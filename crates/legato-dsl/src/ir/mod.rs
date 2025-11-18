use std::{collections::BTreeSet, time::Duration};

use crate::ast::{Object, Value};

pub mod params;
pub mod registry;
/// ValidationError covers logical issues
/// when lowering from the AST to the IR.
///
/// Typically, these might be bad parameters,
/// bad values, nodes that don't exist, etc.
#[derive(Clone, PartialEq, Debug)]
pub enum ValidationError {
    NodeNotFound(String),
    InvalidParameter(String),
    MissingRequiredParameters(String),
    MissingRequiredParameter(String),
}
