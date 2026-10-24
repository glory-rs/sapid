use std::collections::BTreeMap;

use glory::core::Lotus;
use glory::render::{Event, EventKind, FocusMode, NodeId};
use glory::style::Style;
use glory::{Cage, Scope};

use crate::{Size, Theme};

pub(crate) type Handlers = BTreeMap<EventKind, Vec<Box<dyn FnMut(Event)>>>;

pub(crate) struct Common {
    pub theme: Theme,
    pub size: Size,
    pub unstyled: bool,
    pub class: Lotus<String>,
    pub style: Lotus<Style>,
    pub name: Option<Lotus<String>>,
    pub enabled: Lotus<bool>,
    pub focus: FocusMode,
    pub node_ref: Option<Cage<Option<NodeId>>>,
    pub handlers: Handlers,
}

impl Default for Common {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            size: Size::default(),
            unstyled: false,
            class: "".into(),
            style: Style::new().into(),
            name: None,
            enabled: true.into(),
            focus: FocusMode::Auto,
            node_ref: None,
            handlers: BTreeMap::new(),
        }
    }
}

impl Common {
    pub fn appearance(
        &self,
        scope: &Scope,
        class: &str,
        base: Style,
    ) -> (Lotus<String>, Lotus<Style>) {
        let prefix = format!(
            "{class} {} {}{}",
            self.size.class(),
            self.theme.class(),
            if self.unstyled { " sapid-unstyled" } else { "" }
        );
        let classes = self.class.clone();
        let class_value = move || {
            let extra = classes.get();
            format!("{prefix} {extra}")
        };
        let classes = if self.class.is_reactive() {
            scope.bond(class_value).into()
        } else {
            class_value().into()
        };
        let base = if self.unstyled { Style::new() } else { base };
        let custom = self.style.clone();
        let style_value = move || merge(base.clone(), custom.get());
        let styles = if self.style.is_reactive() {
            scope.bond(style_value).into()
        } else {
            style_value().into()
        };
        (classes, styles)
    }
}

// Keep every optional declaration independent; absent overrides preserve tokens.
// The exhaustive destructure makes new Glory Style fields a compile-time review.
pub(crate) fn merge(mut base: Style, overrides: Style) -> Style {
    macro_rules! fields {
        ($($field:ident),+ $(,)?) => {
            let Style { $($field),+ } = overrides;
            $(if $field.is_some() { base.$field = $field; })+
        };
    }
    fields!(
        position,
        top,
        left,
        right,
        bottom,
        display,
        grid_columns,
        grid_rows,
        grid_column,
        grid_row,
        aspect_ratio,
        object_fit,
        pointer_events,
        flex_direction,
        flex_wrap,
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        gap,
        padding,
        margin,
        grow,
        shrink,
        align,
        justify,
        overflow_x,
        overflow_y,
        scroll_anchor,
        color,
        background,
        font_size,
        line_height,
        font_family,
        font_weight,
        white_space,
        text_overflow,
        field_sizing,
        border_width,
        border_color,
        border_style,
        radius,
        opacity,
        box_shadow,
    );
    base
}

/// Shared configuration remains available on the concrete public builders.
macro_rules! common_methods {
    () => {
        pub fn theme(mut self, theme: $crate::Theme) -> Self {
            self.common.theme = theme;
            self
        }
        pub fn size(mut self, size: $crate::Size) -> Self {
            self.common.size = size;
            self
        }
        /// Remove theme declarations; explicit style overrides still apply.
        pub fn unstyled(mut self) -> Self {
            self.common.unstyled = true;
            self
        }
        /// Set user classes, appended to the widget's own classes.
        pub fn class(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.common.class = value.into();
            self
        }
        /// Override supplied declarations, preserving unspecified theme tokens.
        pub fn style(mut self, value: impl Into<glory::Lotus<glory::style::Style>>) -> Self {
            self.common.style = value.into();
            self
        }
        pub fn name(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
            self.common.name = Some(value.into());
            self
        }
        pub fn enabled(mut self, value: impl Into<glory::Lotus<bool>>) -> Self {
            self.common.enabled = value.into();
            self
        }
        pub fn focus(mut self, value: glory::render::FocusMode) -> Self {
            self.common.focus = value;
            self
        }
        pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
        /// Append an observer; internal behavior runs first. Repeated observers
        /// for the same kind run in registration order while the Owner is live.
        pub fn on(
            mut self,
            kind: glory::render::EventKind,
            handler: impl FnMut(glory::render::Event) + 'static,
        ) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(handler));
            self
        }
    };
}
pub(crate) use common_methods;

macro_rules! configure {
    ($common:expr, $widget:expr, $scope:expr, $class:expr, $base:expr) => {{
        let (classes, style) = $common.appearance($scope, $class, $base);
        let mut widget = $widget.class(classes).style(style).focus($common.focus);
        if let Some(name) = $common.name.take() {
            widget = widget.name(name);
        }
        if let Some(reference) = $common.node_ref {
            widget = widget.node_ref(reference);
        }
        widget
    }};
}
pub(crate) use configure;

pub(crate) fn observe(
    handlers: &mut [Box<dyn FnMut(Event)>],
    owner: glory::core::InstanceId,
    event: Event,
) {
    for handler in handlers {
        if !owner.is_alive() {
            break;
        }
        handler(event.clone());
    }
}
