use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::tree_choices_form(theme, false)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::style::{Display, FlexDirection, FlexWrap, Length, Style};
    use sapid::{
        TreeChoiceModel, TreeChoicePolicy, TreeChoices, TreeKind, TreeModel, TreeNode,
        cascader, tree_select,
    };
    use std::collections::BTreeSet;
    pub fn nodes() -> Vec<TreeNode> {
        vec![
            TreeNode::new("source", "Source").kind(TreeKind::Branch),
            TreeNode::new("main", "Main file").parent("source"),
            TreeNode::new("disabled", "Disabled file")
                .parent("source")
                .disabled(true),
            TreeNode::new("lib", "Library file").parent("source"),
            TreeNode::new("docs", "Documents").kind(TreeKind::Branch),
            TreeNode::new("guide", "Guide")
                .parent("docs")
                .kind(TreeKind::Branch),
            TreeNode::new("remote", "Remote files").kind(TreeKind::Lazy),
            TreeNode::new("deep", "Deep guide").parent("guide"),
        ]
    }
    pub fn tree_choices_form(theme: sapid::Theme, columns: bool) -> impl Widget {
        widget(move |scope| {
            let data = scope.cage(nodes());
            let value = scope.cage(TreeChoices::default());
            let refuse = scope.cage(false);
            let enabled = scope.cage(true);
            let readonly = scope.cage(false);
            let mounted = scope.cage(true);
            let proposals = scope.cage(0usize);
            let loads = scope.cage(0usize);
            let expanded = scope.cage(BTreeSet::from(["source".into()]));
            let opening_theme = theme.clone();
            sapid::form("path-demo")
                .theme(theme.clone())
                .fill(
                    col()
                        .style(
                            Style::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column)
                                .gap(16.)
                                .min_width(Length::Px(0.)),
                        )
                        .fill((
                            row()
                                .style(
                                    Style::new()
                                        .display(Display::Flex)
                                        .flex_direction(FlexDirection::Row)
                                        .gap(12.)
                                        .flex_wrap(FlexWrap::Wrap),
                                )
                                .fill((
                                    sapid::checkbox()
                                        .theme(theme.clone())
                                        .label("Refuse path selection")
                                        .checked(refuse)
                                        .on_checked_change(move |v| {
                                            refuse.revise(|old| *old = v)
                                        }),
                                    sapid::checkbox()
                                        .theme(theme.clone())
                                        .label("Enable path choice")
                                        .checked(enabled)
                                        .on_checked_change(move |v| {
                                            enabled.revise(|old| *old = v)
                                        }),
                                    sapid::checkbox()
                                        .theme(theme.clone())
                                        .label("Readonly path choice")
                                        .checked(readonly)
                                        .on_checked_change(move |v| {
                                            readonly.revise(|old| *old = v)
                                        }),
                                )),
                            when(mounted, move || {
                                macro_rules! main {
                                    ($w:expr) => {
                                        $w.theme(opening_theme.clone())
                                            .name("Project destination")
                                            .class("main-path-choice")
                                            .nodes(data)
                                            .value(value)
                                            .expanded(expanded)
                                            .enabled(enabled)
                                            .readonly(readonly)
                                            .form_name("destination")
                                            .clearable(true)
                                            .on_value_change(move |v| {
                                                proposals.revise(|n| *n += 1);
                                                if !refuse.get() {
                                                    value.revise(|old| *old = v)
                                                }
                                            })
                                            .on_expanded_change(move |v| {
                                                expanded.revise(|old| *old = v)
                                            })
                                            .on_load_children(move |_| {
                                                loads.revise(|n| *n += 1)
                                            })
                                    };
                                }
                                if columns {
                                    Box::new(main!(cascader("main-destination")))
                                        as Box<dyn Widget>
                                } else {
                                    Box::new(main!(tree_select("main-destination")))
                                        as Box<dyn Widget>
                                }
                            }),
                            text()
                                .class("path-choice-value")
                                .content(value.map(|value| {
                                    format!(
                                        "Path: {}",
                                        value
                                            .paths()
                                            .iter()
                                            .map(|p| p.keys().join("/"))
                                            .collect::<Vec<_>>()
                                            .join(",")
                                    )
                                })),
                            text()
                                .class("path-choice-proposals")
                                .content(proposals.map(|n| format!("Proposals: {n}"))),
                            text()
                                .class("path-choice-loads")
                                .content(loads.map(|n| format!("Loads: {n}"))),
                            row()
                                .style(
                                    Style::new()
                                        .display(Display::Flex)
                                        .flex_direction(FlexDirection::Row)
                                        .gap(8.)
                                        .flex_wrap(FlexWrap::Wrap),
                                )
                                .fill((
                                    sapid::button()
                                        .theme(theme.clone())
                                        .label("Publish remote children")
                                        .on_press(move || {
                                            data.revise(|data| {
                                                if data[6].kind == TreeKind::Lazy {
                                                    data[6].kind = TreeKind::Branch;
                                                    data.push(
                                                        TreeNode::new("loaded", "Loaded child")
                                                            .parent("remote"),
                                                    );
                                                }
                                            })
                                        }),
                                    sapid::button()
                                        .theme(theme.clone())
                                        .label("Reorder path items")
                                        .on_press(move || data.revise(|data| data.swap(1, 3))),
                                    sapid::button()
                                        .theme(theme.clone())
                                        .label("Select library externally")
                                        .on_press(move || {
                                            let model = TreeChoiceModel::new(
                                                TreeModel::new(data.get()).unwrap(),
                                                TreeChoicePolicy::Leaves,
                                                false,
                                            )
                                            .unwrap();
                                            value.revise(|old| {
                                                *old = TreeChoices::new([model
                                                    .path("lib")
                                                    .unwrap()])
                                                .unwrap()
                                            });
                                        }),
                                    sapid::button()
                                        .theme(theme.clone())
                                        .label("Toggle path Owner")
                                        .on_press(move || mounted.revise(|v| *v = !*v)),
                                )),
                            tree_select("checked-destinations")
                                .theme(theme.clone())
                                .name("Checked destinations")
                                .class("multiple-path-choice")
                                .nodes(nodes())
                                .multiple(true)
                                .policy(TreeChoicePolicy::DescendantLeaves)
                                .default_expanded(BTreeSet::from(["source".into()])),
                            tree_select("rtl-destinations")
                                .theme(theme.clone())
                                .name("RTL destination")
                                .class("rtl-path-choice")
                                .nodes(nodes())
                                .direction(glory::render::Direction::Rtl),
                            tree_select("disabled-destinations")
                                .theme(theme.clone())
                                .name("Disabled destination")
                                .nodes(nodes())
                                .enabled(false),
                        )),
                )
                .show_in(scope)
        })
    }
}
