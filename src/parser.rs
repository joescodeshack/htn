//! Parser for the `.htn` task-network language.
//!
//! Grammar (informally):
//!
//! ```text
//! task        := selector | sequence | action
//! selector    := "selector" precondition* "{" task* "}"
//! sequence    := "sequence" precondition* "{" task* "}"
//! action      := "action" STRING precondition* effects?
//!
//! precondition := "(" condition ")"
//! condition    := or
//! or           := and ("or"  and)*
//! and          := not ("and" not)*
//! not          := "not"* atom
//! atom         := comparison | "(" condition ")"
//! comparison   := IDENT cmp_op value
//! cmp_op       := "==" | "!=" | "<=" | ">=" | "<" | ">"
//!
//! effects     := "[" (effect ("," effect)* ","?)? "]"
//! effect      := IDENT eff_op value
//! eff_op      := "=" | "+=" | "-=" | "*=" | "/="
//!
//! value       := bool | number | string
//! ```

use chumsky::prelude::*;

use super::{
    condition::{ComparisonOp, Condition},
    effect::{ArithmeticOp, Effect},
    task::Task,
    value::Value,
};

/// Error/recovery configuration shared by every sub-parser.
type Extra<'a> = extra::Err<Rich<'a, char>>;

/// Parse a complete `.htn` source string into its root [`Task`].
///
/// On failure returns the collection of [`Rich`] errors describing what went
/// wrong (and where).
pub fn parse(src: &str) -> Result<Task, Vec<Rich<'_, char>>> {
    parser().parse(src).into_result()
}

/// A literal `value`: a bool, an integer, a float, or a quoted string.
fn value<'a>() -> impl Parser<'a, &'a str, Value, Extra<'a>> + Clone {
    let boolean = choice((
        text::ascii::keyword("true").to(Value::Bool(true)),
        text::ascii::keyword("false").to(Value::Bool(false)),
    ));

    let number = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .try_map(|s: &str, span| {
            if s.contains('.') {
                s.parse()
                    .map(Value::Float)
                    .map_err(|_| Rich::custom(span, format!("invalid float literal `{s}`")))
            } else {
                s.parse().map(Value::Int).map_err(|_| {
                    Rich::custom(
                        span,
                        format!("integer literal `{s}` does not fit in an i32"),
                    )
                })
            }
        });

    let string = just('"')
        .ignore_then(none_of("\"").repeated().to_slice())
        .then_ignore(just('"'))
        .map(|s: &str| Value::String(s.to_string()));

    choice((boolean, number, string))
}

/// A comparison operator used inside conditions.
fn comparison_op<'a>() -> impl Parser<'a, &'a str, ComparisonOp, Extra<'a>> + Clone {
    choice((
        just("==").to(ComparisonOp::E),
        just("!=").to(ComparisonOp::NE),
        just("<=").to(ComparisonOp::LTE),
        just(">=").to(ComparisonOp::GTE),
        just("<").to(ComparisonOp::LT),
        just(">").to(ComparisonOp::GT),
    ))
    .padded()
}

/// A boolean condition expression (the contents of a precondition's parens),
/// supporting `and` / `or` / `not` and nested grouping.
fn condition<'a>() -> impl Parser<'a, &'a str, Condition, Extra<'a>> + Clone {
    recursive(|cond| {
        let ident = text::ascii::ident().padded();

        let comparison = ident.then(comparison_op()).then(value().padded()).map(
            |((key, op), val): ((&str, ComparisonOp), Value)| Condition::compare(key, op, val),
        );

        let atom = choice((comparison, cond.delimited_by(just('('), just(')')).padded()));

        // `not` binds tightest: any number of leading `not`s flip the atom.
        let not = text::ascii::keyword("not")
            .padded()
            .repeated()
            .foldr(atom, |_, inner| Condition::Not(Box::new(inner)));

        // `and` binds tighter than `or`.
        let and = not
            .separated_by(text::ascii::keyword("and").padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|mut cs| {
                if cs.len() == 1 {
                    cs.pop().unwrap()
                } else {
                    Condition::All(cs)
                }
            });

        and.separated_by(text::ascii::keyword("or").padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|mut cs| {
                if cs.len() == 1 {
                    cs.pop().unwrap()
                } else {
                    Condition::Any(cs)
                }
            })
    })
}

