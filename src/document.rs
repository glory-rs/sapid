//! Intrinsic document text and structure; no HTML/backend escape hatches.
use glory::render::{Descriptor, DocumentPart, NodeId, Props, TextKind};
use glory::style::{BorderStyle, Display, Edges, Family, FlexDirection, Style};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::appearance::{Common, configure};

pub const MAX_DOCUMENT_ITEMS: usize = 256;
pub(crate) fn extend_bounded<T>(
    values: &mut Vec<T>,
    incoming: impl IntoIterator<Item = T>,
    limit: usize,
) {
    let remaining = (limit + 1).saturating_sub(values.len());
    values.extend(incoming.into_iter().take(remaining));
}

macro_rules! document_methods {
    ($($theme_override:ident, $size_override:ident)?) => {
        pub fn theme(mut self, theme: crate::Theme) -> Self {
            self.common.theme = theme;
            $(self.$theme_override=Some(self.common.theme.clone());)?
            self
        }
        pub fn size(mut self, size: crate::Size) -> Self {
            self.common.size = size;
            $(self.$size_override=Some(size);)?
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, class: impl Into<Vary<String>>) -> Self {
            self.common.class = class.into();
            self
        }
        pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
            self.common.style = style.into();
            self
        }
        pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(reference);
            self
        }
    };
}
fn take(children: &mut Children) -> Children {
    std::mem::replace(children, ().into_children())
}
fn column(gap: f64) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
}
fn inline_style(common: &Common, kind: TextKind) -> Style {
    let mut style = common.theme.label(common.size);
    style.font_weight = Some(400.0);
    match kind {
        TextKind::Code => {
            style.font_family = Some(vec![Family::Mono]);
            style.background = Some(common.theme.accent_soft);
            style.color = Some(common.theme.accent_text);
            style.padding = Some(Edges {
                top: 1.0,
                bottom: 1.0,
                left: 4.0,
                right: 4.0,
            });
            style.radius = Some(common.theme.radius);
        }
        TextKind::Kbd => {
            style.font_family = Some(vec![Family::Mono]);
            style.background = Some(common.theme.surface);
            style.padding = Some(Edges {
                top: 1.0,
                bottom: 1.0,
                left: 5.0,
                right: 5.0,
            });
            style.border_width = Some(Edges {
                top: 1.0,
                bottom: 2.0,
                left: 1.0,
                right: 1.0,
            });
            style.border_style = Some(BorderStyle::Solid);
            style.border_color = Some(common.theme.border);
            style.radius = Some(common.theme.radius);
        }
        TextKind::Mark => {
            style.background = Some(common.theme.accent_soft);
            style.color = Some(common.theme.accent_text);
        }
        _ => {}
    }
    style
}
macro_rules! inline_text {
    ($name:ident,$factory:ident,$kind:expr,$class:literal) => {
        pub struct $name {
            common: Common,
            content: Vary<String>,
        }
        pub fn $factory() -> $name {
            $name {
                common: Common::default(),
                content: "".into(),
            }
        }
        impl $name {
            document_methods!();
            pub fn content(mut self, content: impl Into<Vary<String>>) -> Self {
                self.content = content.into();
                self
            }
        }
        impl Widget for $name {
            fn build(&mut self, scope: &Scope) -> Result {
                let base = inline_style(&self.common, $kind);
                configure!(self.common, glory::ui::text(), scope, $class, base)
                    .kind($kind)
                    .content(self.content.clone())
                    .show_in(scope)
            }
        }
    };
}
inline_text!(Code, code, TextKind::Code, "sapid-code");
inline_text!(Mark, mark, TextKind::Mark, "sapid-mark");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KbdPlatform {
    #[default]
    Windows,
    Mac,
    Linux,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KbdKey {
    Control,
    Alt,
    Shift,
    Meta,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
}
impl KbdKey {
    fn label(self, platform: KbdPlatform) -> &'static str {
        match (self, platform) {
            (Self::Control, KbdPlatform::Mac) => "Control",
            (Self::Control, _) => "Ctrl",
            (Self::Alt, KbdPlatform::Mac) => "Option",
            (Self::Alt, _) => "Alt",
            (Self::Meta, KbdPlatform::Mac) => "Command",
            (Self::Meta, KbdPlatform::Windows) => "Windows",
            (Self::Meta, KbdPlatform::Linux) => "Super",
            (Self::Enter, KbdPlatform::Mac) => "Return",
            (Self::Enter, _) => "Enter",
            (Self::Shift, _) => "Shift",
            (Self::Escape, _) => "Escape",
            (Self::Backspace, _) => "Backspace",
            (Self::Tab, _) => "Tab",
            (Self::Space, _) => "Space",
        }
    }
}
/// A keyboard hint. Platform selection is explicit; no shortcut is registered.
pub struct Kbd {
    common: Common,
    content: Vary<String>,
    key: Option<KbdKey>,
    platform: KbdPlatform,
}
pub fn kbd() -> Kbd {
    Kbd {
        common: Common::default(),
        content: "".into(),
        key: None,
        platform: KbdPlatform::Windows,
    }
}
impl Kbd {
    document_methods!();
    pub fn content(mut self, content: impl Into<Vary<String>>) -> Self {
        self.content = content.into();
        self.key = None;
        self
    }
    pub fn key(mut self, key: KbdKey) -> Self {
        self.key = Some(key);
        self
    }
    pub fn platform(mut self, platform: KbdPlatform) -> Self {
        self.platform = platform;
        self
    }
}
impl Widget for Kbd {
    fn build(&mut self, scope: &Scope) -> Result {
        let content = self.key.map_or_else(
            || self.content.clone(),
            |key| key.label(self.platform).into(),
        );
        let base = inline_style(&self.common, TextKind::Kbd);
        configure!(self.common, glory::ui::text(), scope, "sapid-kbd", base)
            .kind(TextKind::Kbd)
            .content(content)
            .show_in(scope)
    }
}

