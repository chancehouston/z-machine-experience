//! Recognising the family of the committed story files (docs/introspection.md
//! section 3, cited I§3; SC-008).

mod common;

use common::*;
use zmachine_introspect::{Exactness, Family, Introspector, Unknown};

// I§3.1, I§3.2: the two games are recognised, each as its own family.
#[test]
fn the_games_are_recognised() {
    for (story, family) in [
        ("stories/zork1/zork1.z3", Family::Zil),
        ("stories/advent/advent.z5", Family::Inform6),
        ("stories/advent/advent.z8", Family::Inform6),
    ] {
        let machine = load(story, 1);
        assert_eq!(
            Introspector::new(&machine.view()).family(),
            family,
            "{story}"
        );
    }
}

// I§3.3, SC-008: the test programs are compiled with Inform but have no library.
// They are unrecognised; the family-dependent queries say so, and the tree and
// dictionary are still answered in full.
#[test]
fn the_test_programs_are_unrecognised() {
    for story in [
        "stories/czech/czech.z3",
        "stories/czech/czech.z5",
        "stories/czech/czech.z8",
        "stories/praxix/praxix.z5",
    ] {
        let machine = load(story, 1);
        let view = machine.view();
        let i = Introspector::new(&view);
        assert_eq!(i.family(), Family::Unrecognised, "{story}");
        assert_eq!(
            i.location(&view).value.err(),
            Some(Unknown::UnrecognisedFamily),
            "{story}"
        );
        assert_eq!(
            i.inventory(&view).value.err(),
            Some(Unknown::UnrecognisedFamily),
            "{story}"
        );
        assert_eq!(
            i.exits(&view).value.err(),
            Some(Unknown::UnrecognisedFamily),
            "{story}"
        );
        let scope = i.scope(&view);
        assert_eq!(
            scope.value.err(),
            Some(Unknown::UnrecognisedFamily),
            "{story}"
        );
        assert_eq!(
            scope.exactness,
            Exactness::Approximate,
            "{story}: scope is approximate even when unknown"
        );
        assert!(!i.object_tree(&view).value.unwrap().is_empty(), "{story}");
        let words = i.dictionary(&view).value.unwrap();
        assert!(
            words
                .iter()
                .all(|w| w.parts == Err(Unknown::UnrecognisedFamily)),
            "{story}"
        );
    }
}

// I§3: the family is decided from the file as shipped, so it is the same at
// every stop of a game, from before it has run to after it has ended.
#[test]
fn the_family_does_not_change_during_play() {
    for (game, family) in [
        (ZORK1, Family::Zil),
        (ADVENT5, Family::Inform6),
        (ADVENT8, Family::Inform6),
    ] {
        let fresh = load(game.story, game.seed);
        assert_eq!(
            Introspector::new(&fresh.view()).family(),
            family,
            "{} before it runs",
            game.name
        );
        let mut stops = 0;
        walk(game, |machine, stop| {
            if stop.number % 25 == 1 || stop.wants == Wants::Nothing {
                assert_eq!(
                    Introspector::new(&machine.view()).family(),
                    family,
                    "{} at stop {}",
                    game.name,
                    stop.number
                );
                stops += 1;
            }
        });
        assert!(stops >= 7, "{}", game.name);
    }
}
