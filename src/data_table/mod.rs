//! Bounded explicit local queries or authoritative server pages, displayed by
//! Glory's keyed windowed rows. No source query runs on a scroll notification.
mod local;
mod model;
use crate::{
    appearance::Common,
    composite,
    state::{Control, Dirty, Value},
};
use glory::render::{NodeId, Role, Semantics, SortDirection, TableMetadata};
use glory::style::{Display, Edges, FlexDirection, Length, Overflow, Style};
use glory::ui::{Result, Widget, list::ListStats};
use glory::{Cage, Scope, Vary};
pub use model::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

enum Source {
    Local(Vary<Vec<DataRecord>>),
    Server(Vary<DataPage>),
}
#[derive(Clone, Copy)]
enum Accepted {
    Local(Cage<Rc<Vec<DataRecord>>>),
    Server(Cage<Option<DataPage>>),
}
impl Accepted {
    fn plan(
        self,
        columns: &[DataColumn],
        query: &DataTableQuery,
        local: &RefCell<Option<local::Matches>>,
    ) -> Result<Resolved> {
        let (plan, keys) = match self {
            Self::Local(rows) => rows.try_with(|rows| {
                let plan = local::plan(columns, rows, query, &mut local.borrow_mut());
                let keys = plan
                    .indices
                    .iter()
                    .map(|index| rows[*index].key.clone())
                    .collect::<Vec<_>>();
                Ok::<_, glory::ui::Error>((plan, keys))
            })??,
            Self::Server(page) => page.try_with(|page| {
                let Some(page) = page else {
                    return Ok((
                        DataPlan {
                            indices: vec![],
                            total: None,
                            pages: None,
                            has_next: false,
                            loading: true,
                        },
                        vec![],
                    ));
                };
                let plan = model::server_page_plan(page, query);
                let keys = plan
                    .indices
                    .iter()
                    .map(|index| page.rows[*index].key.clone())
                    .collect::<Vec<_>>();
                Ok::<_, glory::ui::Error>((plan, keys))
            })??,
        };
        let positions = keys
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, key)| (key, index))
            .collect();
        Ok(Resolved {
            plan,
            keys,
            positions,
        })
    }
    fn cell(self, index: usize, key: &str, column: usize) -> String {
        let read = |rows: &Vec<DataRecord>| {
            rows.get(index)
                .filter(|row| row.key == key)
                .and_then(|row| row.cells.get(column))
                .cloned()
                .unwrap_or_default()
        };
        match self {
            Self::Local(rows) => rows.with(|rows| read(rows)),
            Self::Server(page) => page.with(|page| {
                page.as_ref()
                    .map(|page| read(&page.rows))
                    .unwrap_or_default()
            }),
        }
    }
}
fn accept_server_page(
    scope: &Scope,
    source: Vary<DataPage>,
    columns: Rc<Vec<DataColumn>>,
    query: Cage<DataTableQuery>,
) -> Result<Cage<Option<DataPage>>> {
    // Column visibility does not change the server request or require the same
    // accepted page's payload to be validated/copied again.
    let request =
        composite::validated(scope, &query.map(|q| q.request.clone()).into(), |_| Ok(()))?;
    let read = |page: &DataPage, expected: &DataRequest| -> Result<Option<DataPage>> {
        if &page.request != expected {
            return Ok(None);
        }
        server_data_plan(
            &columns,
            page,
            &DataTableQuery {
                request: expected.clone(),
                hidden: BTreeSet::new(),
            },
        )
        .map_err(error)?;
        Ok(Some(page.clone()))
    };
    let initial =
        glory::core::untracked(|| source.try_with(|page| read(page, &request.get_untracked())))??;
    let current = scope.cage(initial);
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let expected = request.try_get()?;
            let next = source.try_with(|page| {
                if page.request != expected {
                    return Ok(None);
                }
                if glory::core::untracked(|| current.with(|old| old.as_ref() == Some(page))) {
                    return Ok(None);
                }
                server_data_plan(
                    &columns,
                    page,
                    &DataTableQuery {
                        request: expected.clone(),
                        hidden: BTreeSet::new(),
                    },
                )
                .map_err(error)?;
                Ok::<_, glory::ui::Error>(Some(page.clone()))
            })??;
            if let Some(next) = next
                && current.with_untracked(|old| old.as_ref() != Some(&next))
            {
                current.try_revise(|old| *old = Some(next))?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error)
        }
    });
    Ok(current)
}

