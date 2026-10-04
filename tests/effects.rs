//! Effect behaviour. An effect that can't be applied (bad arithmetic, mismatched
//! types, arithmetic on a missing key) makes its action fail, so a parent selector
//! falls back to its next child.

use htn::prelude::*;
use htn::{ArithmeticOp, Condition, Effect, Value};

fn steps(names: &[&str]) -> Result<Vec<String>, PlanError> {
    Ok(names.iter().map(|s| s.to_string()).collect())
}

/// Plans an action with `effect`, then an action that only runs if `check` holds.
fn apply_then_check(
    effect: Effect,
    check: Condition,
    bb: &std::collections::HashMap<String, Value>,
) -> Result<Vec<String>, PlanError> {
    let root = sequence!(
        conditions = [],
        tasks = [
            action!("apply", conditions = [], effects = [effect]),
            action!("check", conditions = [check]),
        ]
    );
    plan(&root, bb)
}

/// Asserts that the action carrying `effect` fails and the selector falls back.
fn assert_action_fails(effect: Effect, bb: &std::collections::HashMap<String, Value>) {
    let root = selector!(
        conditions = [],
        tasks = [
            action!("risky", conditions = [], effects = [effect.clone()]),
            action!("fallback"),
        ]
    );
    assert_eq!(plan(&root, bb), steps(&["fallback"]), "effect {effect:?}");
}

// ---- valid effects -----------------------------------------------------------

#[test]
fn int_arithmetic_updates_the_blackboard() {
    let bb = blackboard! { "x" => 10 };

    assert_eq!(
        apply_then_check(effect!("x" + 5), cond!("x" == 15), &bb),
        steps(&["apply", "check"])
    );
    assert_eq!(
        apply_then_check(effect!("x" - 5), cond!("x" == 5), &bb),
        steps(&["apply", "check"])
    );
    assert_eq!(
        apply_then_check(effect!("x" * 3), cond!("x" == 30), &bb),
        steps(&["apply", "check"])
    );
    assert_eq!(
        apply_then_check(effect!("x" / 2), cond!("x" == 5), &bb),
        steps(&["apply", "check"])
    );
}

#[test]
fn int_plus_float_becomes_float() {
    let bb = blackboard! { "x" => 1 };

    assert_eq!(
        apply_then_check(effect!("x" + 0.5), cond!("x" == 1.5), &bb),
        steps(&["apply", "check"])
    );
}

#[test]
fn set_creates_a_missing_key() {
    assert_eq!(
        apply_then_check(effect!("gold" = 5), cond!("gold" == 5), &blackboard! {}),
        steps(&["apply", "check"])
    );
}

#[test]
fn set_can_change_a_keys_type() {
    let bb = blackboard! { "target" => 0 };

    assert_eq!(
        apply_then_check(
            effect!("target" = "player"),
            cond!("target" == "player"),
            &bb
        ),
        steps(&["apply", "check"])
    );
}

// ---- invalid arithmetic ------------------------------------------------------

#[test]
fn int_divide_by_zero_fails_the_action() {
    assert_action_fails(effect!("x" / 0), &blackboard! { "x" => 10 });
}

#[test]
fn int_add_overflow_fails_the_action() {
    assert_action_fails(effect!("x" + 1), &blackboard! { "x" => i32::MAX });
}

#[test]
fn int_sub_underflow_fails_the_action() {
    assert_action_fails(effect!("x" - 1), &blackboard! { "x" => i32::MIN });
}

#[test]
fn int_mul_overflow_fails_the_action() {
    assert_action_fails(effect!("x" * 2), &blackboard! { "x" => i32::MAX });
}

#[test]
fn int_div_overflow_fails_the_action() {
    // i32::MIN / -1 doesn't fit in an i32.
    assert_action_fails(effect!("x" / -1), &blackboard! { "x" => i32::MIN });
}

#[test]
fn failed_action_fails_its_sequence() {
    let root = sequence!(
        conditions = [],
        tasks = [
            action!("aim"),
            action!("fire", conditions = [], effects = [effect!("ammo" / 0)]),
        ]
    );

    assert_eq!(
        plan(&root, &blackboard! { "ammo" => 3 }),
        Err(PlanError::NoPlan)
    );
}

// ---- type mismatches ---------------------------------------------------------

#[test]
fn string_plus_int_fails_the_action() {
    assert_action_fails(effect!("name" + 5), &blackboard! { "name" => "bob" });
}

#[test]
fn int_plus_string_fails_the_action() {
    assert_action_fails(effect!("x" + "five"), &blackboard! { "x" => 1 });
}

#[test]
fn bool_arithmetic_fails_the_action() {
    assert_action_fails(effect!("flag" + 1), &blackboard! { "flag" => true });
    assert_action_fails(effect!("flag" + true), &blackboard! { "flag" => true });
    assert_action_fails(effect!("x" * false), &blackboard! { "x" => 2 });
}

#[test]
fn vector_plus_int_fails_the_action() {
    let effect = Effect::new("pos", ArithmeticOp::Add, Value::Int(1));

    assert_action_fails(effect, &blackboard! { "pos" => Value::Vector2(0.0, 0.0) });
}

// ---- arithmetic on missing keys ----------------------------------------------

#[test]
fn arithmetic_on_missing_key_fails_the_action() {
    for effect in [
        effect!("gold" + 5),
        effect!("gold" - 5),
        effect!("gold" * 5),
        effect!("gold" / 5),
    ] {
        assert_action_fails(effect, &blackboard! {});
    }
}
