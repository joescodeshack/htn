/// Why [`plan`](crate::plan) could not produce a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanError {
    /// No branch of the task network could be decomposed with the current blackboard.
    NoPlan,
    /// A condition checked a key that is not on the blackboard.
    MissingKey,
    /// A selector or sequence with no child tasks was reached while planning.
    EmptyTask,
    /// A condition compared a blackboard value with a value of a different type.
    TypeMismatch,
}

/// Why an [`Effect`](crate::Effect) could not be applied to the blackboard.
///
/// During planning, an effect error makes its action fail, so a parent selector
/// moves on to its next child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectError {
    /// Arithmetic was applied to a key that is not on the blackboard.
    MissingKey,
    /// The key's current value can't be combined with the effect's value.
    TypeMismatch,
    /// Integer arithmetic overflowed or divided by zero.
    InvalidArithmetic,
}

/// Why [`parse`](crate::parse) rejected its input.
///
/// Each variant carries the byte offset in the source where the problem starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// A character that the grammar doesn't allow at this position.
    UnexpectedChar { offset: usize },
    /// The input ended before the task network was complete.
    UnexpectedEnd { offset: usize },
    /// An integer literal that does not fit in an `i32`.
    IntOutOfRange { offset: usize },
}
