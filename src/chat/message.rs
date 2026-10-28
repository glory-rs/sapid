use crate::appearance::{Common, merge};
use crate::{Size, Theme};
use glory::render::{NodeId, SemanticId, Semantics, TextKind};
use glory::style::{
    Align, BorderStyle, Display, Edges, FlexDirection, Length, Limit, Overflow, Style,
    WhiteSpace,
};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};

pub const MAX_MESSAGE_BODY_BYTES: usize = 64 * 1024;
pub const MAX_MESSAGE_QUOTE_BYTES: usize = 4096;
pub const MAX_MESSAGE_AUTHOR_BYTES: usize = 128;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn label(value: &str, limit: usize) -> Result {
    if value.len() > limit || value.trim().is_empty() || value.contains(['\0', '\r', '\n']) {
        return Err(invalid("invalid bounded message label"));
    }
    Ok(())
}
pub fn message_id(
    key: impl AsRef<str>,
) -> std::result::Result<SemanticId, glory::render::Error> {
    let key = key.as_ref();
    if key.len() > 64 {
        return Err(glory::render::Error::InvalidProps(
            "message keys are limited to 64 bytes",
        ));
    }
    SemanticId::new(key)?;
    SemanticId::new(format!("sapid-message-{}_{key}", key.len()))
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageStatus {
    #[default]
    None,
    Sending,
    Sent,
    Failed,
    Cancelled,
}
impl MessageStatus {
    fn text(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Sending => "Sending",
            Self::Sent => "Sent",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageGroup {
    #[default]
    Single,
    First,
    Middle,
    Last,
}
impl MessageGroup {
    fn header(self) -> bool {
        matches!(self, Self::Single | Self::First)
    }
    fn footer(self) -> bool {
        matches!(self, Self::Single | Self::Last)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageAlignment {
    #[default]
    Start,
    Center,
    End,
}
impl MessageAlignment {
    fn align(self) -> Align {
        match self {
            Self::Start => Align::Start,
            Self::Center => Align::Center,
            Self::End => Align::End,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageQuote {
    pub source: String,
    pub body: String,
    pub target: Option<SemanticId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageData {
    pub author: String,
    pub body: String,
    /// Caller-formatted display text; no timezone or clock is inferred.
    pub timestamp: Option<String>,
    pub status: MessageStatus,
    /// Optional application-localized replacement for the finite status label.
    pub status_label: Option<String>,
    pub group: MessageGroup,
    pub alignment: MessageAlignment,
    pub quote: Option<MessageQuote>,
}
impl MessageData {
    pub fn new(author: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            author: author.into(),
            body: body.into(),
            timestamp: None,
            status: MessageStatus::None,
            status_label: None,
            group: MessageGroup::Single,
            alignment: MessageAlignment::Start,
            quote: None,
        }
    }
    pub fn validate(&self) -> Result {
        label(&self.author, MAX_MESSAGE_AUTHOR_BYTES)?;
        if self.body.len() > MAX_MESSAGE_BODY_BYTES {
            return Err(invalid("message body exceeds 65536 UTF8 bytes"));
        }
        if let Some(value) = &self.timestamp {
            label(value, 64)?;
        }
        if let Some(value) = &self.status_label {
            label(value, 128)?;
        }
        if let Some(quote) = &self.quote {
            label(&quote.source, 128)?;
            if quote.body.len() > MAX_MESSAGE_QUOTE_BYTES {
                return Err(invalid("message quote exceeds 4096 UTF8 bytes"));
            }
        }
        Ok(())
    }
    fn status_text(&self) -> String {
        self.status_label
            .clone()
            .unwrap_or_else(|| self.status.text().to_owned())
    }
}
macro_rules! appearance {
    () => {
        pub fn theme(mut self, value: Theme) -> Self {
            self.common.theme = value;
            self
        }
        pub fn size(mut self, value: Size) -> Self {
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
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.common.node_ref = Some(value);
            self
        }
    };
}
pub struct Bubble {
    common: Common,
    text: Vary<String>,
}
pub fn bubble(text: impl Into<Vary<String>>) -> Bubble {
    Bubble {
        common: Common::default(),
        text: text.into(),
    }
}
impl Bubble {
    appearance!();
}
impl Widget for Bubble {
    fn build(&mut self, scope: &Scope) -> Result {
        let text = crate::composite::validated(scope, &self.text, |text| {
            if text.len() > MAX_MESSAGE_BODY_BYTES {
                return Err(invalid("bubble body exceeds 65536 UTF8 bytes"));
            }
            Ok(())
        })?;
        let mut cosmetic = Style::new();
        if !self.common.unstyled {
            cosmetic = Style {
                background: Some(self.common.theme.surface),
                color: Some(self.common.theme.foreground),
                radius: Some(self.common.theme.radius),
                border_color: Some(self.common.theme.border),
                border_style: Some(BorderStyle::Solid),
                border_width: Some(Edges::all(1.)),
                padding: Some(Edges::all(self.common.theme.spacing(self.common.size))),
                ..Default::default()
            };
        }
        let (class, custom) = self.common.appearance(scope, "sapid-bubble", cosmetic);
        let style = scope.bond(move || {
            merge(
                Style::new()
                    .min_width(Length::Px(0.))
                    .max_width(Limit::Percent(100.))
                    .overflow_x(Overflow::Auto),
                custom.get(),
            )
        });
        let content = scope.bond(move || {
            text.with(|text| glory::render::normalize_text(text, true).into_owned())
        });
        let mut root = glory::ui::view().class(class).style(style);
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference);
        }
        let mut body = crate::text()
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .content(content)
            .style(Style::new().white_space(WhiteSpace::PreWrap))
            .class("sapid-bubble-body");
        if self.common.unstyled {
            body = body.unstyled();
        }
        root.fill(body).show_in(scope)
    }
}
pub struct Marker {
    common: Common,
    target: Vary<SemanticId>,
    label: Vary<String>,
    navigate: Option<Box<dyn FnMut(crate::AnchorRequest)>>,
}
pub fn marker(target: impl Into<Vary<SemanticId>>) -> Marker {
    Marker {
        common: Common::default(),
        target: target.into(),
        label: "Reference".into(),
        navigate: None,
    }
}
impl Marker {
    appearance!();
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn on_navigate(mut self, value: impl FnMut(crate::AnchorRequest) + 'static) -> Self {
        self.navigate = Some(Box::new(value));
        self
    }
}
impl Widget for Marker {
    fn build(&mut self, scope: &Scope) -> Result {
        let label = crate::composite::validated(scope, &self.label, |value| label(value, 128))?;
        let target = self.target.clone();
        let href = scope.bond(move || format!("#{}", target.get().as_str()));
        let extra = self.common.class.clone();
        let class = scope.bond(move || format!("sapid-marker {}", extra.get()));
        let mut anchor = crate::anchor()
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .class(class)
            .style(self.common.style.clone())
            .href(href)
            .label(label);
        if self.common.unstyled {
            anchor = anchor.unstyled();
        }
        if let Some(reference) = self.common.node_ref {
            anchor = anchor.node_ref(reference);
        }
        if let Some(callback) = self.navigate.take() {
            anchor = anchor.on_navigate(callback);
        }
        anchor.show_in(scope)
    }
}
pub struct Message {
    common: Common,
    id: std::result::Result<SemanticId, glory::render::Error>,
    data: Vary<MessageData>,
    actions: Option<Children>,
}
pub fn message(key: impl AsRef<str>, data: impl Into<Vary<MessageData>>) -> Message {
    Message {
        common: Common::default(),
        id: message_id(key),
        data: data.into(),
        actions: None,
    }
}
impl Message {
    appearance!();
    pub fn actions(mut self, children: impl IntoChildren) -> Self {
        self.actions = Some(children.into_children());
        self
    }
}
impl Widget for Message {
    fn build(&mut self, scope: &Scope) -> Result {
        let id = self.id.clone()?;
        let data = crate::composite::validated(scope, &self.data, MessageData::validate)?;
        let (class, custom) = self.common.appearance(scope, "sapid-message", Style::new());
        let gap = self.common.theme.spacing(Size::ExtraSmall);
        let style = scope.bond(move || {
            merge(
                Style {
                    display: Some(Display::Flex),
                    flex_direction: Some(FlexDirection::Column),
                    align: Some(data.with(|value| value.alignment.align())),
                    gap: Some(gap),
                    width: Some(Length::Percent(100.)),
                    min_width: Some(Length::Px(0.)),
                    ..Default::default()
                },
                custom.get(),
            )
        });
        let mut root = glory::ui::view()
            .class(class)
            .style(style)
            .name(
                scope.bond(move || data.with(|value| format!("Message from {}", value.author))),
            )
            .semantics(Some(Box::new(Semantics {
                id: Some(id),
                ..Default::default()
            })));
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference);
        }
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let mut body = bubble(data.map(|value| value.body.clone()))
            .theme(theme.clone())
            .size(size);
        if unstyled {
            body = body.unstyled();
        }
        let header = glory::ui::view()
            .class("sapid-message-header")
            .semantics(data.map(|value| {
                Some(Box::new(Semantics {
                    hidden: !value.group.header(),
                    ..Default::default()
                }))
            }))
            .fill(
                glory::ui::text()
                    .content(data.map(|value| value.author.clone()))
                    .style(if unstyled {
                        Style::new()
                    } else {
                        theme.label(size)
                    }),
            );
        let footer = glory::ui::view()
            .class("sapid-message-footer")
            .style(if unstyled {
                Style::new()
            } else {
                theme.description(size)
            })
            .semantics(data.map(|value| {
                Some(Box::new(Semantics {
                    hidden: !value.group.footer()
                        || (value.timestamp.is_none()
                            && value.status == MessageStatus::None
                            && value.status_label.is_none()),
                    ..Default::default()
                }))
            }))
            .fill((
                glory::ui::text()
                    .class("sapid-message-time")
                    .content(data.map(|value| value.timestamp.clone().unwrap_or_default())),
                glory::ui::text()
                    .class("sapid-message-status")
                    .content(data.map(MessageData::status_text)),
            ));
        let mut quote = crate::blockquote()
            .theme(theme.clone())
            .size(size)
            .class("sapid-message-quote")
            .fill(
                glory::ui::text()
                    .kind(TextKind::Paragraph)
                    .content(data.map(|value| {
                        value
                            .quote
                            .as_ref()
                            .map(|quote| {
                                glory::render::normalize_text(&quote.body, true).into_owned()
                            })
                            .unwrap_or_default()
                    })),
            );
        if unstyled {
            quote = quote.unstyled();
        }
        let quote_target = data.map(|value| {
            value
                .quote
                .as_ref()
                .and_then(|quote| quote.target.clone())
                .unwrap_or_else(|| SemanticId::new("unavailable-message-reference").unwrap())
        });
        let mut reference =
            marker(quote_target)
                .theme(theme.clone())
                .size(size)
                .label(data.map(|value| {
                    value
                        .quote
                        .as_ref()
                        .map_or_else(|| "Reference".to_owned(), |quote| quote.source.clone())
                }));
        if unstyled {
            reference = reference.unstyled();
        }
        let citation = glory::ui::view()
            .semantics(data.map(|value| {
                Some(Box::new(Semantics {
                    hidden: value
                        .quote
                        .as_ref()
                        .is_none_or(|quote| quote.target.is_none()),
                    ..Default::default()
                }))
            }))
            .fill(reference);
        let plain = glory::ui::view()
            .semantics(data.map(|value| {
                Some(Box::new(Semantics {
                    hidden: value
                        .quote
                        .as_ref()
                        .is_none_or(|quote| quote.target.is_some()),
                    ..Default::default()
                }))
            }))
            .fill(glory::ui::text().content(data.map(|value| {
                value
                    .quote
                    .as_ref()
                    .map(|quote| quote.source.clone())
                    .unwrap_or_default()
            })));
        let quote = glory::ui::view()
            .semantics(data.map(|value| {
                Some(Box::new(Semantics {
                    hidden: value.quote.is_none(),
                    ..Default::default()
                }))
            }))
            .fill(quote.citation((citation, plain)));
        let mut root = root.fill((header, body, quote, footer));
        if let Some(actions) = self.actions.take() {
            root = root.fill(
                glory::ui::view()
                    .class("sapid-message-actions")
                    .style(Style::new().display(Display::Flex).gap(gap))
                    .fill(actions),
            );
        }
        root.show_in(scope)
    }
}