/// The full grammar, producing the root [`Task`].
fn parser<'a>() -> impl Parser<'a, &'a str, Task, Extra<'a>> {
    let precondition = condition().delimited_by(just('('), just(')')).padded();

    let eff_op = choice((
        just("+=").to(ArithmeticOp::Add),
        just("-=").to(ArithmeticOp::Sub),
        just("*=").to(ArithmeticOp::Mult),
        just("/=").to(ArithmeticOp::Div),
        just("=").to(ArithmeticOp::Eq),
    ))
    .padded();

    let effect = text::ascii::ident()
        .padded()
        .then(eff_op)
        .then(value().padded())
        .map(|((key, op), val): ((&str, ArithmeticOp), Value)| Effect::new(key, op, val));

    let effects = effect
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('[').padded(), just(']').padded());

    let string_lit = just('"')
        .ignore_then(none_of("\"").repeated().to_slice())
        .then_ignore(just('"'))
        .map(|s: &str| s.to_string())
        .padded();

    let task = recursive(|task| {
        let body = task
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded());

        let action = text::ascii::keyword("action")
            .padded()
            .ignore_then(string_lit)
            .then(precondition.clone().repeated().collect::<Vec<_>>())
            .then(effects.or_not())
            .map(|((action, preconditions), effects)| Task::Action {
                preconditions,
                action,
                effects: effects.unwrap_or_default(),
            });

        let selector = text::ascii::keyword("selector")
            .padded()
            .ignore_then(precondition.clone().repeated().collect::<Vec<_>>())
            .then(body.clone())
            .map(|(preconditions, tasks)| Task::Selector {
                preconditions,
                tasks,
            });

        let sequence = text::ascii::keyword("sequence")
            .padded()
            .ignore_then(precondition.clone().repeated().collect::<Vec<_>>())
            .then(body)
            .map(|(preconditions, tasks)| Task::Sequence {
                preconditions,
                tasks,
            });

        choice((selector, sequence, action)).padded()
    });

    task.then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- small constructor helpers, mirroring the macros in `htn.rs` -------

    fn cmp(key: &str, op: ComparisonOp, v: impl Into<Value>) -> Condition {
        Condition::compare(key, op, v.into())
    }

    fn eff(key: &str, op: ArithmeticOp, v: impl Into<Value>) -> Effect {
        Effect::new(key, op, v.into())
    }

    fn action(name: &str) -> Task {
        Task::Action {
            preconditions: vec![],
            action: name.to_string(),
            effects: vec![],
        }
    }

    // ---- values ------------------------------------------------------------

    #[test]
    fn parses_value_kinds() {
        // Drive the value parser through a single-condition action.
        let got = parse(r#"action "a" (x == 42)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![cmp("x", ComparisonOp::E, 42)],
                action: "a".to_string(),
                effects: vec![],
            }
        );

        let got = parse(r#"action "a" (x == 3.5)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![cmp("x", ComparisonOp::E, 3.5_f32)],
                action: "a".to_string(),
                effects: vec![],
            }
        );

        let got = parse(r#"action "a" (x == -7)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![cmp("x", ComparisonOp::E, -7)],
                action: "a".to_string(),
                effects: vec![],
            }
        );

        let got = parse(r#"action "a" (name == "bob")"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![cmp("name", ComparisonOp::E, "bob")],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    // ---- actions -----------------------------------------------------------

    #[test]
    fn parses_bare_action() {
        assert_eq!(
            parse(r#"action "go_to_sleep""#).unwrap(),
            action("go_to_sleep")
        );
    }

    #[test]
    fn parses_action_with_multiple_preconditions() {
        let got = parse(r#"action "use_heal_item" (health < 40) (has_heal_item == true)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![
                    cmp("health", ComparisonOp::LT, 40),
                    cmp("has_heal_item", ComparisonOp::E, true),
                ],
                action: "use_heal_item".to_string(),
                effects: vec![],
            }
        );
    }

    #[test]
    fn parses_all_comparison_operators() {
        let got =
            parse(r#"action "a" (a == 1) (b != 2) (c < 3) (d > 4) (e <= 5) (f >= 6)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![
                    cmp("a", ComparisonOp::E, 1),
                    cmp("b", ComparisonOp::NE, 2),
                    cmp("c", ComparisonOp::LT, 3),
                    cmp("d", ComparisonOp::GT, 4),
                    cmp("e", ComparisonOp::LTE, 5),
                    cmp("f", ComparisonOp::GTE, 6),
                ],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    // ---- effects -----------------------------------------------------------

    #[test]
    fn parses_action_with_effects() {
        let got = parse(
            r#"action "go_home" [
                is_home = true,
                is_outside = false
            ]"#,
        )
        .unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![],
                action: "go_home".to_string(),
                effects: vec![
                    eff("is_home", ArithmeticOp::Eq, true),
                    eff("is_outside", ArithmeticOp::Eq, false),
                ],
            }
        );
    }

    #[test]
    fn parses_all_effect_operators() {
        let got = parse(r#"action "a" [ a = 1, b += 2, c -= 3, d *= 4, e /= 5 ]"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![],
                action: "a".to_string(),
                effects: vec![
                    eff("a", ArithmeticOp::Eq, 1),
                    eff("b", ArithmeticOp::Add, 2),
                    eff("c", ArithmeticOp::Sub, 3),
                    eff("d", ArithmeticOp::Mult, 4),
                    eff("e", ArithmeticOp::Div, 5),
                ],
            }
        );
    }

    #[test]
    fn parses_preconditions_and_effects_together() {
        let got = parse(r#"action "find_cover" (in_cover == false) [ in_cover = true ]"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![cmp("in_cover", ComparisonOp::E, false)],
                action: "find_cover".to_string(),
                effects: vec![eff("in_cover", ArithmeticOp::Eq, true)],
            }
        );
    }

    // ---- condition combinators --------------------------------------------

    #[test]
    fn parses_or_condition() {
        let got = parse(r#"action "setup_camp" (hour < 6 or hour > 16)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![Condition::Any(vec![
                    cmp("hour", ComparisonOp::LT, 6),
                    cmp("hour", ComparisonOp::GT, 16),
                ])],
                action: "setup_camp".to_string(),
                effects: vec![],
            }
        );
    }

    #[test]
    fn parses_and_condition() {
        let got = parse(r#"action "a" (x > 0 and y > 0)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![Condition::All(vec![
                    cmp("x", ComparisonOp::GT, 0),
                    cmp("y", ComparisonOp::GT, 0),
                ])],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    #[test]
    fn parses_not_condition() {
        let got = parse(r#"action "a" (not x == true)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![Condition::Not(Box::new(cmp("x", ComparisonOp::E, true)))],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    #[test]
    fn and_binds_tighter_than_or() {
        // `a or b and c`  =>  Any[ a, All[ b, c ] ]
        let got = parse(r#"action "a" (a == 1 or b == 2 and c == 3)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![Condition::Any(vec![
                    cmp("a", ComparisonOp::E, 1),
                    Condition::All(vec![
                        cmp("b", ComparisonOp::E, 2),
                        cmp("c", ComparisonOp::E, 3),
                    ]),
                ])],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    #[test]
    fn parens_override_precedence() {
        // `(a or b) and c`  =>  All[ Any[ a, b ], c ]
        let got = parse(r#"action "a" ((a == 1 or b == 2) and c == 3)"#).unwrap();
        assert_eq!(
            got,
            Task::Action {
                preconditions: vec![Condition::All(vec![
                    Condition::Any(vec![
                        cmp("a", ComparisonOp::E, 1),
                        cmp("b", ComparisonOp::E, 2),
                    ]),
                    cmp("c", ComparisonOp::E, 3),
                ])],
                action: "a".to_string(),
                effects: vec![],
            }
        );
    }

    // ---- compound tasks ----------------------------------------------------

    #[test]
    fn parses_nested_selectors_and_sequences() {
        let got = parse(
            r#"
            selector {
                selector (in_combat == true) {
                    action "find_cover" (in_cover == false) [ in_cover = true ]
                    action "shoot_at_player" (in_cover == true)
                }
            }
            "#,
        )
        .unwrap();

        assert_eq!(
            got,
            Task::Selector {
                preconditions: vec![],
                tasks: vec![Task::Selector {
                    preconditions: vec![cmp("in_combat", ComparisonOp::E, true)],
                    tasks: vec![
                        Task::Action {
                            preconditions: vec![cmp("in_cover", ComparisonOp::E, false)],
                            action: "find_cover".to_string(),
                            effects: vec![eff("in_cover", ArithmeticOp::Eq, true)],
                        },
                        Task::Action {
                            preconditions: vec![cmp("in_cover", ComparisonOp::E, true)],
                            action: "shoot_at_player".to_string(),
                            effects: vec![],
                        },
                    ],
                }],
            }
        );
    }

    #[test]
    fn parses_sequence_with_preconditions() {
        let got = parse(
            r#"
            sequence (health < 40) (in_town == true) {
                action "go_to_store"
                action "buy_heal_item"
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            got,
            Task::Sequence {
                preconditions: vec![
                    cmp("health", ComparisonOp::LT, 40),
                    cmp("in_town", ComparisonOp::E, true),
                ],
                tasks: vec![action("go_to_store"), action("buy_heal_item")],
            }
        );
    }

    #[test]
    fn parses_empty_selector_body() {
        assert_eq!(
            parse("selector {}").unwrap(),
            Task::Selector {
                preconditions: vec![],
                tasks: vec![],
            }
        );
    }

    // ---- a full, realistic program ----------------------------------------

    #[test]
    fn parses_full_program() {
        // The `simple.htn` example, with its `actionn` typo corrected.
        let src = r#"
            selector {
                action "use_heal_item" (health < 40) (has_heal_item == true)
                selector (in_town == true) {
                    sequence (energy < 40) {
                        action "go_to_sleep"
                    }
                }
                action "setup_camp" (hour < 6 or hour > 16)
                sequence (health < 40) (has_heal_item == false) (in_town == true) {
                    action "go_to_store"
                    action "buy_heal_item"
                    action "use_heal_item"
                }
                sequence {
                    action "go_home" [
                        is_home = true,
                        is_outside = false
                    ]
                    action "go_to_sleep"
                }
            }
        "#;

        let root = parse(src).expect("full program should parse");

        // Spot-check the shape rather than spelling out the whole tree.
        match root {
            Task::Selector {
                preconditions,
                tasks,
            } => {
                assert!(preconditions.is_empty());
                assert_eq!(tasks.len(), 5);
            }
            other => panic!("expected a root selector, got {other:?}"),
        }
    }

    // ---- failure cases -----------------------------------------------------

    #[test]
    fn rejects_unknown_keyword() {
        // `actionn` is not a valid task keyword.
        assert!(parse(r#"selector { actionn "oops" }"#).is_err());
    }

    #[test]
    fn rejects_unclosed_brace() {
        assert!(parse(r#"selector { action "a" "#).is_err());
    }

    #[test]
    fn rejects_action_without_name() {
        assert!(parse("action").is_err());
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse(r#"action "a" nonsense"#).is_err());
    }
}
