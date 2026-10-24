//! Explicit application usage, sources and citations. These widgets neither
//! fetch previews nor independently verify an application's trust metadata.
use crate::appearance::{Common, configure, observe};
use crate::composite;
use glory::render::{DocumentPart, LinkOpen, TextKind};
use glory::style::{Display, FlexDirection, Length, Limit, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};
use std::collections::BTreeSet;
pub const MAX_CONTEXT_UNITS: u64 = 1_000_000_000_000;
pub const MAX_SOURCES: usize = 128;
pub const MAX_SOURCE_METADATA_BYTES: usize = 262_144;
macro_rules! display_methods {
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
        pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
            self.common.name = Some(value.into());
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
        pub fn on(
            mut self,
            kind: glory::render::EventKind,
            value: impl FnMut(glory::render::Event) + 'static,
        ) -> Self {
            self.common
                .handlers
                .entry(kind)
                .or_default()
                .push(Box::new(value));
            self
        }
    };
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str, max: usize, empty: bool) -> bool {
    value.len() <= max && !value.contains(['\0', '\r']) && (empty || !value.trim().is_empty())
}
fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextUsage {
    pub used: u64,
    pub capacity: Option<u64>,
    pub unit: String,
}
impl Default for ContextUsage {
    fn default() -> Self {
        Self {
            used: 0,
            capacity: None,
            unit: "tokens".into(),
        }
    }
}
impl ContextUsage {
    pub fn validate(&self) -> Result {
        if self.used > MAX_CONTEXT_UNITS
            || self
                .capacity
                .is_some_and(|v| v == 0 || v > MAX_CONTEXT_UNITS)
            || !text(&self.unit, 64, false)
            || self.unit.contains('\n')
        {
            return Err(invalid(
                "context units must be bounded and capacity must be positive when known",
            ));
        }
        Ok(())
    }
    fn description(&self) -> String {
        match self.capacity {
            Some(capacity) => format!(
                "{} of {} {}{}",
                self.used,
                capacity,
                self.unit,
                if self.used > capacity {
                    format!("; {} over capacity", self.used - capacity)
                } else {
                    String::new()
                }
            ),
            None => format!("{} {}; capacity unknown", self.used, self.unit),
        }
    }
}
pub struct Context {
    common: Common,
    usage: Lotus<ContextUsage>,
}
pub fn context() -> Context {
    Context {
        common: Common {
            name: Some("Context usage".into()),
            ..Common::default()
        },
        usage: ContextUsage::default().into(),
    }
}
impl Context {
    display_methods!();
    pub fn usage(mut self, value: impl Into<Lotus<ContextUsage>>) -> Self {
        self.usage = value.into();
        self
    }
}
impl Widget for Context {
    fn build(&mut self, scope: &Scope) -> Result {
        let usage = composite::validated(scope, &self.usage, ContextUsage::validate)?;
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let mut base = self.common.theme.label(self.common.size);
        base.min_width = Some(Length::Px(0.));
        base.background = Some(self.common.theme.surface);
        base.padding = Some(glory::style::Edges::all(8.));
        base.radius = Some(self.common.theme.radius);
        let mut root = configure!(self.common, glory::ui::view(), scope, "sapid-context", base)
            .fill(glory::ui::component(move |scope| {
                glory::ui::text()
                    .class("sapid-context-description")
                    .content(scope.bond(move || usage.with(ContextUsage::description)))
                    .show_in(scope)?;
                let percentage = scope.bond(move || {
                    usage.with(|u| {
                        u.capacity
                            .map(|v| (u.used as f64 / v as f64 * 100.).min(100.))
                            .unwrap_or(0.)
                    })
                });
                let unknown = scope.bond(move || usage.with(|u| u.capacity.is_none()));
                let mut progress = crate::progress(percentage)
                    .class("sapid-context-progress")
                    .name("Context capacity")
                    .value_text(scope.bond(move || usage.with(ContextUsage::description)))
                    .indeterminate(unknown)
                    .theme(theme.clone())
                    .width(Length::Percent(100.));
                if unstyled {
                    progress = progress.unstyled();
                }
                progress.show_in(scope)
            }));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceTrust {
    #[default]
    Unverified,
    Verified,
    Conflicting,
}
impl SourceTrust {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unverified => "Application trust: unverified",
            Self::Verified => "Application trust: verified",
            Self::Conflicting => "Application trust: conflicting",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRecord {
    pub key: String,
    pub title: String,
    pub url: String,
    pub preview: Option<String>,
    pub trust: SourceTrust,
}
impl SourceRecord {
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            url: url.into(),
            preview: None,
            trust: SourceTrust::Unverified,
        }
    }
    pub fn validate(&self) -> Result {
        if !key(&self.key)
            || !text(&self.title, 256, false)
            || self.title.contains('\n')
            || self.preview.as_ref().is_some_and(|v| !text(v, 8192, false))
            || !destination(&self.url)
        {
            return Err(invalid(
                "invalid finite source key/title/preview or HTTP(S)/fragment destination",
            ));
        }
        Ok(())
    }
}
/// Finite safe URL subset: ASCII DNS/IPv4 host with optional numeric port, or
/// an application-local identifier. No ambient path, credentials or fetch.
fn destination(url: &str) -> bool {
    if url.len() > 4096 || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    if let Some(fragment) = url.strip_prefix('#') {
        return key(fragment);
    }
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let (host, port) = authority
        .split_once(':')
        .map_or((authority, None), |(h, p)| (h, Some(p)));
    if host.is_empty()
        || host.len() > 253
        || host.split('.').any(|part| {
            part.is_empty()
                || part.len() > 63
                || part.starts_with('-')
                || part.ends_with('-')
                || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return false;
    }
    !port.is_some_and(|p| {
        p.is_empty()
            || p.len() > 5
            || !p.bytes().all(|b| b.is_ascii_digit())
            || p.parse::<u16>().ok().filter(|p| *p > 0).is_none()
    })
}
fn validate_sources(records: &[SourceRecord]) -> Result {
    if records.len() > MAX_SOURCES {
        return Err(invalid("sources admit at most 128 current records"));
    }
    let mut keys = BTreeSet::new();
    let mut bytes = 0;
    for r in records {
        r.validate()?;
        if !keys.insert(&r.key) {
            return Err(invalid("source keys must be distinct"));
        }
        bytes += r.key.len()
            + r.title.len()
            + r.url.len()
            + r.preview.as_ref().map_or(0, String::len);
        if bytes > MAX_SOURCE_METADATA_BYTES {
            return Err(invalid("source metadata exceeds 256 KiB"));
        }
    }
    Ok(())
}
pub struct Sources {
    common: Common,
    key: String,
    records: Lotus<Vec<SourceRecord>>,
    navigate: Option<Box<dyn FnMut(SourceNavigation)>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceNavigation {
    pub key: String,
    pub url: String,
    pub open: LinkOpen,
}
pub fn sources(key: impl Into<String>) -> Sources {
    Sources {
        common: Common {
            name: Some("Sources".into()),
            ..Common::default()
        },
        key: key.into(),
        records: Vec::new().into(),
        navigate: None,
    }
}
impl Sources {
    display_methods!();
    pub fn records(mut self, value: impl Into<Lotus<Vec<SourceRecord>>>) -> Self {
        self.records = value.into();
        self
    }
    pub fn on_navigate(mut self, value: impl FnMut(SourceNavigation) + 'static) -> Self {
        self.navigate = Some(Box::new(value));
        self
    }
}
impl Widget for Sources {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("sources", &self.key, "root", "root")?;
        let records = composite::validated(scope, &self.records, |v| validate_sources(v))?;
        let keys = scope.bond(move || {
            records.with(|v| v.iter().map(|r| r.key.clone()).collect::<Vec<_>>())
        });
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let has_callback = self.navigate.is_some();
        let callback = crate::composite::Callback::new(self.navigate.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let namespace = self.key.clone();
        let owner = scope.id();
        let mut base = self.common.theme.label(self.common.size);
        base.min_width = Some(Length::Px(0.));
        base.background = Some(self.common.theme.surface);
        base.padding = Some(glory::style::Edges::all(8.));
        base.radius = Some(self.common.theme.radius);
        let mut root = configure!(self.common, glory::ui::view(), scope, "sapid-sources", base)
            .document_part(DocumentPart::UnorderedList)
            .fill(glory::ui::each(
                keys,
                |k| k.clone(),
                move |key, _| {
                    let key = key.get_untracked();
                    let theme = theme.clone();
                    let callback = callback.clone();
                    let namespace = namespace.clone();
                    glory::ui::component(move |scope| {
                        let href = key.clone();
                        let title = key.clone();
                        let trust = key.clone();
                        let preview = key.clone();
                        let destination = key.clone();
                        let mut link = crate::anchor()
                            .class("sapid-source-link")
                            .theme(theme.clone())
                            .size(size)
                            .semantic_id(composite::id("sources", &namespace, "entry", &key)?)
                            .label(scope.bond(move || {
                                records.with(|v| {
                                    v.iter()
                                        .find(|r| r.key == title)
                                        .expect("live source")
                                        .title
                                        .clone()
                                })
                            }))
                            .href(scope.bond(move || {
                                records.with(|v| {
                                    v.iter()
                                        .find(|r| r.key == href)
                                        .expect("live source")
                                        .url
                                        .clone()
                                })
                            }));
                        if unstyled {
                            link = link.unstyled();
                        }
                        let callback = callback.clone();
                        if has_callback {
                            link = link.on_navigate(move |request| {
                                callback.emit(
                                    owner,
                                    SourceNavigation {
                                        key: destination.clone(),
                                        url: request.href,
                                        open: request.open,
                                    },
                                )
                            });
                        }
                        glory::ui::view()
                            .class("sapid-source")
                            .document_part(DocumentPart::ListItem)
                            .style(
                                Style::new()
                                    .display(Display::Flex)
                                    .flex_direction(FlexDirection::Column)
                                    .gap(4.)
                                    .min_width(Length::Px(0.)),
                            )
                            .fill((
                                link,
                                glory::ui::text().class("sapid-source-trust").content(
                                    scope.bond(move || {
                                        records.with(|v| {
                                            v.iter()
                                                .find(|r| r.key == trust)
                                                .expect("live source")
                                                .trust
                                                .label()
                                                .to_owned()
                                        })
                                    }),
                                ),
                                glory::ui::text()
                                    .kind(TextKind::Paragraph)
                                    .class("sapid-source-preview")
                                    .content(scope.bond(move || {
                                        records.with(|v| {
                                            v.iter()
                                                .find(|r| r.key == preview)
                                                .expect("live source")
                                                .preview
                                                .clone()
                                                .unwrap_or_default()
                                        })
                                    }))
                                    .style(Style {
                                        white_space: Some(WhiteSpace::PreWrap),
                                        max_width: Some(Limit::Percent(100.)),
                                        overflow_x: Some(Overflow::Auto),
                                        ..Style::new()
                                    }),
                            ))
                            .show_in(scope)
                    })
                },
            ));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}
pub struct InlineCitation {
    common: Common,
    source: Lotus<SourceRecord>,
    label: Lotus<String>,
    navigate: Option<Box<dyn FnMut(SourceNavigation)>>,
}
pub fn inline_citation(source: impl Into<Lotus<SourceRecord>>) -> InlineCitation {
    InlineCitation {
        common: Common::default(),
        source: source.into(),
        label: "[1]".into(),
        navigate: None,
    }
}
impl InlineCitation {
    display_methods!();
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn on_navigate(mut self, value: impl FnMut(SourceNavigation) + 'static) -> Self {
        self.navigate = Some(Box::new(value));
        self
    }
}
impl Widget for InlineCitation {
    fn build(&mut self, scope: &Scope) -> Result {
        let source = composite::validated(scope, &self.source, SourceRecord::validate)?;
        let label = composite::validated(scope, &self.label, |v| {
            if text(v, 64, false) && !v.contains('\n') {
                Ok(())
            } else {
                Err(invalid(
                    "inline citation label must be nonempty and at most 64 bytes",
                ))
            }
        })?;
        let mut anchor = crate::anchor()
            .class(scope.bond({
                let class = self.common.class.clone();
                move || format!("sapid-inline-citation {}", class.get())
            }))
            .label(label)
            .name(scope.bond(move || {
                source.with(|s| {
                    format!("Citation {}: {}; {}", label.get(), s.title, s.trust.label())
                })
            }))
            .href(scope.bond(move || source.with(|s| s.url.clone())))
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .style(self.common.style.clone())
            .focus(self.common.focus);
        if let Some(name) = self.common.name.take() {
            anchor = anchor.name(name);
        }
        if let Some(reference) = self.common.node_ref {
            anchor = anchor.node_ref(reference);
        }
        if let Some(mut callback) = self.navigate.take() {
            anchor = anchor.on_navigate(move |request| {
                callback(SourceNavigation {
                    key: source.with(|s| s.key.clone()),
                    url: request.href,
                    open: request.open,
                })
            });
        }
        if self.common.unstyled {
            anchor = anchor.unstyled();
        }
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let observers = crate::composite::Observers::new(handlers);
            let owner = scope.id();
            anchor = anchor.on(kind, move |event| observers.emit(owner, event));
        }
        anchor.show_in(scope)
    }
}
