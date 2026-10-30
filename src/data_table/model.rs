//! Explicit local query work and independently supplied server pages. Scrolling
//! consumes the accepted page's indices, never reruns a business-source query.
use std::collections::BTreeSet;

pub const MAX_DATA_COLUMNS: usize = 32;
pub const MAX_LOCAL_DATA_ROWS: usize = 4096;
pub const MAX_DATA_TEXT_BYTES: usize = 4096;
pub const MAX_DATA_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataColumn {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub width: u16,
}
impl DataColumn {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            sortable: true,
            width: 160,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataRecord {
    pub key: String,
    /// Values follow the configured column order, including hidden columns.
    pub cells: Vec<String>,
}
impl DataRecord {
    pub fn new(key: impl Into<String>, cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            cells: cells
                .into_iter()
                .take(MAX_DATA_COLUMNS + 1)
                .map(Into::into)
                .collect(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSort {
    pub column: String,
    pub descending: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataRequest {
    /// One-based page. Page changes do not implicitly clear explicit selection.
    pub page: usize,
    pub page_size: usize,
    pub filter: String,
    pub sort: Option<DataSort>,
}
impl Default for DataRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 25,
            filter: String::new(),
            sort: None,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataTableQuery {
    pub request: DataRequest,
    /// Presentation only; server requests do not include column visibility.
    pub hidden: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataPage {
    /// Echo the exact request which produced this page. Obsolete pages are
    /// never interpreted as the response to a more recent query.
    pub request: DataRequest,
    pub rows: Vec<DataRecord>,
    pub total: Option<usize>,
    pub has_next: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataPlan {
    pub indices: Vec<usize>,
    pub total: Option<usize>,
    pub pages: Option<usize>,
    pub has_next: bool,
    pub loading: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTableError {
    Columns,
    Key,
    DuplicateKey,
    Text,
    Rows,
    Payload,
    Query,
    Page,
}
impl DataTableError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Columns => {
                "data table requires 1..32 uniquely keyed columns with 48..600px widths"
            }
            Self::Key => "data table keys require 1..64 semantic ASCII bytes",
            Self::DuplicateKey => "data table current rows require unique complete keys",
            Self::Text => "data table values require bounded canonical text",
            Self::Rows => {
                "data table local source exceeds 4096 current records or its column shape"
            }
            Self::Payload => "data table current text payload exceeds 8 MiB",
            Self::Query => "data table query exceeds finite bounds or references an unknown column",
            Self::Page => "data table server page conflicts with its echoed request or total",
        }
    }
}
impl std::fmt::Display for DataTableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}
impl std::error::Error for DataTableError {}

type Result<T> = std::result::Result<T, DataTableError>;
fn valid_key(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && glory::render::SemanticId::new(value).is_ok()
}
fn valid_text(value: &str, budget: usize) -> bool {
    value.len() <= budget && !value.contains(['\0', '\r'])
}
pub fn validate_data_columns(columns: &[DataColumn]) -> Result<()> {
    if columns.is_empty() || columns.len() > MAX_DATA_COLUMNS {
        return Err(DataTableError::Columns);
    }
    let mut keys = BTreeSet::new();
    for column in columns {
        if !valid_key(&column.key) {
            return Err(DataTableError::Key);
        }
        if !(48..=600).contains(&column.width) || !keys.insert(&column.key) {
            return Err(DataTableError::Columns);
        }
        if !valid_text(&column.label, 256)
            || column.label.trim().is_empty()
            || column.label.contains('\n')
        {
            return Err(DataTableError::Text);
        }
    }
    Ok(())
}
pub fn validate_data_query(columns: &[DataColumn], query: &DataTableQuery) -> Result<()> {
    let request = &query.request;
    if request.page == 0
        || !(1..=MAX_LOCAL_DATA_ROWS).contains(&request.page_size)
        || request
            .page
            .checked_sub(1)
            .and_then(|p| p.checked_mul(request.page_size))
            .is_none_or(|start| {
                start >= u32::MAX as usize
                    || start.saturating_add(request.page_size) >= u32::MAX as usize
            })
        || !valid_text(&request.filter, 256)
        || request.filter.contains('\n')
        || query.hidden.len() >= columns.len()
        || query
            .hidden
            .iter()
            .any(|key| !columns.iter().any(|column| &column.key == key))
        || request.sort.as_ref().is_some_and(|sort| {
            !columns
                .iter()
                .any(|column| column.key == sort.column && column.sortable)
        })
    {
        return Err(DataTableError::Query);
    }
    Ok(())
}
pub fn validate_data_records(columns: &[DataColumn], rows: &[DataRecord]) -> Result<()> {
    #[cfg(test)]
    work::VALIDATION_PASSES.with(|count| count.set(count.get() + 1));
    if rows.len() > MAX_LOCAL_DATA_ROWS {
        return Err(DataTableError::Rows);
    }
    let mut keys = BTreeSet::new();
    let mut bytes = 0usize;
    for row in rows {
        if !valid_key(&row.key) {
            return Err(DataTableError::Key);
        }
        if !keys.insert(&row.key) {
            return Err(DataTableError::DuplicateKey);
        }
        if row.cells.len() != columns.len() {
            return Err(DataTableError::Rows);
        }
        for cell in &row.cells {
            if !valid_text(cell, MAX_DATA_TEXT_BYTES) {
                return Err(DataTableError::Text);
            }
            bytes = bytes
                .checked_add(cell.len())
                .filter(|bytes| *bytes <= MAX_DATA_PAYLOAD_BYTES)
                .ok_or(DataTableError::Payload)?;
        }
    }
    Ok(())
}
/// Validate all inputs and resolve one standalone local query. Stable equal
/// values retain business order in both directions. Mounted DataTable widgets
/// reuse admitted matches when only pagination or column visibility changes.
pub fn local_data_plan(
    columns: &[DataColumn],
    rows: &[DataRecord],
    query: &DataTableQuery,
) -> Result<DataPlan> {
    validate_data_columns(columns)?;
    validate_data_query(columns, query)?;
    validate_data_records(columns, rows)?;
    let needle = query.request.filter.to_lowercase();
    let indices = local_match_indices(columns, rows, &needle, query.request.sort.as_ref());
    Ok(local_page_plan(&indices, &query.request))
}

// Internal callers have already admitted the columns, records and query.
pub(super) fn local_match_indices(
    columns: &[DataColumn],
    rows: &[DataRecord],
    needle: &str,
    sort: Option<&DataSort>,
) -> Vec<usize> {
    #[cfg(test)]
    MATCH_PASSES.with(|count| count.set(count.get() + 1));
    let mut indices = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            (needle.is_empty()
                || row
                    .cells
                    .iter()
                    .any(|cell| cell.to_lowercase().contains(needle)))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if let Some(sort) = sort {
        let column = columns
            .iter()
            .position(|column| column.key == sort.column)
            .expect("validated sort column");
        indices.sort_by(|a, b| {
            let order = rows[*a].cells[column].cmp(&rows[*b].cells[column]);
            if sort.descending {
                order.reverse()
            } else {
                order
            }
        });
    }
    indices
}

pub(super) fn local_page_plan(indices: &[usize], request: &DataRequest) -> DataPlan {
    let total = indices.len();
    let start = (request.page - 1) * request.page_size;
    let end = start.saturating_add(request.page_size).min(total);
    let selected = if start < end {
        indices[start..end].to_vec()
    } else {
        Vec::new()
    };
    DataPlan {
        indices: selected,
        total: Some(total),
        pages: Some(total.div_ceil(request.page_size).max(1)),
        has_next: end < total,
        loading: false,
    }
}

#[cfg(test)]
pub(super) mod work {
    use std::cell::Cell;
    thread_local! {
        pub static MATCH_PASSES: Cell<usize> = const { Cell::new(0) };
        pub static VALIDATION_PASSES: Cell<usize> = const { Cell::new(0) };
    }
}
#[cfg(test)]
use work::MATCH_PASSES;
/// Server pages are already sorted/filtered. Their finite rows are validated,
/// then indexed as supplied; no local comparator or filter is applied.
pub fn server_data_plan(
    columns: &[DataColumn],
    page: &DataPage,
    query: &DataTableQuery,
) -> Result<DataPlan> {
    validate_data_columns(columns)?;
    validate_data_query(columns, query)?;
    validate_data_query(
        columns,
        &DataTableQuery {
            request: page.request.clone(),
            hidden: BTreeSet::new(),
        },
    )?;
    validate_data_records(columns, &page.rows)?;
    if page.rows.len() > page.request.page_size
        || page.total.is_some_and(|total| total >= u32::MAX as usize)
    {
        return Err(DataTableError::Page);
    }
    let start = (page.request.page - 1) * page.request.page_size;
    if let Some(total) = page.total {
        let expected = total.saturating_sub(start).min(page.request.page_size);
        if page.rows.len() != expected
            || page.has_next != (start.saturating_add(page.rows.len()) < total)
        {
            return Err(DataTableError::Page);
        }
    }
    Ok(server_page_plan(page, query))
}

pub(super) fn server_page_plan(page: &DataPage, query: &DataTableQuery) -> DataPlan {
    if page.request != query.request {
        return DataPlan {
            indices: Vec::new(),
            total: None,
            pages: None,
            has_next: false,
            loading: true,
        };
    }
    DataPlan {
        indices: (0..page.rows.len()).collect(),
        total: page.total,
        pages: page
            .total
            .map(|total| total.div_ceil(page.request.page_size).max(1)),
        has_next: page.has_next,
        loading: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn columns() -> Vec<DataColumn> {
        vec![
            DataColumn::new("name", "Name"),
            DataColumn::new("team", "Team"),
        ]
    }
    fn rows() -> Vec<DataRecord> {
        vec![
            DataRecord::new("ada", ["Ada", "Compiler"]),
            DataRecord::new("grace", ["Grace", "Compiler"]),
            DataRecord::new("linus", ["Linus", "Kernel"]),
        ]
    }
    #[test]
    fn explicit_local_sort_filter_page_and_empty_ranges_keep_complete_keys() {
        let cols = columns();
        let rows = rows();
        let mut q = DataTableQuery::default();
        q.request.page_size = 1;
        q.request.sort = Some(DataSort {
            column: "name".into(),
            descending: true,
        });
        let p = local_data_plan(&cols, &rows, &q).unwrap();
        assert_eq!(p.indices, [2]);
        assert_eq!(p.total, Some(3));
        assert!(p.has_next);
        q.request.page = 2;
        assert_eq!(local_data_plan(&cols, &rows, &q).unwrap().indices, [1]);
        q.request.filter = "cOmPiLeR".into();
        assert_eq!(local_data_plan(&cols, &rows, &q).unwrap().indices, [0]);
        q.request.page = 100;
        assert!(
            local_data_plan(&cols, &rows, &q)
                .unwrap()
                .indices
                .is_empty()
        );
    }
    #[test]
    fn descending_equal_values_keep_original_business_order() {
        let q = DataTableQuery {
            request: DataRequest {
                sort: Some(DataSort {
                    column: "team".into(),
                    descending: true,
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            local_data_plan(&columns(), &rows(), &q).unwrap().indices,
            [2, 0, 1]
        );
    }
    #[test]
    fn server_order_is_authoritative_and_old_request_never_becomes_current() {
        let q = DataTableQuery::default();
        let p = DataPage {
            request: q.request.clone(),
            rows: rows(),
            total: Some(3),
            has_next: false,
        };
        assert_eq!(
            server_data_plan(&columns(), &p, &q).unwrap().indices,
            [0, 1, 2]
        );
        let mut new = q.clone();
        new.request.filter = "absent".into();
        let plan = server_data_plan(&columns(), &p, &new).unwrap();
        assert!(plan.loading && plan.indices.is_empty());
    }
    #[test]
    fn malformed_columns_records_queries_and_server_totals_are_rejected() {
        let cols = columns();
        let mut data = rows();
        data[1].key = data[0].key.clone();
        assert_eq!(
            validate_data_records(&cols, &data),
            Err(DataTableError::DuplicateKey)
        );
        let mut q = DataTableQuery {
            hidden: cols.iter().map(|c| c.key.clone()).collect(),
            ..DataTableQuery::default()
        };
        assert_eq!(validate_data_query(&cols, &q), Err(DataTableError::Query));
        q = DataTableQuery::default();
        q.request.page = usize::MAX;
        assert_eq!(validate_data_query(&cols, &q), Err(DataTableError::Query));
        let p = DataPage {
            request: DataRequest::default(),
            rows: rows(),
            total: Some(2),
            has_next: false,
        };
        assert_eq!(
            server_data_plan(&cols, &p, &DataTableQuery::default()),
            Err(DataTableError::Page)
        );
    }
    #[test]
    fn current_text_and_iterator_shape_have_independent_limits() {
        let row = DataRecord::new("oversized", (0..1_000_000).map(|_| "value"));
        assert_eq!(row.cells.len(), MAX_DATA_COLUMNS + 1);
        assert_eq!(
            validate_data_records(&columns(), &[row]),
            Err(DataTableError::Rows)
        );
        let row = DataRecord::new("text", ["a".repeat(4097), "b".into()]);
        assert_eq!(
            validate_data_records(&columns(), &[row]),
            Err(DataTableError::Text)
        );
    }
}
