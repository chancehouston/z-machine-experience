//! The compass, from hand-made introspection answers
//! (specs/004-first-demo/research.md R5; data-model.md "Compass";
//! contracts/experience-api.md, cited E§; docs/introspection.md section 8, I§8).

use crate::compass::{Compass, NOTE_NONE, NOTE_UNKNOWN, Place, PlaceState};
use zmachine_introspect::{Answer, Exactness, Exit, ExitKind, Exits, Unknown, UnknownDirection};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|w| w.to_string()).collect()
}

fn fixed(direction: u8, list: &[&str], to: &str) -> Exit {
    Exit {
        direction,
        words: words(list),
        kind: ExitKind::Fixed,
        destination: Some(100 + direction as u16),
        destination_name: Some(Ok(to.to_string())),
        door: None,
        state: None,
        dark_ahead: zmachine_introspect::Tri::Unknown,
        source: zmachine_introspect::Source::File,
    }
}

fn conditional(direction: u8, list: &[&str]) -> Exit {
    Exit {
        direction,
        words: words(list),
        kind: ExitKind::Conditional,
        destination: None,
        destination_name: None,
        door: None,
        state: None,
        dark_ahead: zmachine_introspect::Tri::Unknown,
        source: zmachine_introspect::Source::File,
    }
}

fn compass(exits: Vec<Exit>, word_len: usize) -> Compass {
    Compass::of(
        &Answer::exact(Exits {
            exits,
            unknown: vec![],
        }),
        word_len,
    )
}

fn state(c: &Compass, place: Place) -> PlaceState {
    c.place(place)
        .unwrap_or_else(|| panic!("{place:?} is not listed"))
        .state
}

// Data-model "Compass": the twelve places are always listed, in order, each with
// its identifier and label.
#[test]
fn the_twelve_places_are_always_listed_in_order() {
    let c = compass(vec![], 6);
    let ids: Vec<&str> = c.places.iter().map(|p| p.place.id()).collect();
    assert_eq!(
        ids,
        [
            "north",
            "northeast",
            "east",
            "southeast",
            "south",
            "southwest",
            "west",
            "northwest",
            "up",
            "down",
            "in",
            "out"
        ]
    );
    let labels: Vec<&str> = c.places.iter().map(|p| p.label).collect();
    assert_eq!(
        labels,
        [
            "North",
            "Northeast",
            "East",
            "Southeast",
            "South",
            "Southwest",
            "West",
            "Northwest",
            "Up",
            "Down",
            "In",
            "Out"
        ]
    );
    for place in Place::ALL {
        assert_eq!(Place::from_id(place.id()), Some(place));
        assert_eq!(place.label().to_lowercase(), place.id());
    }
    assert_eq!(Place::from_id("northe"), None);
    assert_eq!(Place::from_id("North"), None);
    assert_eq!(
        Compass::idle().places.len(),
        12,
        "an idle compass lists them too"
    );
}

// R5, I§8: a fixed exit whose words include a place's word, cut to the length
// the dictionary keeps, is that place: open, sending the full English word, and
// leading to the destination's name.
#[test]
fn a_fixed_exit_lights_its_place() {
    // Version 3 keeps six characters: `northeast` is stored as `northe`.
    let c = compass(
        vec![
            fixed(31, &["n", "north"], "Forest Path"),
            fixed(27, &["ne", "northe"], "Clearing"),
        ],
        6,
    );
    let north = c.place(Place::North).unwrap();
    assert_eq!(north.state, PlaceState::Open);
    assert_eq!(north.command.as_deref(), Some("north"));
    assert_eq!(north.leads_to.as_deref(), Some("Forest Path"));
    let northeast = c.place(Place::Northeast).unwrap();
    assert_eq!(
        (northeast.state, northeast.command.as_deref()),
        (PlaceState::Open, Some("northeast"))
    );
    assert_eq!(northeast.leads_to.as_deref(), Some("Clearing"));
    assert!(c.offered && c.known);
    assert_eq!(c.note, None);
    assert!(c.extras.is_empty());
    // Every other place is closed, with nothing to send.
    for place in Place::ALL {
        if place != Place::North && place != Place::Northeast {
            let p = c.place(place).unwrap();
            assert_eq!(
                (p.state, p.command.as_deref(), p.leads_to.as_deref()),
                (PlaceState::Closed, None, None),
                "{place:?}"
            );
        }
    }
}

