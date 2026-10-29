//! Accessible plain text with a zero-sized, clipped, out-of-flow box.
use glory::render::{Change, NodeId, Patch, SemanticId, Semantics};
use glory::style::{Display, Edges, Length, Limit, Overflow, Position, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

pub const MAX_VISUALLY_HIDDEN_BYTES: usize = 65_536;

/// Plain text only: invisible interactive descendants cannot enter the Tab order.
pub struct VisuallyHidden {
    content: Vary<String>,
    class: Vary<String>,
    id: Option<SemanticId>,
    node_ref: Option<Cage<Option<NodeId>>>,
}
pub fn visually_hidden() -> VisuallyHidden {
    VisuallyHidden {
        content: "".into(),
        class: "".into(),
        id: None,
        node_ref: None,
    }
}
impl VisuallyHidden {
    pub fn content(mut self, value: impl Into<Vary<String>>) -> Self {
        self.content = value.into();
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.class = value.into();
        self
    }
    /// Stable target for an explicit label/description relation.
    pub fn semantic_id(mut self, value: SemanticId) -> Self {
        self.id = Some(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.node_ref = Some(value);
        self
    }
}
fn validate(value: &str) -> Result {
    if value.len() > MAX_VISUALLY_HIDDEN_BYTES {
        return Err(
            glory::render::Error::InvalidProps("visually hidden text exceeds 64KiB").into(),
        );
    }
    Ok(())
}
impl Widget for VisuallyHidden {
    fn build(&mut self, scope: &Scope) -> Result {
        let initial = self.content.try_get()?;
        validate(&initial)?;
        let classes = self.class.clone();
        let classes = scope.bond(move || {
            let custom = classes.get();
            format!("sapid-visually-hidden {custom}")
        });
        let reference = self.node_ref.unwrap_or_else(|| scope.cage(None));
        // Width/height apply because this span is an absolute block. Both axes
        // clip at zero, without opacity/visibility/hidden accessibility changes.
        glory::ui::text()
            .content(initial.clone())
            .class(classes)
            .style(Style {
                position: Some(Position::Absolute),
                top: Some(Length::Px(0.0)),
                left: Some(Length::Px(0.0)),
                display: Some(Display::Block),
                width: Some(Length::Px(0.0)),
                height: Some(Length::Px(0.0)),
                min_width: Some(Length::Px(0.0)),
                min_height: Some(Length::Px(0.0)),
                max_width: Some(Limit::Px(0.0)),
                max_height: Some(Limit::Px(0.0)),
                padding: Some(Edges::all(0.0)),
                margin: Some(Edges::all(0.0)),
                border_width: Some(Edges::all(0.0)),
                overflow_x: Some(Overflow::Hidden),
                overflow_y: Some(Overflow::Hidden),
                white_space: Some(WhiteSpace::NoWrap),
                ..Style::new()
            })
            .selectable(false)
            .semantics(Some(Box::new(Semantics {
                id: self.id.clone(),
                ..Default::default()
            })))
            .node_ref(reference)
            .show_in(scope)?;
        let source = self.content.clone();
        if source.is_reactive() {
            let weak = scope.downgrade();
            let mut previous = initial;
            scope.binding(move || {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let next = source.try_get()?;
                    if next == previous {
                        return Ok(());
                    }
                    validate(&next)?;
                    if let Some(id) = reference.try_get()? {
                        scope.patch(id, Patch::Text(Change::Set(next.clone())))?;
                        previous = next;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        }
        Ok(())
    }
}

impl Default for VisuallyHidden {
    fn default() -> Self {
        visually_hidden()
    }
}
