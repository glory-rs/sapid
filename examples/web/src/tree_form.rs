use glory::prelude::*;
use sapid::{TreeKind, TreeNode, TreeSelectionMode, tree};
use std::collections::BTreeSet;

fn nodes() -> Vec<TreeNode> {
    let mut nodes = vec![
        TreeNode::new("source", "Source").kind(TreeKind::Branch),
        TreeNode::new("main", "Main file").parent("source"),
        TreeNode::new("disabled", "Disabled file")
            .parent("source")
            .disabled(true),
        TreeNode::new("library", "Library file").parent("source"),
        TreeNode::new("docs", "Documents").kind(TreeKind::Branch),
        TreeNode::new("remote", "Remote files").kind(TreeKind::Lazy),
    ];
    nodes.extend(
        (0..96)
            .map(|i| TreeNode::new(format!("doc{i}"), format!("Document {i:02}")).parent("docs")),
    );
    nodes
}
pub fn tree_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let data = scope.cage(nodes());
        let expanded = scope.cage(BTreeSet::from(["source".into()]));
        let selected = scope.cage(BTreeSet::new());
        let multiple = scope.cage(BTreeSet::new());
        let refuse = scope.cage(false);
        let enabled = scope.cage(true);
        let mounted = scope.cage(true);
        let loads = scope.cage(0usize);
        let proposals = scope.cage(0usize);
        let editor_theme = theme.clone();
        let widget_theme = theme.clone();
        col()
            .style(Style::new().gap(12.).min_width(Length::Px(0.)))
            .fill((
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Refuse tree changes")
                    .checked(refuse)
                    .on_checked_change(move |value| refuse.revise(|old| *old = value)),
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Enable tree")
                    .checked(enabled)
                    .on_checked_change(move |value| enabled.revise(|old| *old = value)),
                when(mounted, move || {
                    let theme = widget_theme.clone();
                    let editor_theme = editor_theme.clone();
                    tree("project-files")
                        .theme(theme.clone())
                        .name("Project files")
                        .class("project-tree")
                        .height(240.)
                        .nodes(data)
                        .expanded(expanded)
                        .selected(selected)
                        .enabled(enabled)
                        .on_expanded_change(move |value| {
                            proposals.revise(|n| *n += 1);
                            if !refuse.get() {
                                expanded.revise(|old| *old = value);
                            }
                        })
                        .on_selection_change(move |value| {
                            if !refuse.get() {
                                selected.revise(|old| *old = value);
                            }
                        })
                        .on_load_children(move |_| loads.revise(|n| *n += 1))
                        .render_label(move |node| {
                            let editor_theme = editor_theme.clone();
                            row()
                                .style(Style::new().gap(8.).min_width(Length::Px(0.)))
                                .fill((
                                    text().content(node.with(|node| node.label.clone())),
                                    when(node.with(|node| node.key == "main"), move || {
                                        sapid::text_field()
                                            .theme(editor_theme.clone())
                                            .name("Tree editor")
                                            .default_value("Uncommitted draft")
                                    }),
                                ))
                        })
                }),
                text().class("tree-selected").content(selected.map(|keys| {
                    format!(
                        "Selected: {}",
                        keys.iter().cloned().collect::<Vec<_>>().join(",")
                    )
                })),
                text()
                    .class("tree-loads")
                    .content(loads.map(|n| format!("Loads: {n}"))),
                text()
                    .class("tree-proposals")
                    .content(proposals.map(|n| format!("Expansion proposals: {n}"))),
                sapid::button()
                    .theme(theme.clone())
                    .label("Publish lazy files")
                    .on_press(move || {
                        data.revise(|nodes| {
                            if nodes[5].kind == TreeKind::Lazy {
                                nodes[5].kind = TreeKind::Branch;
                                nodes.push(TreeNode::new("loaded", "Loaded file").parent("remote"));
                            }
                        })
                    }),
                sapid::button()
                    .theme(theme.clone())
                    .label("Reorder tree files")
                    .on_press(move || data.revise(|nodes| nodes.swap(1, 3))),
                sapid::button()
                    .theme(theme.clone())
                    .label("Collapse all tree branches")
                    .on_press(move || expanded.revise(BTreeSet::clear)),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle tree Owner")
                    .on_press(move || mounted.revise(|value| *value = !*value)),
            ))
            .fill((
                tree("multi-files")
                    .theme(theme.clone())
                    .name("Selected files")
                    .selection_mode(TreeSelectionMode::Multiple)
                    .nodes(vec![
                        TreeNode::new("alpha", "Alpha"),
                        TreeNode::new("beta", "Beta"),
                        TreeNode::new("gamma", "Gamma"),
                    ])
                    .selected(multiple)
                    .on_selection_change(move |value| multiple.revise(|old| *old = value)),
                text()
                    .class("tree-multi-selected")
                    .content(multiple.map(|keys| {
                        format!(
                            "Multiple: {}",
                            keys.iter().cloned().collect::<Vec<_>>().join(",")
                        )
                    })),
                tree("rtl-files")
                    .theme(theme.clone())
                    .name("RTL files")
                    .direction(glory::render::Direction::Rtl)
                    .nodes(vec![
                        TreeNode::new("rtl", "RTL branch").kind(TreeKind::Branch),
                        TreeNode::new("rtl-child", "RTL child").parent("rtl"),
                    ]),
                sapid::file_tree(
                    "package-files",
                    [
                        sapid::FileEntry {
                            key: "package".into(),
                            parent: None,
                            path: "package".into(),
                            kind: TreeKind::Branch,
                            disabled: false,
                        },
                        sapid::FileEntry {
                            key: "config".into(),
                            parent: Some("package".into()),
                            path: "package/config.toml".into(),
                            kind: TreeKind::Leaf,
                            disabled: false,
                        },
                    ],
                )?
                .theme(theme.clone())
                .name("Package files")
                .default_expanded(BTreeSet::from(["package".into()])),
            ))
            .show_in(scope)
    })
}