// R5: versions 5 and 8 keep nine characters, so `northeast` is stored whole,
// and `north` must not be taken for it.
#[test]
fn the_cut_follows_the_dictionary() {
    let c = compass(
        vec![
            fixed(11, &["ne", "northeast"], "A"),
            fixed(7, &["n", "north"], "B"),
            fixed(14, &["sw", "southwest"], "C"),
        ],
        9,
    );
    assert_eq!(state(&c, Place::Northeast), PlaceState::Open);
    assert_eq!(state(&c, Place::North), PlaceState::Open);
    assert_eq!(state(&c, Place::Southwest), PlaceState::Open);
    assert_eq!(
        c.place(Place::Northeast).unwrap().leads_to.as_deref(),
        Some("A")
    );
    // With a six-character cut the same words do not match the long places.
    let c = compass(vec![fixed(11, &["ne", "northeast"], "A")], 6);
    assert_eq!(state(&c, Place::Northeast), PlaceState::Closed);
    assert_eq!(c.extras.len(), 1, "an exit that fits no place is an extra");
}

// I§8, data-model: a conditional exit is `maybe`: it sends its word and names no
// destination.
#[test]
fn a_conditional_exit_is_maybe() {
    let c = compass(
        vec![
            conditional(22, &["d", "down"]),
            conditional(21, &["in", "inside", "into"]),
        ],
        6,
    );
    let down = c.place(Place::Down).unwrap();
    assert_eq!(
        (
            down.state,
            down.command.as_deref(),
            down.leads_to.as_deref()
        ),
        (PlaceState::Maybe, Some("down"), None)
    );
    assert_eq!(state(&c, Place::In), PlaceState::Maybe);
    assert_eq!(c.note, None, "a maybe is an exit");
}

// R5, FR-013: an exit that matches no place is an extra, labelled with and
// sending its longest word.
#[test]
fn an_exit_with_no_place_is_an_extra() {
    let c = compass(
        vec![
            fixed(19, &["land"], "Shore"),
            conditional(18, &["x", "cross", "ford"]),
            fixed(31, &["n", "north"], "A"),
        ],
        6,
    );
    assert_eq!(c.extras.len(), 2);
    assert_eq!(
        (
            c.extras[0].label.as_str(),
            c.extras[0].command.as_str(),
            c.extras[0].state
        ),
        ("Land", "land", PlaceState::Open)
    );
    assert_eq!(c.extras[0].leads_to.as_deref(), Some("Shore"));
    assert_eq!(
        (
            c.extras[1].command.as_str(),
            c.extras[1].state,
            c.extras[1].leads_to.as_deref()
        ),
        ("cross", PlaceState::Maybe, None)
    );
    assert!(c.known);
}

// Data-model: an exit the interface reports with no words cannot be named or
// sent. It is not an extra, and the compass is not known.
#[test]
fn an_exit_with_no_words_makes_the_compass_unknown() {
    let c = compass(
        vec![fixed(17, &[], "Somewhere"), fixed(31, &["north"], "A")],
        6,
    );
    assert!(c.extras.is_empty());
    assert!(!c.known);
    assert_eq!(c.note, Some(NOTE_UNKNOWN));
    assert_eq!(
        state(&c, Place::North),
        PlaceState::Open,
        "what is known is still shown"
    );
}

// R5, E§1.4 of the introspection rules in spirit: two exits that fit one place
// are not chosen between. The place is unknown.
#[test]
fn two_exits_for_one_place_make_it_unknown() {
    let c = compass(
        vec![fixed(31, &["north"], "A"), fixed(30, &["n", "north"], "B")],
        6,
    );
    let north = c.place(Place::North).unwrap();
    assert_eq!(
        (
            north.state,
            north.command.as_deref(),
            north.leads_to.as_deref()
        ),
        (PlaceState::Unknown, None, None)
    );
    assert!(!c.known);
}

// I§8, FR-014: a direction the interface could not determine makes its place
// unknown; an unknown answer makes every place unknown.
#[test]
fn unknown_exits_light_nothing() {
    let exits = Exits {
        exits: vec![fixed(30, &["e", "east"], "A")],
        unknown: vec![UnknownDirection {
            direction: 28,
            words: words(&["s", "south"]),
            reason: Unknown::NotDeterminable,
        }],
    };
    let c = Compass::of(&Answer::exact(exits), 6);
    assert_eq!(state(&c, Place::South), PlaceState::Unknown);
    assert_eq!(state(&c, Place::East), PlaceState::Open);
    assert_eq!(state(&c, Place::West), PlaceState::Closed);
    assert!(!c.known);
    assert_eq!(c.note, Some(NOTE_UNKNOWN));

    for reason in [
        Unknown::UnrecognisedFamily,
        Unknown::NoPlayer,
        Unknown::Damaged,
    ] {
        let c = Compass::of(&Answer::unknown(reason, Exactness::Exact), 6);
        assert!(c.offered && !c.known);
        assert!(
            c.places
                .iter()
                .all(|p| p.state == PlaceState::Unknown && p.command.is_none()),
            "{reason:?}"
        );
        assert!(c.extras.is_empty());
        assert_eq!(c.note, Some(NOTE_UNKNOWN));
    }
}

