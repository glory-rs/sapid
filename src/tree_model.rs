//! Bounded current tree data. Validation and flattening happen on explicit
//! source/expansion changes; scrolling never visits this model.
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_TREE_NODES: usize = 2048;
pub const MAX_TREE_DEPTH: usize = 64;
pub const MAX_TREE_LABEL_BYTES: usize = 4096;
pub const MAX_TREE_PAYLOAD_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKind {
    Leaf,
    Branch,
    Lazy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub key: String,
    pub parent: Option<String>,
    pub label: String,
    /// Lazy nodes have no loaded children. The application replaces them with
    /// Branch and publishes children after its own Resource finishes.
    pub kind: TreeKind,
    pub disabled: bool,
}
impl TreeNode {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            parent: None,
            label: label.into(),
            kind: TreeKind::Leaf,
            disabled: false,
        }
    }
    pub fn parent(mut self, key: impl Into<String>) -> Self {
        self.parent = Some(key.into());
        self
    }
    pub fn kind(mut self, kind: TreeKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeError {
    Nodes,
    Key,
    DuplicateKey,
    Label,
    Payload,
    Parent,
    CycleOrDepth,
    State,
    Path,
}
impl TreeError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Nodes => "tree current data exceeds 2048 nodes",
            Self::Key => "tree keys require 1..64 semantic ASCII bytes",
            Self::DuplicateKey => "tree current keys must be unique",
            Self::Label => "tree labels require nonempty canonical text of at most 4096 bytes",
            Self::Payload => "tree current text payload exceeds 2 MiB",
            Self::Parent => "tree parents must name loaded branches in the current model",
            Self::CycleOrDepth => "tree parents contain a cycle or exceed 64 levels",
            Self::State => "tree state exceeds its finite selection or key budget",
            Self::Path => {
                "hierarchical paths require bounded distinct segments and complete current ancestry"
            }
        }
    }
}
pub fn valid_tree_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 64 && glory::render::SemanticId::new(key).is_ok()
}
pub fn validate_tree_keys(keys: &BTreeSet<String>) -> Result<(), TreeError> {
    if keys.len() > MAX_TREE_NODES || keys.iter().any(|key| !valid_tree_key(key)) {
        Err(TreeError::State)
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    pub key: String,
    pub index: usize,
    pub level: u8,
    pub position: u16,
    pub count: u16,
    pub parent: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeModel {
    pub(crate) nodes: Vec<TreeNode>,
    indices: BTreeMap<String, usize>,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    parents: Vec<Option<usize>>,
    levels: Vec<u8>,
}
impl TreeModel {
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }
    /// Admit a borrowed business value before cloning any labels.
    pub fn validate(nodes: &[TreeNode]) -> Result<(), TreeError> {
        if nodes.len() > MAX_TREE_NODES {
            return Err(TreeError::Nodes);
        }
        let mut indices = BTreeMap::new();
        let mut bytes = 0usize;
        for (index, node) in nodes.iter().enumerate() {
            if !valid_tree_key(&node.key)
                || node.parent.as_ref().is_some_and(|key| !valid_tree_key(key))
            {
                return Err(TreeError::Key);
            }
            if indices.insert(node.key.as_str(), index).is_some() {
                return Err(TreeError::DuplicateKey);
            }
            if node.label.len() > MAX_TREE_LABEL_BYTES
                || node.label.trim().is_empty()
                || node.label.contains(['\0', '\r'])
            {
                return Err(TreeError::Label);
            }
            bytes = bytes.saturating_add(
                node.label.len() + node.key.len() + node.parent.as_ref().map_or(0, String::len),
            );
            if bytes > MAX_TREE_PAYLOAD_BYTES {
                return Err(TreeError::Payload);
            }
        }
        for node in nodes {
            let mut parent = node.parent.as_deref();
            let mut depth = 1;
            while let Some(key) = parent {
                let index = indices.get(key).copied().ok_or(TreeError::Parent)?;
                if nodes[index].kind != TreeKind::Branch {
                    return Err(TreeError::Parent);
                }
                depth += 1;
                if depth > MAX_TREE_DEPTH {
                    return Err(TreeError::CycleOrDepth);
                }
                parent = nodes[index].parent.as_deref();
            }
        }
        Ok(())
    }
    pub fn new(nodes: impl IntoIterator<Item = TreeNode>) -> Result<Self, TreeError> {
        let nodes = nodes
            .into_iter()
            .take(MAX_TREE_NODES + 1)
            .collect::<Vec<_>>();
        if nodes.len() > MAX_TREE_NODES {
            return Err(TreeError::Nodes);
        }
        let mut indices = BTreeMap::new();
        let mut bytes = 0usize;
        for (index, node) in nodes.iter().enumerate() {
            if !valid_tree_key(&node.key)
                || node.parent.as_ref().is_some_and(|key| !valid_tree_key(key))
            {
                return Err(TreeError::Key);
            }
            if indices.insert(node.key.clone(), index).is_some() {
                return Err(TreeError::DuplicateKey);
            }
            if node.label.len() > MAX_TREE_LABEL_BYTES
                || node.label.trim().is_empty()
                || node.label.contains(['\0', '\r'])
            {
                return Err(TreeError::Label);
            }
            bytes = bytes.saturating_add(
                node.label.len() + node.key.len() + node.parent.as_ref().map_or(0, String::len),
            );
            if bytes > MAX_TREE_PAYLOAD_BYTES {
                return Err(TreeError::Payload);
            }
        }
        let mut children = vec![Vec::new(); nodes.len()];
        let mut roots = Vec::new();
        let mut parents = vec![None; nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            if let Some(key) = &node.parent {
                let parent = indices.get(key).copied().ok_or(TreeError::Parent)?;
                if nodes[parent].kind != TreeKind::Branch {
                    return Err(TreeError::Parent);
                }
                parents[index] = Some(parent);
                children[parent].push(index);
            } else {
                roots.push(index);
            }
        }
        let mut levels = vec![0; nodes.len()];
        for (index, level) in levels.iter_mut().enumerate() {
            let mut cursor = parents[index];
            let mut depth = 1;
            while let Some(parent) = cursor {
                depth += 1;
                if depth > MAX_TREE_DEPTH {
                    return Err(TreeError::CycleOrDepth);
                }
                cursor = parents[parent];
            }
            *level = depth as u8;
        }
        Ok(Self {
            nodes,
            indices,
            children,
            roots,
            parents,
            levels,
        })
    }
    pub fn index(&self, key: &str) -> Option<usize> {
        self.indices.get(key).copied()
    }
    pub fn children(&self, index: usize) -> &[usize] {
        self.children.get(index).map_or(&[], Vec::as_slice)
    }
    pub fn parent(&self, index: usize) -> Option<usize> {
        self.parents.get(index).copied().flatten()
    }
    pub(crate) fn validate_column_path(
        &self,
        expanded: &BTreeSet<String>,
    ) -> Result<(), TreeError> {
        if expanded.len() > MAX_TREE_DEPTH
            || self.roots.len() > 64
            || self.children.iter().any(|children| children.len() > 64)
        {
            return Err(TreeError::State);
        }
        let deepest = expanded
            .iter()
            .map(|key| self.index(key).ok_or(TreeError::Path))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max_by_key(|index| self.levels[*index]);
        let mut expected = BTreeSet::new();
        let mut current = deepest;
        while let Some(index) = current {
            expected.insert(self.nodes[index].key.clone());
            current = self.parents[index];
        }
        if expected != *expanded {
            return Err(TreeError::Path);
        }
        Ok(())
    }
    pub fn visible(&self, expanded: &BTreeSet<String>) -> Result<Vec<TreeRow>, TreeError> {
        validate_tree_keys(expanded)?;
        let mut result = Vec::new();
        let mut stack = self
            .roots
            .iter()
            .enumerate()
            .rev()
            .map(|(position, index)| (*index, position, self.roots.len()))
            .collect::<Vec<_>>();
        while let Some((index, position, count)) = stack.pop() {
            result.push(TreeRow {
                key: self.nodes[index].key.clone(),
                index,
                level: self.levels[index],
                position: position as u16,
                count: count as u16,
                parent: self.parents[index],
            });
            if expanded.contains(&self.nodes[index].key) {
                let children = &self.children[index];
                stack.extend(
                    children
                        .iter()
                        .enumerate()
                        .rev()
                        .map(|(position, child)| (*child, position, children.len())),
                );
            }
        }
        Ok(result)
    }
}

/// FileTree is an explicit data adapter, with no filesystem or transport access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub key: String,
    pub parent: Option<String>,
    pub path: String,
    pub kind: TreeKind,
    pub disabled: bool,
}
pub fn file_tree_nodes(
    entries: impl IntoIterator<Item = FileEntry>,
) -> Result<Vec<TreeNode>, TreeError> {
    let mut nodes = Vec::new();
    let mut paths = BTreeSet::new();
    let mut key_paths = BTreeMap::new();
    for entry in entries.into_iter().take(MAX_TREE_NODES + 1) {
        if nodes.len() == MAX_TREE_NODES {
            return Err(TreeError::Nodes);
        }
        if !valid_tree_key(&entry.key)
            || entry
                .parent
                .as_ref()
                .is_some_and(|key| !valid_tree_key(key))
        {
            return Err(TreeError::Key);
        }
        if entry.path.is_empty()
            || entry.path.len() > 1024
            || entry.path.contains(['\\', '\0', '\r', '\n', ':'])
            || entry.path.split('/').any(|part| {
                part.is_empty() || matches!(part, "." | "..") || part.trim() != part
            })
            || !paths.insert(entry.path.clone())
        {
            return Err(TreeError::Path);
        }
        let label = entry.path.rsplit('/').next().expect("nonempty").to_owned();
        key_paths.insert(entry.key.clone(), entry.path);
        nodes.push(TreeNode {
            key: entry.key,
            parent: entry.parent,
            label,
            kind: entry.kind,
            disabled: entry.disabled,
        });
    }
    for node in &nodes {
        let path = &key_paths[&node.key];
        match (&node.parent, path.rsplit_once('/')) {
            (None, None) => {}
            (Some(parent), Some((prefix, _)))
                if key_paths.get(parent).is_some_and(|path| path == prefix) => {}
            _ => return Err(TreeError::Path),
        }
    }
    TreeModel::new(nodes.clone())?;
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn nodes() -> Vec<TreeNode> {
        vec![
            TreeNode::new("src", "src").kind(TreeKind::Branch),
            TreeNode::new("main", "main.rs").parent("src"),
            TreeNode::new("lib", "lib.rs").parent("src"),
            TreeNode::new("tests", "tests").kind(TreeKind::Lazy),
        ]
    }
    #[test]
    fn sibling_coordinates_and_visible_order_follow_current_data() {
        let model = TreeModel::new(nodes()).unwrap();
        assert_eq!(
            model
                .visible(&BTreeSet::new())
                .unwrap()
                .iter()
                .map(|r| r.index)
                .collect::<Vec<_>>(),
            [0, 3]
        );
        let rows = model.visible(&BTreeSet::from(["src".into()])).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        assert_eq!(
            (
                rows[2].level,
                rows[2].position,
                rows[2].count,
                rows[2].parent
            ),
            (2, 1, 2, Some(0))
        );
        assert_eq!(model.index("lib"), Some(2));
    }
    #[test]
    fn cycles_missing_and_unloaded_parents_are_rejected() {
        let mut data = nodes();
        data[0].parent = Some("src".into());
        assert_eq!(TreeModel::new(data), Err(TreeError::CycleOrDepth));
        let mut data = nodes();
        data[1].parent = Some("unknown".into());
        assert_eq!(TreeModel::new(data), Err(TreeError::Parent));
        let mut data = nodes();
        data[1].parent = Some("tests".into());
        assert_eq!(TreeModel::new(data), Err(TreeError::Parent));
        let deep = (0..33).map(|i| {
            TreeNode::new(format!("k{i}"), "node")
                .kind(TreeKind::Branch)
                .parent(if i == 0 {
                    "k32".into()
                } else {
                    format!("k{}", i - 1)
                })
        });
        assert_eq!(TreeModel::new(deep), Err(TreeError::CycleOrDepth));
    }
    #[test]
    fn independent_key_text_payload_and_iterator_limits() {
        let mut data = nodes();
        data[1].key = "src".into();
        assert_eq!(TreeModel::new(data), Err(TreeError::DuplicateKey));
        assert_eq!(
            TreeModel::new([TreeNode::new("bad/key", "label")]),
            Err(TreeError::Key)
        );
        assert_eq!(
            TreeModel::new([TreeNode::new("key", "\0")]),
            Err(TreeError::Label)
        );
        assert_eq!(
            TreeModel::new((0..600).map(|i| TreeNode::new(format!("k{i}"), "a".repeat(4096)))),
            Err(TreeError::Payload)
        );
        let calls = std::cell::Cell::new(0);
        assert_eq!(
            TreeModel::new((0..1_000_000).map(|i| {
                calls.set(calls.get() + 1);
                TreeNode::new(format!("k{i}"), "x")
            })),
            Err(TreeError::Nodes)
        );
        assert_eq!(calls.get(), MAX_TREE_NODES + 1);
    }
    #[test]
    fn replacing_a_lazy_branch_has_no_visited_key_history() {
        let mut data = nodes();
        data[3].kind = TreeKind::Branch;
        data.push(TreeNode::new("case", "case.rs").parent("tests"));
        let model = TreeModel::new(data).unwrap();
        let expanded = BTreeSet::from(["tests".into(), "future".into()]);
        assert_eq!(
            model
                .visible(&expanded)
                .unwrap()
                .iter()
                .map(|r| r.index)
                .collect::<Vec<_>>(),
            [0, 3, 4]
        );
        assert!(model.index("future").is_none());
        assert_eq!(
            TreeModel::new(nodes())
                .unwrap()
                .visible(&expanded)
                .unwrap()
                .len(),
            2
        );
    }
    #[test]
    fn file_adapter_requires_complete_relative_paths_and_explicit_parents() {
        let root = FileEntry {
            key: "src".into(),
            parent: None,
            path: "src".into(),
            kind: TreeKind::Branch,
            disabled: false,
        };
        let child = FileEntry {
            key: "main".into(),
            parent: Some("src".into()),
            path: "src/main.rs".into(),
            kind: TreeKind::Leaf,
            disabled: false,
        };
        assert_eq!(
            file_tree_nodes([root.clone(), child.clone()]).unwrap()[1].label,
            "main.rs"
        );
        for path in ["../secret", "/root", "a//b", "a\\b", "a/./b", "a:", "a/ "] {
            let mut bad = child.clone();
            bad.path = path.into();
            assert_eq!(file_tree_nodes([root.clone(), bad]), Err(TreeError::Path));
        }
        let mut bad = child;
        bad.parent = None;
        assert_eq!(file_tree_nodes([root, bad]), Err(TreeError::Path));
    }
}
