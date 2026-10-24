//! Inline messages use intrinsic live-region roles; severity is independent of
//! whether the application requests an announcement.
use crate::appearance::Common;
use crate::composite::{self, Callback};
use crate::state::Value;
use glory::render::{Role, Semantics, TextKind};
use glory::style::{BorderStyle, Color, Edges, Length, Style};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Lotus, Scope};

pub const MAX_ALERT_TEXT_BYTES: usize = 64 * 1024;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Announcement {
    Off,
    #[default]
    Polite,
    Assertive,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}
pub struct Alert {
    key: String,
    common: Common,
    content: Lotus<String>,
    title: Option<Lotus<String>>,
    severity: AlertSeverity,
    announcement: Announcement,
    visible: Value<bool>,
    closable: bool,
    close_label: Lotus<String>,
    close: Option<Box<dyn FnMut(())>>,
    children: Option<Children>,
}
pub fn alert(key: impl Into<String>) -> Alert {
    Alert {
        key: key.into(),
        common: Common::default(),
        content: "".into(),
        title: None,
        severity: AlertSeverity::Info,
        announcement: Announcement::Polite,
        visible: Value {
            source: None,
            default: true,
        },
        closable: false,
        close_label: "Dismiss message".into(),
        close: None,
        children: None,
    }
}
impl Alert {
    pub fn content(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.content = value.into();
        self
    }
    pub fn title(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.title = Some(value.into());
        self
    }
    pub fn severity(mut self, value: AlertSeverity) -> Self {
        self.severity = value;
        self
    }
    pub fn announcement(mut self, value: Announcement) -> Self {
        self.announcement = value;
        self
    }
    pub fn visible(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.visible.source = Some(value.into());
        self
    }
    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }
    pub fn close_label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.close_label = value.into();
        self
    }
    pub fn on_close(mut self, mut handler: impl FnMut() + 'static) -> Self {
        self.close = Some(Box::new(move |()| handler()));
        self
    }
    pub fn fill(mut self, value: impl IntoChildren) -> Self {
        self.children = Some(value.into_children());
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
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
}
fn message(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > MAX_ALERT_TEXT_BYTES {
        return Err(glory::render::Error::InvalidProps(
            "alert text must be nonempty and within 64 KiB",
        )
        .into());
    }
    Ok(())
}
impl Widget for Alert {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = composite::id("alert", &self.key, "root", "root")?;
        let title_id = composite::id("alert", &self.key, "title", "root")?;
        let content_id = composite::id("alert", &self.key, "message", "root")?;
        let content = composite::validated(scope, &self.content, |value| message(value))?;
        let title = self
            .title
            .as_ref()
            .map(|value| composite::validated(scope, value, |value| message(value)))
            .transpose()?;
        let close_label = if self.closable {
            Some(composite::validated(scope, &self.close_label, |value| {
                message(value)
            })?)
        } else {
            None
        };
        let visible = std::mem::take(&mut self.visible).mount(scope);
        let source = visible.source.clone();
        let mut root = glory::ui::view();
        if let Some(role) = match self.announcement {
            Announcement::Off => None,
            Announcement::Polite => Some(Role::Status),
            Announcement::Assertive => Some(Role::Alert),
        } {
            root = root.role(role);
        }
        let labelled_by = if self.common.name.is_none() && title.is_some() {
            Some(title_id.clone())
        } else {
            None
        };
        let description_id = content_id.clone();
        root = root.semantics(scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(id.clone()),
                labelled_by: labelled_by.clone(),
                described_by: Some(description_id.clone()),
                hidden: !source.get(),
                ..Default::default()
            }))
        }));
        let theme = &self.common.theme;
        let (background, color) = match self.severity {
            AlertSeverity::Info => (theme.accent_soft, theme.accent_text),
            AlertSeverity::Success => {
                let palette = crate::Theme::new(crate::Hue::Green, theme.scheme);
                (palette.accent_soft, palette.accent_text)
            }
            AlertSeverity::Warning => {
                if theme.scheme == crate::ColorScheme::Light {
                    (Color(255, 249, 219, 255), Color(102, 77, 3, 255))
                } else {
                    (Color(56, 43, 13, 255), Color(255, 224, 102, 255))
                }
            }
            AlertSeverity::Danger => (theme.danger_soft, theme.danger),
        };
        let mut base = composite::column(8.0)
            .min_width(Length::Px(0.0))
            .padding(Edges::all(14.0))
            .radius(theme.radius)
            .border_width(Edges::all(1.0))
            .border_style(BorderStyle::Solid)
            .border_color(color);
        base.background = Some(background);
        base.color = Some(color);
        let (classes, style) = self.common.appearance(scope, "sapid-alert", base);
        root = root.class(classes).style(style);
        if let Some(name) = self.common.name.take() {
            root = root.name(name)
        }
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference)
        }
        let mut children = self.children.take();
        let close = Callback::new(self.close.take());
        let weak = scope.downgrade();
        let owner = scope.id();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        // All child captures belong to the caller's mounting Owner.
        root.fill(glory::ui::widget(move |scope| {
            if let Some(title) = title {
                glory::ui::text()
                    .class("sapid-alert-title")
                    .kind(TextKind::Heading(3))
                    .content(title)
                    .semantics(Some(Box::new(Semantics {
                        id: Some(title_id.clone()),
                        ..Default::default()
                    })))
                    .show_in(scope)?;
            }
            glory::ui::text()
                .class("sapid-alert-content")
                .content(content)
                .semantics(Some(Box::new(Semantics {
                    id: Some(content_id.clone()),
                    ..Default::default()
                })))
                .show_in(scope)?;
            if let Some(children) = children.take() {
                glory::ui::group().fill(children).show_in(scope)?;
            }
            if let Some(label) = close_label {
                let weak = weak.clone();
                let visible = visible.clone();
                let close = close.clone();
                let mut button = crate::button()
                    .theme(theme.clone())
                    .size(size)
                    .class("sapid-alert-close")
                    .label(label)
                    .on_press(move || {
                        let Some(scope) = weak.upgrade() else { return };
                        match visible.accept(&false) {
                            Ok(()) => close.emit(owner, ()),
                            Err(error) => scope.report_error(error),
                        }
                    });
                if unstyled {
                    button = button.unstyled();
                }
                button.show_in(scope)?;
            }
            Ok(())
        }))
        .show_in(scope)
    }
}
