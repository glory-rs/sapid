//! Projection retains only current paths and numeric live keys, never visited JSON.
use super::State;
use crate::{JsonKind, JsonTree, TreeKind, TreeNode};
use glory::render::TextKind;
use glory::style::{Length, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};
pub(super) struct Projection {
    tree: Rc<JsonTree>,
    nodes: Vec<TreeNode>,
    paths: HashMap<String, String>,
    keys: BTreeMap<String, usize>,
    next: u64,
}
fn kind(kind: JsonKind) -> &'static str {
    match kind {
        JsonKind::Object => "Object",
        JsonKind::Array => "Array",
        JsonKind::String => "String",
        JsonKind::Number => "Number",
        JsonKind::Boolean => "Boolean",
        JsonKind::Null => "Null",
    }
}
impl Projection {
    pub(super) fn prepare(tree: Rc<JsonTree>, previous: Option<&Self>) -> Result<Rc<Self>> {
        if tree
            .nodes()
            .iter()
            .any(|node| node.depth >= crate::MAX_TREE_DEPTH)
        {
            return Err(glory::render::Error::InvalidProps("JSON Tree navigation supports at most 64 visible semantic levels including the root").into());
        }
        let mut next = previous.map_or(0, |value| value.next);
        let mut paths = HashMap::with_capacity(tree.nodes().len());
        for node in tree.nodes() {
            let key = if let Some(key) = previous.and_then(|value| value.paths.get(&node.path))
            {
                key.clone()
            } else {
                next = next
                    .checked_add(1)
                    .ok_or(glory::render::Error::InvalidProps(
                        "JSON Tree identity exhausted",
                    ))?;
                format!("j{next}")
            };
            paths.insert(node.path.clone(), key);
        }
        let mut keys = BTreeMap::new();
        let mut nodes = Vec::with_capacity(tree.nodes().len());
        for (index, node) in tree.nodes().iter().enumerate() {
            let key = paths[&node.path].clone();
            keys.insert(key.clone(), index);
            let label = glory::render::normalize_text(
                &format!("{}: {}", node.label, kind(node.kind)),
                true,
            )
            .into_owned();
            let mut value = TreeNode::new(key, label).kind(
                if matches!(node.kind, JsonKind::Object | JsonKind::Array) {
                    TreeKind::Branch
                } else {
                    TreeKind::Leaf
                },
            );
            if let Some(parent) = node.parent {
                value.parent = Some(paths[&tree.nodes()[parent].path].clone());
            }
            nodes.push(value);
        }
        crate::TreeModel::validate(&nodes).map_err(|_| {
            glory::render::Error::InvalidProps(
                "JSON Tree projection exceeds the Tree model budget",
            )
        })?;
        Ok(Rc::new(Self {
            tree,
            nodes,
            paths,
            keys,
            next,
        }))
    }
}
pub(super) fn show(
    state: Rc<State>,
    projection: Cage<Rc<Projection>>,
    scope: &Scope,
) -> Result {
    let data = scope.bond(move || projection.with(|value| value.nodes.clone()));
    let expand_state = state.clone();
    let expanded = scope.bond(move || {
        let accepted = expand_state
            .accepted_paths
            .get()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let search = expand_state.search.get();
        projection.with(|value| {
            value
                .tree
                .nodes()
                .iter()
                .filter(|node| {
                    matches!(node.kind, JsonKind::Object | JsonKind::Array)
                        && (accepted.contains(&node.path) || search.open.contains(&node.path))
                })
                .map(|node| value.paths[&node.path].clone())
                .collect::<BTreeSet<_>>()
        })
    });
    let change_state = state.clone();
    let weak = scope.downgrade();
    let owner = scope.id();
    let label_state = state.clone();
    let mut widget = crate::tree(format!("json-tree-{}", state.namespace))
        .class("sapid-json-tree")
        .name("JSON tree")
        .theme(state.theme.clone())
        .size(state.size)
        .nodes(data)
        .expanded(expanded)
        .enabled(state.enabled.clone())
        .selection_mode(crate::TreeSelectionMode::None)
        .on_expanded_change(move |keys| {
            if !change_state.query.with(String::is_empty) {
                return;
            }
            let result = (|| -> Result {
                let mut proposal = change_state.expanded.source.try_with(|paths| {
                    super::check_paths(paths)?;
                    Ok::<_, glory::ui::Error>(paths.clone())
                })??;
                projection.with(|value| {
                    proposal.retain(|path| {
                        value
                            .paths
                            .get(path)
                            .and_then(|key| value.keys.get(key))
                            .is_none_or(|index| {
                                !matches!(
                                    value.tree.nodes()[*index].kind,
                                    JsonKind::Object | JsonKind::Array
                                )
                            })
                    });
                    for key in keys {
                        if let Some(index) = value.keys.get(&key) {
                            proposal.push(value.tree.nodes()[*index].path.clone());
                        }
                    }
                });
                super::check_paths(&proposal)?;
                change_state.expanded.accept(&proposal)?;
                change_state.changed.emit(owner, proposal);
                Ok(())
            })();
            if let Err(error) = result
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error)
            }
        })
        .render_label(move |node| {
            let state = label_state.clone();
            glory::ui::component(move |scope| {
                let node = node.clone();
                let mark_node = node.clone();
                let marked_state = state.clone();
                let marked = scope.bond(move || {
                    projection.with(|value| {
                        value.keys.get(&mark_node.get().key).is_some_and(|index| {
                            marked_state.matching(&value.tree.nodes()[*index].path)
                        })
                    })
                });
                let text = scope.bond(move || {
                    projection.with(|value| {
                        let Some(index) = value.keys.get(&node.get().key).copied() else {
                            return String::new();
                        };
                        let item = &value.tree.nodes()[index];
                        if matches!(item.kind, JsonKind::Array | JsonKind::Object) {
                            format!(
                                "{}: {} {}",
                                item.label,
                                kind(item.kind),
                                item.children.len()
                            )
                        } else {
                            format!("{}: {}", item.label, value.tree.value(index).unwrap_or(""))
                        }
                    })
                });
                let mut style = Style::new()
                    .min_width(Length::Px(0.))
                    .overflow_x(Overflow::Auto);
                style.grow = Some(1.);
                let theme = state.theme.clone();
                let unstyled = state.unstyled;
                glory::ui::view()
                    .class("sapid-json-tree-label")
                    .style(style)
                    .fill(
                        glory::ui::text()
                            .kind(TextKind::Code)
                            .class("sapid-json-tree-value")
                            .content(text)
                            .selectable(true)
                            .style(scope.bond(move || Style {
                                white_space: Some(WhiteSpace::Pre),
                                background:
                                    (!unstyled && marked.get()).then_some(theme.accent_soft),
                                ..Style::new()
                            })),
                    )
                    .show_in(scope)
            })
        });
    if state.unstyled {
        widget = widget.unstyled();
    }
    widget.show_in(scope)
}
