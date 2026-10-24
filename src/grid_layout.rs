//! Finite passive CSS grid layout, distinct from an interactive data grid.
use crate::appearance::{Common, merge};
use glory::render::{NodeId, SemanticId, Semantics};
use glory::style::{
    AspectRatio as Ratio, Display, GridPlacement, GridTrack, GridTracks, Length, Style,
};
use glory::ui::{Children, IntoChildren, Result, View, Widget};
use glory::{Cage, Lotus, Scope};
use std::collections::HashSet;
pub const MAX_GRID_ITEMS: usize = 256;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
macro_rules! appearance {
    () => {
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
            self.common.class = class.into();
            self
        }
        pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
            self.common.style = style.into();
            self
        }
        pub fn name(mut self, name: impl Into<Lotus<String>>) -> Self {
            self.common.name = Some(name.into());
            self
        }
        pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(reference);
            self
        }
    };
}
fn configured(common: &mut Common, scope: &Scope, class: &str, base: Lotus<Style>) -> View {
    let (class, custom) = common.appearance(scope, class, Style::new());
    let style: Lotus<Style> = if base.is_reactive() || custom.is_reactive() {
        scope.bond(move || merge(base.get(), custom.get())).into()
    } else {
        merge(base.get(), custom.get()).into()
    };
    let mut root = glory::ui::view().class(class).style(style);
    if let Some(name) = common.name.take() {
        root = root.name(name);
    }
    if let Some(reference) = common.node_ref {
        root = root.node_ref(reference);
    }
    root
}
pub struct Grid {
    common: Common,
    namespace: String,
    columns: Vec<GridTrack>,
    rows: GridTracks,
    gap: f64,
    items: Vec<GridItem>,
    simple: bool,
}
pub fn grid(namespace: impl Into<String>) -> Grid {
    Grid {
        common: Common::default(),
        namespace: namespace.into(),
        columns: vec![GridTrack::Fr(1.0); 12],
        rows: GridTracks::None,
        gap: 12.0,
        items: Vec::new(),
        simple: false,
    }
}
pub struct SimpleGrid {
    grid: Grid,
}
pub fn simple_grid(namespace: impl Into<String>) -> SimpleGrid {
    let mut grid = grid(namespace);
    grid.columns = vec![GridTrack::Fr(1.0); 3];
    grid.simple = true;
    SimpleGrid { grid }
}
impl Grid {
    appearance!();
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.common.size = size;
        self
    }
    /// Equal fractional tracks. Invalid counts are rejected before node creation.
    pub fn columns(mut self, count: u16) -> Self {
        self.columns = vec![GridTrack::Fr(1.0); usize::from(count.min(65))];
        self
    }
    pub fn column_tracks(mut self, tracks: impl IntoIterator<Item = GridTrack>) -> Self {
        self.columns = tracks.into_iter().take(65).collect();
        self
    }
    pub fn row_tracks(mut self, tracks: impl IntoIterator<Item = GridTrack>) -> Self {
        self.rows = GridTracks::Tracks(tracks.into_iter().take(65).collect());
        self
    }
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }
    pub fn item(mut self, item: GridItem) -> Self {
        crate::document::extend_bounded(&mut self.items, std::iter::once(item), MAX_GRID_ITEMS);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = GridItem>) -> Self {
        crate::document::extend_bounded(&mut self.items, items, MAX_GRID_ITEMS);
        self
    }
}
impl SimpleGrid {
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.grid = self.grid.theme(theme);
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.grid = self.grid.size(size);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.grid = self.grid.unstyled();
        self
    }
    pub fn class(mut self, class: impl Into<Lotus<String>>) -> Self {
        self.grid = self.grid.class(class);
        self
    }
    pub fn style(mut self, style: impl Into<Lotus<Style>>) -> Self {
        self.grid = self.grid.style(style);
        self
    }
    pub fn name(mut self, name: impl Into<Lotus<String>>) -> Self {
        self.grid = self.grid.name(name);
        self
    }
    pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.grid = self.grid.node_ref(reference);
        self
    }
    pub fn columns(mut self, columns: u16) -> Self {
        self.grid = self.grid.columns(columns);
        self
    }
    pub fn gap(mut self, gap: f64) -> Self {
        self.grid = self.grid.gap(gap);
        self
    }
    pub fn item(mut self, item: GridItem) -> Self {
        self.grid = self.grid.item(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = GridItem>) -> Self {
        self.grid = self.grid.items(items);
        self
    }
}
pub struct GridItem {
    common: Common,
    theme: Option<crate::Theme>,
    size: Option<crate::Size>,
    key: String,
    column: Option<u16>,
    row: Option<u16>,
    span: u16,
    row_span: u16,
    children: Children,
}
pub fn grid_item(key: impl Into<String>) -> GridItem {
    GridItem {
        common: Common::default(),
        theme: None,
        size: None,
        key: key.into(),
        column: None,
        row: None,
        span: 1,
        row_span: 1,
        children: ().into_children(),
    }
}
impl GridItem {
    appearance!();
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.theme = Some(theme);
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.size = Some(size);
        self
    }
    pub fn column(mut self, start: u16) -> Self {
        self.column = Some(start);
        self
    }
    pub fn row(mut self, start: u16) -> Self {
        self.row = Some(start);
        self
    }
    pub fn span(mut self, span: u16) -> Self {
        self.span = span;
        self
    }
    pub fn row_span(mut self, span: u16) -> Self {
        self.row_span = span;
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
#[derive(Clone, Copy)]
struct Place {
    column: u16,
    row: u16,
    span: u16,
    row_span: u16,
}
fn placements(columns: usize, items: &[GridItem]) -> Result<Vec<Place>> {
    let mut occupied = [0u64; 64];
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let span = usize::from(item.span);
        let height = usize::from(item.row_span);
        if span == 0 || span > columns || height == 0 || height > 64 {
            return Err(invalid(
                "grid spans must fit the configured columns and 64 rows",
            ));
        }
        let (first_column, last_column) = if let Some(column) = item.column {
            let column = usize::from(column);
            if column == 0 || column + span > columns + 1 {
                return Err(invalid("grid column placement exceeds configured tracks"));
            }
            (column - 1, column - 1)
        } else {
            (0, columns - span)
        };
        let (first_row, last_row) = if let Some(row) = item.row {
            let row = usize::from(row);
            if row == 0 || row + height > 65 {
                return Err(invalid("grid row placement exceeds 64 rows"));
            }
            (row - 1, row - 1)
        } else {
            (0, 64 - height)
        };
        let bits = if span == 64 {
            u64::MAX
        } else {
            (1u64 << span) - 1
        };
        let mut found = None;
        'search: for row in first_row..=last_row {
            for column in first_column..=last_column {
                let mask = bits << column;
                if occupied[row..row + height]
                    .iter()
                    .all(|occupied| occupied & mask == 0)
                {
                    for occupied in &mut occupied[row..row + height] {
                        *occupied |= mask;
                    }
                    found = Some(Place {
                        column: (column + 1) as u16,
                        row: (row + 1) as u16,
                        span: item.span,
                        row_span: item.row_span,
                    });
                    break 'search;
                }
            }
        }
        result.push(found.ok_or_else(|| {
            invalid("grid items overlap or exceed the finite 64 by 64 placement budget")
        })?);
    }
    Ok(result)
}
impl Widget for Grid {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.len() > MAX_GRID_ITEMS {
            return Err(invalid("grid accepts at most 256 items"));
        }
        let mut base = Style::new()
            .display(Display::Grid)
            .min_width(Length::Px(0.0))
            .gap(self.gap)
            .grid_columns(GridTracks::Tracks(self.columns.clone()))
            .grid_rows(self.rows.clone());
        base.validate()
            .map_err(glory::render::Error::InvalidProps)?;
        let kind = if self.simple { "simple-grid" } else { "grid" };
        let root_id = crate::composite::id(kind, &self.namespace, "root", "root")?;
        let mut keys = HashSet::with_capacity(self.items.len());
        let mut ids = Vec::with_capacity(self.items.len());
        for item in &self.items {
            ids.push(crate::composite::id(
                kind,
                &self.namespace,
                "item",
                &item.key,
            )?);
            if !keys.insert(item.key.as_str()) {
                return Err(invalid("duplicate grid item key"));
            }
            if self.simple
                && (item.span != 1
                    || item.row_span != 1
                    || item.column.is_some()
                    || item.row.is_some())
            {
                return Err(invalid(
                    "simple grid items occupy one automatic cell; use grid for spanning or explicit placement",
                ));
            }
        }
        let places = placements(self.columns.len(), &self.items)?;
        if !self.common.unstyled {
            base.color = Some(self.common.theme.foreground);
        }
        let mut children = Vec::with_capacity(self.items.len());
        for ((mut item, id), place) in self.items.drain(..).zip(ids).zip(places) {
            item.common.theme = item
                .theme
                .take()
                .unwrap_or_else(|| self.common.theme.clone());
            item.common.size = item.size.unwrap_or(self.common.size);
            item.common.unstyled |= self.common.unstyled;
            children.push(RenderedItem { item, id, place });
        }
        configured(
            &mut self.common,
            scope,
            if self.simple {
                "sapid-grid sapid-simple-grid"
            } else {
                "sapid-grid"
            },
            base.into(),
        )
        .semantics(Some(Box::new(Semantics {
            id: Some(root_id),
            ..Default::default()
        })))
        .fill(children)
        .show_in(scope)
    }
}
impl Widget for SimpleGrid {
    fn build(&mut self, scope: &Scope) -> Result {
        self.grid.build(scope)
    }
}
struct RenderedItem {
    item: GridItem,
    id: SemanticId,
    place: Place,
}
impl Widget for RenderedItem {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut base = Style::new()
            .min_width(Length::Px(0.0))
            .grid_column(GridPlacement::At {
                start: self.place.column,
                span: self.place.span,
            })
            .grid_row(GridPlacement::At {
                start: self.place.row,
                span: self.place.row_span,
            });
        if !self.item.common.unstyled {
            base.color = Some(self.item.common.theme.foreground);
        }
        configured(&mut self.item.common, scope, "sapid-grid-item", base.into())
            .semantics(Some(Box::new(Semantics {
                id: Some(self.id.clone()),
                ..Default::default()
            })))
            .fill(std::mem::replace(
                &mut self.item.children,
                ().into_children(),
            ))
            .show_in(scope)
    }
}
pub struct AspectRatio {
    common: Common,
    ratio: Lotus<f64>,
    width: Length,
    height: Option<Length>,
    children: Children,
}
pub fn aspect_ratio(ratio: impl Into<Lotus<f64>>) -> AspectRatio {
    AspectRatio {
        common: Common::default(),
        ratio: ratio.into(),
        width: Length::Percent(100.0),
        height: None,
        children: ().into_children(),
    }
}
impl AspectRatio {
    appearance!();
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn size(mut self, size: crate::Size) -> Self {
        self.common.size = size;
        self
    }
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for AspectRatio {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut fixed = Style::new().width(self.width).min_width(Length::Px(0.0));
        fixed.height = self.height;
        if !self.common.unstyled {
            fixed.color = Some(self.common.theme.foreground);
        }
        let initial = self
            .ratio
            .try_with(|value| {
                let mut style = fixed.clone();
                style.aspect_ratio = Some(Ratio::Ratio(*value));
                style.validate().map(|()| *value)
            })?
            .map_err(glory::render::Error::InvalidProps)?;
        let ratio = scope.cage(initial);
        if self.ratio.is_reactive() {
            let source = self.ratio.clone();
            let weak = scope.downgrade();
            scope.effect(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let value = source
                        .try_with(|value| {
                            Style::new()
                                .aspect_ratio(Ratio::Ratio(*value))
                                .validate()
                                .map(|()| *value)
                        })?
                        .map_err(glory::render::Error::InvalidProps)?;
                    if ratio.try_with_untracked(|accepted| *accepted != value)? {
                        ratio.try_revise(|accepted| *accepted = value)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        }
        let value = move || {
            let mut style = fixed.clone();
            style.aspect_ratio = Some(Ratio::Ratio(ratio.get()));
            style
        };
        let base = if self.ratio.is_reactive() {
            scope.bond(value).into()
        } else {
            value().into()
        };
        configured(&mut self.common, scope, "sapid-aspect-ratio", base)
            .fill(std::mem::replace(&mut self.children, ().into_children()))
            .show_in(scope)
    }
}
