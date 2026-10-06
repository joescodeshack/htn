# HTN (Hierarchical Task Network)

HTN stands for Hierarchical Task Network.

It is a strategy to create AI, primarily used in Game NPC to dictate their behavior. A similar strategy would be a Behavior Tree. HTN differs by constructing
a plan, or list of actions for an agent based on information from a Blackboard.

The Blackboard is the NPC's view of the world, a set of named values like `health`, `hunger` or `danger_nearby`. The game keeps the Blackboard up to date, and the HTN reads from it to decide what the NPC should do next.

The tree is built from three kinds of tasks:

- **Selector** tries each of its children in order and picks the first one that can succeed. This is how you give an NPC priorities, running from danger before eating, eating before sleeping.
- **Sequence** runs all of its children in order, every one of them has to succeed for the sequence to succeed.
- **Action** is a single thing the NPC can do, like "eat" or "go_to_sleep". These are the leaves of the tree and are what end up in the plan.

Every task can have conditions that must be met for it to run, and actions can have effects that change the Blackboard, such as eating lowering `hunger`.

As the HTN steps through the tree, it can affect the state of the Blackboard. These effects are made on a copy of the Blackboard for planning, so the Blackboard changes can be thrown away if the selector/sequence/task fails its conditions. Additionally, the accrued plans will be removed.

This is what makes HTN useful, later tasks in a sequence can depend on what earlier tasks would have done. An NPC can plan to buy food and then eat it, even though it has no food yet. The result is a list of action names, it's up to your game to carry them out, and to plan again when the world changes.


### Getting Started

To utilize the features of htn programatically you can leverage the macros to define
your htn with clear, terse code:

```rust
use htn::prelude::*;

fn main() {
    let blackboard = blackboard! {
        "hunger" => 100,
        "energy" => 40,
        "danger" => 0
    };

    let htn = selector!(
        conditions = [],
        tasks = [
            action!(
                "eat",
                conditions = [
                    cond!("hunger" > 60)
                ]
            ),
            action!(
                "sleep",
                conditions = [
                    cond!("energy" <= 10)
                ]
            ),
            action!(
                "run",
                conditions = [
                    cond!("danger" > 80)
                ]
            )
        ]
    );

    let plan = plan(&htn, &blackboard);
    assert_eq!(plan, Ok(vec!["eat".to_string()]));
}
```

### htn lang
I use chumsky to create a relatively simple language to express the HTN, below is an example of the syntax. In order to parse this language you just have to call `htn::parse` and hand the result to `plan`:

```rust
use htn::{blackboard, parse, plan};

fn main() {
    let tree = parse(r#"
        selector {
            selector (danger_nearby == true) {
                sequence (health < 30 or not has_weapon == true) {
                    action "drop_what_youre_doing" [ is_sleeping = false, is_eating = false ]
                    action "run_to_safe_spot" (stamina > 10) [
                        stamina -= 25,
                        at_safe_spot = true,
                        danger_nearby = false
                    ]
                }
                action "hide" (at_safe_spot == false) [ hidden = true ]
                action "fight_back" (has_weapon == true and health >= 30) [ stamina -= 15 ]
            }

            sequence (hunger > 70) {
                selector {
                    action "eat_from_pack" (food_in_pack > 0) [ food_in_pack -= 1, hunger -= 50 ]
                    sequence (in_town == true) (gold >= 5) {
                        action "go_to_tavern"
                        action "buy_meal" [ gold -= 5 ]
                        action "eat_meal" [ hunger -= 60 ]
                    }
                    sequence {
                        action "forage" [ food_in_pack += 1, stamina -= 10 ]
                        action "eat_from_pack" [ food_in_pack -= 1, hunger -= 30 ]
                    }
                }
            }

            sequence (energy < 25.0 or (hour >= 22 or hour < 6) and energy < 60.0) {
                selector {
                    action "go_home" (in_town == true) [ at_home = true ]
                    action "make_camp" (in_town == false) [ at_camp = true ]
                }
                action "go_to_sleep" [ is_sleeping = true, energy = 100.0, stamina *= 2 ]
            }

            action "wander" (stamina > 0) [ stamina -= 5, energy -= 1.5 ]
            action "idle"
        }
    "#)
    .expect("htn source should parse");

    let blackboard = blackboard! {
        // danger
        "danger_nearby" => false,
        "health" => 100,
        "has_weapon" => true,
        "stamina" => 50,
        "at_safe_spot" => false,
        "hidden" => false,
        // eating
        "hunger" => 10,
        "food_in_pack" => 0,
        "in_town" => true,
        "gold" => 10,
        "is_eating" => false,
        // sleeping
        "energy" => 80.0_f32,
        "hour" => 12,
        "at_home" => false,
        "at_camp" => false,
        "is_sleeping" => false,
    };

    let plan = plan(&tree, &blackboard);
    assert_eq!(plan, Ok(vec!["wander".to_string()]));
}
```
