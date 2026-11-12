//! The family-dependent queries on Zork I, checked against the game itself at
//! every prompt of the walkthrough (docs/introspection.md, cited I§; SC-002 to
//! SC-006). Zork I is of the ZIL family.

mod common;

use common::*;
use zmachine_introspect::{Introspector, Tri, Unknown};

const EXITS: &str = "stories/zork1/introspection/exits.txt";

// I§6, SC-002, FR-037: at every command prompt the location is known, exact, and
// the room the game names. ZIL does not record light.
#[test]
fn location_agrees_with_the_game_at_every_prompt() {
    // Zork I announces a dark room with these words in place of its name.
    let report = check_locations(ZORK1, EXITS, |stop, _| stop.text.contains("pitch black"));
    assert!(
        report.problems.is_empty(),
        "{} problems:\n{}",
        report.problems.len(),
        report.problems.join("\n")
    );
    assert_eq!(report.prompts, 332);
    assert!(
        report.headings >= 150,
        "room headings checked: {}",
        report.headings
    );
    assert!(report.changes >= 150, "moves checked: {}", report.changes);
    assert_eq!(
        (report.lit, report.dark),
        (0, 0),
        "ZIL does not record light: always unknown"
    );
}

// I§6: the opening, a named room mid-game, and the player inside the boat.
#[test]
fn location_at_known_moments() {
    let mut seen = Vec::new();
    let mut in_boat = 0;
    walk(ZORK1, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        let view = machine.view();
        let i = Introspector::new(&view);
        let location = i.location(&view).value.unwrap();
        if stop.number == 1 {
            assert_eq!(location.name, Ok("West of House".to_string()));
            assert_eq!(location.holder, Ok(None));
            assert_eq!(location.lit, Tri::Unknown);
        }
        if let Ok(Some(holder)) = location.holder {
            let tree = i.object_tree(&view).value.unwrap();
            assert_eq!(
                tree[holder as usize - 1].name,
                "magic boat",
                "stop {}",
                stop.number
            );
            in_boat += 1;
        }
        seen.push(location.name.unwrap());
    });
    assert!(seen.contains(&"Kitchen".to_string()) && seen.contains(&"Treasure Room".to_string()));
    assert!(in_boat >= 5, "prompts spent in the boat: {in_boat}");
}

// I§7, SC-005: the inventory is what the game lists when a copy of it is asked.
// It is unknown only by a tie for the player, and at no more than 3% of prompts.
#[test]
fn inventory_agrees_with_the_game_at_every_prompt() {
    let report = check_inventories(ZORK1, "You are carrying:");
    assert!(
        report.problems.is_empty(),
        "{} problems:\n{}",
        report.problems.len(),
        report.problems.join("\n")
    );
    assert_eq!(report.prompts, 332);
    assert!(
        report.not_an_inventory.len() <= 2,
        "prompts that were not for a command: {:?}",
        report.not_an_inventory
    );
    assert!(
        report
            .unknown
            .iter()
            .all(|(_, reason)| *reason == Unknown::AmbiguousPlayer),
        "{:?}",
        report.unknown
    );
    println!("Zork I: inventory unknown at stops {:?}", report.unknown);
    assert!(
        report.unknown.len() * 100 <= report.prompts * 3,
        "unknown at {} of {} prompts",
        report.unknown.len(),
        report.prompts
    );
    assert!(
        report.compared >= 320 && report.empty >= 2 && report.most_carried >= 6,
        "{report:?}"
    );
}

// I§7: a carried container's contents are nested under it, not listed beside it,
// and an object just taken is carried at the next prompt.
#[test]
fn inventory_at_known_moments() {
    let mut nested = 0;
    let mut after_take = 0;
    let mut previous = String::new();
    walk(ZORK1, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        let view = machine.view();
        let Ok(carried) = Introspector::new(&view).inventory(&view).value else {
            return;
        };
        for item in &carried {
            if !item.contents.is_empty() {
                nested += 1;
                let inner = item.contents[0].name.clone().unwrap();
                assert!(
                    !carried
                        .iter()
                        .any(|c| c.name.as_deref() == Ok(inner.as_str())),
                    "{inner} is listed twice"
                );
            }
            assert_eq!(item.worn, Tri::Unknown, "ZIL does not record what is worn");
        }
        if previous == "take egg" {
            assert!(
                carried
                    .iter()
                    .any(|c| c.name.as_deref() == Ok("jewel-encrusted egg")),
                "{carried:?}"
            );
            after_take += 1;
        }
        previous = stop.line.clone().map(|(_, text)| text).unwrap_or_default();
    });
    assert_eq!(after_take, 3, "the walkthrough takes the egg three times");
    assert!(
        nested >= 10,
        "prompts with a container that holds something: {nested}"
    );
}

// I§8, SC-004: at 59 moments in the game, in 49 rooms, the reported exits are the
// ones worked out by hand from the game's ZIL source, including every form of
// exit and flag exits before and after their flags change.
#[test]
fn exits_equal_the_hand_made_expectations() {
    let (checked, problems) = check_expected_exits(ZORK1, EXITS, 6);
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
    assert!(checked >= 50, "entries checked: {checked}");
    let rooms: std::collections::BTreeSet<String> = expectations(EXITS).into_iter().map(|e| e.room).collect();
    assert!(rooms.len() >= 25, "SC-004 asks for at least 25 rooms: {}", rooms.len());
}

// I§8, SC-003, FR-038: every move the walkthrough makes through an exit reported
// with a destination arrives there.
#[test]
fn moves_through_fixed_exits_arrive_where_the_exits_say() {
    let report = check_moves(ZORK1, 6);
    assert!(report.problems.is_empty(), "{} problems:\n{}", report.problems.len(), report.problems.join("\n"));
    println!("Zork I: {} arrived, {} conditional, stayed: {:?}", report.arrived, report.conditional, report.stayed);
    assert!(report.arrived >= 100, "SC-003 asks for at least 100 such moves: {}", report.arrived);
    assert_eq!(report.wordless, 0, "every ZIL direction has a word");
    assert!(report.exits_seen >= 1000, "{}", report.exits_seen);
    assert!(report.stayed.len() <= 3, "a fixed exit the game did not let the player through: {:?}", report.stayed);
}
