//! What was reported, and the verdict of a trial on it
//! (specs/005-corpus-testing/spec.md FR-006; data-model.md "Reported", "Verdict").

use zmachine_corpus::report::Case;
use zmachine_corpus::trial::Outcome;
use zmachine_corpus::verdict::{Reported, Verdict, judge, reported, verdict};
use zmachine_introspect::{Exit, ExitKind, Exits, Unknown, UnknownDirection};

fn exit(direction: u8, kind: ExitKind, destination: Option<u16>) -> Exit {
    Exit {
        direction,
        words: vec![],
        kind,
        destination,
        destination_name: None,
    }
}

// data-model.md "Reported", row by row.
#[test]
fn what_was_reported_is_read_from_the_exits_answer() {
    let exits = Ok(Exits {
        exits: vec![
            exit(31, ExitKind::Fixed, Some(7)),
            exit(30, ExitKind::Conditional, None),
        ],
        unknown: vec![UnknownDirection {
            direction: 29,
            words: vec![],
            reason: Unknown::Damaged,
        }],
    });
    assert_eq!(reported(&exits, 31), Reported::Fixed(7));
    assert_eq!(reported(&exits, 30), Reported::Conditional);
    assert_eq!(reported(&exits, 29), Reported::Unknown);
    // In neither list: not an exit.
    assert_eq!(reported(&exits, 28), Reported::None);
    // The whole answer unknown: every direction is.
    let unknown = Err(Unknown::NoPlayer);
    assert_eq!(reported(&unknown, 31), Reported::Unknown);
    assert_eq!(reported(&unknown, 28), Reported::Unknown);
}

// data-model.md "Verdict": every pair of what can be reported and what can
// happen, 24 in all.
#[test]
fn every_report_and_outcome_has_one_verdict() {
    use Outcome::*;
    use Verdict as V;
    let cases = [
        // Leads to room 7.
        (Reported::Fixed(7), Moved(7), V::Confirmed),
        (Reported::Fixed(7), Moved(8), V::Wrong),
        (Reported::Fixed(7), Stayed, V::Wrong),
        (Reported::Fixed(7), Question, V::Wrong),
        (Reported::Fixed(7), Ended, V::Wrong),
        (Reported::Fixed(7), Fault, V::Wrong),
        (Reported::Fixed(7), Lost, V::NoVerdict),
        // Not an exit.
        (Reported::None, Stayed, V::Confirmed),
        (Reported::None, Moved(7), V::Wrong),
        (Reported::None, Question, V::NoneOther),
        (Reported::None, Ended, V::NoneOther),
        (Reported::None, Fault, V::NoneOther),
        (Reported::None, Lost, V::NoVerdict),
        // Conditional: introspection claimed nothing, so never wrong.
        (Reported::Conditional, Moved(7), V::Conditional),
        (Reported::Conditional, Stayed, V::Conditional),
        (Reported::Conditional, Question, V::Conditional),
        (Reported::Conditional, Ended, V::Conditional),
        (Reported::Conditional, Fault, V::Conditional),
        (Reported::Conditional, Lost, V::NoVerdict),
        // Unknown.
        (Reported::Unknown, Moved(7), V::Unknown),
        (Reported::Unknown, Stayed, V::Unknown),
        (Reported::Unknown, Question, V::Unknown),
        (Reported::Unknown, Ended, V::Unknown),
        (Reported::Unknown, Fault, V::Unknown),
        (Reported::Unknown, Lost, V::NoVerdict),
    ];
    for (said, happened, expected) in cases {
        assert_eq!(verdict(said, happened), expected, "{said:?} {happened:?}");
    }
}

// spec.md Edge Cases, FR-006: a move to the room the player is already in cannot
// be told from staying. An exit reported as leading to the player's own room is
// neither confirmed nor wrong when the player is still there: no verdict.
#[test]
fn an_exit_to_the_room_the_player_is_in_has_no_verdict_when_the_player_stays() {
    let case = |room, reported, outcome| Case {
        room,
        direction: 31,
        reported,
        outcome,
    };
    assert_eq!(
        judge(&case(7, Reported::Fixed(7), Outcome::Stayed)),
        Verdict::NoVerdict
    );
    // From any other room the same report and outcome are wrong.
    assert_eq!(
        judge(&case(8, Reported::Fixed(7), Outcome::Stayed)),
        Verdict::Wrong
    );
    // And it is wrong if the player ends up somewhere else.
    assert_eq!(
        judge(&case(7, Reported::Fixed(7), Outcome::Moved(9))),
        Verdict::Wrong
    );
    // Otherwise a case is judged by its report and outcome alone.
    assert_eq!(
        judge(&case(7, Reported::None, Outcome::Stayed)),
        Verdict::Confirmed
    );
}
