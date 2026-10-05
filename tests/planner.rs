//! Planner behaviour: selector fallback, root preconditions, empty compound tasks,
//! conditions that check keys missing from the blackboard, and conditions that
//! compare values of different types.

use htn::prelude::*;
use htn::{Condition, Value, parse};

fn steps(names: &[&str]) -> Result<Vec<String>, PlanError> {
    Ok(names.iter().map(|s| s.to_string()).collect())
}

// ---- selector fallback -------------------------------------------------------

#[test]
fn selector_falls_back_when_child_fails_to_decompose() {
    // The sequence's own (empty) preconditions pass, but its only step can't run.
    let root = selector!(
        conditions = [],
        tasks = [
            sequence!(
                conditions = [],
                tasks = [action!(
                    "attack",
                    conditions = [cond!("has_weapon" == true)]
                )]
            ),
            action!("idle"),
        ]
    );
    let bb = blackboard! { "has_weapon" => false };

    assert_eq!(plan(&root, &bb), steps(&["idle"]));
}

#[test]
fn selector_skips_several_failing_children() {
    let root = selector!(
        conditions = [],
        tasks = [
            sequence!(
                conditions = [],
                tasks = [action!("reload", conditions = [cond!("ammo" > 0)])]
            ),
            selector!(
                conditions = [cond!("in_combat" == true)],
                tasks = [action!("shoot", conditions = [cond!("ammo" > 0)])]
            ),
            action!("idle"),
        ]
    );
    let bb = blackboard! { "ammo" => 0, "in_combat" => true };

    assert_eq!(plan(&root, &bb), steps(&["idle"]));
}

#[test]
fn selector_fallback_discards_effects_of_failed_branch() {
    // The first branch applies `armed = true` before failing; the second branch
    // must still see the original blackboard.
    let root = selector!(
        conditions = [],
        tasks = [
            sequence!(
                conditions = [],
                tasks = [
                    action!("arm", conditions = [], effects = [effect!("armed" = true)]),
                    action!("fire", conditions = [cond!("ammo" > 0)]),
                ]
            ),
            action!("flee", conditions = [cond!("armed" == false)]),
        ]
    );
    let bb = blackboard! { "armed" => false, "ammo" => 0 };

    assert_eq!(plan(&root, &bb), steps(&["flee"]));
}

#[test]
fn selector_with_no_valid_child_has_no_plan() {
    let root = selector!(
        conditions = [],
        tasks = [
            action!("heal", conditions = [cond!("health" < 50)]),
            action!("rest", conditions = [cond!("energy" < 50)]),
        ]
    );
    let bb = blackboard! { "health" => 100, "energy" => 100 };

    assert_eq!(plan(&root, &bb), Err(PlanError::NoPlan));
}

// ---- root preconditions ------------------------------------------------------

#[test]
fn root_action_preconditions_are_checked() {
    let root = action!("attack", conditions = [cond!("has_weapon" == true)]);

    assert_eq!(
        plan(&root, &blackboard! { "has_weapon" => false }),
        Err(PlanError::NoPlan)
    );
    assert_eq!(
        plan(&root, &blackboard! { "has_weapon" => true }),
        steps(&["attack"])
    );
}

#[test]
fn root_selector_preconditions_are_checked() {
    let root = selector!(
        conditions = [cond!("in_combat" == true)],
        tasks = [action!("shoot")]
    );

    assert_eq!(
        plan(&root, &blackboard! { "in_combat" => false }),
        Err(PlanError::NoPlan)
    );
    assert_eq!(
        plan(&root, &blackboard! { "in_combat" => true }),
        steps(&["shoot"])
    );
}

#[test]
fn root_sequence_preconditions_are_checked() {
    let root = sequence!(
        conditions = [cond!("ammo" == 0)],
        tasks = [action!("take_cover"), action!("reload")]
    );

    assert_eq!(
        plan(&root, &blackboard! { "ammo" => 5 }),
        Err(PlanError::NoPlan)
    );
    assert_eq!(
        plan(&root, &blackboard! { "ammo" => 0 }),
        steps(&["take_cover", "reload"])
    );
}

// ---- empty compound tasks ----------------------------------------------------

#[test]
fn empty_root_sequence_is_an_error() {
    let root = sequence!(conditions = [], tasks = []);

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::EmptyTask));
}

#[test]
fn empty_root_selector_is_an_error() {
    let root = selector!(conditions = [], tasks = []);

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::EmptyTask));
}

#[test]
fn empty_sequence_reached_inside_selector_is_an_error() {
    let root = selector!(
        conditions = [],
        tasks = [sequence!(conditions = [], tasks = []), action!("idle")]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::EmptyTask));
}

#[test]
fn empty_selector_reached_inside_sequence_is_an_error() {
    let root = sequence!(
        conditions = [],
        tasks = [
            action!("look_around"),
            selector!(conditions = [], tasks = [])
        ]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::EmptyTask));
}

#[test]
fn empty_task_parsed_from_dsl_is_an_error() {
    let root = parse(r#"selector { sequence {} action "idle" }"#).unwrap();

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::EmptyTask));
}

// ---- conditions on missing keys ----------------------------------------------

#[test]
fn missing_key_in_action_condition_is_an_error() {
    // The error aborts planning; it does not fall back to "idle".
    let root = selector!(
        conditions = [],
        tasks = [
            action!("heal", conditions = [cond!("health" < 50)]),
            action!("idle")
        ]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::MissingKey));
}

