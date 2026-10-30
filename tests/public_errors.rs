//! Application-style propagation through the public APIs and a standard error boundary.
use sapid::{DataTableError, TreeError, TreeModel, TreeNode, data_table::validate_data_columns};
use std::error::Error;

#[test]
fn data_validation_propagates_with_question_mark_and_keeps_its_type() {
    fn load() -> Result<(), Box<dyn Error>> {
        validate_data_columns(&[])?;
        Ok(())
    }
    let error = load().unwrap_err();
    assert_eq!(error.to_string(), DataTableError::Columns.message());
    assert_eq!(error.downcast_ref(), Some(&DataTableError::Columns));
}

#[test]
fn tree_construction_propagates_with_question_mark_and_keeps_its_type() {
    fn load() -> Result<TreeModel, Box<dyn Error>> {
        Ok(TreeModel::new([TreeNode::new("", "Invalid key")])?)
    }
    let error = load().unwrap_err();
    assert_eq!(error.to_string(), TreeError::Key.message());
    assert_eq!(error.downcast_ref(), Some(&TreeError::Key));
}

#[cfg(feature = "charts")]
#[test]
fn chart_domain_propagates_with_question_mark_and_keeps_its_type() {
    use sapid::charts::{ChartDomain, ChartError};
    fn load() -> Result<ChartDomain, Box<dyn Error>> {
        Ok(ChartDomain::new(1.0, 1.0)?)
    }
    let error = load().unwrap_err();
    assert_eq!(error.to_string(), ChartError::Domain.message());
    assert_eq!(error.downcast_ref(), Some(&ChartError::Domain));
}
