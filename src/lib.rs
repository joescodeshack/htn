mod condition;
mod effect;
mod error;
mod macros;
mod parser;
mod task;
mod value;

pub use condition::{ComparisonOp, Condition};
pub use effect::{ArithmeticOp, Effect};
pub use error::{EffectError, PlanError};
pub use task::{Task, plan};
pub use value::Value;

pub use parser::parse;

/// Everything needed to build and plan with a task network.
///
/// ```
/// use htn::prelude::*;
///
/// let root = selector!(
///     conditions = [],
///     tasks = [
///         action!("heal", conditions = [cond!("health" < 50)]),
///         action!("attack"),
///     ]
/// );
/// let blackboard = blackboard! { "health" => 40 };
///
/// assert_eq!(plan(&root, &blackboard), Ok(vec!["heal".to_string()]));
/// ```
pub mod prelude {
    pub use crate::error::PlanError;
    pub use crate::task::{Task, plan};
    pub use crate::{action, blackboard, cond, effect, selector, sequence};
}
