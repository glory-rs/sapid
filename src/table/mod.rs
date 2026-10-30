//! A finite intrinsic table; all content factories run after grid admission.
use crate::appearance::Common;
use crate::composite;
use glory::render::{DocumentPart, NodeId, SemanticId, Semantics, TableMetadata};
use glory::style::{BorderStyle, Edges, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::collections::BTreeSet;
mod model;
pub use model::{MAX_TABLE_CELLS, MAX_TABLE_COLUMNS, MAX_TABLE_ROWS};
type Factory = Box<dyn FnMut() -> Box<dyn Widget>>;
fn invalid(value: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(value).into()
}
fn key(value: &str) -> Result {
    if value.len() > 64 {
        return Err(invalid("table keys require at most 64 ASCII bytes"));
    }
    SemanticId::new(value)?;
    Ok(())
}
fn id(table: &str, row: &str, cell: &str) -> Result<SemanticId> {
    Ok(SemanticId::new(format!(
        "sapid-table-cell-{}_{table}-{}_{row}-{}_{cell}",
        table.len(),
        row.len(),
        cell.len()
    ))?)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableCellKind {
    Data,
    ColumnHeader,
    RowHeader,
}
pub struct TableCell {
    key: String,
    kind: TableCellKind,
    text: Vary<String>,
    content: Option<Factory>,
    span: model::Span,
    common: Common,
    theme: Option<crate::Theme>,
    size: Option<crate::Size>,
}
pub fn table_cell(key: impl Into<String>) -> TableCell {
    TableCell {
        key: key.into(),
        kind: TableCellKind::Data,
        text: String::new().into(),
        content: None,
        span: model::Span {
            rows: 1,
            columns: 1,
        },
        common: Common::default(),
        theme: None,
        size: None,
    }
}
pub fn table_header_cell(key: impl Into<String>) -> TableCell {
    let mut value = table_cell(key);
    value.kind = TableCellKind::ColumnHeader;
    value
}
pub fn table_row_header(key: impl Into<String>) -> TableCell {
    let mut value = table_cell(key);
    value.kind = TableCellKind::RowHeader;
    value
}
macro_rules! appearance {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.theme = Some(value);
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.size = Some(value);
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
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
    };
}
impl TableCell {
    appearance!();
    pub fn text(mut self, value: impl Into<Vary<String>>) -> Self {
        self.text = value.into();
        self.content = None;
        self
    }
    pub fn content<W: Widget>(mut self, mut value: impl FnMut() -> W + 'static) -> Self {
        self.content = Some(Box::new(move || Box::new(value())));
        self
    }
    pub fn row_span(mut self, value: u16) -> Self {
        self.span.rows = value;
        self
    }
    pub fn column_span(mut self, value: u16) -> Self {
        self.span.columns = value;
        self
    }
}
pub struct TableRow {
    key: String,
    cells: Vec<TableCell>,
    common: Common,
    theme: Option<crate::Theme>,
    size: Option<crate::Size>,
}
pub fn table_row(key: impl Into<String>) -> TableRow {
    TableRow {
        key: key.into(),
        cells: Vec::new(),
        common: Common::default(),
        theme: None,
        size: None,
    }
}
impl TableRow {
    appearance!();
    pub fn cell(mut self, value: TableCell) -> Self {
        if self.cells.len() <= MAX_TABLE_COLUMNS {
            self.cells.push(value)
        }
        self
    }
    pub fn cells(mut self, values: impl IntoIterator<Item = TableCell>) -> Self {
        let left = (MAX_TABLE_COLUMNS + 1).saturating_sub(self.cells.len());
        self.cells.extend(values.into_iter().take(left));
        self
    }
}
#[derive(Clone, Copy)]
enum SectionKind {
    Head,
    Body,
    Foot,
}
pub struct TableSection {
    kind: SectionKind,
    rows: Vec<TableRow>,
}
fn section(kind: SectionKind) -> TableSection {
    TableSection {
        kind,
        rows: Vec::new(),
    }
}
pub fn table_head() -> TableSection {
    section(SectionKind::Head)
}
pub fn table_body() -> TableSection {
    section(SectionKind::Body)
}
pub fn table_foot() -> TableSection {
    section(SectionKind::Foot)
}
impl TableSection {
    pub fn row(mut self, value: TableRow) -> Self {
        if self.rows.len() <= MAX_TABLE_ROWS {
            self.rows.push(value)
        }
        self
    }
    pub fn rows(mut self, values: impl IntoIterator<Item = TableRow>) -> Self {
        let left = (MAX_TABLE_ROWS + 1).saturating_sub(self.rows.len());
        self.rows.extend(values.into_iter().take(left));
        self
    }
}
pub struct Table {
    key: String,
    common: Common,
    caption: Option<Vary<String>>,
    sections: Vec<TableSection>,
    striped: bool,
    bordered: bool,
}
pub fn table(key: impl Into<String>) -> Table {
    Table {
        key: key.into(),
        common: Common::default(),
        caption: None,
        sections: Vec::new(),
        striped: false,
        bordered: false,
    }
}
impl Table {
    pub fn caption(mut self, value: impl Into<Vary<String>>) -> Self {
        self.caption = Some(value.into());
        self
    }
    pub fn head(mut self, value: TableSection) -> Self {
        self.sections
            .retain(|s| !matches!(s.kind, SectionKind::Head));
        self.sections.push(TableSection {
            kind: SectionKind::Head,
            ..value
        });
        self
    }
    pub fn body(mut self, value: TableSection) -> Self {
        self.sections
            .retain(|s| !matches!(s.kind, SectionKind::Body));
        self.sections.push(TableSection {
            kind: SectionKind::Body,
            ..value
        });
        self
    }
    pub fn foot(mut self, value: TableSection) -> Self {
        self.sections
            .retain(|s| !matches!(s.kind, SectionKind::Foot));
        self.sections.push(TableSection {
            kind: SectionKind::Foot,
            ..value
        });
        self
    }
    pub fn striped(mut self, value: bool) -> Self {
        self.striped = value;
        self
    }
    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
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
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
}
fn inherited(
    common: &mut Common,
    theme: Option<&crate::Theme>,
    size: Option<crate::Size>,
    parent: &Common,
) {
    common.theme = theme.unwrap_or(&parent.theme).clone();
    common.size = size.unwrap_or(parent.size);
    common.unstyled |= parent.unstyled;
}
struct Header {
    row: usize,
    row_end: usize,
    column: usize,
    end: usize,
    id: SemanticId,
    kind: TableCellKind,
}
impl Widget for Table {
    fn build(&mut self, scope: &Scope) -> Result {
        key(&self.key)?;
        self.sections.sort_by_key(|s| match s.kind {
            SectionKind::Head => 0,
            SectionKind::Body => 1,
            SectionKind::Foot => 2,
        });
        let mut row_keys = BTreeSet::new();
        let mut plans = Vec::with_capacity(self.sections.len());
        let mut columns = 0;
        let mut row_count = 0;
        let mut cells = 0;
        for section in &self.sections {
            row_count += section.rows.len();
            if row_count > MAX_TABLE_ROWS {
                return Err(invalid("table exceeds 256 live rows"));
            }
            let mut spans = Vec::with_capacity(section.rows.len());
            for row in &section.rows {
                key(&row.key)?;
                if !row_keys.insert(&row.key) {
                    return Err(invalid("table row keys must be unique"));
                }
                let mut keys = BTreeSet::new();
                cells += row.cells.len();
                if cells > MAX_TABLE_CELLS {
                    return Err(invalid("table exceeds 4096 live cells"));
                }
                for cell in &row.cells {
                    key(&cell.key)?;
                    if !keys.insert(&cell.key) {
                        return Err(invalid("table cell keys must be unique within their row"));
                    }
                }
                spans.push(row.cells.iter().map(|cell| cell.span).collect());
            }
            let plan = model::grid(&spans)?;
            columns = columns.max(plan.columns);
            plans.push(plan);
        }
        let mut headers = Vec::new();
        let mut row_index = 0;
        for (section, plan) in self.sections.iter().zip(&plans) {
            for (row, positions) in section.rows.iter().zip(&plan.cells) {
                for (cell, column) in row.cells.iter().zip(positions) {
                    if cell.kind != TableCellKind::Data {
                        headers.push(Header {
                            row: row_index,
                            row_end: row_index + usize::from(cell.span.rows),
                            column: usize::from(*column),
                            end: usize::from(*column) + usize::from(cell.span.columns),
                            id: id(&self.key, &row.key, &cell.key)?,
                            kind: cell.kind,
                        })
                    }
                }
                row_index += 1;
            }
        }
        // Resolve only the current finite declaration. Validate all header
        // budgets and text before constructing any application content.
        let mut associations = Vec::with_capacity(self.sections.len());
        let mut texts = Vec::with_capacity(self.sections.len());
        row_index = 0;
        for (section, plan) in self.sections.iter().zip(&plans) {
            let mut group_headers = Vec::with_capacity(section.rows.len());
            let mut group_texts = Vec::with_capacity(section.rows.len());
            for (row, positions) in section.rows.iter().zip(&plan.cells) {
                let mut row_headers = Vec::with_capacity(row.cells.len());
                let mut row_texts = Vec::with_capacity(row.cells.len());
                for (cell, column) in row.cells.iter().zip(positions) {
                    let cell_id = id(&self.key, &row.key, &cell.key)?;
                    let related = headers
                        .iter()
                        .filter(|header| {
                            header.id != cell_id
                                && match header.kind {
                                    TableCellKind::ColumnHeader => {
                                        header.row < row_index
                                            && header.column
                                                < usize::from(*column)
                                                    + usize::from(cell.span.columns)
                                            && header.end > usize::from(*column)
                                    }
                                    TableCellKind::RowHeader => {
                                        header.row <= row_index && row_index < header.row_end
                                    }
                                    TableCellKind::Data => false,
                                }
                        })
                        .map(|header| header.id.clone())
                        .collect::<Vec<_>>();
                    if related.len() > glory::render::MAX_CELL_HEADERS {
                        return Err(invalid("table cell has more than 32 header associations"));
                    }
                    row_headers.push(related);
                    row_texts.push(if cell.content.is_none() {
                        Some(composite::validated(scope, &cell.text, |value| {
                            if value.len() > 4096 || value.contains('\0') {
                                Err(invalid("table cell text exceeds 4096 canonical bytes"))
                            } else {
                                Ok(())
                            }
                        })?)
                    } else {
                        None
                    });
                }
                group_headers.push(row_headers);
                group_texts.push(row_texts);
                row_index += 1;
            }
            associations.push(group_headers);
            texts.push(group_texts);
        }
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let caption_id = composite::id("table-caption", &self.key, "caption", "caption")?;
        let caption = self
            .caption
            .as_ref()
            .map(|source| {
                composite::validated(scope, source, |value| {
                    if value.len() > 4096 || value.contains('\0') {
                        Err(invalid("table caption exceeds 4096 canonical bytes"))
                    } else {
                        Ok(())
                    }
                })
            })
            .transpose()?;
        let mut base = Style::new().width(Length::Percent(100.));
        if !self.common.unstyled {
            base.color = Some(self.common.theme.foreground);
            base.background = Some(self.common.theme.surface);
        }
        let (classes, style) = self.common.appearance(scope, "sapid-table", base);
        let mut root_view = glory::ui::view()
            .class(classes)
            .style(style)
            .node_ref(root)
            .semantics(composite::metadata(Semantics {
                document_part: Some(DocumentPart::Table),
                id: Some(composite::id("table-root", &self.key, "table", "root")?),
                table: Some(TableMetadata::Table {
                    rows: Some(row_count as u32),
                    columns: (columns > 0).then_some(columns as u16),
                }),
                ..Default::default()
            }));
        if let Some(name) = self.common.name.take() {
            root_view = root_view.name(name)
        }
        root_view.show_in(scope)?;
        let parent = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
        if let Some(caption) = caption {
            let mut caption_style = self.common.theme.label(self.common.size);
            caption_style.color = Some(self.common.theme.muted);
            caption_style.padding = Some(Edges::all(8.));
            if self.common.unstyled {
                caption_style = Style::new()
            }
            glory::ui::view()
                .class("sapid-table-caption")
                .style(caption_style)
                .semantics(composite::metadata(Semantics {
                    document_part: Some(DocumentPart::TableCaption),
                    id: Some(caption_id),
                    ..Default::default()
                }))
                .fill(glory::ui::text().content(caption))
                .show_in(&parent)?;
        }
        row_index = 0;
        for (section_index, (section, plan)) in self.sections.iter_mut().zip(&plans).enumerate() {
            let group = parent.cage(None);
            let part = match section.kind {
                SectionKind::Head => DocumentPart::TableHead,
                SectionKind::Body => DocumentPart::TableBody,
                SectionKind::Foot => DocumentPart::TableFoot,
            };
            glory::ui::view()
                .class(match section.kind {
                    SectionKind::Head => "sapid-table-head",
                    SectionKind::Body => "sapid-table-body",
                    SectionKind::Foot => "sapid-table-foot",
                })
                .document_part(part)
                .node_ref(group)
                .show_in(&parent)?;
            let group_scope = parent.child_in(group.get().ok_or(glory::ui::Error::Gone)?)?;
            for (local_row, (row, positions)) in
                section.rows.iter_mut().zip(&plan.cells).enumerate()
            {
                inherited(&mut row.common, row.theme.as_ref(), row.size, &self.common);
                let row_ref = row
                    .common
                    .node_ref
                    .unwrap_or_else(|| group_scope.cage(None));
                let mut style = Style::new();
                if self.striped && row_index % 2 == 1 && !row.common.unstyled {
                    style.background = Some(row.common.theme.accent_soft)
                }
                let (classes, style) =
                    row.common
                        .appearance(&group_scope, "sapid-table-row", style);
                glory::ui::view()
                    .class(classes)
                    .style(style)
                    .node_ref(row_ref)
                    .semantics(composite::metadata(Semantics {
                        document_part: Some(DocumentPart::TableRow),
                        id: Some(composite::id("table-row", &self.key, "row", &row.key)?),
                        table: Some(TableMetadata::Row {
                            index: row_index as u32,
                        }),
                        ..Default::default()
                    }))
                    .show_in(&group_scope)?;
                let row_scope =
                    group_scope.child_in(row_ref.get().ok_or(glory::ui::Error::Gone)?)?;
                for (cell_index, (cell, column)) in row.cells.iter_mut().zip(positions).enumerate()
                {
                    inherited(
                        &mut cell.common,
                        cell.theme.as_ref(),
                        cell.size,
                        &row.common,
                    );
                    let mut style = cell.common.theme.label(cell.common.size);
                    style.padding = Some(Edges {
                        top: 8.,
                        bottom: 8.,
                        left: 12.,
                        right: 12.,
                    });
                    style.color = Some(cell.common.theme.foreground);
                    if cell.kind != TableCellKind::Data {
                        style.background = Some(cell.common.theme.accent_soft);
                        style.font_weight = Some(700.);
                    }
                    if self.bordered {
                        style.border_width = Some(Edges::all(1.));
                        style.border_style = Some(BorderStyle::Solid);
                        style.border_color = Some(cell.common.theme.border);
                    }
                    let (classes, style) =
                        cell.common
                            .appearance(&row_scope, "sapid-table-cell", style);
                    let cell_id = id(&self.key, &row.key, &cell.key)?;
                    let related =
                        std::mem::take(&mut associations[section_index][local_row][cell_index]);
                    let part = match cell.kind {
                        TableCellKind::Data => DocumentPart::TableCell,
                        TableCellKind::ColumnHeader => DocumentPart::ColumnHeader,
                        TableCellKind::RowHeader => DocumentPart::RowHeader,
                    };
                    let mut view = glory::ui::view().class(classes).style(style).semantics(
                        composite::metadata(Semantics {
                            document_part: Some(part),
                            id: Some(cell_id),
                            table: Some(TableMetadata::Cell {
                                row: row_index as u32,
                                column: *column,
                                row_span: cell.span.rows,
                                column_span: cell.span.columns,
                                headers: related,
                            }),
                            ..Default::default()
                        }),
                    );
                    if let Some(reference) = cell.common.node_ref {
                        view = view.node_ref(reference)
                    }
                    if let Some(mut factory) = cell.content.take() {
                        view = view.fill(glory::ui::widget(move |scope| factory().show_in(scope)))
                    } else {
                        view = view.fill(glory::ui::text().content(
                            texts[section_index][local_row][cell_index].expect("validated text"),
                        ))
                    }
                    view.show_in(&row_scope)?;
                }
                group_scope.retain(row_scope)?;
                row_index += 1;
            }
            parent.retain(group_scope)?;
        }
        scope.retain(parent)?;
        Ok(())
    }
}
