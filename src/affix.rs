//! Host-positioned viewport, container and sticky content without measurement loops.
use crate::appearance::{Common, configure, merge, observe};
use crate::composite;
use glory::core::InstanceId;
use glory::render::{Event, EventKind, FocusMode, NodeId, Semantics};
use glory::style::{Length, Overflow, Position, Style};
use glory::ui::{IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AffixHorizontal {
    Left(f64),
    Right(f64),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AffixVertical {
    Top(f64),
    Bottom(f64),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffixOffset {
    pub horizontal: AffixHorizontal,
    pub vertical: AffixVertical,
}
impl Default for AffixOffset {
    fn default() -> Self {
        Self {
            horizontal: AffixHorizontal::Right(16.0),
            vertical: AffixVertical::Bottom(16.0),
        }
    }
}
impl AffixOffset {
    pub fn validate(self) -> Result {
        let horizontal = match self.horizontal {
            AffixHorizontal::Left(v) | AffixHorizontal::Right(v) => v,
        };
        let vertical = match self.vertical {
            AffixVertical::Top(v) | AffixVertical::Bottom(v) => v,
        };
        if [horizontal, vertical]
            .into_iter()
            .all(|v| v.is_finite() && v.abs() <= 1_000_000.0)
        {
            Ok(())
        } else {
            Err(glory::render::Error::InvalidProps(
                "affix offsets must be finite and within +/-1000000 pixels",
            )
            .into())
        }
    }
    fn apply(self, mut style: Style, position: Position) -> Style {
        style.position = Some(position);
        style.top = None;
        style.right = None;
        style.bottom = None;
        style.left = None;
        match self.horizontal {
            AffixHorizontal::Left(v) => style.left = Some(Length::Px(v)),
            AffixHorizontal::Right(v) => style.right = Some(Length::Px(v)),
        }
        match self.vertical {
            AffixVertical::Top(v) => style.top = Some(Length::Px(v)),
            AffixVertical::Bottom(v) => style.bottom = Some(Length::Px(v)),
        }
        style
    }
}
/// A live, explicitly constructed relative positioning ancestor.
#[derive(Clone, Copy)]
pub struct AffixContext {
    owner: InstanceId,
    node: NodeId,
}
impl AffixContext {
    pub fn node(&self) -> NodeId {
        self.node
    }
    fn validate(self, scope: &Scope) -> Result {
        if !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        if !scope.is_within(self.node)? {
            return Err(glory::render::Error::InvalidProps(
                "affix context must be a construction ancestor",
            )
            .into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum Target {
    Viewport,
    Container(AffixContext),
    Sticky(AffixContext),
}
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    offset: AffixOffset,
    visible: bool,
}

macro_rules! appearance_methods {
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
        pub fn focus(mut self, value: FocusMode) -> Self {
            self.common.focus = value;
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(handler));
            self
        }
    };
}

pub struct Affix {
    key: String,
    common: Common,
    target: Target,
    offset: Vary<AffixOffset>,
    explicit_offset: bool,
    visible: Vary<bool>,
    children: Option<glory::ui::Children>,
}
pub fn affix(key: impl Into<String>) -> Affix {
    Affix {
        key: key.into(),
        common: Common::default(),
        target: Target::Viewport,
        offset: AffixOffset::default().into(),
        explicit_offset: false,
        visible: true.into(),
        children: None,
    }
}
impl Affix {
    appearance_methods!();
    pub fn offset(mut self, value: impl Into<Vary<AffixOffset>>) -> Self {
        self.offset = value.into();
        self.explicit_offset = true;
        self
    }
    pub fn visible(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.visible = value.into();
        self
    }
    pub fn within(mut self, context: AffixContext) -> Self {
        self.target = Target::Container(context);
        self
    }
    /// Sticky participates in normal flow and is constrained by this scroll ancestor.
    /// Native hosts explicitly reject Sticky until their layout engine supports it.
    pub fn sticky(mut self, context: AffixContext) -> Self {
        self.target = Target::Sticky(context);
        if !self.explicit_offset {
            self.offset = AffixOffset {
                horizontal: AffixHorizontal::Left(0.0),
                vertical: AffixVertical::Top(0.0),
            }
            .into();
        }
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = Some(children.into_children());
        self
    }
}
impl Widget for Affix {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("affix", &self.key, "root", "root")?;
        let position = match self.target {
            Target::Viewport => Position::Fixed,
            Target::Container(context) => {
                context.validate(scope)?;
                Position::Absolute
            }
            Target::Sticky(context) => {
                context.validate(scope)?;
                Position::Sticky
            }
        };
        let offset = self.offset.clone();
        let visible = self.visible.clone();
        let raw = scope.bond(move || Snapshot {
            offset: offset.get(),
            visible: visible.get(),
        });
        let accepted = composite::validated(scope, &raw.into(), |state: &Snapshot| {
            state.offset.validate()
        })?;
        let custom = self.common.style.clone();
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-affix",
            Style::new()
        )
        .style(scope.bond(move || {
            accepted
                .get()
                .offset
                .apply(merge(Style::new(), custom.get()), position)
        }))
        .semantics(scope.bond(move || {
            composite::metadata(Semantics {
                id: Some(id.clone()),
                hidden: !accepted.get().visible,
                ..Default::default()
            })
        }));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        if let Some(children) = self.children.take() {
            root = root.fill(children);
        }
        root.show_in(scope)
    }
}

type Content = Box<dyn FnMut(AffixContext, &Scope) -> Result>;
pub struct AffixContainer {
    key: String,
    common: Common,
    content: Option<Content>,
}
pub fn affix_container(key: impl Into<String>) -> AffixContainer {
    AffixContainer {
        key: key.into(),
        common: Common::default(),
        content: None,
    }
}
impl AffixContainer {
    appearance_methods!();
    pub fn content<W: Widget>(
        mut self,
        mut factory: impl FnMut(AffixContext) -> W + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context, scope| {
            factory(context).show_in(scope)
        }));
        self
    }
}
impl Widget for AffixContainer {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("affix-container", &self.key, "root", "root")?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let custom = self.common.style.clone();
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-affix-container",
            Style::new()
        )
        .node_ref(reference)
        .style(scope.bond(move || {
            let mut style = merge(
                Style::new()
                    .width(Length::Percent(100.0))
                    .height(Length::Px(240.0))
                    .overflow_x(Overflow::Auto)
                    .overflow_y(Overflow::Auto),
                custom.get(),
            );
            style.position = Some(Position::Relative);
            style
        }))
        .semantics(composite::metadata(Semantics {
            id: Some(id),
            ..Default::default()
        }));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        if let Some(factory) = self.content.as_mut() {
            let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
            let child = scope.child_in(node)?;
            factory(
                AffixContext {
                    node,
                    owner: scope.id(),
                },
                &child,
            )?;
            scope.retain(child)?;
        }
        Ok(())
    }
}
