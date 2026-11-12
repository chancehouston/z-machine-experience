//! Game packs on the games the repository holds (docs/introspection.md section
//! 12, cited I§12.n; specs/006-game-packs). The rules themselves are tested on
//! built stories in `src/pack_tests.rs`. Rooms, doors and attributes are by
//! number here.

mod common;

use common::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use zmachine_introspect::{DoorState, ExitKind, Introspector, PackState, Source};

/// A pack for a story file, with its identity worked out from the file.
fn pack_for(story: &str, entries: &str) -> String {
    let image = read(story);
    let digest: String = Sha256::digest(&image)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!(
        "pack: 1\nsha256: {digest}\nrelease: {}\nserial: {}\n{entries}",
        u16::from_be_bytes([image[2], image[3]]),
        String::from_utf8(image[0x12..0x18].to_vec()).unwrap()
    )
}

/// Zork I's "open" attribute (specs/006-game-packs/research.md R5).
const ZORK1_OPEN: &str = "open-attribute: 11\n";

// I§12.3, I§12.5; US1 scenarios 1 and 2: along Zork I's walkthrough, with the
// attribute given, every 5-byte exit is a door, seen both closed and open, to
// the same room each time, and marked. Nothing else is marked.
#[test]
fn zork1s_doors_are_open_or_closed() {
    let pack = pack_for(ZORK1.story, ZORK1_OPEN);
    let mut seen: BTreeSet<(u16, u8, u16, u16, bool)> = BTreeSet::new();
    let mut conditional_with = 0;
    let mut conditional_without = 0;
    walk(ZORK1, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        let view = machine.view();
        let with = Introspector::with_pack(&view, &pack);
        assert_eq!(with.pack(), PackState::InForce);
        let room = with.location(&view).value.unwrap().room;
        let exits = with.exits(&view).value.unwrap();
        let plain = Introspector::new(&view).exits(&view).value.unwrap();
        assert_eq!(exits.exits.len(), plain.exits.len());
        assert_eq!(exits.unknown, plain.unknown);
        for (exit, was) in exits.exits.iter().zip(&plain.exits) {
            conditional_with += (exit.kind == ExitKind::Conditional) as usize;
            conditional_without += (was.kind == ExitKind::Conditional) as usize;
            if exit.kind != ExitKind::Door {
                assert_eq!(exit, was, "an exit that is not a door is unchanged");
                assert_eq!(exit.source, Source::File);
                continue;
            }
            assert_eq!(was.kind, ExitKind::Conditional);
            assert_eq!(exit.source, Source::Pack);
            let (to, door) = (exit.destination.unwrap(), exit.door.unwrap());
            seen.insert((
                room,
                exit.direction,
                to,
                door,
                exit.state == Some(DoorState::Open),
            ));
        }
    });
    // Two doors, at three rooms; each door seen both ways somewhere.
    let doors: BTreeSet<u16> = seen.iter().map(|s| s.3).collect();
    assert_eq!(doors.into_iter().collect::<Vec<_>>(), [240, 243]);
    for door in [240, 243] {
        for open in [false, true] {
            assert!(
                seen.iter().any(|s| s.3 == door && s.4 == open),
                "door {door} open {open}: {seen:?}"
            );
        }
    }
    // The room behind a door does not depend on the door's state.
    for a in &seen {
        for b in &seen {
            if (a.0, a.1) == (b.0, b.1) {
                assert_eq!((a.2, a.3), (b.2, b.3));
            }
        }
    }
    assert!(
        conditional_with < conditional_without,
        "{conditional_with} of {conditional_without} conditional exits remain"
    );
}

// I§12.2, I§12.6; FR-013, FR-014; SC-006: with a pack that is not in force every
// answer is the answer without one; with one in force only `exits` differs; and
// the walkthrough's transcript is the reference, byte for byte, with every query
// asked with the pack at every stop.
#[test]
fn a_pack_changes_only_exits_and_no_transcript() {
    for game in GAMES {
        let entries = if game.name == ZORK1.name {
            ZORK1_OPEN
        } else {
            ""
        };
        let pack = pack_for(game.story, entries);
        // The same pack with the last character of its serial changed.
        let at = pack.find("serial: ").unwrap() + "serial: ".len() + 5;
        let last = if pack.as_bytes()[at] == b'0' {
            "1"
        } else {
            "0"
        };
        let mut other = pack.clone();
        other.replace_range(at..at + 1, last);
        let mut asked = 0;
        let replay = walk(game, |machine, _| {
            let view = machine.view();
            let plain = Introspector::new(&view);
            let with = Introspector::with_pack(&view, &pack);
            assert_eq!(with.pack(), PackState::InForce, "{}", game.name);
            assert_eq!(with.location(&view), plain.location(&view));
            assert_eq!(with.inventory(&view), plain.inventory(&view));
            assert_eq!(with.scope(&view), plain.scope(&view));
            for i in [
                Introspector::with_pack(&view, &other),
                Introspector::with_pack(&view, "pack: 1\n"),
            ] {
                assert_ne!(i.pack(), PackState::InForce);
                assert_eq!(i.exits(&view), plain.exits(&view));
            }
            for exit in &plain
                .exits(&view)
                .value
                .map(|e| e.exits)
                .unwrap_or_default()
            {
                assert_eq!(
                    (exit.door, exit.state, exit.source),
                    (None, None, Source::File)
                );
            }
            asked += 1;
        });
        assert_eq!(
            replay.transcript,
            read_text(game.reference),
            "{}",
            game.name
        );
        assert!(asked >= 170, "{}: {asked}", game.name);
    }
}

// I§12.2: the tables are the same with a pack, on a whole game's file.
#[test]
fn the_tables_are_the_same_with_a_pack() {
    let machine = load(ZORK1.story, ZORK1.seed);
    let view = machine.view();
    let pack = pack_for(ZORK1.story, ZORK1_OPEN);
    let (plain, with) = (
        Introspector::new(&view),
        Introspector::with_pack(&view, &pack),
    );
    assert_eq!(with.object_tree(&view), plain.object_tree(&view));
    assert_eq!(with.dictionary(&view), plain.dictionary(&view));
    assert_eq!(with.family(), plain.family());
}
