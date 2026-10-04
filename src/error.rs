use std::{error::Error, fmt};

/// Why [`plan`](crate::plan) could not produce a plan.
#[derive(Clone, Debug, PartialEq)]
pub enum PlanError {
    /// No branch of the task network could be decomposed with the current blackboard.
    NoPlan,
    /// A condition checked a key that is not on the blackboard.
    MissingKey(String),
    /// A selector or sequence with no child tasks was reached while planning.
    EmptyTask,
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPlan => write!(f, "no valid plan for the current blackboard"),
            Self::MissingKey(key) => write!(f, "condition checks missing blackboard key `{key}`"),
            Self::EmptyTask => write!(f, "selector or sequence has no child tasks"),
        }
    }
}

impl Error for PlanError {}

/// Why an [`Effect`](crate::Effect) could not be applied to the blackboard.
///
/// During planning, an effect error makes its action fail, so a parent selector
/// moves on to its next child.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectError {
    /// Arithmetic was applied to a key that is not on the blackboard.
    MissingKey(String),
    /// The key's current value can't be combined with the effect's value.
    TypeMismatch(String),
    /// Integer arithmetic on the key overflowed or divided by zero.
    InvalidArithmetic(String),
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key) => write!(f, "effect changes missing blackboard key `{key}`"),
            Self::TypeMismatch(key) => write!(f, "effect value has the wrong type for key `{key}`"),
            Self::InvalidArithmetic(key) => {
                write!(f, "integer overflow or division by zero on key `{key}`")
            }
        }
    }
}

impl Error for EffectError {}