#[cfg(test)]
mod derivation_tests {
    use super::*;
    use model::work::{MATCH_PASSES, VALIDATION_PASSES};

    #[test]
    fn mounted_local_table_only_rederives_matching_on_admitted_data_filter_or_sort_changes() {
        let app = glory::prelude::App::memory();
        let rows = app.scope().cage(
            (0..12)
                .map(|i| {
                    DataRecord::new(format!("r{i}"), [format!("Person {i}"), "Compiler".into()])
                })
                .collect::<Vec<_>>(),
        );
        let query = app.scope().cage(DataTableQuery {
            request: DataRequest {
                page_size: 3,
                ..Default::default()
            },
            ..Default::default()
        });
        MATCH_PASSES.set(0);
        VALIDATION_PASSES.set(0);
        app.mount(
            data_table("passes")
                .columns([
                    DataColumn::new("name", "Name"),
                    DataColumn::new("team", "Team"),
                ])
                .local_rows(rows)
                .query(query),
        )
        .unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (1, 1));
        for page in [2, 3, 1] {
            query.revise(|q| q.request.page = page);
            app.flush().unwrap();
        }
        query.revise(|q| {
            q.request.page_size = 4;
            q.hidden.insert("team".into());
        });
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (1, 1));
        query.revise(|q| q.request.filter = "compiler".into());
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (2, 1));
        query.revise(|q| q.request.filter = "COMPILER".into());
        app.flush().unwrap();
        assert_eq!(MATCH_PASSES.get(), 2);
        query.revise(|q| {
            q.request.sort = Some(DataSort {
                column: "name".into(),
                descending: true,
            })
        });
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (3, 1));
        rows.revise(|rows| rows[0].cells[1] = "Runtime".into());
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (4, 2));
        rows.revise(|rows| rows[1].key = rows[0].key.clone());
        assert!(app.flush().is_err());
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (4, 3));
        query.revise(|q| q.request.page = 2);
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (4, 3));
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }

    #[test]
    fn mounted_server_table_reuses_validated_page_for_visibility_and_waits_for_new_requests() {
        let app = glory::prelude::App::memory();
        let query = app.scope().cage(DataTableQuery::default());
        let page = app.scope().cage(DataPage {
            request: DataRequest::default(),
            rows: vec![DataRecord::new("a", ["Ada", "Compiler"])],
            total: Some(1),
            has_next: false,
        });
        MATCH_PASSES.set(0);
        VALIDATION_PASSES.set(0);
        app.mount(
            data_table("server-passes")
                .columns([
                    DataColumn::new("name", "Name"),
                    DataColumn::new("team", "Team"),
                ])
                .server_page(page)
                .query(query),
        )
        .unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (0, 1));
        query.revise(|q| {
            q.hidden.insert("team".into());
        });
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (0, 1));
        query.revise(|q| q.request.filter = "Runtime".into());
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (0, 1));
        page.revise(|page| page.request = query.with(|q| q.request.clone()));
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (0, 2));
        query.revise(|q| q.hidden.clear());
        app.flush().unwrap();
        assert_eq!((MATCH_PASSES.get(), VALIDATION_PASSES.get()), (0, 2));
    }
}
fn error(value: DataTableError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(value.message()).into()
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Resolved {
    plan: DataPlan,
    keys: Vec<String>,
    positions: BTreeMap<String, usize>,
}
struct QueryControl {
    control: Control<DataTableQuery>,
    accepted: Cage<DataTableQuery>,
    columns: Rc<Vec<DataColumn>>,
    pending: RefCell<Option<DataTableQuery>>,
    dirty: Dirty,
    callback: composite::Callback<DataTableQuery>,
    owner: glory::core::InstanceId,
}
impl QueryControl {
    fn propose(&self, edit: impl FnOnce(&mut DataTableQuery)) -> Result {
        let mut next = self
            .pending
            .borrow()
            .clone()
            .unwrap_or_else(|| self.accepted.get_untracked());
        edit(&mut next);
        validate_data_query(&self.columns, &next).map_err(error)?;
        self.pending.replace(Some(next.clone()));
        self.control.accept(&next)?;
        self.dirty.touch()?;
        self.callback.emit(self.owner, next);
        Ok(())
    }
}
struct SelectionControl {
    control: Control<BTreeSet<String>>,
    accepted: Cage<BTreeSet<String>>,
    pending: RefCell<Option<BTreeSet<String>>>,
    dirty: Dirty,
    callback: composite::Callback<BTreeSet<String>>,
    owner: glory::core::InstanceId,
}
fn selection(value: &BTreeSet<String>) -> Result {
    if value.len() > MAX_LOCAL_DATA_ROWS
        || value
            .iter()
            .any(|key| key.len() > 64 || glory::render::SemanticId::new(key).is_err())
    {
        return Err(error(DataTableError::Key));
    }
    Ok(())
}
impl SelectionControl {
    fn propose(&self, edit: impl FnOnce(&mut BTreeSet<String>)) -> Result {
        let mut next = self
            .pending
            .borrow()
            .clone()
            .unwrap_or_else(|| self.accepted.get_untracked());
        edit(&mut next);
        selection(&next)?;
        self.pending.replace(Some(next.clone()));
        self.control.accept(&next)?;
        self.dirty.touch()?;
        self.callback.emit(self.owner, next);
        Ok(())
    }
}
type CellFactory = Rc<RefCell<dyn FnMut(Cage<usize>, String, Vary<String>) -> Box<dyn Widget>>>;
pub struct DataTable {
    common: Common,
    key: String,
    columns: Vec<DataColumn>,
    source: Option<Source>,
    query: Value<DataTableQuery>,
    selected: Value<BTreeSet<String>>,
    changed: Option<Box<dyn FnMut(DataTableQuery)>>,
    selection_changed: Option<Box<dyn FnMut(BTreeSet<String>)>>,
    selectable: bool,
    height: f64,
    row_height: f64,
    position: Option<Vary<f64>>,
    scroll: Option<Box<dyn FnMut(f64)>>,
    inspect: Option<Box<dyn FnMut(ListStats)>>,
    renderers: BTreeMap<String, CellFactory>,
    empty: String,
    loading: String,
}
pub fn data_table(key: impl Into<String>) -> DataTable {
    DataTable {
        common: Common::default(),
        key: key.into(),
        columns: Vec::new(),
        source: None,
        query: Value::default(),
        selected: Value::default(),
        changed: None,
        selection_changed: None,
        selectable: true,
        height: 320.,
        row_height: 44.,
        position: None,
        scroll: None,
        inspect: None,
        renderers: BTreeMap::new(),
        empty: "No matching rows".into(),
        loading: "Loading rows".into(),
    }
}
impl DataTable {
    pub fn columns(mut self, value: impl IntoIterator<Item = DataColumn>) -> Self {
        self.columns = value.into_iter().take(MAX_DATA_COLUMNS + 1).collect();
        self
    }
    pub fn local_rows(mut self, value: impl Into<Vary<Vec<DataRecord>>>) -> Self {
        self.source = Some(Source::Local(value.into()));
        self
    }
    pub fn server_page(mut self, value: impl Into<Vary<DataPage>>) -> Self {
        self.source = Some(Source::Server(value.into()));
        self
    }
    pub fn query(mut self, value: impl Into<Vary<DataTableQuery>>) -> Self {
        self.query.source = Some(value.into());
        self
    }
    pub fn default_query(mut self, value: DataTableQuery) -> Self {
        self.query.default = value;
        self
    }
    pub fn on_query_change(mut self, value: impl FnMut(DataTableQuery) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn selected(mut self, value: impl Into<Vary<BTreeSet<String>>>) -> Self {
        self.selected.source = Some(value.into());
        self
    }
    pub fn default_selected(mut self, value: BTreeSet<String>) -> Self {
        self.selected.default = value;
        self
    }
    pub fn on_selection_change(mut self, value: impl FnMut(BTreeSet<String>) + 'static) -> Self {
        self.selection_changed = Some(Box::new(value));
        self
    }
    pub fn selectable(mut self, value: bool) -> Self {
        self.selectable = value;
        self
    }
    pub fn height(mut self, value: f64) -> Self {
        self.height = value;
        self
    }
    pub fn row_height(mut self, value: f64) -> Self {
        self.row_height = value;
        self
    }
    pub fn position(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.position = Some(value.into());
        self
    }
    pub fn on_scroll(mut self, value: impl FnMut(f64) + 'static) -> Self {
        self.scroll = Some(Box::new(value));
        self
    }
    pub fn inspect(mut self, value: impl FnMut(ListStats) + 'static) -> Self {
        self.inspect = Some(Box::new(value));
        self
    }
    pub fn render_cell<W: Widget>(
        mut self,
        key: impl Into<String>,
        mut factory: impl FnMut(Cage<usize>, String, Vary<String>) -> W + 'static,
    ) -> Self {
        self.renderers.insert(
            key.into(),
            Rc::new(RefCell::new(move |index, key, value| {
                Box::new(factory(index, key, value)) as Box<dyn Widget>
            })),
        );
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn empty_label(mut self, value: impl Into<String>) -> Self {
        self.empty = value.into();
        self
    }
    pub fn loading_label(mut self, value: impl Into<String>) -> Self {
        self.loading = value.into();
        self
    }
}
fn fixed_width(width: f64) -> Style {
    let mut style = Style::new().width(Length::Px(width));
    style.shrink = Some(0.);
    style
}
fn row_style() -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
}
fn metadata(value: TableMetadata) -> Option<Box<Semantics>> {
    Some(Box::new(Semantics {
        table: Some(value),
        ..Default::default()
    }))
}
fn column_index(
    columns: &[DataColumn],
    query: &DataTableQuery,
    index: usize,
    selectable: bool,
) -> u16 {
    (usize::from(selectable)
        + columns[..index]
            .iter()
            .filter(|column| !query.hidden.contains(&column.key))
            .count()) as u16
}
impl Widget for DataTable {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("data-table", &self.key, "root", "root")?;
        validate_data_columns(&self.columns).map_err(error)?;
        if !self.height.is_finite()
            || self.height <= 0.
            || !self.row_height.is_finite()
            || self.row_height < 24.
            || self.row_height > 256.
        {
            return Err(glory::render::Error::InvalidProps(
                "data table requires finite positive viewport and 24..256px rows",
            )
            .into());
        }
        if self.empty.len() > 256
            || self.loading.len() > 256
            || self.empty.contains(['\0', '\r'])
            || self.loading.contains(['\0', '\r'])
        {
            return Err(error(DataTableError::Text));
        }
        if self
            .renderers
            .keys()
            .any(|key| !self.columns.iter().any(|column| &column.key == key))
        {
            return Err(error(DataTableError::Columns));
        }
        let columns = Rc::new(std::mem::take(&mut self.columns));
        let validation = columns.clone();
        let raw_query = std::mem::take(&mut self.query).mount(scope);
        let query = composite::validated(scope, &raw_query.source, move |q| {
            validate_data_query(&validation, q).map_err(error)
        })?;
        let validation = columns.clone();
        let data = match self
            .source
            .take()
            .ok_or_else(|| error(DataTableError::Rows))?
        {
            Source::Local(source) => Accepted::Local(local::accept(scope, &source, validation)?),
            Source::Server(source) => {
                Accepted::Server(accept_server_page(scope, source, validation, query)?)
            }
        };
        let validation = columns.clone();
        let local = RefCell::new(None);
        let plan_source = scope.bond(move || data.plan(&validation, &query.get(), &local));
        let initial = plan_source.get()?;
        let resolved = scope.cage(initial);
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            match plan_source.get() {
                Ok(next) => {
                    if resolved.with(|old| old != &next) {
                        resolved.revise(|old| *old = next)
                    }
                }
                Err(error) => scope.report_error(error),
            }
        });
        let dirty = Dirty::new(scope);
        let controller = Rc::new(QueryControl {
            control: raw_query,
            accepted: query,
            columns: columns.clone(),
            pending: RefCell::new(None),
            dirty: dirty.clone(),
            callback: composite::Callback::new(self.changed.take()),
            owner: scope.id(),
        });
        let reset = Rc::downgrade(&controller);
        scope.effect(move || {
            if dirty.take()
                && let Some(control) = reset.upgrade()
            {
                control.pending.take();
            }
        });
        let raw_selection = std::mem::take(&mut self.selected).mount(scope);
        let selected = composite::validated(scope, &raw_selection.source, selection)?;
        let dirty = Dirty::new(scope);
        let selection = Rc::new(SelectionControl {
            control: raw_selection,
            accepted: selected,
            pending: RefCell::new(None),
            dirty: dirty.clone(),
            callback: composite::Callback::new(self.selection_changed.take()),
            owner: scope.id(),
        });
        let reset = Rc::downgrade(&selection);
        scope.effect(move || {
            if dirty.take()
                && let Some(control) = reset.upgrade()
            {
                control.pending.take();
            }
        });
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let (classes, style) =
            self.common
                .appearance(scope, "sapid-data-table", composite::column(8.));
        glory::ui::view()
            .class(classes)
            .style(style)
            .node_ref(root)
            .show_in(scope)?;
        let parent = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
        let theme = self.common.theme.clone();
        let enabled = self.common.enabled.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let edit = controller.clone();
        let weak = scope.downgrade();
        let mut filter = crate::text_field()
            .theme(theme.clone())
            .size(size)
            .name("Filter table rows")
            .enabled(enabled.clone())
            .value(query.map(|q| q.request.filter.clone()))
            .on_value_change(move |value| {
                if let Err(error) = edit.propose(move |q| {
                    q.request.filter = value;
                    q.request.page = 1
                }) && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(error)
                }
            });
        if unstyled {
            filter = filter.unstyled()
        }
        filter.show_in(&parent)?;
        let visibility = parent.cage(None);
        glory::ui::view()
            .class("sapid-data-table-columns")
            .style(row_style().gap(8.))
            .node_ref(visibility)
            .show_in(&parent)?;
        let visibility_scope = parent.child_in(visibility.get().unwrap())?;
        for column in columns.iter() {
            let key = column.key.clone();
            let name = format!("Show {}", column.label);
            let edit = controller.clone();
            let weak = scope.downgrade();
            let current = key.clone();
            let validation = columns.clone();
            let source = enabled.clone();
            let visibility_key = key.clone();
            let active = scope.bond(move || {
                source.get()
                    && (query.with(|q| {
                        q.hidden.contains(&visibility_key) || q.hidden.len() + 1 < validation.len()
                    }))
            });
            let mut check = crate::checkbox()
                .theme(theme.clone())
                .size(size)
                .label(name)
                .enabled(active)
                .checked(query.map(move |q| !q.hidden.contains(&current)))
                .on_checked_change(move |checked| {
                    if let Err(error) = edit.propose(|q| {
                        if checked {
                            q.hidden.remove(&key);
                        } else {
                            q.hidden.insert(key.clone());
                        }
                    }) && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error)
                    }
                });
            if unstyled {
                check = check.unstyled()
            }
            check.show_in(&visibility_scope)?;
        }
        parent.retain(visibility_scope)?;
        let scroll = parent.cage(None);
        glory::ui::view()
            .class("sapid-data-table-horizontal")
            .style(Style::new().overflow_x(Overflow::Auto))
            .node_ref(scroll)
            .show_in(&parent)?;
        let horizontal = parent.child_in(scroll.get().unwrap())?;
        let table = horizontal.cage(None);
        let selectable = self.selectable;
        let widths = columns.clone();
        let total_width = scope.bond(move || {
            f64::from(
                u32::from(selectable) * 48
                    + widths
                        .iter()
                        .filter(|column| !query.with(|q| q.hidden.contains(&column.key)))
                        .map(|column| u32::from(column.width))
                        .sum::<u32>(),
            )
        });
        let table_style = scope.bond(move || Style::new().min_width(Length::Px(total_width.get())));
        let column_count = columns.len();
        let total = scope.bond(move || {
            metadata(TableMetadata::Table {
                rows: resolved
                    .with(|value| value.plan.total)
                    .and_then(|total| u32::try_from(total).ok())
                    .and_then(|total| total.checked_add(1))
                    .filter(|count| *count < u32::MAX),
                columns: Some(
                    (columns_count(query, column_count) + usize::from(selectable)) as u16,
                ),
            })
        });
        let name = self
            .common
            .name
            .take()
            .unwrap_or_else(|| "Data table".into());
        glory::ui::view()
            .class("sapid-data-table-grid")
            .role(Role::Table)
            .name(name)
            .node_ref(table)
            .style(table_style)
            .semantics(total)
            .show_in(&horizontal)?;
        let table_scope = horizontal.child_in(table.get().unwrap())?;
        let head = table_scope.cage(None);
        glory::ui::view()
            .role(Role::Row)
            .semantics(metadata(TableMetadata::Row { index: 0 }))
            .class("sapid-data-table-head")
            .style(row_style())
            .node_ref(head)
            .show_in(&table_scope)?;
        let head_scope = table_scope.child_in(head.get().unwrap())?;
        if selectable {
            glory::ui::view()
                .role(Role::ColumnHeader)
                .name("Selection")
                .style(fixed_width(48.))
                .fill(glory::ui::text().content("Select"))
                .show_in(&head_scope)?;
        }
        for (index, column) in columns.iter().enumerate() {
            let key = column.key.clone();
            let layout = columns.clone();
            let visible = key.clone();
            let sort_key = key.clone();
            let sortable = column.sortable;
            let header = scope.bond(move || {
                let q = query.get();
                Some(Box::new(Semantics {
                    hidden: q.hidden.contains(&visible),
                    sort: if sortable {
                        column_sort(&q, &sort_key)
                    } else {
                        None
                    },
                    table: Some(TableMetadata::Cell {
                        row: 0,
                        column: column_index(&layout, &q, index, selectable),
                        row_span: 1,
                        column_span: 1,
                        headers: vec![],
                    }),
                    ..Default::default()
                }))
            });
            let mut cell = glory::ui::view()
                .role(Role::ColumnHeader)
                .semantics(header)
                .class("sapid-data-table-header")
                .style(fixed_width(f64::from(column.width)));
            if column.sortable {
                let edit = controller.clone();
                let weak = scope.downgrade();
                let sort_key = key.clone();
                let label = column.label.clone();
                let text = scope.bond(move || {
                    format!(
                        "{}{}",
                        label,
                        match query.with(|q| column_sort(q, &sort_key)) {
                            Some(SortDirection::Ascending) => " ↑",
                            Some(SortDirection::Descending) => " ↓",
                            _ => "",
                        }
                    )
                });
                let mut button = crate::button()
                    .theme(theme.clone())
                    .size(size)
                    .enabled(enabled.clone())
                    .name(format!("Sort by {}", column.label))
                    .label(text)
                    .on_press(move || {
                        if let Err(error) = edit.propose(|q| {
                            q.request.sort = match &q.request.sort {
                                Some(sort) if sort.column == key && !sort.descending => {
                                    Some(DataSort {
                                        column: key.clone(),
                                        descending: true,
                                    })
                                }
                                Some(sort) if sort.column == key => None,
                                _ => Some(DataSort {
                                    column: key.clone(),
                                    descending: false,
                                }),
                            };
                            q.request.page = 1
                        }) && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error)
                        }
                    });
                if unstyled {
                    button = button.unstyled()
                }
                cell = cell.fill(button);
            } else {
                cell = cell.fill(glory::ui::text().content(column.label.clone()))
            }
            cell.show_in(&head_scope)?;
        }
        table_scope.retain(head_scope)?;
        let count = scope.bond(move || resolved.with(|page| page.keys.len()));
        let key_at = resolved;
        let offset = scope.bond(move || {
            ((query.with(|q| (q.request.page - 1) * q.request.page_size)) + 1) as u32
        });
        let body_columns = columns.clone();
        let body_theme = theme.clone();
        let body_enabled = enabled.clone();
        let row_height = self.row_height;
        let renderers = Rc::new(std::mem::take(&mut self.renderers));
        let selected_control = selection.clone();
        let mut rows = glory::ui::list(
            count,
            move |index| key_at.with(|page| page.keys[index].clone()),
            move |index, key| {
                let columns = body_columns.clone();
                let theme = body_theme.clone();
                let enabled = body_enabled.clone();
                let selection = selected_control.clone();
                let renderers = renderers.clone();
                glory::ui::widget(move |row_scope| {
                    let root = row_scope.cage(None);
                    glory::ui::view()
                        .class("sapid-data-table-row")
                        .style(row_style().height(Length::Px(row_height)))
                        .node_ref(root)
                        .show_in(row_scope)?;
                    let content = row_scope.child_in(root.get().unwrap())?;
                    if selectable {
                        let key = key.clone();
                        let check_key = key.clone();
                        let weak = row_scope.downgrade();
                        let selected_row = selected.map(move |set| set.contains(&check_key));
                        let selection = selection.clone();
                        let mut check = crate::checkbox()
                            .theme(theme.clone())
                            .size(size)
                            .name(format!("Select row {key}"))
                            .label("")
                            .enabled(enabled.clone())
                            .checked(selected_row)
                            .on_checked_change(move |checked| {
                                if let Err(error) = selection.propose(|set| {
                                    if checked {
                                        set.insert(key.clone());
                                    } else {
                                        set.remove(&key);
                                    }
                                }) && let Some(scope) = weak.upgrade()
                                {
                                    scope.report_error(error)
                                }
                            });
                        if unstyled {
                            check = check.unstyled()
                        }
                        glory::ui::view()
                            .role(Role::Cell)
                            .style(fixed_width(48.).overflow_x(Overflow::Clip))
                            .fill(check)
                            .show_in(&content)?;
                    }
                    for (column_index_value, column) in columns.iter().enumerate() {
                        let row_key = key.clone();
                        let value = row_scope.bond(move || {
                            resolved
                                .with(|page| page.plan.indices.get(index.get()).copied())
                                .map(|source_index| {
                                    data.cell(source_index, &row_key, column_index_value)
                                })
                                .unwrap_or_default()
                        });
                        let column_key = column.key.clone();
                        let metadata_columns = columns.clone();
                        let meta = row_scope.bond(move || {
                            let q = query.get();
                            Some(Box::new(Semantics {
                                hidden: q.hidden.contains(&column_key),
                                table: Some(TableMetadata::Cell {
                                    row: offset.get() + index.get() as u32,
                                    column: column_index(
                                        &metadata_columns,
                                        &q,
                                        column_index_value,
                                        selectable,
                                    ),
                                    row_span: 1,
                                    column_span: 1,
                                    headers: vec![],
                                }),
                                ..Default::default()
                            }))
                        });
                        let mut style = fixed_width(f64::from(column.width))
                            .height(Length::Percent(100.))
                            .overflow_x(Overflow::Clip)
                            .overflow_y(Overflow::Clip);
                        if !unstyled {
                            style.padding = Some(Edges::all(8.));
                            style.color = Some(theme.foreground);
                            style.font_size = theme.label(size).font_size;
                            style.background = Some(theme.surface);
                        }
                        let mut cell = glory::ui::view()
                            .class("sapid-data-table-cell")
                            .role(Role::Cell)
                            .style(style)
                            .semantics(meta);
                        if let Some(factory) = renderers.get(&column.key) {
                            let factory = factory.clone();
                            let key = key.clone();
                            cell = cell.fill(glory::ui::widget(move |scope| {
                                let widget =
                                    (factory.borrow_mut())(index, key.clone(), value.into());
                                widget.show_in(scope)
                            }));
                        } else {
                            cell = cell.fill(glory::ui::text().content(value))
                        }
                        cell.show_in(&content)?;
                    }
                    row_scope.retain(content)?;
                    Ok(())
                })
            },
        )
        .table_rows(offset)
        .height(self.height)
        .row_height(self.row_height)
        .overscan(2)
        .row_limit(256)
        .locate(move |key| resolved.with(|page| page.positions.get(key).copied()));
        if let Some(position) = self.position.take() {
            rows = rows.position(position)
        }
        if let Some(callback) = self.scroll.take() {
            rows = rows.on_scroll(callback)
        }
        if let Some(inspect) = self.inspect.take() {
            rows = rows.inspect(inspect)
        }
        rows.show_in(&table_scope)?;
        let empty = self.empty.clone();
        let loading = self.loading.clone();
        glory::ui::text()
            .class("sapid-data-table-status")
            .content(scope.bond(move || {
                resolved.with(|page| {
                    if page.plan.loading {
                        loading.clone()
                    } else if page.keys.is_empty() {
                        empty.clone()
                    } else {
                        String::new()
                    }
                })
            }))
            .show_in(&table_scope)?;
        horizontal.retain(table_scope)?;
        parent.retain(horizontal)?;
        let footer = parent.cage(None);
        glory::ui::view()
            .style(row_style().gap(8.))
            .node_ref(footer)
            .show_in(&parent)?;
        let footer_scope = parent.child_in(footer.get().unwrap())?;
        for next in [false, true] {
            let edit = controller.clone();
            let weak = scope.downgrade();
            let active = enabled.clone();
            let available = scope.bond(move || {
                active.get()
                    && if next {
                        resolved.with(|page| page.plan.has_next)
                    } else {
                        query.with(|q| q.request.page > 1)
                    }
            });
            let mut button = crate::button()
                .theme(theme.clone())
                .size(size)
                .enabled(available)
                .label(if next {
                    "Next table page"
                } else {
                    "Previous table page"
                })
                .on_press(move || {
                    if let Err(error) = edit.propose(|q| {
                        q.request.page = if next {
                            q.request.page.saturating_add(1)
                        } else {
                            q.request.page.saturating_sub(1).max(1)
                        }
                    }) && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error)
                    }
                });
            if unstyled {
                button = button.unstyled()
            }
            button.show_in(&footer_scope)?;
        }
        glory::ui::text()
            .content(scope.bond(move || {
                format!(
                    "Page {} · {} selected",
                    query.with(|q| q.request.page),
                    selected.with(BTreeSet::len)
                )
            }))
            .show_in(&footer_scope)?;
        parent.retain(footer_scope)?;
        scope.retain(parent)?;
        Ok(())
    }
}
fn columns_count(query: Cage<DataTableQuery>, count: usize) -> usize {
    query.with(|q| count - q.hidden.len())
}
fn column_sort(query: &DataTableQuery, key: &str) -> Option<SortDirection> {
    Some(match &query.request.sort {
        Some(sort) if sort.column == key => {
            if sort.descending {
                SortDirection::Descending
            } else {
                SortDirection::Ascending
            }
        }
        _ => SortDirection::None,
    })
}