/// A finite textual list, separate from Glory's virtual/windowed `List`.
pub struct TextList {
    common: Common,
    ordered: bool,
    items: Vec<TextListItem>,
}
pub fn text_list() -> TextList {
    TextList {
        common: Common::default(),
        ordered: false,
        items: Vec::new(),
    }
}
impl TextList {
    document_methods!();
    pub fn ordered(mut self, ordered: bool) -> Self {
        self.ordered = ordered;
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn item(mut self, item: TextListItem) -> Self {
        extend_bounded(&mut self.items, std::iter::once(item), MAX_DOCUMENT_ITEMS);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = TextListItem>) -> Self {
        extend_bounded(&mut self.items, items, MAX_DOCUMENT_ITEMS);
        self
    }
}
impl Widget for TextList {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.len() > MAX_DOCUMENT_ITEMS {
            return Err(glory::render::Error::InvalidProps(
                "text list accepts at most 256 items",
            )
            .into());
        }
        let mut style = self.common.theme.label(self.common.size);
        style.font_weight = Some(400.0);
        style.padding = Some(Edges {
            top: 0.0,
            bottom: 0.0,
            left: 24.0,
            right: 0.0,
        });
        style.margin = Some(Edges::all(0.0));
        let part = if self.ordered {
            DocumentPart::OrderedList
        } else {
            DocumentPart::UnorderedList
        };
        for item in &mut self.items {
            item.common.theme = item
                .theme_override
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.common.theme.clone());
            item.common.size = item.size_override.unwrap_or(self.common.size);
            item.common.unstyled |= self.common.unstyled;
        }
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-text-list",
            style
        )
        .document_part(part)
        .fill(std::mem::take(&mut self.items))
        .show_in(scope)
    }
}
pub struct TextListItem {
    common: Common,
    theme_override: Option<crate::Theme>,
    size_override: Option<crate::Size>,
    children: Children,
}
pub fn text_list_item() -> TextListItem {
    TextListItem {
        common: Common::default(),
        theme_override: None,
        size_override: None,
        children: ().into_children(),
    }
}
impl TextListItem {
    document_methods!(theme_override, size_override);
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for TextListItem {
    fn build(&mut self, scope: &Scope) -> Result {
        let base = if self.theme_override.is_some() || self.size_override.is_some() {
            let mut style = self.common.theme.label(self.common.size);
            style.font_weight = Some(400.0);
            style
        } else {
            Style::new()
        };
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-text-list-item",
            base
        )
        .document_part(DocumentPart::ListItem)
        .fill(take(&mut self.children))
        .show_in(scope)
    }
}

/// A real quotation with independent source content and optional attribution.
pub struct Blockquote {
    common: Common,
    children: Children,
    citation: Children,
}
pub fn blockquote() -> Blockquote {
    Blockquote {
        common: Common::default(),
        children: ().into_children(),
        citation: ().into_children(),
    }
}
impl Blockquote {
    document_methods!();
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
    pub fn citation(mut self, children: impl IntoChildren) -> Self {
        self.citation = children.into_children();
        self
    }
}
impl Widget for Blockquote {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut base = column(8.0)
            .padding(Edges {
                top: 12.0,
                bottom: 12.0,
                left: 16.0,
                right: 12.0,
            })
            .border_width(Edges {
                top: 0.0,
                bottom: 0.0,
                left: 3.0,
                right: 0.0,
            })
            .border_style(BorderStyle::Solid)
            .border_color(self.common.theme.accent);
        base.color = Some(self.common.theme.foreground);
        base.background = Some(self.common.theme.accent_soft);
        base.margin = Some(Edges::all(0.0));
        let mut citation = glory::ui::view()
            .class("sapid-blockquote-citation")
            .fill(take(&mut self.citation));
        if !self.common.unstyled {
            citation = citation.style(self.common.theme.description(self.common.size));
        }
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-blockquote",
            base
        )
        .document_part(DocumentPart::Blockquote)
        .fill(take(&mut self.children))
        .fill(citation)
        .show_in(scope)
    }
}