// FR-014: "exits unknown" and "no exits" are different, and say so differently.
#[test]
fn no_exits_is_not_the_same_as_unknown() {
    let none = compass(vec![], 6);
    assert!(none.offered && none.known);
    assert!(none.places.iter().all(|p| p.state == PlaceState::Closed));
    assert_eq!(none.note, Some(NOTE_NONE));
    assert_ne!(NOTE_NONE, NOTE_UNKNOWN);
    assert_eq!(
        (NOTE_UNKNOWN, NOTE_NONE),
        ("Exits unknown: type a direction", "No exits here")
    );
    // An extra is an exit: no note.
    assert_eq!(compass(vec![fixed(19, &["land"], "Shore")], 6).note, None);
}

// FR-015, data-model: when the game is not waiting for a command nothing is
// offered. Every place is idle, which is not "closed": it says nothing of exits.
#[test]
fn an_idle_compass_offers_nothing_and_claims_nothing() {
    let c = Compass::idle();
    assert!(!c.offered);
    assert!(
        c.places
            .iter()
            .all(|p| p.state == PlaceState::Idle && p.command.is_none() && p.leads_to.is_none())
    );
    assert!(c.extras.is_empty());
    assert_eq!(c.note, None);
    // When the compass is offered, no place is idle.
    assert!(
        compass(vec![], 6)
            .places
            .iter()
            .all(|p| p.state != PlaceState::Idle)
    );
}

// E§ "The words the layer owns", FR-030: what a screen reader says for each state.
#[test]
fn each_state_has_its_spoken_text() {
    let mut unnamed = fixed(29, &["w", "west"], "");
    unnamed.destination_name = Some(Err(Unknown::NotDeterminable));
    let exits = Exits {
        exits: vec![
            fixed(31, &["n", "north"], "Forest Path"),
            conditional(30, &["e", "east"]),
            unnamed,
            fixed(19, &["land"], "Shore"),
        ],
        unknown: vec![UnknownDirection {
            direction: 28,
            words: words(&["s", "south"]),
            reason: Unknown::NotDeterminable,
        }],
    };
    let c = Compass::of(&Answer::exact(exits), 6);
    let spoken = |place: Place| c.place(place).unwrap().spoken.as_str();
    assert_eq!(spoken(Place::North), "North, to Forest Path");
    assert_eq!(spoken(Place::East), "East, may be blocked");
    assert_eq!(
        spoken(Place::West),
        "West",
        "open, with a destination that has no name"
    );
    assert_eq!(c.place(Place::West).unwrap().leads_to, None);
    assert_eq!(spoken(Place::South), "South, unknown");
    assert_eq!(spoken(Place::Up), "Up, no exit");
    assert_eq!(c.extras[0].spoken, "Land, to Shore");
    assert_eq!(
        Compass::idle().place(Place::Out).unwrap().spoken,
        "Out, not available now"
    );
    for (state, word) in [
        (PlaceState::Open, "open"),
        (PlaceState::Maybe, "maybe"),
        (PlaceState::Closed, "closed"),
        (PlaceState::Unknown, "unknown"),
        (PlaceState::Idle, "idle"),
    ] {
        assert_eq!(state.as_str(), word);
    }
}

// Data-model validation: no two places, and no place and extra, send the same
// line; a command is present exactly when the state is open or maybe.
#[test]
fn commands_are_distinct_and_present_only_where_offered() {
    let c = compass(
        vec![
            fixed(31, &["n", "north"], "A"),
            conditional(30, &["e", "east"]),
            fixed(23, &["u", "up"], "B"),
            fixed(20, &["out"], "C"),
            fixed(19, &["land"], "D"),
        ],
        6,
    );
    let mut commands: Vec<String> = c
        .places
        .iter()
        .filter_map(|p| p.command.clone())
        .chain(c.extras.iter().map(|e| e.command.clone()))
        .collect();
    assert_eq!(commands.len(), 5);
    commands.sort();
    commands.dedup();
    assert_eq!(commands.len(), 5);
    for p in &c.places {
        assert_eq!(
            p.command.is_some(),
            matches!(p.state, PlaceState::Open | PlaceState::Maybe),
            "{:?}",
            p.place
        );
        assert!(p.leads_to.is_none() || p.state == PlaceState::Open);
    }
}

