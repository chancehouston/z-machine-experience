//! What a trial says of one of introspection's reports
//! (specs/005-corpus-testing/spec.md FR-006; data-model.md "Verdict").

use crate::trial::Outcome;
use zmachine_introspect::{DoorState, ExitKind, Exits, Source, Unknown};

/// What introspection said of one direction at one place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reported {
    Fixed(u16),
    None,
    Conditional,
    Unknown,
    /// A door a game pack let be read as open, to this room.
    Open(u16),
    /// A door read as closed, with this room behind it.
    Closed(u16),
    /// Conditional because a game pack names the exit's variable as untrusted.
    Distrusted,
}

impl Reported {
    /// Whether the report rests on a game pack (docs/introspection.md, 12.5).
    pub fn marked(self) -> bool {
        matches!(
            self,
            Reported::Open(_) | Reported::Closed(_) | Reported::Distrusted
        )
    }
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
            (ExitKind::Conditional, _) if exit.source == Source::Pack => Reported::Distrusted,
            (ExitKind::Conditional, _) => Reported::Conditional,
            (ExitKind::Door, Some(behind)) => match exit.state {
                Some(DoorState::Open) => Reported::Open(behind),
                Some(DoorState::Closed) => Reported::Closed(behind),
                None => Reported::Unknown,
            },
            (ExitKind::Door, None) => Reported::Unknown,
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
        (Reported::Open(said), Outcome::Moved(to)) if said == to => Verdict::Confirmed,
        (Reported::Open(_), _) => Verdict::Wrong,
        (Reported::Closed(_), Outcome::Stayed) => Verdict::Confirmed,
        (Reported::Closed(_), Outcome::Moved(_)) => Verdict::Wrong,
        (Reported::Closed(_), _) => Verdict::NoneOther,
        (Reported::Distrusted, _) => Verdict::Conditional,
    }
}

/// The verdict on a case, which knows the room the trial started in. An exit
/// reported as leading to that same room has no verdict when the player is
/// still there: a move to the room one is in cannot be told from staying.
pub fn judge(case: &crate::report::Case) -> Verdict {
    match (case.reported, case.outcome) {
        (Reported::Fixed(to) | Reported::Open(to), Outcome::Stayed) if to == case.room => {
            Verdict::NoVerdict
        }
        (reported, outcome) => verdict(reported, outcome),
    }
}
