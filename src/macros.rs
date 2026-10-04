//! Macros for building task networks and blackboards in code.

/// Builds a [`Condition`](crate::Condition) comparing a blackboard key to a value.
///
/// Supports `==`, `!=`, `<`, `>`, `<=` and `>=`.
///
/// ```
/// use htn::prelude::*;
///
/// let low_health = cond!("health" < 50);
/// let armed = cond!("has_weapon" == true);
/// ```
#[macro_export]
macro_rules! cond {
    ($key:literal == $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::E, $crate::Value::from($val))
    };
    ($key:literal != $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::NE, $crate::Value::from($val))
    };
    ($key:literal <= $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::LTE, $crate::Value::from($val))
    };
    ($key:literal >= $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::GTE, $crate::Value::from($val))
    };
    ($key:literal < $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::LT, $crate::Value::from($val))
    };
    ($key:literal > $val:expr) => {
        $crate::Condition::compare($key, $crate::ComparisonOp::GT, $crate::Value::from($val))
    };
}

/// Builds an [`Effect`](crate::Effect) that changes a blackboard key when an action is planned.
///
/// `=` sets the value; `+`, `-`, `*` and `/` apply arithmetic to the current value.
///
/// ```
/// use htn::prelude::*;
///
/// let take_damage = effect!("health" - 10);
/// let arm = effect!("has_weapon" = true);
/// ```
#[macro_export]
macro_rules! effect {
    ($key:literal = $val:expr) => {
        $crate::Effect::new($key, $crate::ArithmeticOp::Eq, $crate::Value::from($val))
    };
    ($key:literal + $val:expr) => {
        $crate::Effect::new($key, $crate::ArithmeticOp::Add, $crate::Value::from($val))
    };
    ($key:literal - $val:expr) => {
        $crate::Effect::new($key, $crate::ArithmeticOp::Sub, $crate::Value::from($val))
    };
    ($key:literal * $val:expr) => {
        $crate::Effect::new($key, $crate::ArithmeticOp::Mult, $crate::Value::from($val))
    };
    ($key:literal / $val:expr) => {
        $crate::Effect::new($key, $crate::ArithmeticOp::Div, $crate::Value::from($val))
    };
}

/// Builds a [`Task::Action`](crate::Task::Action), a leaf task that appears in the final plan.
///
/// ```
/// use htn::prelude::*;
///
/// let idle = action!("idle");
/// let attack = action!("attack", conditions = [cond!("has_weapon" == true)]);
/// let heal = action!(
///     "heal",
///     conditions = [cond!("health" < 50)],
///     effects = [effect!("health" + 25)]
/// );
/// ```
#[macro_export]
macro_rules! action {
    ($name:literal) => {
        $crate::Task::Action {
            preconditions: ::std::vec![],
            effects: ::std::vec![],
            action: ::std::string::String::from($name),
        }
    };
    ($name:literal, conditions = [$($cond:expr),* $(,)?]) => {
        $crate::Task::Action {
            preconditions: ::std::vec![$($cond),*],
            effects: ::std::vec![],
            action: ::std::string::String::from($name),
        }
    };
    ($name:literal, conditions = [$($cond:expr),* $(,)?], effects = [$($eff:expr),* $(,)?]) => {
        $crate::Task::Action {
            preconditions: ::std::vec![$($cond),*],
            effects: ::std::vec![$($eff),*],
            action: ::std::string::String::from($name),
        }
    };
}

/// Builds a [`Task::Selector`](crate::Task::Selector), which decomposes the first child
/// whose preconditions are met.
///
/// ```
/// use htn::prelude::*;
///
/// let root = selector!(
///     conditions = [],
///     tasks = [
///         action!("flee", conditions = [cond!("health" < 20)]),
///         action!("fight"),
///     ]
/// );
/// ```
#[macro_export]
macro_rules! selector {
    (conditions = [$($cond:expr),* $(,)?], tasks = [$($child:expr),* $(,)?]) => {
        $crate::Task::Selector {
            preconditions: ::std::vec![$($cond),*],
            tasks: ::std::vec![$($child),*],
        }
    };
}

/// Builds a [`Task::Sequence`](crate::Task::Sequence), which decomposes every child in order,
/// applying each action's effects before the next child is checked.
///
/// ```
/// use htn::prelude::*;
///
/// let reload = sequence!(
///     conditions = [cond!("ammo" == 0)],
///     tasks = [action!("take_cover"), action!("reload", conditions = [], effects = [effect!("ammo" = 30)])]
/// );
/// ```
#[macro_export]
macro_rules! sequence {
    (conditions = [$($cond:expr),* $(,)?], tasks = [$($child:expr),* $(,)?]) => {
        $crate::Task::Sequence {
            preconditions: ::std::vec![$($cond),*],
            tasks: ::std::vec![$($child),*],
        }
    };
}

/// Builds a blackboard (`HashMap<String, Value>`) from `key => value` pairs.
///
/// ```
/// use htn::prelude::*;
///
/// let blackboard = blackboard! {
///     "health" => 40,
///     "has_weapon" => true,
/// };
/// ```
#[macro_export]
macro_rules! blackboard {
    ($($key:literal => $val:expr),* $(,)?) => {{
        let mut map: ::std::collections::HashMap<::std::string::String, $crate::Value> =
            ::std::collections::HashMap::new();
        $(map.insert(::std::string::String::from($key), $crate::Value::from($val));)*
        map
    }};
}
