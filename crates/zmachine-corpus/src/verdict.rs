//! What a trial says of one of introspection's reports
//! (specs/005-corpus-testing/spec.md FR-006; data-model.md "Verdict").

use crate::trial::Outcome;
use zmachine_introspect::{Exits, Unknown};

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

/// What was reported for `direction`, from the answer to the exits query.
pub fn reported(_exits: &Result<Exits, Unknown>, _direction: u8) -> Reported {
    Reported::Unknown
}

pub fn verdict(_reported: Reported, _outcome: Outcome) -> Verdict {
    Verdict::NoVerdict
}
