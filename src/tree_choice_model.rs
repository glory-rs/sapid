//! Full ancestry values for finite hierarchical choices. No labels, visited
//! nodes, host identities or loading results are retained in a selection.
use crate::tree_model::{valid_tree_key, *};
use glory::render::TreeCheckState;
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

pub const MAX_TREE_CHOICES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreePath(Vec<String>);
impl TreePath {
    pub fn new(keys: impl IntoIterator<Item = String>) -> Result<Self, TreeError> {
        let keys = keys
            .into_iter()
            .take(MAX_TREE_DEPTH + 1)
            .collect::<Vec<_>>();
        if keys.is_empty()
            || keys.len() > MAX_TREE_DEPTH
            || keys.iter().any(|key| !valid_tree_key(key))
            || keys.iter().collect::<BTreeSet<_>>().len() != keys.len()
        {
            return Err(TreeError::Path);
        }
        Ok(Self(keys))
    }
    pub fn keys(&self) -> &[String] {
        &self.0
    }
    pub fn leaf(&self) -> &str {
        self.0.last().expect("nonempty validated path")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeChoices(Vec<TreePath>);
impl TreeChoices {
    pub fn new(paths: impl IntoIterator<Item = TreePath>) -> Result<Self, TreeError> {
        let mut paths = paths
            .into_iter()
            .take(MAX_TREE_CHOICES + 1)
            .collect::<Vec<_>>();
        if paths.len() > MAX_TREE_CHOICES {
            return Err(TreeError::State);
        }
        paths.sort();
        if paths.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(TreeError::State);
        }
        Ok(Self(paths))
    }
    pub fn paths(&self) -> &[TreePath] {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|path| path.leaf() == key)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeChoicePolicy {
    /// A loaded branch or lazy node may itself be the submitted value.
    AnyNode,
    #[default]
    Leaves,
    /// Multiple choice only. A branch toggles its current enabled loaded leaves.
    /// Lazy descendants are neither fabricated nor automatically selected later.
    DescendantLeaves,
}

#[derive(Clone, Debug)]
pub struct TreeChoiceModel {
    tree: Rc<TreeModel>,
    policy: TreeChoicePolicy,
    multiple: bool,
}
impl TreeChoiceModel {
    pub fn new(
        tree: TreeModel,
        policy: TreeChoicePolicy,
        multiple: bool,
    ) -> Result<Self, TreeError> {
        if policy == TreeChoicePolicy::DescendantLeaves && !multiple {
            return Err(TreeError::State);
        }
        Ok(Self {
            tree: Rc::new(tree),
            policy,
            multiple,
        })
    }
    pub fn tree(&self) -> &TreeModel {
        &self.tree
    }
    pub fn policy(&self) -> TreeChoicePolicy {
        self.policy
    }
    pub fn multiple(&self) -> bool {
        self.multiple
    }
    pub(crate) fn validate_columns(
        &self,
        expanded: &BTreeSet<String>,
    ) -> Result<(), TreeError> {
        self.tree.validate_column_path(expanded)
    }
    pub fn path(&self, key: &str) -> Result<TreePath, TreeError> {
        let mut index = self.tree.index(key).ok_or(TreeError::Path)?;
        let mut keys = Vec::new();
        loop {
            keys.push(self.tree.nodes()[index].key.clone());
            let Some(parent) = self.tree.parent(index) else {
                break;
            };
            index = parent;
        }
        keys.reverse();
        TreePath::new(keys)
    }
    pub fn validate(&self, choices: &TreeChoices) -> Result<(), TreeError> {
        if !self.multiple && choices.paths().len() > 1 {
            return Err(TreeError::State);
        }
        for path in choices.paths() {
            if self.path(path.leaf())? != *path {
                return Err(TreeError::Path);
            }
            let index = self.tree.index(path.leaf()).ok_or(TreeError::Path)?;
            if self.policy != TreeChoicePolicy::AnyNode
                && self.tree.nodes()[index].kind != TreeKind::Leaf
            {
                return Err(TreeError::State);
            }
        }
        Ok(())
    }
    fn leaves(&self, root: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut pending = vec![root];
        while let Some(index) = pending.pop() {
            let node = &self.tree.nodes()[index];
            if node.kind == TreeKind::Leaf && !node.disabled {
                result.push(index);
            }
            pending.extend(self.tree.children(index).iter().rev());
        }
        result
    }
    pub fn toggle(&self, current: &TreeChoices, key: &str) -> Result<TreeChoices, TreeError> {
        self.validate(current)?;
        let index = self.tree.index(key).ok_or(TreeError::Path)?;
        let node = &self.tree.nodes()[index];
        if node.disabled {
            return Ok(current.clone());
        }
        let targets = if self.policy == TreeChoicePolicy::DescendantLeaves {
            self.leaves(index)
        } else if self.policy == TreeChoicePolicy::Leaves && node.kind != TreeKind::Leaf {
            Vec::new()
        } else {
            vec![index]
        };
        if targets.is_empty() {
            return Ok(current.clone());
        }
        let remove = self.multiple
            && targets
                .iter()
                .all(|index| current.contains(&self.tree.nodes()[*index].key));
        let mut next = if self.multiple {
            current.0.clone()
        } else {
            Vec::new()
        };
        for index in targets {
            let key = &self.tree.nodes()[index].key;
            next.retain(|path| path.leaf() != key);
            if !remove {
                next.push(self.path(key)?);
            }
            if next.len() > MAX_TREE_CHOICES {
                return Err(TreeError::State);
            }
        }
        TreeChoices::new(next)
    }
    pub fn checked(
        &self,
        choices: &TreeChoices,
    ) -> Result<BTreeMap<String, TreeCheckState>, TreeError> {
        self.validate(choices)?;
        let selected = choices
            .paths()
            .iter()
            .map(TreePath::leaf)
            .collect::<BTreeSet<_>>();
        let mut result = BTreeMap::new();
        // One postorder pass over this admitted finite current model. No scroll
        // callback or key history participates in parent/half-checked derivation.
        let mut counts = vec![(0usize, 0usize); self.tree.nodes().len()];
        let expanded = self
            .tree
            .nodes()
            .iter()
            .filter(|node| node.kind == TreeKind::Branch)
            .map(|node| node.key.clone())
            .collect();
        let rows = self.tree.visible(&expanded)?;
        for row in rows.iter().rev() {
            let node = &self.tree.nodes()[row.index];
            let (mut total, mut hits) = counts[row.index];
            if node.kind == TreeKind::Leaf && !node.disabled {
                total += 1;
                hits += usize::from(selected.contains(node.key.as_str()));
            }
            let checked = if self.policy != TreeChoicePolicy::DescendantLeaves {
                if selected.contains(node.key.as_str()) {
                    TreeCheckState::Checked
                } else {
                    TreeCheckState::Unchecked
                }
            } else if total > 0 && hits == total {
                TreeCheckState::Checked
            } else if hits > 0 {
                TreeCheckState::Mixed
            } else {
                TreeCheckState::Unchecked
            };
            result.insert(node.key.clone(), checked);
            if let Some(parent) = row.parent {
                counts[parent].0 += total;
                counts[parent].1 += hits;
            }
        }
        Ok(result)
    }
    /// Root column followed by only the current loaded branch chain. A lazy
    /// node ends the columns until the application publishes its children.
    pub fn columns(&self, path: Option<&TreePath>) -> Result<Vec<Vec<String>>, TreeError> {
        let mut result = vec![
            self.tree
                .nodes()
                .iter()
                .filter(|node| node.parent.is_none())
                .map(|node| node.key.clone())
                .collect(),
        ];
        if let Some(path) = path {
            if self.path(path.leaf())? != *path {
                return Err(TreeError::Path);
            }
            for key in path.keys() {
                let index = self.tree.index(key).ok_or(TreeError::Path)?;
                let children = self.tree.children(index);
                if children.is_empty() {
                    break;
                }
                result.push(
                    children
                        .iter()
                        .map(|index| self.tree.nodes()[*index].key.clone())
                        .collect(),
                );
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exactly_64_levels_are_valid_in_both_admission_paths_and_65_are_rejected() {
        let nodes = (0..64)
            .map(|i| {
                let mut node = TreeNode::new(format!("k{i}"), "Node").kind(TreeKind::Branch);
                if i > 0 {
                    node.parent = Some(format!("k{}", i - 1));
                }
                node
            })
            .collect::<Vec<_>>();
        TreeModel::validate(&nodes).unwrap();
        let model = TreeChoiceModel::new(
            TreeModel::new(nodes.clone()).unwrap(),
            TreeChoicePolicy::AnyNode,
            false,
        )
        .unwrap();
        let path = model.path("k63").unwrap();
        assert_eq!(path.keys().len(), 64);
        let mut deeper = nodes;
        deeper.push(TreeNode::new("too-deep", "Node").parent("k63"));
        assert_eq!(TreeModel::validate(&deeper), Err(TreeError::CycleOrDepth));
        assert_eq!(TreeModel::new(deeper), Err(TreeError::CycleOrDepth));
    }
    fn model(policy: TreeChoicePolicy, multiple: bool) -> TreeChoiceModel {
        TreeChoiceModel::new(
            TreeModel::new([
                TreeNode::new("root", "Root").kind(TreeKind::Branch),
                TreeNode::new("a", "A").parent("root"),
                TreeNode::new("b", "B").parent("root"),
                TreeNode::new("off", "Disabled")
                    .parent("root")
                    .disabled(true),
                TreeNode::new("lazy", "Lazy")
                    .kind(TreeKind::Lazy)
                    .parent("root"),
            ])
            .unwrap(),
            policy,
            multiple,
        )
        .unwrap()
    }
    #[test]
    fn paths_preserve_full_ancestry_and_reject_reparented_values() {
        let model = model(TreeChoicePolicy::Leaves, false);
        let path = model.path("a").unwrap();
        assert_eq!(path.keys(), &["root", "a"]);
        assert!(
            model
                .validate(&TreeChoices::new([TreePath::new(["a".into()]).unwrap()]).unwrap())
                .is_err()
        );
        assert!(
            model
                .validate(&TreeChoices::new([model.path("root").unwrap()]).unwrap())
                .is_err()
        );
        assert!(
            model
                .validate(&TreeChoices::new([path.clone()]).unwrap())
                .is_ok()
        );
        assert!(TreeChoices::new([path.clone(), path]).is_err());
    }
    #[test]
    fn cascade_uses_only_current_enabled_loaded_leaves_and_derives_mixed() {
        let model = model(TreeChoicePolicy::DescendantLeaves, true);
        let one = model.toggle(&TreeChoices::default(), "a").unwrap();
        assert_eq!(model.checked(&one).unwrap()["root"], TreeCheckState::Mixed);
        let all = model.toggle(&one, "root").unwrap();
        assert_eq!(all.paths().len(), 2);
        assert_eq!(
            model.checked(&all).unwrap()["root"],
            TreeCheckState::Checked
        );
        assert!(!all.contains("off"));
        assert!(!all.contains("lazy"));
        assert!(model.toggle(&all, "root").unwrap().is_empty());
        assert_eq!(model.toggle(&all, "off").unwrap(), all);
    }
    #[test]
    fn independent_branches_and_leaf_only_policy_are_distinct() {
        let any = model(TreeChoicePolicy::AnyNode, true);
        let branch = any.toggle(&TreeChoices::default(), "root").unwrap();
        assert_eq!(branch.paths().len(), 1);
        assert!(branch.contains("root"));
        assert_eq!(
            any.checked(&branch).unwrap()["a"],
            TreeCheckState::Unchecked
        );
        let leaves = model(TreeChoicePolicy::Leaves, false);
        assert!(
            leaves
                .toggle(&TreeChoices::default(), "root")
                .unwrap()
                .is_empty()
        );
        let a = leaves.toggle(&TreeChoices::default(), "a").unwrap();
        let b = leaves.toggle(&a, "b").unwrap();
        assert!(!b.contains("a"));
        assert!(b.contains("b"));
    }
    #[test]
    fn columns_stop_at_lazy_and_keep_source_sibling_order() {
        let m = model(TreeChoicePolicy::AnyNode, false);
        assert_eq!(
            m.columns(Some(&m.path("lazy").unwrap())).unwrap(),
            vec![vec!["root"], vec!["a", "b", "off", "lazy"]]
        );
    }
    #[test]
    fn finite_path_selection_and_branch_admission() {
        assert!(TreePath::new(std::iter::empty()).is_err());
        assert!(TreePath::new(["a".into(), "a".into()]).is_err());
        assert!(TreePath::new((0..65).map(|i| format!("k{i}"))).is_err());
        let nodes = (0..129)
            .map(|i| TreeNode::new(format!("k{i}"), "Label"))
            .collect::<Vec<_>>();
        let m = TreeChoiceModel::new(
            TreeModel::new(nodes).unwrap(),
            TreeChoicePolicy::Leaves,
            true,
        )
        .unwrap();
        let accepted =
            TreeChoices::new((0..128).map(|i| m.path(&format!("k{i}")).unwrap())).unwrap();
        assert!(m.toggle(&accepted, "k128").is_err());
        assert_eq!(accepted.paths().len(), 128);
        assert!(
            TreeChoiceModel::new(
                TreeModel::new([]).unwrap(),
                TreeChoicePolicy::DescendantLeaves,
                false
            )
            .is_err()
        );
    }
}
