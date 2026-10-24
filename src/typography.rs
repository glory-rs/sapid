//! Typed paragraphs, headings and explicitly themed rich content.
use glory::render::{EventKind, NodeId, TextKind};
use glory::style::{Color, Display, Edges, Family, Overflow, Style, TextOverflow, WhiteSpace};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, configure, merge, observe};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextVariant {
    #[default]
    Body,
    Lead,
    Muted,
    Small,
    Caption,
}
#[derive(Default)]
struct Metrics {
    font_size: Option<f64>,
    line_height: Option<f64>,
    weight: Option<f64>,
    color: Option<Color>,
    truncate: bool,
}
impl Metrics {
    fn apply(&self, mut style: Style) -> Style {
        if let Some(value) = self.font_size {
            style.font_size = Some(value);
        }
        if let Some(value) = self.line_height {
            style.line_height = Some(value);
        }
        if let Some(value) = self.weight {
            style.font_weight = Some(value);
        }
        if let Some(value) = self.color {
            style.color = Some(value);
        }
        if self.truncate {
            style.display = Some(Display::Block);
            style.overflow_x = Some(Overflow::Hidden);
            style.white_space = Some(WhiteSpace::NoWrap);
            style.text_overflow = Some(TextOverflow::Ellipsis);
        }
        style
    }
}
fn apply_metrics(common: &mut Common, scope: &Scope, metrics: &Metrics) {
    let metrics = metrics.apply(Style::new());
    let overrides = common.style.clone();
    common.style = if overrides.is_reactive() {
        scope
            .bond(move || merge(metrics.clone(), overrides.get()))
            .into()
    } else {
        merge(metrics, overrides.get()).into()
    };
}
macro_rules! text_methods {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.common.theme = value;
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.common.size = value;
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.content = value.into();
            self
        }
        pub fn font_size(mut self, value: f64) -> Self {
            self.metrics.font_size = Some(value);
            self
        }
        pub fn line_height(mut self, value: f64) -> Self {
            self.metrics.line_height = Some(value);
            self
        }
        pub fn weight(mut self, value: f64) -> Self {
            self.metrics.weight = Some(value);
            self
        }
        pub fn color(mut self, value: Color) -> Self {
            self.metrics.color = Some(value);
            self
        }
        /// Single-line ellipsis; supply a finite width/max_width via style.
        pub fn truncate(mut self, value: bool) -> Self {
            self.metrics.truncate = value;
            self
        }
        pub fn selectable(mut self, value: impl Into<Lotus<bool>>) -> Self {
            self.selectable = value.into();
            self
        }
        pub fn on(
            mut self,
            kind: EventKind,
            callback: impl FnMut(glory::render::Event) + 'static,
        ) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(callback));
            self
        }
    };
}
pub struct Text {
    common: Common,
    content: Lotus<String>,
    variant: TextVariant,
    inline: bool,
    metrics: Metrics,
    selectable: Lotus<bool>,
}
pub fn text() -> Text {
    Text {
        common: Common::default(),
        content: "".into(),
        variant: TextVariant::Body,
        inline: false,
        metrics: Metrics::default(),
        selectable: true.into(),
    }
}
impl Text {
    text_methods!();
    pub fn variant(mut self, value: TextVariant) -> Self {
        self.variant = value;
        self
    }
    pub fn inline(mut self, value: bool) -> Self {
        self.inline = value;
        self
    }
}
fn body(common: &Common, variant: TextVariant) -> Style {
    let role = match variant {
        TextVariant::Body | TextVariant::Muted => crate::TypographyRole::Body,
        TextVariant::Lead => crate::TypographyRole::Lead,
        TextVariant::Small => crate::TypographyRole::Small,
        TextVariant::Caption => crate::TypographyRole::Caption,
    };
    let metrics = common.theme.typography(role, common.size);
    Style {
        font_size: Some(metrics.font_size),
        line_height: Some(metrics.line_height),
        font_weight: metrics.weight,
        color: Some(
            if matches!(variant, TextVariant::Muted | TextVariant::Caption) {
                common.theme.muted
            } else {
                common.theme.foreground
            },
        ),
        margin: Some(Edges::all(0.0)),
        ..Style::new().font_family([Family::System])
    }
}
impl Widget for Text {
    fn build(&mut self, scope: &Scope) -> Result {
        apply_metrics(&mut self.common, scope, &self.metrics);
        let style = body(&self.common, self.variant);
        let mut widget = configure!(self.common, glory::ui::text(), scope, "sapid-text", style)
            .kind(if self.inline {
                TextKind::Inline
            } else {
                TextKind::Paragraph
            })
            .content(self.content.clone())
            .selectable(self.selectable.clone());
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            widget = widget.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        widget.show_in(scope)
    }
}
pub struct Title {
    common: Common,
    content: Lotus<String>,
    level: u8,
    metrics: Metrics,
    selectable: Lotus<bool>,
}
pub fn title(level: u8) -> Title {
    Title {
        common: Common::default(),
        content: "".into(),
        level,
        metrics: Metrics::default(),
        selectable: true.into(),
    }
}
impl Title {
    text_methods!();
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
}
impl Widget for Title {
    fn build(&mut self, scope: &Scope) -> Result {
        if !(1..=6).contains(&self.level) {
            return Err(glory::render::Error::InvalidProps("title level must be 1..=6").into());
        }
        let metrics = self.common.theme.typography(
            crate::TypographyRole::heading(self.level).expect("validated heading"),
            self.common.size,
        );
        apply_metrics(&mut self.common, scope, &self.metrics);
        let style = Style {
            font_size: Some(metrics.font_size),
            line_height: Some(metrics.line_height),
            font_weight: metrics.weight,
            color: Some(self.common.theme.foreground),
            margin: Some(Edges::all(0.0)),
            ..Style::new().font_family([Family::System])
        };
        let mut widget =
            configure!(self.common, glory::ui::text(), scope, "sapid-title", style)
                .kind(TextKind::Heading(self.level))
                .content(self.content.clone())
                .selectable(self.selectable.clone());
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            widget = widget.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        widget.show_in(scope)
    }
}
#[derive(Clone)]
pub struct TypographyContext {
    pub theme: crate::Theme,
    pub size: crate::Size,
    pub unstyled: bool,
}
impl TypographyContext {
    pub fn text(&self) -> Text {
        let text = text().theme(self.theme.clone()).size(self.size);
        if self.unstyled { text.unstyled() } else { text }
    }
    pub fn title(&self, level: u8) -> Title {
        let title = title(level).theme(self.theme.clone()).size(self.size);
        if self.unstyled {
            title.unstyled()
        } else {
            title
        }
    }
    pub fn anchor(&self) -> crate::Anchor {
        let anchor = crate::anchor().theme(self.theme.clone()).size(self.size);
        if self.unstyled {
            anchor.unstyled()
        } else {
            anchor
        }
    }
}
type Content = Box<dyn FnMut(TypographyContext) -> glory::ui::Children>;
pub struct Typography {
    common: Common,
    children: Option<glory::ui::Children>,
    content: Option<Content>,
}
pub fn typography() -> Typography {
    Typography {
        common: Common::default(),
        children: None,
        content: None,
    }
}
impl Typography {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn size(mut self, value: crate::Size) -> Self {
        self.common.size = value;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn fill(mut self, value: impl IntoChildren) -> Self {
        self.children = Some(value.into_children());
        self
    }
    /// Explicitly share theme and size with rich child builders.
    pub fn content<C: IntoChildren>(
        mut self,
        mut factory: impl FnMut(TypographyContext) -> C + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| factory(context).into_children()));
        self
    }
}
impl Widget for Typography {
    fn build(&mut self, scope: &Scope) -> Result {
        let context = TypographyContext {
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
        };
        let mut style = body(&self.common, TextVariant::Body);
        style.display = Some(Display::Flex);
        style.flex_direction = Some(glory::style::FlexDirection::Column);
        style.gap = Some(16.0);
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-typography",
            style
        );
        if let Some(mut factory) = self.content.take() {
            root = root.fill(factory(context));
        }
        if let Some(children) = self.children.take() {
            root = root.fill(children);
        }
        root.show_in(scope)
    }
}