#[test]
fn missing_key_is_an_error_for_every_comparison_operator() {
    let conditions = [
        cond!("x" == 1),
        cond!("x" != 1),
        cond!("x" < 1),
        cond!("x" > 1),
        cond!("x" <= 1),
        cond!("x" >= 1),
    ];

    for c in conditions {
        let root = action!("a", conditions = [c.clone()]);
        assert_eq!(
            plan(&root, &blackboard! {}),
            Err(PlanError::MissingKey),
            "condition {c:?}"
        );
    }
}

#[test]
fn missing_key_inside_not_is_an_error() {
    let root = action!(
        "a",
        conditions = [Condition::Not(Box::new(cond!("x" == 1)))]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::MissingKey));
}

#[test]
fn missing_key_inside_all_is_an_error() {
    let root = action!(
        "a",
        conditions = [Condition::All(vec![
            cond!("present" == true),
            cond!("x" == 1)
        ])]
    );

    assert_eq!(
        plan(&root, &blackboard! { "present" => true }),
        Err(PlanError::MissingKey)
    );
}

#[test]
fn missing_key_in_compound_task_precondition_is_an_error() {
    let root = selector!(
        conditions = [],
        tasks = [
            sequence!(
                conditions = [cond!("in_town" == true)],
                tasks = [action!("shop")]
            ),
            action!("idle"),
        ]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::MissingKey));
}

#[test]
fn missing_key_in_root_precondition_is_an_error() {
    let root = selector!(
        conditions = [cond!("alive" == true)],
        tasks = [action!("idle")]
    );

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::MissingKey));
}

#[test]
fn missing_key_parsed_from_dsl_is_an_error() {
    let root = parse(r#"selector { action "heal" (health < 50) action "idle" }"#).unwrap();

    assert_eq!(plan(&root, &blackboard! {}), Err(PlanError::MissingKey));
}

#[test]
fn key_set_by_earlier_effect_is_not_missing() {
    let root = sequence!(
        conditions = [],
        tasks = [
            action!(
                "pick_up_key",
                conditions = [],
                effects = [effect!("has_key" = true)]
            ),
            action!("open_door", conditions = [cond!("has_key" == true)]),
        ]
    );

    assert_eq!(
        plan(&root, &blackboard! {}),
        steps(&["pick_up_key", "open_door"])
    );
}

#[test]
fn missing_key_in_unreached_branch_is_not_an_error() {
    // "cast_spell" is never considered because "attack" is chosen first.
    let root = selector!(
        conditions = [],
        tasks = [
            action!("attack", conditions = [cond!("health" > 0)]),
            action!("cast_spell", conditions = [cond!("mana" > 0)]),
        ]
    );

    assert_eq!(
        plan(&root, &blackboard! { "health" => 10 }),
        steps(&["attack"])
    );
}

// ---- conditions comparing different types ------------------------------------

#[test]
fn int_vs_float_is_an_error_for_every_comparison_operator() {
    let conditions = [
        cond!("x" == 1.0),
        cond!("x" != 1.0),
        cond!("x" < 1.0),
        cond!("x" > 1.0),
        cond!("x" <= 1.0),
        cond!("x" >= 1.0),
    ];

    for c in conditions {
        let root = action!("a", conditions = [c.clone()]);
        assert_eq!(
            plan(&root, &blackboard! { "x" => 1 }),
            Err(PlanError::TypeMismatch),
            "condition {c:?}"
        );
    }
}

#[test]
fn mismatched_types_are_an_error() {
    let cases = [
        (cond!("x" == "high"), Value::Int(1)),
        (cond!("x" != 1), Value::Bool(true)),
        (cond!("x" == true), Value::String("yes".into())),
        (
            cond!("x" == Value::Vector2(0.0, 0.0)),
            Value::Vector3(0.0, 0.0, 0.0),
        ),
    ];

    for (c, current) in cases {
        let root = action!("a", conditions = [c.clone()]);
        assert_eq!(
            plan(&root, &blackboard! { "x" => current }),
            Err(PlanError::TypeMismatch),
            "condition {c:?}"
        );
    }
}

#[test]
fn type_mismatch_aborts_planning_instead_of_falling_back() {
    let root = selector!(
        conditions = [],
        tasks = [
            action!("heal", conditions = [cond!("health" < 50)]),
            action!("idle"),
        ]
    );

    assert_eq!(
        plan(&root, &blackboard! { "health" => 40.0 }),
        Err(PlanError::TypeMismatch)
    );
}

#[test]
fn type_mismatch_inside_not_is_an_error() {
    // `not` must not turn a failed comparison into `true`.
    let root = action!(
        "a",
        conditions = [Condition::Not(Box::new(cond!("x" == "one")))]
    );

    assert_eq!(
        plan(&root, &blackboard! { "x" => 1 }),
        Err(PlanError::TypeMismatch)
    );
}

#[test]
fn type_mismatch_parsed_from_dsl_is_an_error() {
    let root = parse(r#"selector { action "heal" (health < 50) action "idle" }"#).unwrap();

    assert_eq!(
        plan(&root, &blackboard! { "health" => 40.0 }),
        Err(PlanError::TypeMismatch)
    );
}

#[test]
fn type_mismatch_in_unreached_branch_is_not_an_error() {
    let root = selector!(
        conditions = [],
        tasks = [
            action!("attack", conditions = [cond!("health" > 0)]),
            action!("cast_spell", conditions = [cond!("mana" > 0.0)]),
        ]
    );

    assert_eq!(
        plan(&root, &blackboard! { "health" => 10, "mana" => 5 }),
        steps(&["attack"])
    );
}
