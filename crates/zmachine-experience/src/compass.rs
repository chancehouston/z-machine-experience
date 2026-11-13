//! The compass: which of twelve places lead somewhere, and what a tap on each
//! sends (specs/004-first-demo/data-model.md "Compass"; research R5).

use zmachine_introspect::{Answer, DoorState, ExitKind, Exits, Tri};

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
        match self {
            Place::North => "north",
            Place::Northeast => "northeast",
            Place::East => "east",
            Place::Southeast => "southeast",
            Place::South => "south",
            Place::Southwest => "southwest",
            Place::West => "west",
            Place::Northwest => "northwest",
            Place::Up => "up",
            Place::Down => "down",
            Place::In => "in",
            Place::Out => "out",
        }
    }

    /// What to show for the place.
    pub fn label(self) -> &'static str {
        match self {
            Place::North => "North",
            Place::Northeast => "Northeast",
            Place::East => "East",
            Place::Southeast => "Southeast",
            Place::South => "South",
            Place::Southwest => "Southwest",
            Place::West => "West",
            Place::Northwest => "Northwest",
            Place::Up => "Up",
            Place::Down => "Down",
            Place::In => "In",
            Place::Out => "Out",
        }
    }

    pub fn from_id(id: &str) -> Option<Place> {
        Place::ALL.into_iter().find(|place| place.id() == id)
    }
}

/// What the compass shows for a place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceState {
    /// Leads somewhere for certain.
    Open,
    /// The game decides.
    Maybe,
    /// A closed door: a way that is there, and shut (feature 006).
    Door,
    /// Does not lead anywhere.
    Closed,
    /// Cannot be told.
    Unknown,
    /// Nothing is offered now: the game is not waiting for a command.
    Idle,
}

impl PlaceState {
    pub fn as_str(self) -> &'static str {
        match self {
            PlaceState::Open => "open",
            PlaceState::Maybe => "maybe",
            PlaceState::Door => "door",
            PlaceState::Closed => "closed",
            PlaceState::Unknown => "unknown",
            PlaceState::Idle => "idle",
        }
    }
}

/// One of the twelve places as the compass shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompassPlace {
    pub place: Place,
    pub label: &'static str,
    pub state: PlaceState,
    /// The line a tap sends; present exactly when the state is open, maybe or
    /// door.
    pub command: Option<String>,
    /// The destination's name, for an open place where it is known.
    pub leads_to: Option<String>,
    /// Whether the way leads into darkness, as far as is known (feature 008).
    pub dark_ahead: bool,
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
    /// Whether the way leads into darkness, as far as is known (feature 008).
    pub dark_ahead: bool,
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

/// What a screen reader says for a place or an extra.
fn spoken(label: &str, state: PlaceState, leads_to: Option<&str>, dark_ahead: bool) -> String {
    let said = spoken_of(label, state, leads_to);
    if dark_ahead {
        format!("{said}{SPOKEN_DARK}")
    } else {
        said
    }
}

/// What is added to what a screen reader says for a way into darkness.
pub const SPOKEN_DARK: &str = ", into darkness";

fn spoken_of(label: &str, state: PlaceState, leads_to: Option<&str>) -> String {
    match (state, leads_to) {
        (PlaceState::Open, Some(to)) => format!("{label}, to {to}"),
        (PlaceState::Open, None) => label.to_string(),
        (PlaceState::Maybe, _) => format!("{label}, may be blocked"),
        (PlaceState::Door, _) => format!("{label}, closed door"),
        (PlaceState::Closed, _) => format!("{label}, no exit"),
        (PlaceState::Unknown, _) => format!("{label}, unknown"),
        (PlaceState::Idle, _) => format!("{label}, not available now"),
    }
}

fn place_in(
    place: Place,
    state: PlaceState,
    leads_to: Option<String>,
    dark_ahead: bool,
) -> CompassPlace {
    let offered = matches!(
        state,
        PlaceState::Open | PlaceState::Maybe | PlaceState::Door
    );
    CompassPlace {
        place,
        label: place.label(),
        state,
        command: offered.then(|| place.id().to_string()),
        spoken: spoken(place.label(), state, leads_to.as_deref(), dark_ahead),
        leads_to,
        dark_ahead,
    }
}

/// Every place in one state, with nothing to send.
fn all(state: PlaceState) -> Vec<CompassPlace> {
    Place::ALL
        .into_iter()
        .map(|place| place_in(place, state, None, false))
        .collect()
}

/// The first letter in upper case, for showing a word as a label.
fn capitalised(word: &str) -> String {
    let mut letters = word.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => String::new(),
    }
}