// ---------------------------------------------------------------------------
// Doors (feature 006, User Story 5; specs/006-game-packs/research.md R7;
// docs/introspection.md I§12.3)
// ---------------------------------------------------------------------------

fn door(direction: u8, list: &[&str], to: &str, open: bool) -> Exit {
    use zmachine_introspect::{DoorState, Source};
    Exit {
        kind: ExitKind::Door,
        door: Some(40),
        state: Some(if open {
            DoorState::Open
        } else {
            DoorState::Closed
        }),
        source: Source::Pack,
        ..fixed(direction, list, to)
    }
}

// FR-030, FR-031: a closed door is a place of its own state. It is offered, a
// tap sends the direction like any other, and the room behind it is not told.
#[test]
fn a_closed_door_is_shown_as_closed_and_can_be_tried() {
    let c = compass(vec![door(29, &["e", "east"], "Kitchen", false)], 6);
    let east = c.place(Place::East).unwrap();
    assert_eq!(east.state, PlaceState::Door);
    assert_eq!(PlaceState::Door.as_str(), "door");
    assert_eq!(east.command.as_deref(), Some("east"));
    assert_eq!(east.spoken, "East, closed door");
    assert_eq!(
        east.leads_to, None,
        "the room behind a closed door is kept back"
    );
    assert!(c.known && c.offered);
    assert_eq!(c.note, None, "a closed door is a way out to try");
    // It is none of the other states.
    for other in [
        PlaceState::Open,
        PlaceState::Maybe,
        PlaceState::Closed,
        PlaceState::Unknown,
        PlaceState::Idle,
    ] {
        assert_ne!(other.as_str(), "door");
        assert_ne!(east.state, other);
    }
}

// FR-030: an open door is an open way, exactly as a fixed exit to the same room.
#[test]
fn an_open_door_is_an_open_way() {
    let through = compass(vec![door(29, &["e", "east"], "Kitchen", true)], 6);
    let plain = compass(vec![fixed(29, &["e", "east"], "Kitchen")], 6);
    assert_eq!(through, plain);
    let east = through.place(Place::East).unwrap();
    assert_eq!(east.state, PlaceState::Open);
    assert_eq!(east.spoken, "East, to Kitchen");
    assert_eq!(east.leads_to.as_deref(), Some("Kitchen"));
}

// US5 scenario 4: an exit that is conditional because its variable is untrusted
// is shown as any conditional exit is; a pack does not change "may be blocked".
#[test]
fn an_exit_on_an_untrusted_variable_may_be_blocked() {
    let mut distrusted = conditional(29, &["e", "east"]);
    distrusted.source = zmachine_introspect::Source::Pack;
    let c = compass(vec![distrusted], 6);
    assert_eq!(c, compass(vec![conditional(29, &["e", "east"])], 6));
    assert_eq!(c.place(Place::East).unwrap().spoken, "East, may be blocked");
}

// A closed door in a direction the compass has no place for is an extra, with
// the same state and no room told.
#[test]
fn a_closed_door_may_be_an_extra() {
    let c = compass(vec![door(19, &["land"], "Shore", false)], 6);
    assert_eq!(c.extras.len(), 1);
    let extra = &c.extras[0];
    assert_eq!(
        (extra.state, extra.command.as_str(), extra.spoken.as_str()),
        (PlaceState::Door, "land", "Land, closed door")
    );
    assert_eq!(extra.leads_to, None);
}

// ---------------------------------------------------------------------------
// Darkness ahead (feature 008; specs/008-darkness/research.md R10; FR-018,
// FR-019, FR-021)
// ---------------------------------------------------------------------------

fn ahead(mut exit: Exit, dark: zmachine_introspect::Tri) -> Exit {
    exit.dark_ahead = dark;
    exit.source = zmachine_introspect::Source::Pack;
    exit
}

