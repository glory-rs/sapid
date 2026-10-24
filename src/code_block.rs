//! Portable code spans. No parser, markup injection, host or network dependency.
use crate::appearance::{Common, configure, observe};
use crate::code_model::*;
use glory::render::{Event, EventKind, NodeId, Semantics, TextKind};
use glory::style::{
    Color, Display, Edges, Family, FlexDirection, Length, Overflow, Style, WhiteSpace,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq)]
struct Span {
    text: String,
    kind: CodeTokenKind,
    highlighted: bool,
}
#[derive(Clone, PartialEq, Eq)]
struct Presentation {
    document: CodeDocument,
    language: CodeLanguage,
    spans: Vec<Rc<Span>>,
    numbers: String,
}
fn invalid(error: CodeError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(match error {
        CodeError::Bytes => "code exceeds 65536 UTF-8 bytes",
        CodeError::Tokens => "code exceeds 8192 lexical tokens",
        CodeError::Lines => "code exceeds 2048 lines",
        CodeError::CommentDepth => "code comments exceed 32 nesting levels",
        CodeError::RawStringHashes => "code raw strings exceed 16 hash delimiters",
    })
    .into()
}
fn prepare(
    source: &str,
    language: CodeLanguage,
    highlighted: &[usize],
    start: u32,
) -> Result<Presentation> {
    if source.len() > MAX_CODE_BYTES {
        return Err(invalid(CodeError::Bytes));
    }
    let source = glory::render::normalize_text(source, true);
    let document = tokenize_code(&source, language).map_err(invalid)?;
    let mut line = 1;
    let mut spans = Vec::new();
    for token in document.tokens() {
        for part in document.source()[token.range.clone()].split_inclusive('\n') {
            spans.push(Rc::new(Span {
                text: part.into(),
                kind: token.kind,
                highlighted: highlighted.contains(&line),
            }));
            line += usize::from(part.ends_with('\n'));
        }
    }
    let mut numbers = String::new();
    for index in 0..document.line_starts().len() {
        if index > 0 {
            numbers.push('\n');
        }
        use std::fmt::Write;
        write!(numbers, "{}", u64::from(start) + index as u64).expect("String write");
    }
    Ok(Presentation {
        document,
        language,
        spans,
        numbers,
    })
}
struct Config {
    common: Common,
    source: Lotus<String>,
    language: Lotus<CodeLanguage>,
    numbers: bool,
    start: u32,
    highlighted: Vec<usize>,
}
impl Config {
    fn new(language: CodeLanguage) -> Self {
        Self {
            common: Common::default(),
            source: "".into(),
            language: language.into(),
            numbers: false,
            start: 1,
            highlighted: Vec::new(),
        }
    }
}
pub struct CodeBlock {
    config: Config,
}
pub struct CodeHighlight {
    config: Config,
}
pub fn code_block() -> CodeBlock {
    CodeBlock {
        config: Config::new(CodeLanguage::Plain),
    }
}
pub fn code_highlight() -> CodeHighlight {
    CodeHighlight {
        config: Config::new(CodeLanguage::Rust),
    }
}
macro_rules! methods {
    ($name:ident) => {
        impl $name {
            pub fn theme(mut self, value: crate::Theme) -> Self {
                self.config.common.theme = value;
                self
            }
            pub fn size(mut self, value: crate::Size) -> Self {
                self.config.common.size = value;
                self
            }
            pub fn unstyled(mut self) -> Self {
                self.config.common.unstyled = true;
                self
            }
            pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
                self.config.common.class = value.into();
                self
            }
            pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
                self.config.common.style = value.into();
                self
            }
            pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
                self.config.common.node_ref = Some(value);
                self
            }
            pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
                self.config.common.name = Some(value.into());
                self
            }
            pub fn source(mut self, value: impl Into<Lotus<String>>) -> Self {
                self.config.source = value.into();
                self
            }
            pub fn language(mut self, value: impl Into<Lotus<CodeLanguage>>) -> Self {
                self.config.language = value.into();
                self
            }
            pub fn line_numbers(mut self, value: bool) -> Self {
                self.config.numbers = value;
                self
            }
            pub fn start_line(mut self, value: u32) -> Self {
                self.config.start = value;
                self
            }
            /// Relative one-based line numbers. Consume at most 2049 iterator entries.
            pub fn highlighted_lines(
                mut self,
                values: impl IntoIterator<Item = usize>,
            ) -> Self {
                crate::document::extend_bounded(
                    &mut self.config.highlighted,
                    values,
                    MAX_CODE_LINES,
                );
                self
            }
            pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
                self.config
                    .common
                    .handlers
                    .entry(kind)
                    .or_default()
                    .push(Box::new(value));
                self
            }
        }
        impl Widget for $name {
            fn build(&mut self, scope: &Scope) -> Result {
                build(&mut self.config, scope)
            }
        }
    };
}
methods!(CodeBlock);
methods!(CodeHighlight);
fn color(theme: &crate::Theme, kind: CodeTokenKind) -> Option<Color> {
    match kind {
        CodeTokenKind::Keyword => Some(theme.accent_text),
        CodeTokenKind::String => Some(if theme.scheme == crate::ColorScheme::Dark {
            crate::theme::GREEN[3]
        } else {
            crate::theme::GREEN[9]
        }),
        CodeTokenKind::Comment => Some(theme.muted),
        CodeTokenKind::Number => Some(theme.danger),
        _ => None,
    }
}
fn build(config: &mut Config, scope: &Scope) -> Result {
    if config.start == 0
        || config.highlighted.len() > MAX_CODE_LINES
        || config
            .highlighted
            .iter()
            .any(|line| !(1..=MAX_CODE_LINES).contains(line))
    {
        return Err(glory::render::Error::InvalidProps(
            "code lines require start >=1 and at most 2048 relative lines in 1..=2048",
        )
        .into());
    }
    let language = config.language.try_get()?;
    let initial = config
        .source
        .try_with(|source| prepare(source, language, &config.highlighted, config.start))??;
    let accepted = scope.cage(Rc::new(initial));
    let source = config.source.clone();
    let language = config.language.clone();
    let highlighted = std::mem::take(&mut config.highlighted);
    let start = config.start;
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let language = language.try_get()?;
            let next = source.try_with(|source| {
                let same = glory::core::untracked(|| {
                    accepted.try_with(|old| {
                        old.language == language && old.document.source() == source
                    })
                })?;
                if same {
                    return Ok::<_, glory::ui::Error>(None);
                }
                prepare(source, language, &highlighted, start).map(Some)
            })??;
            if let Some(next) = next {
                accepted.try_revise(|old| *old = Rc::new(next))?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    let common = &mut config.common;
    let reference = common.node_ref.unwrap_or_else(|| scope.cage(None));
    let theme = common.theme.clone();
    let unstyled = common.unstyled;
    let mut base = common.theme.label(common.size);
    base.font_family = Some(vec![Family::Mono]);
    base.font_weight = Some(400.);
    base.background = Some(common.theme.input);
    base.padding = Some(Edges::all(12.));
    base.radius = Some(common.theme.radius);
    base.min_width = Some(Length::Px(0.));
    let mut root = configure!(common, glory::ui::view(), scope, "sapid-code-block", base)
        .node_ref(reference);
    let owner = scope.id();
    for (kind, mut handlers) in std::mem::take(&mut common.handlers) {
        root = root.on(kind, move |event| observe(&mut handlers, owner, event));
    }
    root.show_in(scope)?;
    let area = scope.child_in(reference.try_get()?.ok_or(glory::ui::Error::Gone)?)?;
    let mut row = glory::ui::view().class("sapid-code-scroll").style(
        Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(16.)
            .overflow_x(Overflow::Auto),
    );
    let gutter_style = Style {
        color: (!unstyled).then_some(theme.muted),
        white_space: Some(WhiteSpace::Pre),
        font_family: Some(vec![Family::Mono]),
        shrink: Some(0.),
        ..Style::new()
    };
    if config.numbers {
        row = row.fill(
            glory::ui::view()
                .class("sapid-code-gutter")
                .semantics(Some(Box::new(Semantics {
                    inert: true,
                    ..Default::default()
                })))
                .fill(
                    glory::ui::text()
                        .selectable(false)
                        .style(gutter_style)
                        .content(
                            scope.bond(move || accepted.with(|value| value.numbers.clone())),
                        ),
                ),
        );
    }
    let spans = scope.bond(move || {
        accepted.with(|value| value.spans.iter().cloned().enumerate().collect::<Vec<_>>())
    });
    row.fill(
        glory::ui::view()
            .class("sapid-code-source")
            .style(Style {
                white_space: Some(WhiteSpace::Pre),
                font_family: Some(vec![Family::Mono]),
                shrink: Some(0.),
                ..Style::new()
            })
            .fill(glory::ui::each(
                spans,
                |value| value.0,
                move |span, _| {
                    let theme = theme.clone();
                    glory::ui::component(move |scope| {
                        let theme = theme.clone();
                        glory::ui::text()
                            .kind(TextKind::Code)
                            .selectable(true)
                            .class("sapid-code-token")
                            .content(
                                scope.bond(move || span.with(|(_, value)| value.text.clone())),
                            )
                            .style(scope.bond(move || {
                                span.with(|(_, value)| Style {
                                    color: if unstyled {
                                        None
                                    } else {
                                        color(&theme, value.kind)
                                    },
                                    background: (!unstyled && value.highlighted)
                                        .then_some(theme.accent_soft),
                                    ..Style::new()
                                })
                            }))
                            .show_in(scope)
                    })
                },
            ))
            // A zero-width inline box gives a terminal LF a real following
            // line. Chromium otherwise omits it from rendered selections,
            // even though the DOM Range contains the exact source bytes.
            .fill(
                glory::ui::view().class("sapid-code-end").style(
                    Style::new()
                        .display(Display::InlineFlex)
                        .width(Length::Px(0.))
                        .height(Length::Px(1.)),
                ),
            ),
    )
    .show_in(&area)?;
    scope.retain(area)?;
    Ok(())
}
