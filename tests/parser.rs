//! Parser robustness: malformed input returns an error instead of panicking.

use htn::{ComparisonOp, Condition, Task, Value, parse};

#[test]
fn int_literal_too_large_is_an_error() {
    assert!(parse(r#"action "a" (x == 99999999999)"#).is_err());
}

#[test]
fn int_literal_too_small_is_an_error() {
    assert!(parse(r#"action "a" (x == -2147483649)"#).is_err());
}

#[test]
fn int_literal_too_large_in_effect_is_an_error() {
    assert!(parse(r#"action "a" [ x = 99999999999 ]"#).is_err());
}

#[test]
fn i32_bounds_parse() {
    let got = parse(r#"action "a" (x == 2147483647) (y == -2147483648)"#).unwrap();

    assert_eq!(
        got,
        Task::Action {
            preconditions: vec![
                Condition::compare("x", ComparisonOp::E, Value::Int(i32::MAX)),
                Condition::compare("y", ComparisonOp::E, Value::Int(i32::MIN)),
            ],
            action: "a".to_string(),
            effects: vec![],
        }
    );
}
