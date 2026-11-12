//! Trials change nothing in the game being walked (specs/005-corpus-testing/
//! spec.md FR-002, FR-008; SC-002).

mod common;

use common::*;
use zmachine_corpus::directions::directions;
use zmachine_corpus::trial::{LIMIT, trial};
use zmachine_corpus::walk::walkthrough;
use zmachine_introspect::Introspector;

// SC-002: each walkthrough's transcript is the reference, byte for byte, with a
// trial of every direction run at every command prompt.
#[test]
fn a_walkthrough_with_trials_gives_the_reference_transcript() {
    for game in GAMES {
        let mut trials = 0;
        let mut tools = None;
        let replay = walkthrough(
            read(game.story),
            &read_text(game.walkthrough),
            game.seed,
            |driver, _| {
                let (introspector, found) = tools.get_or_insert_with(|| {
                    let view = driver.machine.view();
                    let introspector = Introspector::new(&view);
                    let found = directions(&introspector, &view);
                    (introspector, found)
                });
                let address = driver.command_address.unwrap();
                for direction in found.iter() {
                    if let Some(word) = &direction.word {
                        trial(&driver.machine, introspector, address, word, LIMIT);
                        trials += 1;
                    }
                }
            },
        );
        assert!(trials >= 12 * 150, "{}: {trials} trials", game.name);
        assert!(
            replay.transcript == read_text(game.reference),
            "{}: the transcript differs from the reference",
            game.name
        );
    }
}
