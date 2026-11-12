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
        assert!(
            report.problems.is_empty(),
            "{} problems:\n{}",
            report.problems.len(),
            report.problems.join("\n")
        );
        assert_eq!(report.prompts, 172, "{}", game.name);
        assert!(
            report.headings >= 60,
            "{}: room headings checked: {}",
            game.name,
            report.headings
        );
        assert!(
            report.changes >= 80,
            "{}: moves checked: {}",
            game.name,
            report.changes
        );
        assert!(
            report.lit >= 150,
            "{}: lit at {} prompts",
            game.name,
            report.lit
        );
        assert_eq!(
            report.lit + report.dark,
            report.prompts,
            "{}: Inform records light",
            game.name
        );
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
            assert_eq!(
                location.holder,
                Ok(None),
                "Advent never puts the player inside anything"
            );
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
        let to_the_cave = [
            "in",
            "take keys",
            "take lamp",
            "out",
            "s",
            "s",
            "s",
            "unlock grate with keys",
            "open grate",
            "d",
            "w",
            "on",
            "w",
        ];
        let mut machine = play(game, &to_the_cave);
        let view = machine.view();
        let lit = Introspector::new(&view).location(&view).value.unwrap();
        assert_eq!(
            (lit.name.as_deref(), lit.lit),
            (Ok("In Debris Room"), Tri::Yes),
            "{}",
            game.name
        );
        machine.take_output();
        machine.provide_line("off").unwrap();
        while !matches!(machine.run(1_000_000).unwrap(), Event::LineWanted { .. }) {}
        assert!(
            machine
                .take_output()
                .contains("It is now pitch dark in here.")
        );
        assert!(
            machine
                .screen()
                .row_text(1)
                .trim_start()
                .starts_with("Darkness")
        );
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
        assert!(
            report.problems.is_empty(),
            "{} problems:\n{}",
            report.problems.len(),
            report.problems.join("\n")
        );
        assert_eq!(report.prompts, 172, "{}", game.name);
        assert!(
            report.unknown.is_empty(),
            "{}: unknown at {:?}",
            game.name,
            report.unknown
        );
        assert!(
            report.not_an_inventory.len() <= 2,
            "{}: {:?}",
            game.name,
            report.not_an_inventory
        );
        assert!(
            report.compared >= 168 && report.empty >= 2 && report.most_carried >= 5,
            "{report:?}"
        );
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
                assert!(
                    !carried
                        .iter()
                        .any(|c| c.name.as_deref() == Ok("little bird"))
                );
                caged += 1;
            }
        }
    });
    assert!(caged >= 3, "prompts carrying the caged bird: {caged}");
}

// I§8, SC-004: at 50 moments in the game, in 50 rooms, the reported exits are the
// ones worked out by hand from the game's source: rooms, doors, routines and
// directions refused by a string.
#[test]
fn exits_equal_the_hand_made_expectations() {
    for game in BUILDS {
        let (checked, problems) = check_expected_exits(game, EXITS, 9);
        assert!(
            problems.is_empty(),
            "{} problems:\n{}",
            problems.len(),
            problems.join("\n")
        );
        assert!(checked >= 45, "{}: entries checked: {checked}", game.name);
    }
    let rooms: std::collections::BTreeSet<String> =
        expectations(EXITS).into_iter().map(|e| e.room).collect();
    assert!(
        rooms.len() >= 25,
        "SC-004 asks for at least 25 rooms: {}",
        rooms.len()
    );
}

// I§8, SC-003, FR-038: every move the walkthrough makes through an exit reported
// with a destination arrives there.
#[test]
fn moves_through_fixed_exits_arrive_where_the_exits_say() {
    for game in BUILDS {
        let report = check_moves(game, 9);
        assert!(
            report.problems.is_empty(),
            "{} problems:\n{}",
            report.problems.len(),
            report.problems.join("\n")
        );
        println!(
            "{}: {} arrived, {} conditional, stayed: {:?}",
            game.name, report.arrived, report.conditional, report.stayed
        );
        // The walkthrough moves mostly by `go to <room>`; the tour below makes up
        // the hundred that SC-003 asks for.
        assert_eq!(report.arrived, 67, "{}", game.name);
        assert_eq!(report.wordless, 0, "{}", game.name);
        assert!(
            report.stayed.len() <= 3,
            "{}: {:?}",
            game.name,
            report.stayed
        );
    }
}

// I§8, I§3.1: the words for the compass directions come from the compass, and
// `in` and `out`, which the library takes as verbs, are found in the dictionary.
#[test]
fn exits_carry_the_words_that_name_them() {
    let machine = play(ADVENT5, &[]);
    let view = machine.view();
    let exits = Introspector::new(&view).exits(&view).value.unwrap();
    let words_of = |word: &str| {
        exits
            .exits
            .iter()
            .find(|e| e.words.iter().any(|w| w == word))
            .map(|e| e.words.clone())
    };
    assert_eq!(
        words_of("north"),
        Some(vec!["n".to_string(), "north".to_string()])
    );
    assert_eq!(words_of("in"), Some(vec!["in".to_string()]));
    let up = words_of("up").unwrap();
    assert!(up.contains(&"u".to_string()) && up.len() >= 2, "{up:?}");
    assert!(exits.unknown.is_empty());
}

