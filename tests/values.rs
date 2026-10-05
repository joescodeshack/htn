//! `Value` equality and ordering, and how they show up in conditions.

use std::cmp::Ordering;

use htn::prelude::*;
use htn::{ArithmeticOp, ComparisonOp, Condition, Effect, Value, parse};

// ---- Int / Float -------------------------------------------------------------
//
// Values of different types are never equal or ordered, even when they hold the
// same number. Conditions that compare them return an error instead (see
// `tests/planner.rs`).

#[test]
fn int_does_not_equal_float_with_the_same_value() {
    assert_ne!(Value::Int(1), Value::Float(1.0));
    assert_ne!(Value::Float(1.0), Value::Int(1));
}

#[test]
fn int_does_not_equal_float_with_a_different_value() {
    assert_ne!(Value::Int(1), Value::Float(1.5));
    assert_ne!(Value::Float(1.5), Value::Int(1));
}

#[test]
fn int_and_float_are_unordered() {
    assert_eq!(Value::Int(1).partial_cmp(&Value::Float(1.0)), None);
    assert_eq!(Value::Float(0.5).partial_cmp(&Value::Int(1)), None);
}

#[test]
fn conditions_with_int_and_float_values_are_not_equal() {
    let int = Condition::compare("x", ComparisonOp::E, Value::Int(1));
    let float = Condition::compare("x", ComparisonOp::E, Value::Float(1.0));

    assert_ne!(int, float);
}

#[test]
fn effects_with_int_and_float_values_are_not_equal() {
    let int = Effect::new("x", ArithmeticOp::Add, Value::Int(1));
    let float = Effect::new("x", ArithmeticOp::Add, Value::Float(1.0));

    assert_ne!(int, float);
}

#[test]
fn tasks_parsed_with_int_and_float_values_are_not_equal() {
    let int = parse(r#"action "a" (x == 1) [ y = 2 ]"#).unwrap();
    let float = parse(r#"action "a" (x == 1.0) [ y = 2.0 ]"#).unwrap();

    assert_ne!(int, float);
}

#[test]
fn int_condition_against_float_blackboard_value_is_an_error() {
    let root = action!("a", conditions = [cond!("x" == 1)]);

    assert_eq!(
        plan(&root, &blackboard! { "x" => 1.0 }),
        Err(PlanError::TypeMismatch)
    );
}

// ---- vectors -----------------------------------------------------------------

#[test]
fn vectors_equal_themselves() {
    assert_eq!(Value::Vector2(1.0, 2.0), Value::Vector2(1.0, 2.0));
    assert_eq!(Value::Vector3(1.0, 2.0, 3.0), Value::Vector3(1.0, 2.0, 3.0));
}

#[test]
fn different_vectors_are_not_equal() {
    assert_ne!(Value::Vector2(1.0, 2.0), Value::Vector2(2.0, 1.0));
    assert_ne!(Value::Vector3(1.0, 2.0, 3.0), Value::Vector3(1.0, 2.0, 4.0));
    assert_ne!(Value::Vector2(1.0, 2.0), Value::Vector3(1.0, 2.0, 0.0));
    assert_ne!(Value::Vector2(1.0, 2.0), Value::Int(1));
}

#[test]
fn conditions_with_vector_values_compare_equal() {
    let a = Condition::compare("pos", ComparisonOp::E, Value::Vector3(1.0, 2.0, 3.0));
    let b = Condition::compare("pos", ComparisonOp::E, Value::Vector3(1.0, 2.0, 3.0));

    assert_eq!(a, b);
}

#[test]
fn vector_condition_matches_blackboard_value() {
    let root = action!("a", conditions = [cond!("pos" == Value::Vector2(3.0, 4.0))]);

    assert_eq!(
        plan(&root, &blackboard! { "pos" => Value::Vector2(3.0, 4.0) }),
        Ok(vec!["a".to_string()])
    );
}

// ---- == agrees with partial_cmp ----------------------------------------------

#[test]
fn eq_agrees_with_partial_cmp() {
    // `PartialOrd` requires: a == b  <=>  a.partial_cmp(b) == Some(Equal).
    let pairs = [
        (Value::Int(1), Value::Float(1.0)),
        (Value::Int(1), Value::Float(2.0)),
        (Value::Int(2), Value::Int(2)),
        (Value::Bool(true), Value::Bool(true)),
        (Value::String("a".into()), Value::String("a".into())),
        (Value::Vector2(1.0, 2.0), Value::Vector2(1.0, 2.0)),
        (Value::Vector2(1.0, 2.0), Value::Vector2(2.0, 1.0)),
        (Value::Vector3(1.0, 2.0, 3.0), Value::Vector3(1.0, 2.0, 3.0)),
        (Value::Int(1), Value::Bool(true)),
    ];

    for (a, b) in pairs {
        assert_eq!(
            a == b,
            a.partial_cmp(&b) == Some(Ordering::Equal),
            "{a:?} vs {b:?}"
        );
    }
}
