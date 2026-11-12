//! The compass: which of twelve places lead somewhere, and what a tap on each
//! sends (specs/004-first-demo/data-model.md "Compass"; research R5).

/// A place on the compass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
    Up,
    Down,
    In,
    Out,
}

impl Place {
    /// The twelve places, in the order a snapshot lists them.
    pub const ALL: [Place; 12] = [
        Place::North,
        Place::Northeast,
        Place::East,
        Place::Southeast,
        Place::South,
        Place::Southwest,
        Place::West,
        Place::Northwest,
        Place::Up,
        Place::Down,
        Place::In,
        Place::Out,
    ];

    /// The place's identifier, which is also the English word a tap on it sends.
    pub fn id(self) -> &'static str {
        ""
    }

    /// What to show for the place.
    pub fn label(self) -> &'static str {
        ""
    }

    pub fn from_id(_id: &str) -> Option<Place> {
        None
    }
}

/// What the compass shows for a place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceState {
    /// Leads somewhere for certain.
    Open,
    /// The game decides.
    Maybe,
    /// Does not lead anywhere.
    Closed,
    /// Cannot be told.
    Unknown,
    /// Nothing is offered now: the game is not waiting for a command.
    Idle,
}

impl PlaceState {
    pub fn as_str(self) -> &'static str {
        ""
    }
}

/// One of the twelve places as the compass shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompassPlace {
    pub place: Place,
    pub label: &'static str,
    pub state: PlaceState,
    /// The line a tap sends; present exactly when the state is open or maybe.
    pub command: Option<String>,
    /// The destination's name, for an open place where it is known.
    pub leads_to: Option<String>,
    /// What a screen reader says for it.
    pub spoken: String,
}

/// An exit in a direction the compass has no place for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extra {
    pub label: String,
    pub state: PlaceState,
    pub command: String,
    pub leads_to: Option<String>,
    pub spoken: String,
}

/// The compass after a stop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compass {
    /// False when the game is not waiting for a command: every place is idle.
    pub offered: bool,
    /// False when the exits could not be determined, or one could not be placed.
    pub known: bool,
    /// What to show beneath the compass, if anything.
    pub note: Option<&'static str>,
    /// The twelve places, in order.
    pub places: Vec<CompassPlace>,
    pub extras: Vec<Extra>,
}

/// The note shown when the exits are not known.
pub const NOTE_UNKNOWN: &str = "Exits unknown: type a direction";
/// The note shown when the exits are known and there are none.
pub const NOTE_NONE: &str = "No exits here";

impl Compass {
    /// A compass that offers nothing.
    pub(crate) fn idle() -> Compass {
        Compass {
            offered: false,
            known: false,
            note: None,
            places: Vec::new(),
            extras: Vec::new(),
        }
    }

    pub fn place(&self, place: Place) -> Option<&CompassPlace> {
        self.places.iter().find(|p| p.place == place)
    }
}
