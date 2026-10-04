use std::{borrow::Cow, collections::HashMap};

use super::{
    condition::{Condition, eval},
    effect::Effect,
    error::PlanError,
    value::Value,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    Selector {
        preconditions: Vec<Condition>,
        tasks: Vec<Task>,
    },
    Sequence {
        preconditions: Vec<Condition>,
        tasks: Vec<Task>,
    },
    Action {
        preconditions: Vec<Condition>,
        action: String,
        effects: Vec<Effect>,
    },
}

/// Plans `task` against the blackboard, returning the action names in order.
///
/// Returns [`PlanError::NoPlan`] when no branch can run, and the other
/// [`PlanError`] variants when the task network or blackboard is malformed.
pub fn plan(task: &Task, data: &HashMap<String, Value>) -> Result<Vec<String>, PlanError> {
    if !is_met(get_preconditions(task), data)? {
        return Err(PlanError::NoPlan);
    }

    decompose(task, data)?
        .map(|(plan, _)| plan)
        .ok_or(PlanError::NoPlan)
}

fn is_met(preconditions: &[Condition], data: &HashMap<String, Value>) -> Result<bool, PlanError> {
    for c in preconditions {
        if !eval(c, data)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn get_preconditions(task: &Task) -> &[Condition] {
    match task {
        Task::Selector { preconditions, .. } => preconditions,
        Task::Sequence { preconditions, .. } => preconditions,
        Task::Action { preconditions, .. } => preconditions,
    }
}

/// A successfully decomposed branch: its actions and the blackboard after their effects.
type Decomposition = (Vec<String>, HashMap<String, Value>);

/// Decomposes `task`, whose own preconditions the caller has already checked.
///
/// `Ok(None)` means this branch can't be planned and a parent selector should try
/// its next child; `Err` aborts planning entirely.
fn decompose(
    task: &Task,
    data: &HashMap<String, Value>,
) -> Result<Option<Decomposition>, PlanError> {
    match task {
        Task::Selector { tasks, .. } | Task::Sequence { tasks, .. } if tasks.is_empty() => {
            Err(PlanError::EmptyTask)
        }
        Task::Selector { tasks, .. } => {
            for task in tasks {
                if !is_met(get_preconditions(task), data)? {
                    continue;
                }
                if let Some(decomposition) = decompose(task, data)? {
                    return Ok(Some(decomposition));
                }
            }

            Ok(None)
        }
        Task::Sequence { tasks, .. } => {
            let mut full_plan: Vec<String> = vec![];
            let mut state: Cow<HashMap<String, Value>> = Cow::Borrowed(data);

            for task in tasks {
                if !is_met(get_preconditions(task), &state)? {
                    return Ok(None);
                }

                let Some((plan, new_data)) = decompose(task, &state)? else {
                    return Ok(None);
                };
                full_plan.extend(plan);
                state = Cow::Owned(new_data);
            }

            Ok(Some((full_plan, state.into_owned())))
        }
        Task::Action {
            action, effects, ..
        } => {
            let mut new_data = data.clone();

            for effect in effects {
                // An effect that can't be applied makes this action unplannable.
                if effect.apply(&mut new_data).is_err() {
                    return Ok(None);
                }
            }

            Ok(Some((vec![action.clone()], new_data)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decompose, is_met};
    use crate::{action, blackboard, cond, effect, selector, sequence};

    #[test]
    fn simple_htn() {
        let blackboard = blackboard! {
            "health" => 40,
            "has_heal_item" => true
        };

        let root = selector!(
            conditions = [],
            tasks = [action!(
                "heal",
                conditions = [cond!("health" < 50), cond!("has_heal_item" == true)]
            ),]
        );

        let (plan, _) = decompose(&root, &blackboard)
            .expect("should not error")
            .expect("should not return none");
        assert_eq!(plan, vec!["heal"]);
    }

    #[test]
    fn is_met_returns_true() {
        let conditions = vec![cond!("is_true" == true)];

        let blackboard = blackboard!(
            "is_true" => true
        );

        let result = is_met(&conditions, &blackboard).expect("key is present");

        assert!(result);
    }

    #[test]
    fn is_met_returns_false() {
        let conditions = vec![cond!("is_true" == true)];

        let blackboard = blackboard!(
            "is_true" => false
        );

        let result = is_met(&conditions, &blackboard).expect("key is present");

        assert!(!result);
    }

    #[test]
    fn sequence_succeeds() {
        let root = sequence!(
            conditions = [],
            tasks = [
                action!(
                    "put_clothes_in_machine",
                    conditions = [cond!("has_dirty_laundry" == true)],
                    effects = [effect!("clothes_in" = true)]
                ),
                action!(
                    "put_in_detergent",
                    conditions = [cond!("clothes_in" == true), cond!("has_detergent" == true)],
                    effects = [effect!("has_detergent" = false)]
                ),
                action!(
                    "turn_machine_on",
                    conditions = [cond!("clothes_in" == true)]
                ),
                selector!(conditions = [], tasks = [action!("do_nothing")])
            ]
        );

        let blackboard = blackboard! {
            "has_dirty_laundry" => true,
            "has_detergent" => true,
            "has_clean_laundry" => false,
            "clothes_in" => false
        };

        let (plan, _) = decompose(&root, &blackboard)
            .expect("should not error")
            .expect("should not return none");
        assert_eq!(
            plan,
            vec![
                "put_clothes_in_machine",
                "put_in_detergent",
                "turn_machine_on",
                "do_nothing"
            ]
        )
    }
}
