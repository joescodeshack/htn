//! Parser robustness: malformed input returns an error instead of panicking.

use htn::{ComparisonOp, Condition, ParseError, Task, Value, parse};

// Offsets are byte positions in the source where the problem starts.

#[test]
fn int_literal_too_large_is_an_error() {
    assert_eq!(
        parse(r#"action "a" (x == 99999999999)"#),
        Err(vec![ParseError::IntOutOfRange { offset: 17 }])
    );
}

#[test]
fn int_literal_too_small_is_an_error() {
    assert_eq!(
        parse(r#"action "a" (x == -2147483649)"#),
        Err(vec![ParseError::IntOutOfRange { offset: 17 }])
    );
}

#[test]
fn int_literal_too_large_in_effect_is_an_error() {
    assert_eq!(
        parse(r#"action "a" [ x = 99999999999 ]"#),
        Err(vec![ParseError::IntOutOfRange { offset: 17 }])
    );
}

#[test]
fn unexpected_character_is_an_error() {
    // `nonsense` is not a precondition, effect list or end of input.
    assert_eq!(
        parse(r#"action "a" nonsense"#),
        Err(vec![ParseError::UnexpectedChar { offset: 11 }])
    );
}

#[test]
fn unclosed_brace_is_an_unexpected_end() {
    let src = r#"selector { action "a" "#;

    assert_eq!(
        parse(src),
        Err(vec![ParseError::UnexpectedEnd { offset: src.len() }])
    );
}

#[test]
fn missing_action_name_is_an_unexpected_end() {
    assert_eq!(
        parse("action"),
        Err(vec![ParseError::UnexpectedEnd { offset: 6 }])
    );
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