// A way that leads into darkness is marked, and spoken as such; everything else
// of the place is what it would be without the mark.
#[test]
fn a_way_into_darkness_is_marked_and_nothing_else_changes() {
    use zmachine_introspect::Tri;
    let exits = |dark: Tri| {
        vec![
            ahead(fixed(31, &["north"], "Attic"), dark),
            ahead(fixed(9, &["land"], "Shore"), dark),
        ]
    };
    let (marked, plain) = (compass(exits(Tri::Yes), 6), compass(exits(Tri::Unknown), 6));
    let (north, was) = (
        marked.place(Place::North).unwrap(),
        plain.place(Place::North).unwrap(),
    );
    assert!(north.dark_ahead && !was.dark_ahead);
    assert_eq!(north.spoken, "North, to Attic, into darkness");
    assert_eq!(was.spoken, "North, to Attic");
    assert_eq!(
        (north.state, &north.command, &north.leads_to),
        (was.state, &was.command, &was.leads_to)
    );
    // The same for a way the compass has no place for.
    assert!(marked.extras[0].dark_ahead && !plain.extras[0].dark_ahead);
    assert_eq!(marked.extras[0].spoken, "Land, to Shore, into darkness");
    assert_eq!(
        (marked.extras[0].state, &marked.extras[0].command),
        (plain.extras[0].state, &plain.extras[0].command)
    );
    // Lit ahead is not marked, and is the place it was.
    let lit = compass(exits(Tri::No), 6);
    assert_eq!(lit.places, plain.places);
    assert_eq!(lit.extras, plain.extras);
    // No other place is marked, and the note is untouched.
    assert_eq!(marked.places.iter().filter(|p| p.dark_ahead).count(), 1);
    assert_eq!(marked.note, plain.note);
}

// A closed door is not seen through: what is behind it is not told, dark or
// not. A way that may be blocked has no room to speak of.
#[test]
fn a_closed_door_and_a_way_that_may_be_blocked_are_not_marked() {
    use zmachine_introspect::{DoorState, Tri};
    let mut door = ahead(fixed(31, &["north"], "Cellar"), Tri::Yes);
    door.kind = ExitKind::Door;
    door.door = Some(40);
    door.state = Some(DoorState::Closed);
    let c = compass(
        vec![door.clone(), ahead(conditional(30, &["south"]), Tri::Yes)],
        6,
    );
    let north = c.place(Place::North).unwrap();
    assert_eq!((north.state, north.dark_ahead), (PlaceState::Door, false));
    assert_eq!(north.spoken, "North, closed door");
    let south = c.place(Place::South).unwrap();
    assert_eq!((south.state, south.dark_ahead), (PlaceState::Maybe, false));
    // Opened, it is a way like any other, and is marked.
    door.state = Some(DoorState::Open);
    let c = compass(vec![door], 6);
    assert!(c.place(Place::North).unwrap().dark_ahead);
}

// Two exits for one place leave it unknown, and unmarked.
#[test]
fn a_place_that_is_not_known_is_not_marked() {
    use zmachine_introspect::Tri;
    let c = compass(
        vec![
            ahead(fixed(31, &["north"], "Attic"), Tri::Yes),
            ahead(fixed(30, &["north"], "Loft"), Tri::Yes),
        ],
        6,
    );
    let north = c.place(Place::North).unwrap();
    assert_eq!(
        (north.state, north.dark_ahead),
        (PlaceState::Unknown, false)
    );
}

// ---------------------------------------------------------------------------
// Feature 009: a compass settled by look-ahead
// (specs/009-compass-lookahead/contracts/lookahead.md, cited L§).
// ---------------------------------------------------------------------------

use crate::lookahead::Finding;

fn walked(name: &str, dark: bool) -> Finding {
    Finding::Walked {
        room: 7,
        name: Some(name.to_string()),
        dark,
    }
}

/// North and up are open, south and west conditional, and so are two ways the
/// compass has no place for.
fn unsettled() -> Compass {
    compass(
        vec![
            fixed(31, &["north"], "Attic"),
            conditional(30, &["south"]),
            conditional(28, &["west"]),
            conditional(9, &["land"]),
            conditional(8, &["cross"]),
            fixed(23, &["up"], "Loft"),
        ],
        6,
    )
}

// L§10: only a way that is `maybe` is tried, each once, the places in the
// compass's order and then the extras in theirs.
#[test]
fn settled_asks_only_for_ways_that_may_be_blocked() {
    let mut c = unsettled();
    let mut asked = Vec::new();
    c.settle(|command| {
        asked.push(command.to_string());
        Finding::None
    });
    assert_eq!(asked, ["south", "west", "land", "cross"]);
}