/// Description data as term/definition pairs. No selection model is implied.
pub struct DataList {
    common: Common,
    items: Vec<DataListItem>,
}
pub fn data_list() -> DataList {
    DataList {
        common: Common::default(),
        items: Vec::new(),
    }
}
impl DataList {
    document_methods!();
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn item(mut self, item: DataListItem) -> Self {
        extend_bounded(&mut self.items, std::iter::once(item), MAX_DOCUMENT_ITEMS);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = DataListItem>) -> Self {
        extend_bounded(&mut self.items, items, MAX_DOCUMENT_ITEMS);
        self
    }
}
impl Widget for DataList {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.len() > MAX_DOCUMENT_ITEMS {
            return Err(glory::render::Error::InvalidProps(
                "data list accepts at most 256 items",
            )
            .into());
        }
        let mut base = Style::new();
        base.margin = Some(Edges::all(0.0));
        for item in &mut self.items {
            item.common.theme = item
                .theme_override
                .take()
                .unwrap_or_else(|| self.common.theme.clone());
            item.common.size = item.size_override.unwrap_or(self.common.size);
            item.common.unstyled |= self.common.unstyled;
        }
        configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-data-list",
            base
        )
        .document_part(DocumentPart::DescriptionList)
        .fill(std::mem::take(&mut self.items))
        .show_in(scope)
    }
}
pub struct DataListItem {
    common: Common,
    theme_override: Option<crate::Theme>,
    size_override: Option<crate::Size>,
    term: Vary<String>,
    children: Children,
}
pub fn data_list_item() -> DataListItem {
    DataListItem {
        common: Common::default(),
        theme_override: None,
        size_override: None,
        term: "".into(),
        children: ().into_children(),
    }
}
impl DataListItem {
    document_methods!(theme_override, size_override);
    pub fn term(mut self, term: impl Into<Vary<String>>) -> Self {
        self.term = term.into();
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
}
impl Widget for DataListItem {
    fn build(&mut self, scope: &Scope) -> Result {
        // The pair is boxless so the nearest boxed parent remains DescriptionList.
        let group = scope.create(Descriptor::new(Props::Group))?;
        let content = scope.child_in(group)?;
        let (classes, style) = self.common.appearance(
            scope,
            "sapid-data-list-term",
            self.common.theme.label(self.common.size),
        );
        glory::ui::view()
            .class(classes)
            .style(style)
            .document_part(DocumentPart::Term)
            .fill(glory::ui::text().content(self.term.clone()))
            .show_in(&content)?;
        let mut base = self.common.theme.label(self.common.size);
        base.font_weight = Some(400.0);
        base.margin = Some(Edges {
            top: 0.0,
            bottom: 8.0,
            left: 16.0,
            right: 0.0,
        });
        let (classes, style) =
            self.common
                .appearance(scope, "sapid-data-list-definition", base);
        glory::ui::view()
            .class(classes)
            .style(style)
            .document_part(DocumentPart::Definition)
            .fill(take(&mut self.children))
            .show_in(&content)?;
        scope.retain(content)?;
        publish_reference(scope, self.common.node_ref, group)
    }
}
fn publish_reference(
    scope: &Scope,
    reference: Option<Cage<Option<NodeId>>>,
    node: NodeId,
) -> Result {
    if let Some(reference) = reference {
        reference.try_revise(|current| *current = Some(node))?;
        scope.on_cleanup(move || {
            let _ = reference.try_revise(|current| {
                if *current == Some(node) {
                    *current = None;
                }
            });
        })?;
    }
    Ok(())
}

/// Mount-time exact substring highlighting, with original canonical text and
/// order preserved. Marked ranges are bounded; no language parser or case-fold.
pub struct Highlight {
    common: Common,
    content: String,
    queries: Vec<String>,
}
pub fn highlight() -> Highlight {
    Highlight {
        common: Common::default(),
        content: String::new(),
        queries: Vec::new(),
    }
}
impl Highlight {
    document_methods!();
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
    pub fn query(mut self, query: impl Into<String>) -> Self {
        if self.queries.len() <= 32 {
            self.queries.push(query.into());
        }
        self
    }
    pub fn queries(mut self, queries: impl IntoIterator<Item = String>) -> Self {
        extend_bounded(&mut self.queries, queries, 32);
        self
    }
}
impl Widget for Highlight {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.content.len() > 65536
            || self.queries.len() > 32
            || self.queries.iter().any(|q| q.is_empty() || q.len() > 256)
        {
            return Err(glory::render::Error::InvalidProps(
                "highlight requires <=64KiB text and <=32 nonempty <=256-byte queries",
            )
            .into());
        }
        let content = glory::render::normalize_text(&self.content, true);
        let queries = self
            .queries
            .iter()
            .map(|q| glory::render::normalize_text(q, true))
            .collect::<Vec<_>>();
        let ranges = highlight_ranges(&content, &queries)?;
        let root = scope.create(Descriptor::new(Props::Group))?;
        let children = scope.child_in(root)?;
        for (start, end, marked) in ranges {
            let kind = if marked {
                TextKind::Mark
            } else {
                TextKind::Inline
            };
            let class = if marked {
                "sapid-highlight sapid-highlight-match"
            } else {
                "sapid-highlight sapid-highlight-text"
            };
            let base = if marked {
                inline_style(&self.common, kind)
            } else {
                Style::new()
            };
            let (classes, style) = self.common.appearance(scope, class, base);
            glory::ui::text()
                .class(classes)
                .style(style)
                .kind(kind)
                .content(content[start..end].to_string())
                .show_in(&children)?;
        }
        scope.retain(children)?;
        publish_reference(scope, self.common.node_ref, root)
    }
}
fn highlight_ranges(
    content: &str,
    queries: &[std::borrow::Cow<'_, str>],
) -> Result<Vec<(usize, usize, bool)>> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    let mut push = |start, end, marked| -> Result {
        if start == end {
            return Ok(());
        }
        if let Some((_, previous_end, previous_marked)) = ranges.last_mut()
            && *previous_end == start
            && *previous_marked == marked
        {
            *previous_end = end;
            return Ok(());
        }
        if ranges.len() == 1024 {
            return Err(glory::render::Error::InvalidProps(
                "highlight exceeds 1024 text ranges",
            )
            .into());
        }
        ranges.push((start, end, marked));
        Ok(())
    };
    while cursor < content.len() {
        let next = queries
            .iter()
            .filter_map(|query| {
                content[cursor..]
                    .find(query.as_ref())
                    .map(|index| (cursor + index, query.len()))
            })
            .min_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
        let Some((start, len)) = next else {
            push(cursor, content.len(), false)?;
            break;
        };
        push(cursor, start, false)?;
        push(start, start + len, true)?;
        cursor = start + len;
    }
    if content.is_empty() {
        ranges.push((0, 0, false));
    }
    Ok(ranges)
}

