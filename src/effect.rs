use std::collections::HashMap;

use super::{error::EffectError, value::Value};

#[derive(Clone, Debug, PartialEq)]
pub enum ArithmeticOp {
    Eq,
    Add,
    Sub,
    Mult,
    Div,
}

/// Integer and float versions of one arithmetic operator.
type ArithmeticFns = (fn(i32, i32) -> Option<i32>, fn(f32, f32) -> f32);

#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    blackboard_key: String,
    op: ArithmeticOp,
    value: Value,
}

impl Effect {
    pub fn new(key: impl Into<String>, op: ArithmeticOp, value: Value) -> Self {
        Self {
            blackboard_key: key.into(),
            op,
            value,
        }
    }

    /// Applies the effect to the blackboard.
    ///
    /// `Eq` always succeeds. Arithmetic fails if the key is missing, the types
    /// can't be combined, or integer arithmetic overflows or divides by zero.
    /// On failure the blackboard is left unchanged.
    pub fn apply(&self, data: &mut HashMap<String, Value>) -> Result<(), EffectError> {
        let key = &self.blackboard_key;

        let (int_op, float_op): ArithmeticFns = match self.op {
            ArithmeticOp::Eq => {
                data.insert(key.clone(), self.value.clone());
                return Ok(());
            }
            ArithmeticOp::Add => (i32::checked_add, |a, b| a + b),
            ArithmeticOp::Sub => (i32::checked_sub, |a, b| a - b),
            ArithmeticOp::Mult => (i32::checked_mul, |a, b| a * b),
            ArithmeticOp::Div => (i32::checked_div, |a, b| a / b),
        };

        let current = data
            .get(key)
            .ok_or_else(|| EffectError::MissingKey(key.clone()))?;

        let new_value = match (current, &self.value) {
            (Value::Int(a), Value::Int(b)) => int_op(*a, *b)
                .map(Value::Int)
                .ok_or_else(|| EffectError::InvalidArithmetic(key.clone()))?,
            (Value::Float(a), Value::Float(b)) => Value::Float(float_op(*a, *b)),
            (Value::Int(a), Value::Float(b)) => Value::Float(float_op(*a as f32, *b)),
            (Value::Float(a), Value::Int(b)) => Value::Float(float_op(*a, *b as f32)),
            _ => return Err(EffectError::TypeMismatch(key.clone())),
        };

        data.insert(key.clone(), new_value);
        Ok(())
    }
}