// L§13, 16: with no finding the compass is what it was.
#[test]
fn settled_by_no_finding_is_unchanged() {
    let mut c = unsettled();
    c.settle(|_| Finding::None);
    assert_eq!(c, unsettled());
}

// L§11, 15: a way found walked is an open way to the room found, shown and
// spoken as any open way is.
#[test]
fn settled_a_way_found_walked_is_open() {
    let mut c = unsettled();
    c.settle(|command| match command {
        "south" => walked("Cellar", false),
        "land" => walked("Shore", false),
        _ => Finding::None,
    });
    let south = c.place(Place::South).unwrap();
    assert_eq!(south.state, PlaceState::Open);
    assert_eq!(south.command.as_deref(), Some("south"));
    assert_eq!(south.leads_to.as_deref(), Some("Cellar"));
    assert!(!south.dark_ahead);
    assert_eq!(south.spoken, "South, to Cellar");
    assert_eq!(state(&c, Place::West), PlaceState::Maybe);
    let land = &c.extras[0];
    assert_eq!(
        (land.label.as_str(), land.state),
        ("Land", PlaceState::Open)
    );
    assert_eq!(land.leads_to.as_deref(), Some("Shore"));
    assert_eq!(land.spoken, "Land, to Shore");
    assert_eq!(c.extras[1].state, PlaceState::Maybe);
}

// L§12: a place found refused leads nowhere and sends nothing; an extra found
// refused is not listed.
#[test]
fn settled_a_way_found_refused_is_no_way() {
    let mut c = unsettled();
    c.settle(|command| match command {
        "south" | "land" => Finding::Refused,
        _ => Finding::None,
    });
    let south = c.place(Place::South).unwrap();
    assert_eq!(south.state, PlaceState::Closed);
    assert_eq!(south.command, None);
    assert_eq!(south.spoken, "South, no exit");
    // It is told apart from no direction with no way at all by nothing.
    assert_eq!(
        (
            south.state,
            &south.command,
            south.dark_ahead,
            &south.leads_to
        ),
        {
            let east = c.place(Place::East).unwrap();
            (east.state, &east.command, east.dark_ahead, &east.leads_to)
        }
    );
    assert_eq!(c.extras.len(), 1);
    assert_eq!(c.extras[0].label, "Cross");
}

// L§11; FR-012: a way found walked into a dark room is marked and spoken as
// feature 008 marks a way into darkness; a way found refused never is.
#[test]
fn dark_ahead_is_what_the_copy_found() {
    let mut c = unsettled();
    c.settle(|command| match command {
        "south" => walked("Cellar", true),
        "land" => walked("Cave", true),
        _ => Finding::Refused,
    });
    let south = c.place(Place::South).unwrap();
    assert!(south.dark_ahead);
    assert_eq!(south.spoken, "South, to Cellar, into darkness");
    assert!(c.extras[0].dark_ahead);
    assert_eq!(c.extras[0].spoken, "Land, to Cave, into darkness");
    assert!(!c.place(Place::West).unwrap().dark_ahead);
}

// L§10: a way that is open, a closed door, a place two exits claim and a
// direction the interface could not answer are not tried.
#[test]
fn settled_leaves_what_is_already_settled() {
    let mut c = Compass::of(
        &Answer::exact(Exits {
            exits: vec![
                fixed(31, &["north"], "Attic"),
                conditional(30, &["south"]),
                conditional(29, &["south"]),
            ],
            unknown: vec![UnknownDirection {
                direction: 28,
                words: words(&["west"]),
                reason: Unknown::Damaged,
            }],
        }),
        6,
    );
    let before = c.clone();
    let mut asked = 0;
    c.settle(|_| {
        asked += 1;
        Finding::Refused
    });
    assert_eq!(asked, 0);
    assert_eq!(c, before);
    assert_eq!(c.note, Some(NOTE_UNKNOWN));
}

// L§14 as built: where every way was known and look-ahead leaves none, the
// note says there are no exits.
#[test]
fn settled_to_nothing_says_there_are_no_exits() {
    let mut c = compass(vec![conditional(30, &["south"])], 6);
    assert_eq!(c.note, None);
    c.settle(|_| Finding::Refused);
    assert_eq!(c.note, Some(NOTE_NONE));
}

// L§10: a compass that offers nothing is not tried.
#[test]
fn settled_never_when_nothing_is_offered() {
    let mut c = Compass::idle();
    c.settle(|_| panic!("nothing is tried"));
    assert_eq!(c, Compass::idle());
}
