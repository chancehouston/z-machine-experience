//! The family-dependent queries on Advent, in its version 5 and version 8 builds,
//! checked against the game itself at every prompt of the walkthrough
//! (docs/introspection.md, cited I§; SC-002 to SC-006). Advent is of the Inform 6
//! family.

mod common;

use common::*;
use zmachine_core::{Event, Machine};
use zmachine_introspect::{Introspector, Tri};

const EXITS: &str = "stories/advent/introspection/exits.txt";
const BUILDS: [Game; 2] = [ADVENT5, ADVENT8];

// I§6, SC-002, FR-037: at every command prompt the location is known, exact, the
// room the game names in its headings, and the one its status line shows.
#[test]
fn location_agrees_with_the_game_at_every_prompt() {
    for game in BUILDS {
        let report = check_locations(game, EXITS, |_, location| location.lit == Tri::No);
        assert!(report.problems.is_empty(), "{} problems:\n{}", report.problems.len(), report.problems.join("\n"));
        assert_eq!(report.prompts, 172, "{}", game.name);
        assert!(report.headings >= 60, "{}: room headings checked: {}", game.name, report.headings);
        assert!(report.changes >= 80, "{}: moves checked: {}", game.name, report.changes);
        assert!(report.lit >= 150, "{}: lit at {} prompts", game.name, report.lit);
        assert_eq!(report.lit + report.dark, report.prompts, "{}: Inform records light", game.name);
    }
}

// I§6: the opening room, and the maze rooms, whose printed name is not the name
// the story file stores for them.
#[test]
fn location_at_known_moments() {
    for game in BUILDS {
        let mut mazes = 0;
        walk(game, |machine, stop| {
            if !stop.command_prompt {
                return;
            }
            let view = machine.view();
            let i = Introspector::new(&view);
            let location = i.location(&view).value.unwrap();
            assert_eq!(location.holder, Ok(None), "Advent never puts the player inside anything");
            if stop.number == 1 {
                assert_eq!(location.name, Ok("At End Of Road".to_string()));
                assert_eq!(location.lit, Tri::Yes);
            }
            if location.name.as_deref() == Ok("Maze") {
                let stored = &i.object_tree(&view).value.unwrap()[location.room as usize - 1].name;
                assert!(stored.starts_with("(Alike_Maze_"), "{stored}");
                mazes += 1;
            }
        });
        assert!(mazes >= 3, "{}: prompts in the maze: {mazes}", game.name);
    }
}

/// Plays a few commands and returns the machine at the next prompt.
fn play(game: Game, commands: &[&str]) -> Machine {
    let mut machine = load(game.story, game.seed);
    let mut commands = commands.iter();
    loop {
        match machine.run(1_000_000).unwrap() {
            Event::LineWanted { .. } => match commands.next() {
                Some(command) => machine.provide_line(command).unwrap(),
                None => return machine,
            },
            Event::BudgetExhausted => {}
            other => panic!("unexpected {other:?}"),
        }
    }
}

// I§6: darkness. The walkthrough is never in the dark, so this plays its own way
// there: into the cave with the lamp, then the lamp off. The room is still the
// room; `lit` is `no`; the game's status line says "Darkness".
#[test]
fn a_dark_room_is_still_the_room_and_is_not_lit() {
    for game in BUILDS {
        let to_the_cave = ["in", "take keys", "take lamp", "out", "s", "s", "s", "unlock grate with keys", "open grate", "d", "w", "on", "w"];
        let mut machine = play(game, &to_the_cave);
        let view = machine.view();
        let lit = Introspector::new(&view).location(&view).value.unwrap();
        assert_eq!((lit.name.as_deref(), lit.lit), (Ok("In Debris Room"), Tri::Yes), "{}", game.name);
        machine.take_output();
        machine.provide_line("off").unwrap();
        while !matches!(machine.run(1_000_000).unwrap(), Event::LineWanted { .. }) {}
        assert!(machine.take_output().contains("It is now pitch dark in here!"));
        assert!(machine.screen().row_text(1).trim_start().starts_with("Darkness"));
        let view = machine.view();
        let dark = Introspector::new(&view).location(&view).value.unwrap();
        assert_eq!(dark.room, lit.room, "{}: the same room", game.name);
        assert_eq!(dark.name.as_deref(), Ok("In Debris Room"));
        assert_eq!(dark.lit, Tri::No);
    }
}

// I§7, SC-005: the inventory is what the game lists when a copy of it is asked,
// and in Advent it is never unknown.
#[test]
fn inventory_agrees_with_the_game_at_every_prompt() {
    for game in BUILDS {
        let report = check_inventories(game, "You're carrying:");
        assert!(report.problems.is_empty(), "{} problems:\n{}", report.problems.len(), report.problems.join("\n"));
        assert_eq!(report.prompts, 172, "{}", game.name);
        assert!(report.unknown.is_empty(), "{}: unknown at {:?}", game.name, report.unknown);
        assert!(report.not_an_inventory.len() <= 2, "{}: {:?}", game.name, report.not_an_inventory);
        assert!(report.compared >= 168 && report.empty >= 2 && report.most_carried >= 5, "{report:?}");
    }
}

// I§7: after the lamp is taken it is carried and nothing else is; nothing in
// Advent's walkthrough is worn; the caged bird is inside the cage.
#[test]
fn inventory_at_known_moments() {
    let machine = play(ADVENT5, &["in", "take lamp"]);
    let view = machine.view();
    let carried = Introspector::new(&view).inventory(&view).value.unwrap();
    assert_eq!(carried.len(), 1);
    assert_eq!(carried[0].name.as_deref(), Ok("brass lantern"));
    assert_eq!(carried[0].worn, Tri::No);
    assert!(carried[0].contents.is_empty());

    let mut caged = 0;
    walk(ADVENT5, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        let view = machine.view();
        let carried = Introspector::new(&view).inventory(&view).value.unwrap();
        for item in &carried {
            if item.name.as_deref() == Ok("wicker cage") && !item.contents.is_empty() {
                assert_eq!(item.contents[0].name.as_deref(), Ok("little bird"));
                assert!(!carried.iter().any(|c| c.name.as_deref() == Ok("little bird")));
                caged += 1;
            }
        }
    });
    assert!(caged >= 3, "prompts carrying the caged bird: {caged}");
}