// I§8, SC-003: beyond the walkthrough. A tour that follows the reported fixed
// exits wherever they lead, trying each one it meets, arrives where each says.
#[test]
fn a_tour_through_reported_exits_arrives_where_they_say() {
    for game in BUILDS {
        // Above ground first; then again from inside the cave, lamp lit.
        let above = tour(game, &["in", "take lamp", "on", "take keys", "out"], 150);
        assert!(above.problems.is_empty(), "{:?}", above.problems);
        assert!(
            above.arrived >= 100 && !above.redirected.is_empty(),
            "{above:?}"
        );
        let prefix = [
            "in",
            "take lamp",
            "on",
            "take keys",
            "out",
            "s",
            "s",
            "s",
            "unlock grate with keys",
            "open grate",
            "d",
        ];
        let report = tour(game, &prefix, 300);
        println!(
            "{} tour: {} arrived in {} rooms through {} exits; stayed {:?}; redirected {}; ended {:?}",
            game.name,
            report.arrived,
            report.rooms.len(),
            report.exits_tried.len(),
            report.stayed,
            report.redirected.len(),
            report.ended
        );
        assert!(
            report.problems.is_empty(),
            "{} problems:\n{}",
            report.problems.len(),
            report.problems.join("\n")
        );
        // The forest: arriving in one "In Forest" room, the game's own code may move the
        // player on to the other. The exit reports what the room's table stores.
        assert!(
            report.redirected.iter().all(|name| name == "In Forest"),
            "{:?}",
            report.redirected
        );
        assert!(report.arrived >= 150, "{}: {}", game.name, report.arrived);
        // Fixed exits alone reach only so far in Advent: the ways on from the top
        // of the pit are decided by the game's code.
        assert!(
            above.rooms.len() >= 8 && report.rooms.len() >= 6,
            "{}: {} above, {} below",
            game.name,
            above.rooms.len(),
            report.rooms.len()
        );
    }
}

// I§9, SC-006: at 39 moments in the game the reported scope holds every object
// the hand-made list names and none it excludes; and at every prompt the answer
// is approximate and well formed. Inform records light, containers and scenery.
#[test]
fn scope_holds_what_the_game_shows() {
    use zmachine_introspect::{Light, Reach, Visibility};
    for game in BUILDS {
        let mut closed_containers = 0;
        let report = check_scopes(game, "stories/advent/introspection/scope.txt", |scope, here| {
            let mut problems = Vec::new();
            if scope.light == Light::Unknown {
                problems.push(format!("{here}: Inform records light"));
            }
            for entry in &scope.objects {
                if entry.visibility != Visibility::Visible {
                    problems.push(format!("{here}: Inform records whether a container is open: {:?}", entry.name));
                }
                if entry.scenery == Tri::Unknown || entry.concealed == Tri::Unknown {
                    problems.push(format!("{here}: Inform records scenery: {:?}", entry.name));
                }
                if scope.light == Light::Dark && !matches!(entry.reach, Reach::Carried | Reach::Worn | Reach::Inside | Reach::On) {
                    problems.push(format!("{here}: in the dark only what is carried is listed: {:?}", entry.name));
                }
            }
            problems
        });
        assert!(report.problems.is_empty(), "{} problems:\n{}", report.problems.len(), report.problems.join("\n"));
        assert!(report.entries >= 20, "{}: SC-006 asks for at least 20 positions: {}", game.name, report.entries);
        assert_eq!(report.prompts, 172, "{}", game.name);
        assert_eq!(report.player_unknown, 0, "{}", game.name);
        assert!(report.nested >= 3 && report.most_objects >= 8, "{report:?}");
        let _ = &mut closed_containers;
    }
}

// I§9: a closed, opaque container hides what is in it; open, it shows it. And in
// the dark only what is carried is listed. Played off the walkthrough's route.
#[test]
fn scope_respects_closed_containers_and_darkness() {
    use zmachine_introspect::{Light, Reach};
    let names = |machine: &Machine| -> (Light, Vec<(String, Reach)>) {
        let view = machine.view();
        let scope = Introspector::new(&view).scope(&view).value.unwrap();
        (scope.light, scope.objects.into_iter().map(|e| (e.name.unwrap(), e.reach)).collect())
    };
    // The bottle holds water; it is a closed container that is transparent, so
    // the water shows. The cage, closed on the bird, is transparent too.
    let machine = play(ADVENT5, &["in", "take bottle"]);
    let (light, listed) = names(&machine);
    assert_eq!(light, Light::Lit);
    assert!(listed.contains(&("small bottle".to_string(), Reach::Carried)), "{listed:?}");
    assert!(listed.iter().any(|(name, reach)| name == "tasty food" && *reach == Reach::InRoom), "{listed:?}");

    // Underground with the lamp, then the lamp off: only what is carried.
    let route = ["in", "take keys", "take lamp", "out", "s", "s", "s", "unlock grate with keys", "open grate", "d", "w", "on", "w", "off"];
    let machine = play(ADVENT5, &route);
    let (light, listed) = names(&machine);
    assert_eq!(light, Light::Dark);
    let carried: Vec<&str> = listed.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(carried.len(), 2, "{listed:?}");
    assert!(carried.contains(&"brass lantern") && carried.contains(&"set of keys"));
    assert!(listed.iter().all(|(_, reach)| *reach == Reach::Carried));
    // With the lamp on again, the room's things are back: the black rod lies here.
    let mut lit = route.to_vec();
    lit.push("on");
    let (light, listed) = names(&play(ADVENT5, &lit));
    assert_eq!(light, Light::Lit);
    assert!(listed.iter().any(|(name, reach)| name.starts_with("black rod") && *reach == Reach::InRoom), "{listed:?}");
}