/// A passive content row with a separate media, content and action region.
/// Use inside TextListItem for list membership; it never implies selection.
pub struct Item {
    common: Common,
    title: Option<Vary<String>>,
    description: Option<Vary<String>>,
    media: Children,
    children: Children,
    actions: Children,
}
pub fn item() -> Item {
    Item {
        common: Common::default(),
        title: None,
        description: None,
        media: ().into_children(),
        children: ().into_children(),
        actions: ().into_children(),
    }
}
impl Item {
    document_methods!();
    pub fn title(mut self, title: impl Into<Vary<String>>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn description(mut self, description: impl Into<Vary<String>>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn media(mut self, children: impl IntoChildren) -> Self {
        self.media = children.into_children();
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }
    pub fn actions(mut self, children: impl IntoChildren) -> Self {
        self.actions = children.into_children();
        self
    }
}
impl Widget for Item {
    fn build(&mut self, scope: &Scope) -> Result {
        let base = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(12.0)
            .padding(Edges::all(12.0));
        let mut body = glory::ui::view().class("sapid-item-content");
        if !self.common.unstyled {
            body = body.style(Style {
                grow: Some(1.0),
                ..column(4.0)
            });
        }
        if let Some(title) = self.title.take() {
            let mut text = glory::ui::text().class("sapid-item-title").content(title);
            if !self.common.unstyled {
                text = text.style(self.common.theme.label(self.common.size));
            }
            body = body.fill(text);
        }
        if let Some(description) = self.description.take() {
            let mut text = glory::ui::text()
                .class("sapid-item-description")
                .kind(TextKind::Paragraph)
                .content(description);
            if !self.common.unstyled {
                text = text.style(self.common.theme.description(self.common.size));
            }
            body = body.fill(text);
        }
        body = body.fill(take(&mut self.children));
        configure!(self.common, glory::ui::view(), scope, "sapid-item", base)
            .fill(
                glory::ui::view()
                    .class("sapid-item-media")
                    .fill(take(&mut self.media)),
            )
            .fill(body)
            .fill(
                glory::ui::view()
                    .class("sapid-item-actions")
                    .fill(take(&mut self.actions)),
            )
            .show_in(scope)
    }
}
