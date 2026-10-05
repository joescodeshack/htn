use std::{collections::HashMap, mem::discriminant};

use super::{error::PlanError, value::Value};

#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonOp {
    E,
    NE,
    LT,
    GT,
    GTE,
    LTE,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Compare {
        blackboard_key: String,
        op: ComparisonOp,
        value: Value,
    },
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

impl Condition {
    pub fn compare(key: impl Into<String>, op: ComparisonOp, value: Value) -> Self {
        Self::Compare {
            blackboard_key: key.into(),
            op,
            value,
        }
    }
}

/// Evaluates `cond` against the blackboard.
///
/// Comparing a key that isn't on the blackboard, or comparing values of different
/// types, is an error. `All` and `Any` short-circuit, so conditions after the
/// deciding one are not checked.
pub fn eval(cond: &Condition, data: &HashMap<String, Value>) -> Result<bool, PlanError> {
    match cond {
        Condition::Compare {
            blackboard_key,
            op,
            value,
        } => {
            let left = data.get(blackboard_key).ok_or(PlanError::MissingKey)?;
            if discriminant(left) != discriminant(value) {
                return Err(PlanError::TypeMismatch);
            }
            Ok(match op {
                ComparisonOp::E => left == value,
                ComparisonOp::NE => left != value,
                ComparisonOp::LT => left < value,
                ComparisonOp::GT => left > value,
                ComparisonOp::GTE => left >= value,
                ComparisonOp::LTE => left <= value,
            })
        }
        Condition::All(cs) => {
            for c in cs {
                if !eval(c, data)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Condition::Any(cs) => {
            for c in cs {
                if eval(c, data)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Condition::Not(c) => Ok(!eval(c, data)?),
    }
}
