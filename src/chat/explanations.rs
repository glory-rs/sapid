//! Disclosure of public, application-supplied explanations. No inference service
//! or access to hidden model reasoning is part of these display widgets.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, ContentPolicy};
use glory::render::{DocumentPart, TextKind};
use glory::style::{Edges, Length, Limit, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};
use std::collections::BTreeSet;
pub const MAX_EXPLANATION_STEPS: usize = 64;
pub const MAX_EXPLANATION_BYTES: usize = 65536;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExplanationStatus {
    #[default]
    Pending,
    Active,
    Complete,
    Failed,
}
impl ExplanationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Active => "Active",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplanationStep {
    pub key: String,
    pub label: String,
    pub summary: String,
    pub status: ExplanationStatus,
    pub elapsed_millis: Option<u64>,
}
impl ExplanationStep {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            summary: summary.into(),
            status: ExplanationStatus::Pending,
            elapsed_millis: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicExplanation {
    pub summary: String,
    pub body: String,
    pub steps: Vec<ExplanationStep>,
}
impl Default for PublicExplanation {
    fn default() -> Self {
        Self {
            summary: "Public explanation".into(),
            body: String::new(),
            steps: vec![],
        }
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str, max: usize, empty: bool) -> bool {
    value.len() <= max && !value.contains(['\0', '\r']) && (empty || !value.trim().is_empty())
}
impl PublicExplanation {
    pub fn validate(&self) -> Result {
        if self.steps.len() > MAX_EXPLANATION_STEPS {
            return Err(invalid("public explanation admits at most 64 steps"));
        }
        if !text(&self.summary, 256, false) || !text(&self.body, 16384, true) {
            return Err(invalid("invalid bounded public explanation text"));
        }
        let mut keys = BTreeSet::new();
        let mut bytes = self.summary.len() + self.body.len();
        for s in &self.steps {
            if s.key.is_empty()
                || s.key.len() > 64
                || !s
                    .key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
                || !keys.insert(s.key.as_str())
            {
                return Err(invalid(
                    "public explanation step keys must be distinct finite ASCII identifiers",
                ));
            }
            if !text(&s.label, 256, false)
                || !text(&s.summary, 8192, true)
                || s.elapsed_millis.is_some_and(|v| v > 604_800_000)
            {
                return Err(invalid(
                    "invalid public explanation step text or elapsed time",
                ));
            }
            bytes += s.key.len() + s.label.len() + s.summary.len();
            if bytes > MAX_EXPLANATION_BYTES {
                return Err(invalid("public explanation exceeds its total byte budget"));
            }
        }
        Ok(())
    }
}
pub struct Reasoning {
    common: Common,
    key: String,
    data: Lotus<PublicExplanation>,
    ordered: bool,
    open: Option<Lotus<bool>>,
    default_open: bool,
    on_open: Option<Box<dyn FnMut(bool)>>,
    policy: ContentPolicy,
}
pub fn reasoning(key: impl Into<String>) -> Reasoning {
    Reasoning {
        common: Common::default(),
        key: key.into(),
        data: PublicExplanation::default().into(),
        ordered: false,
        open: None,
        default_open: false,
        on_open: None,
        policy: ContentPolicy::Unmount,
    }
}
pub fn chain_of_thought(key: impl Into<String>) -> Reasoning {
    Reasoning {
        ordered: true,
        ..reasoning(key)
    }
}
impl Reasoning {
    common_methods!();
    pub fn data(mut self, value: impl Into<Lotus<PublicExplanation>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.policy = value;
        self
    }
    pub fn on_open_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.on_open = Some(Box::new(value));
        self
    }
}
impl Widget for Reasoning {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("explanation", &self.key, "root", "root")?;
        let accepted = composite::validated(scope, &self.data, PublicExplanation::validate)?;
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let ordered = self.ordered;
        let mut disclosure = crate::collapsible(&self.key)
            .class("sapid-explanation-disclosure")
            .theme(theme.clone())
            .size(size)
            .enabled(self.common.enabled.clone())
            .label(scope.bond(move || accepted.with(|d| d.summary.clone())))
            .default_open(self.default_open)
            .content_policy(self.policy)
            .content(move || {
                let theme = theme.clone();
                glory::ui::component(move |scope| {
                    let mut style = if unstyled {
                        Style::new()
                    } else {
                        theme.label(size)
                    };
                    style.white_space = Some(WhiteSpace::PreWrap);
                    style.max_width = Some(Limit::Percent(100.));
                    style.overflow_x = Some(Overflow::Auto);
                    glory::ui::text()
                        .kind(TextKind::Paragraph)
                        .class("sapid-explanation-body")
                        .content(scope.bond(move || accepted.with(|d| d.body.clone())))
                        .style(style.clone())
                        .show_in(scope)?;
                    let root = scope.cage(None);
                    glory::ui::view()
                        .class("sapid-explanation-steps")
                        .document_part(if ordered {
                            DocumentPart::OrderedList
                        } else {
                            DocumentPart::UnorderedList
                        })
                        .style(Style {
                            padding: Some(Edges {
                                top: 0.,
                                right: 0.,
                                bottom: 0.,
                                left: 24.,
                            }),
                            max_width: Some(Limit::Percent(100.)),
                            ..Style::new()
                        })
                        .node_ref(root)
                        .show_in(scope)?;
                    let area = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
                    let keys = area.bond(move || {
                        accepted
                            .with(|d| d.steps.iter().map(|s| s.key.clone()).collect::<Vec<_>>())
                    });
                    glory::ui::each(
                        keys,
                        |key| key.clone(),
                        move |key, _| {
                            let key = key.get_untracked();
                            let style = style.clone();
                            glory::ui::component(move |scope| {
                                let text_key = key.clone();
                                let body_key = key.clone();
                                glory::ui::view()
                                    .class("sapid-explanation-step")
                                    .document_part(DocumentPart::ListItem)
                                    .style(Style::new().min_width(Length::Px(0.)))
                                    .fill((
                                        glory::ui::text()
                                            .class("sapid-explanation-step-label")
                                            .content(scope.bond(move || {
                                                accepted.with(|d| {
                                                    let s = d
                                                        .steps
                                                        .iter()
                                                        .find(|s| s.key == text_key)
                                                        .expect("live explanation step");
                                                    format!(
                                                        "{} · {}{}",
                                                        s.label,
                                                        s.status.label(),
                                                        s.elapsed_millis.map_or_else(
                                                            String::new,
                                                            |v| format!(" · {v} ms")
                                                        )
                                                    )
                                                })
                                            })),
                                        glory::ui::text()
                                            .kind(TextKind::Paragraph)
                                            .class("sapid-explanation-step-summary")
                                            .content(scope.bond(move || {
                                                accepted.with(|d| {
                                                    d.steps
                                                        .iter()
                                                        .find(|s| s.key == body_key)
                                                        .expect("live explanation step")
                                                        .summary
                                                        .clone()
                                                })
                                            }))
                                            .style(style.clone()),
                                    ))
                                    .show_in(scope)
                            })
                        },
                    )
                    .show_in(&area)?;
                    scope.retain(area)?;
                    Ok(())
                })
            });
        if let Some(open) = self.open.take() {
            disclosure = disclosure.open(open);
        }
        if let Some(callback) = self.on_open.take() {
            disclosure = disclosure.on_open_change(callback);
        }
        if unstyled {
            disclosure = disclosure.unstyled();
        }
        let mut base = self.common.theme.label(self.common.size);
        base.min_width = Some(Length::Px(0.));
        base.background = Some(self.common.theme.surface);
        base.padding = Some(Edges::all(8.));
        base.radius = Some(self.common.theme.radius);
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            if ordered {
                "sapid-chain-of-thought"
            } else {
                "sapid-reasoning"
            },
            base
        )
        .fill(disclosure);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)
    }
}