impl Compass {
    /// A compass that offers nothing: the game is not waiting for a command.
    pub(crate) fn idle() -> Compass {
        Compass {
            offered: false,
            known: false,
            note: None,
            places: all(PlaceState::Idle),
            extras: Vec::new(),
        }
    }

    /// The compass for a room's exits, as the introspection interface reports
    /// them. `word_len` is how many characters of a word the story file keeps.
    ///
    /// An exit belongs to the place whose English word, cut to that length, is
    /// among its words; one that fits no place is an extra. Nothing is chosen
    /// between: two exits for one place leave the place unknown.
    pub(crate) fn of(exits: &Answer<Exits>, word_len: usize) -> Compass {
        let Ok(exits) = &exits.value else {
            return Compass {
                offered: true,
                known: false,
                note: Some(NOTE_UNKNOWN),
                places: all(PlaceState::Unknown),
                extras: Vec::new(),
            };
        };
        let cut = |place: Place| -> String { place.id().chars().take(word_len).collect() };
        let place_of = |words: &[String]| {
            Place::ALL
                .into_iter()
                .find(|&place| words.contains(&cut(place)))
        };

        let mut known = true;
        let mut found: Vec<(Place, PlaceState, Option<String>, bool)> = Vec::new();
        let mut extras = Vec::new();
        let mut put = |place: Place,
                       state: PlaceState,
                       leads_to: Option<String>,
                       dark: bool,
                       known: &mut bool| {
            match found.iter_mut().find(|(p, ..)| *p == place) {
                // A second exit for the same place: neither is preferred.
                Some(entry) => {
                    *entry = (place, PlaceState::Unknown, None, false);
                    *known = false;
                }
                None => found.push((place, state, leads_to, dark)),
            }
        };
        for exit in &exits.exits {
            let (state, leads_to) = match exit.kind {
                ExitKind::Fixed => (
                    PlaceState::Open,
                    exit.destination_name.clone().and_then(Result::ok),
                ),
                ExitKind::Conditional => (PlaceState::Maybe, None),
                // A door a game pack let be read (docs/introspection.md, 12.3).
                // Open, it is a way like any other. Closed, it is still a way
                // to try, and the room behind it is not told: the player has
                // not seen through it.
                ExitKind::Door => match exit.state {
                    Some(DoorState::Open) => (
                        PlaceState::Open,
                        exit.destination_name.clone().and_then(Result::ok),
                    ),
                    Some(DoorState::Closed) => (PlaceState::Door, None),
                    None => (PlaceState::Maybe, None),
                },
            };
            // A way into darkness is marked only where it is an open way: what
            // is behind a closed door is not told (feature 008).
            let dark = state == PlaceState::Open && exit.dark_ahead == Tri::Yes;
            match place_of(&exit.words) {
                Some(place) => put(place, state, leads_to, dark, &mut known),
                None => match exit
                    .words
                    .iter()
                    .max_by(|a, b| a.len().cmp(&b.len()).then(b.cmp(a)))
                {
                    Some(word) => {
                        let label = capitalised(word);
                        extras.push(Extra {
                            spoken: spoken(&label, state, leads_to.as_deref(), dark),
                            label,
                            state,
                            command: word.clone(),
                            leads_to,
                            dark_ahead: dark,
                        });
                    }
                    // No word names it: it cannot be shown or sent.
                    None => known = false,
                },
            }
        }
        for direction in &exits.unknown {
            known = false;
            if let Some(place) = place_of(&direction.words) {
                match found.iter_mut().find(|(p, ..)| *p == place) {
                    Some(entry) => *entry = (place, PlaceState::Unknown, None, false),
                    None => found.push((place, PlaceState::Unknown, None, false)),
                }
            }
        }
        let places: Vec<CompassPlace> = Place::ALL
            .into_iter()
            .map(|place| match found.iter().find(|(p, ..)| *p == place) {
                Some((_, state, leads_to, dark)) => {
                    place_in(place, *state, leads_to.clone(), *dark)
                }
                None => place_in(place, PlaceState::Closed, None, false),
            })
            .collect();
        let any = places.iter().any(|p| p.command.is_some()) || !extras.is_empty();
        let note = match (known, any) {
            (false, _) => Some(NOTE_UNKNOWN),
            (true, false) => Some(NOTE_NONE),
            (true, true) => None,
        };
        Compass {
            offered: true,
            known,
            note,
            places,
            extras,
        }
    }

    pub fn place(&self, place: Place) -> Option<&CompassPlace> {
        self.places.iter().find(|p| p.place == place)
    }
}
