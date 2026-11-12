//! What a trial says of one of introspection's reports
//! (specs/005-corpus-testing/spec.md FR-006; data-model.md "Verdict").

use crate::trial::Outcome;
use zmachine_introspect::{ExitKind, Exits, Unknown};

/// What introspection said of one direction at one place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reported {
    Fixed(u16),
    None,
    Conditional,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    Confirmed,
    Wrong,
    /// Reported as no exit; the game ended, asked a question or faulted.
    NoneOther,
    Conditional,
    Unknown,
    NoVerdict,
}

/// What was reported for `direction`, from the answer to the exits query. A
/// direction the answer does not mention is not an exit; where the whole answer
/// is unknown, every direction is.
pub fn reported(exits: &Result<Exits, Unknown>, direction: u8) -> Reported {
    let Ok(exits) = exits else {
        return Reported::Unknown;
    };
    if let Some(exit) = exits.exits.iter().find(|e| e.direction == direction) {
        return match (exit.kind, exit.destination) {
            (ExitKind::Fixed, Some(destination)) => Reported::Fixed(destination),
            // A fixed exit with no destination claims nothing that can be tried.
            (ExitKind::Fixed, None) => Reported::Unknown,
            (ExitKind::Conditional, _) => Reported::Conditional,
        };
    }
    if exits.unknown.iter().any(|u| u.direction == direction) {
        return Reported::Unknown;
    }
    Reported::None
}

pub fn verdict(reported: Reported, outcome: Outcome) -> Verdict {
    match (reported, outcome) {
        (_, Outcome::Lost) => Verdict::NoVerdict,
        (Reported::Fixed(said), Outcome::Moved(to)) if said == to => Verdict::Confirmed,
        (Reported::Fixed(_), _) => Verdict::Wrong,
        (Reported::None, Outcome::Stayed) => Verdict::Confirmed,
        (Reported::None, Outcome::Moved(_)) => Verdict::Wrong,
        (Reported::None, _) => Verdict::NoneOther,
        (Reported::Conditional, _) => Verdict::Conditional,
        (Reported::Unknown, _) => Verdict::Unknown,
    }
}
