//! The map's record, fed by hand
//! (specs/012-map-first-slice/contracts/map.md, parts W, P and S, cited so).

use crate::compass::{self, Compass, Extra};
use crate::map::{Lamp, LineKind, Map, Observation};
use crate::{Place, PlaceState, Waiting};
use Place::*;
use PlaceState::{Door, Maybe, Open};

type WayOf<'a> = (Place, PlaceState, Option<&'a str>);

/// A compass with these ways and every other place closed.
fn compass_of(ways: &[WayOf], extras: &[&str]) -> Compass {
    Compass {
        offered: true,
        known: true,
        note: None,
        places: Place::ALL
            .into_iter()
            .map(|place| match ways.iter().find(|w| w.0 == place) {
                Some((_, state, to)) => {
                    compass::place_in(place, *state, to.map(str::to_string), false)
                }
                None => compass::place_in(place, PlaceState::Closed, None, false),
            })
            .collect(),
        extras: extras
            .iter()
            .map(|word| Extra {
                label: word.to_string(),
                state: Open,
                command: word.to_string(),
                leads_to: None,
                dark_ahead: false,
                spoken: word.to_string(),
            })
            .collect(),
    }
}

/// What to feed at one stop; lit and waiting for a command unless changed.
struct At<'a> {
    sent: Option<&'a str>,
    room: Option<(u16, &'a str)>,
    dark: Option<bool>,
    ways: &'a [WayOf<'a>],
    extras: &'a [&'a str],
    jumped: bool,
    waiting: Waiting,
}

impl<'a> At<'a> {
    fn room(sent: Option<&'a str>, id: u16, name: &'a str, ways: &'a [WayOf<'a>]) -> At<'a> {
        At {
            sent,
            room: Some((id, name)),
            dark: Some(false),
            ways,
            extras: &[],
            jumped: false,
            waiting: Waiting::Command,
        }
    }

    fn feed(&self, map: &mut Map) {
        let compass = compass_of(self.ways, self.extras);
        map.observe(Observation {
            sent: self.sent,
            waiting: self.waiting,
            room: self.room.map(|(id, name)| (id, Some(name.to_string()))),
            dark: self.dark,
            compass: &compass,
            jumped: self.jumped,
        });
    }
}

/// Feeds a lit command stop.
fn at(map: &mut Map, sent: Option<&str>, id: u16, name: &str, ways: &[WayOf]) {
    At::room(sent, id, name, ways).feed(map);
}

fn page(map: &Map) -> Vec<String> {
    map.page()
}

const HALL: &[WayOf] = &[(North, Open, Some("Study")), (East, Open, Some("Den"))];
const STUDY: &[WayOf] = &[(South, Open, Some("Hall"))];

// W§1, S§1 to S§3: one place at the start.
#[test]
fn the_map_starts_with_the_place_the_player_is_in() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    let view = map.view().expect("a map");
    assert_eq!((view.sheet, view.sheets, view.here), (1, 1, Some(10)));
    assert_eq!(view.lamp, Lamp::Lit);
    assert_eq!(view.places.len(), 1);
    assert_eq!(
        (
            view.places[0].id,
            view.places[0].x,
            view.places[0].y,
            view.places[0].name.as_deref()
        ),
        (10, 0, 0, Some("Hall"))
    );
    assert!(view.lines.is_empty() && view.leads.is_empty() && view.stubs.is_empty());
    assert_eq!(view.spoken, "Map. You are in Hall.");
    assert_eq!(view.note, None);
}

// W§5, P§2: a direction to a new room is a place one step that way and a line.
#[test]
fn a_walk_draws_a_place_and_a_line() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    let view = map.view().unwrap();
    assert_eq!(view.here, Some(11));
    assert_eq!(map.position(11), Some((1, 0, -1)));
    assert_eq!(view.lines.len(), 1);
    let line = &view.lines[0];
    assert_eq!(
        (line.a, line.b, line.out.as_str(), line.back.as_deref()),
        (10, 11, "north", None)
    );
    assert_eq!(line.kind, LineKind::Straight);
}

// W§5, W§11, W§17: walking it again, or back, or doing something else, adds
// no place; an answer stop is passed over; abbreviations are directions.
#[test]
fn nothing_is_drawn_but_what_was_walked() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("N"), 11, "Study", STUDY);
    at(&mut map, Some("take lamp"), 11, "Study", STUDY);
    at(&mut map, Some("s"), 10, "Hall", HALL);
    at(&mut map, Some("go north"), 11, "Study", STUDY);
    // A question between two command stops: the move is judged across it,
    // by the command and not by the answer.
    let mut asking = At::room(Some("south"), 11, "Study", STUDY);
    asking.waiting = Waiting::Answer;
    asking.feed(&mut map);
    at(&mut map, Some("yes"), 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(view.places.len(), 2);
    assert_eq!(view.lines.len(), 1);
    assert_eq!(view.lines[0].back.as_deref(), Some("south"));
    assert_eq!(map.position(10), Some((1, 0, 0)));
    assert_eq!(map.position(11), Some((1, 0, -1)));
    // The Den was never walked to: it is nowhere on the map (W§11).
    assert!(!page(&map).iter().any(|line| line.contains("Den")));
}

// S§6, S§7, W§3: ways seen and not walked are stubs; of the place the player
// is in they are in `ways`, of the others in `stubs`.
#[test]
fn ways_not_walked_are_stubs() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    let view = map.view().unwrap();
    let ways: Vec<(&str, &str, Option<&str>)> = view
        .ways
        .iter()
        .map(|w| (w.way.as_str(), w.state.as_str(), w.command.as_deref()))
        .collect();
    assert_eq!(
        ways,
        [
            ("north", "open", Some("north")),
            ("east", "open", Some("east"))
        ]
    );
    at(&mut map, Some("north"), 11, "Study", STUDY);
    let view = map.view().unwrap();
    let stubs: Vec<(u16, &str)> = view.stubs.iter().map(|s| (s.of, s.way.as_str())).collect();
    assert_eq!(stubs, [(10, "east")], "north was walked; east was not");
    assert_eq!(view.ways.len(), 1);
}

// W§7, FR-004: no name of where a way leads until it has been walked.
#[test]
fn a_way_names_where_it_leads_only_once_walked() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(view.ways[0].leads_to, None);
    assert_eq!(view.ways[0].spoken, "North");
    at(&mut map, Some("north"), 11, "Study", STUDY);
    // Arrived from the south, but south has not been walked from here.
    let view = map.view().unwrap();
    assert_eq!(
        (view.ways[0].way.as_str(), view.ways[0].leads_to.as_deref()),
        ("south", None)
    );
    assert_eq!(view.ways[0].spoken, "South");
    at(&mut map, Some("south"), 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(view.ways[0].leads_to.as_deref(), Some("Study"));
    assert_eq!(view.ways[0].spoken, "North, to Study");
    assert_eq!(view.ways[1].spoken, "East");
    at(&mut map, Some("north"), 11, "Study", STUDY);
    assert_eq!(map.view().unwrap().ways[0].spoken, "South, to Hall");
}

// W§4: a way's state is as last seen; a door and an uncertain way keep the
// compass's words.
#[test]
fn a_ways_state_is_as_last_seen() {
    let mut map = Map::new(true);
    at(
        &mut map,
        None,
        10,
        "Hall",
        &[(West, Door, None), (Up, Maybe, None)],
    );
    let view = map.view().unwrap();
    assert_eq!(view.ways[0].spoken, "West, closed door");
    assert_eq!(view.ways[1].spoken, "Up, may be blocked");
    at(
        &mut map,
        Some("open door"),
        10,
        "Hall",
        &[(West, Open, Some("Den"))],
    );
    let view = map.view().unwrap();
    assert_eq!(view.ways.len(), 1);
    assert_eq!(
        (view.ways[0].state.as_str(), view.ways[0].spoken.as_str()),
        ("open", "West")
    );
}

// S§7: with the game not waiting for a command, nothing can be sent.
#[test]
fn nothing_can_be_sent_while_the_game_asks_a_question() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    let mut asking = At::room(Some("quit"), 10, "Hall", &[]);
    asking.waiting = Waiting::Answer;
    asking.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.ways.len(), 2, "the ways last seen are still drawn");
    assert!(view.ways.iter().all(|way| way.command.is_none()));
}

// W§15: places of one name are numbered in the order reached, once there are
// two of them.
#[test]
fn places_that_share_a_name_are_numbered() {
    let mut map = Map::new(true);
    let maze: &[WayOf] = &[(East, Open, Some("Maze")), (West, Open, Some("Maze"))];
    at(&mut map, None, 20, "Maze", maze);
    assert_eq!(map.view().unwrap().places[0].name.as_deref(), Some("Maze"));
    at(&mut map, Some("east"), 21, "Maze", maze);
    at(&mut map, Some("east"), 22, "Maze", maze);
    let view = map.view().unwrap();
    let names: Vec<(u16, &str)> = view
        .places
        .iter()
        .map(|p| (p.id, p.name.as_deref().unwrap()))
        .collect();
    assert_eq!(names, [(20, "Maze 1"), (21, "Maze 2"), (22, "Maze 3")]);
    assert_eq!(view.spoken, "Map. You are in Maze 3.");
}

// W§16: a way that leads back to its own place draws no line and is marked.
#[test]
fn a_way_that_returns_is_marked() {
    let mut map = Map::new(true);
    let forest: &[WayOf] = &[(North, Open, Some("Forest")), (South, Open, Some("Path"))];
    at(&mut map, None, 30, "Forest", forest);
    at(&mut map, Some("north"), 30, "Forest", forest);
    let view = map.view().unwrap();
    assert!(view.lines.is_empty());
    assert!(view.ways[0].returns && !view.ways[1].returns);
}

// W§6, P§6: a move by a word is a line marked with the command, and the new
// place starts a sheet.
#[test]
fn a_move_by_a_word_starts_a_sheet() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("Say XYZZY"), 40, "Cave", &[]);
    assert_eq!(map.position(40), Some((2, 0, 0)));
    let view = map.view().unwrap();
    assert_eq!((view.sheet, view.sheets), (2, 2));
    assert!(view.lines.is_empty(), "the line leaves the sheet");
    assert!(
        view.leads.is_empty(),
        "it has not been walked back from here"
    );
    // Back on the first sheet the lead is marked with the command as sent.
    at(&mut map, Some("xyzzy"), 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(view.sheet, 1);
    let lead = &view.leads[0];
    assert_eq!(
        (
            lead.from,
            lead.way.as_str(),
            lead.label.as_str(),
            lead.to.as_deref(),
            lead.sheet
        ),
        (10, "word", "Say XYZZY", Some("Cave"), 2)
    );
}

// P§6: a run of moves by a word, each from the place the last reached, is one
// sheet going down the page.
#[test]
fn a_run_of_moves_by_a_word_is_one_sheet() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("launch"), 50, "River", &[]);
    at(&mut map, Some("wait"), 51, "River", &[]);
    at(&mut map, Some("wait"), 52, "River", &[]);
    assert_eq!(map.position(50), Some((2, 0, 0)));
    assert_eq!(map.position(51), Some((2, 0, 1)));
    assert_eq!(map.position(52), Some((2, 0, 2)));
    let view = map.view().unwrap();
    assert_eq!(view.lines.len(), 2);
    assert!(view.lines.iter().all(|l| l.kind == LineKind::Word));
    assert_eq!(view.lines[0].label.as_deref(), Some("wait"));
    // A word from a place that was walked to starts a sheet of its own.
    at(&mut map, Some("east"), 53, "Shore", &[]);
    at(&mut map, Some("pray"), 54, "Temple", &[]);
    assert_eq!(map.position(53), Some((2, 1, 2)));
    assert_eq!(map.position(54), Some((3, 0, 0)));
}

// P§5: in and out stay on the sheet, beside the place left.
#[test]
fn in_and_out_stay_on_the_sheet() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", &[(In, Open, None)]);
    at(&mut map, Some("in"), 12, "Closet", &[(Out, Open, None)]);
    assert_eq!(map.position(12), Some((1, -1, -1)));
    let view = map.view().unwrap();
    assert_eq!(view.lines[0].kind, LineKind::Word);
    assert_eq!(view.lines[0].out, "in");
}

// An extra way of the compass is a way; walking it is a move by its word.
#[test]
fn an_extra_way_is_offered_and_walked_by_its_word() {
    let mut map = Map::new(true);
    let mut start = At::room(None, 10, "Hall", HALL);
    start.extras = &["land"];
    start.feed(&mut map);
    let view = map.view().unwrap();
    let last = view.ways.last().unwrap();
    assert_eq!(
        (last.way.as_str(), last.command.as_deref()),
        ("land", Some("land"))
    );
    at(&mut map, Some("land"), 13, "Field", &[]);
    assert_eq!(map.position(13), Some((2, 0, 0)));
}

// P§3: up and down go beside the place left, off its compass ways, attached.
#[test]
fn up_and_down_are_attached() {
    let mut map = Map::new(true);
    let kitchen: &[WayOf] = &[
        (Up, Open, None),
        (Down, Open, None),
        (Northeast, Open, None),
    ];
    at(&mut map, None, 10, "Kitchen", kitchen);
    at(&mut map, Some("up"), 11, "Attic", &[(Down, Open, None)]);
    // The corner above-right has the kitchen's northeast way: above-left.
    assert_eq!(map.position(11), Some((1, -1, -1)));
    assert!(map.is_inset(11) && !map.is_inset(10));
    at(&mut map, Some("d"), 10, "Kitchen", kitchen);
    at(&mut map, Some("down"), 12, "Cellar", &[(Up, Open, None)]);
    assert_eq!(map.position(12), Some((1, -1, 0)));
    assert!(map.is_inset(12));
    let view = map.view().unwrap();
    assert!(view.places[1].inset && view.places[2].inset && !view.places[0].inset);
    let attached: Vec<(u16, Option<u16>)> =
        view.places.iter().map(|p| (p.id, p.attached_to)).collect();
    assert_eq!(attached, [(10, None), (11, Some(10)), (12, Some(10))]);
    assert!(view.lines.iter().all(|l| l.kind == LineKind::Ladder));
    assert_eq!(view.lines[0].back.as_deref(), Some("down"));
}

// P§4: a group stays until it has four places, then moves once to a sheet of
// its own, keeping its shape; nothing else moves.
#[test]
fn a_group_of_four_moves_to_its_own_sheet() {
    let mut map = Map::new(true);
    let all: &[WayOf] = &[(Down, Open, None), (East, Open, None), (West, Open, None)];
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    at(&mut map, Some("down"), 60, "Cellar", all);
    at(&mut map, Some("east"), 61, "Vault", all);
    at(&mut map, Some("east"), 62, "Crypt", all);
    // Three: still on the first sheet, at the Study's lower-left corner and
    // on eastward from there, taking no position from the Hall or the Study.
    assert_eq!(map.position(60), Some((1, -1, -1)));
    assert_eq!(map.position(61), Some((1, 0, -1)));
    assert_eq!(map.position(62), Some((1, 1, -1)));
    assert!(map.is_inset(60) && map.is_inset(61) && map.is_inset(62));
    assert_eq!(map.sheet_count(), 1);
    let before = (map.position(10), map.position(11));
    at(&mut map, Some("east"), 63, "Tomb", all);
    // Four: a sheet of their own, the first attached at (0, 0).
    assert_eq!(map.sheet_count(), 2);
    assert_eq!(map.position(60), Some((2, 0, 0)));
    let offset = |id: u16| {
        let (sheet, x, y) = map.position(id).unwrap();
        assert_eq!(sheet, 2);
        (x, y)
    };
    assert_eq!(offset(61).1, 0);
    assert!(offset(61).0 > 0 && offset(62).0 > offset(61).0 && offset(63).0 > offset(62).0);
    assert_eq!(
        (map.position(10), map.position(11)),
        before,
        "nothing else moved"
    );
    let view = map.view().unwrap();
    assert_eq!(view.sheet, 2);
    assert!(
        view.places
            .iter()
            .all(|p| p.attached_to.is_none() && !p.inset)
    );
    assert!(
        view.lines.iter().all(|l| l.kind == LineKind::Straight),
        "{:?}",
        view.lines
    );
    // It does not move again as it grows.
    at(&mut map, Some("east"), 64, "Pit", all);
    at(&mut map, Some("east"), 65, "Well", all);
    assert_eq!(map.sheet_count(), 2);
    assert_eq!(map.position(60), Some((2, 0, 0)));
    // Each end of the way between the sheets leads to the other.
    for line in ["west", "west", "west", "west", "west"] {
        let here = map.view().unwrap().here.unwrap();
        at(&mut map, Some(line), here - 1, "x", all);
    }
    let view = map.view().unwrap();
    assert_eq!(view.here, Some(60));
    at(&mut map, Some("up"), 11, "Study", STUDY);
    let view = map.view().unwrap();
    let lead = &view.leads[0];
    assert_eq!(
        (
            lead.from,
            lead.way.as_str(),
            lead.label.as_str(),
            lead.sheet
        ),
        (11, "down", "Down", 2)
    );
    at(&mut map, Some("down"), 60, "Cellar", all);
    let lead = &map.view().unwrap().leads[0];
    assert_eq!(
        (lead.from, lead.way.as_str(), lead.to.as_deref(), lead.sheet),
        (60, "up", Some("Study"), 1)
    );
}

// W§2, W§12: a place seen only in the dark has no name and no stubs; seen
// lit, it has both, where it was.
#[test]
fn a_place_seen_only_in_the_dark_has_no_name() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Kitchen", &[(Up, Open, None)]);
    let mut attic = At::room(Some("up"), 11, "Attic", &[(Down, Open, None)]);
    attic.dark = Some(true);
    attic.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.lamp, Lamp::Out);
    assert_eq!(view.places[1].name, None);
    assert!(view.ways.is_empty());
    assert_eq!(view.spoken, "Map. You are somewhere dark.");
    let position = map.position(11);
    at(&mut map, Some("down"), 10, "Kitchen", &[(Up, Open, None)]);
    assert!(
        map.view().unwrap().stubs.is_empty(),
        "the attic's ways were not seen"
    );
    at(
        &mut map,
        Some("up"),
        11,
        "Attic",
        &[(Down, Open, None), (East, Open, None)],
    );
    let view = map.view().unwrap();
    assert_eq!(view.places[1].name.as_deref(), Some("Attic"));
    assert_eq!(view.ways.len(), 2);
    assert_eq!(map.position(11), position);
    // Dark again: what was seen lit is kept.
    attic.sent = Some("turn off lamp");
    attic.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.places[1].name.as_deref(), Some("Attic"));
    assert_eq!(view.ways.len(), 2);
}

// W§12: light reported as unknown counts as not dark.
#[test]
fn where_light_is_not_known_the_place_is_shown() {
    let mut map = Map::new(true);
    let mut start = At::room(None, 10, "Hall", HALL);
    start.dark = None;
    start.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.lamp, Lamp::Unknown);
    assert_eq!(view.places[0].name.as_deref(), Some("Hall"));
    assert_eq!(view.ways.len(), 2);
}

// W§8: a direction not offered as open, tried, that leaves the player where
// they were, is drawn as refused.
#[test]
fn a_way_tried_and_refused_is_drawn_as_refused() {
    let mut map = Map::new(true);
    let hall: &[WayOf] = &[(North, Open, Some("Study")), (Up, Maybe, None)];
    at(&mut map, None, 10, "Hall", hall);
    at(&mut map, Some("west"), 10, "Hall", hall);
    at(&mut map, Some("up"), 10, "Hall", hall);
    let view = map.view().unwrap();
    let west = view
        .ways
        .iter()
        .find(|w| w.way == "west")
        .expect("west is drawn");
    assert!(west.refused && west.command.is_none());
    assert_eq!(west.spoken, "West, no exit");
    // An uncertain way that refused is still offered, and marked.
    let up = view.ways.iter().find(|w| w.way == "up").unwrap();
    assert!(up.refused && up.command.is_some());
    // An open way that did not move the player is not a refusal.
    at(&mut map, Some("north"), 10, "Hall", hall);
    assert!(
        !map.view()
            .unwrap()
            .ways
            .iter()
            .any(|w| w.way == "north" && w.refused)
    );
    // Something that is not a direction is not a refusal either.
    at(&mut map, Some("jump"), 10, "Hall", hall);
    assert_eq!(map.view().unwrap().ways.len(), 3);
}

// W§9: a way that named one place and led to another is a carried move: a
// line by the command, and the place reached is not put in that direction.
#[test]
fn a_way_that_did_not_go_where_it_said_is_carried() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 70, "Forest", &[]);
    assert_eq!(map.position(70), Some((2, 0, 0)), "not north of the Hall");
    at(&mut map, Some("x"), 70, "Forest", &[]);
    let view = map.view().unwrap();
    assert!(view.lines.is_empty());
    // The north way of the Hall has not been walked: it is still a stub.
    at(&mut map, Some("teleport"), 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(view.ways[0].leads_to, None);
    assert_eq!(view.leads[0].label, "north");
    assert_eq!(view.leads[0].way, "word");
    // A way that gave no destination has nothing to contradict.
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", &[(North, Maybe, None)]);
    at(&mut map, Some("north"), 70, "Forest", &[]);
    assert_eq!(map.position(70), Some((1, 0, -1)));
}

// W§10: one way that has led to two places keeps both lines and is marked.
#[test]
fn a_way_that_varies_is_marked() {
    let mut map = Map::new(true);
    let hall: &[WayOf] = &[(East, Maybe, None)];
    at(&mut map, None, 10, "Hall", hall);
    at(&mut map, Some("east"), 11, "Den", &[(West, Open, None)]);
    at(&mut map, Some("west"), 10, "Hall", hall);
    at(&mut map, Some("east"), 12, "Bog", &[(West, Open, None)]);
    at(&mut map, Some("west"), 10, "Hall", hall);
    let view = map.view().unwrap();
    assert_eq!(view.lines.len(), 2);
    let east = &view.ways[0];
    assert!(east.varies);
    assert_eq!(east.leads_to, None);
    assert_eq!(map.position(11), Some((1, 1, 0)));
    assert_eq!(map.position(12), Some((1, 2, 0)));
}

// W§13: where the player's place is not known nothing is added, and the next
// arrival is joined to nothing.
#[test]
fn where_the_place_is_not_known_nothing_is_added() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    let mut lost = At::room(Some("north"), 0, "", &[]);
    lost.room = None;
    lost.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.here, None);
    assert_eq!(view.places.len(), 1, "the sheet last shown is still shown");
    assert_eq!(view.note, Some("Where you are is not known."));
    assert_eq!(view.spoken, "Map. Where you are is not known.");
    assert!(view.ways.is_empty());
    at(&mut map, Some("north"), 11, "Study", STUDY);
    let view = map.view().unwrap();
    assert_eq!(view.here, Some(11));
    assert!(view.lines.is_empty() && view.leads.is_empty());
    assert_eq!(map.position(11), Some((2, 0, 0)));
}

// W§14: when the game jumped, no line is drawn.
#[test]
fn a_jump_draws_no_line() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    let mut undone = At::room(Some("undo"), 10, "Hall", HALL);
    undone.jumped = true;
    undone.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.here, Some(10));
    assert_eq!(view.lines.len(), 1, "only the walk north");
    assert_eq!(view.lines[0].back, None);
    // A jump to a place not on the map puts it on a new sheet, unjoined.
    let mut restored = At::room(Some("north"), 80, "Shrine", &[]);
    restored.jumped = true;
    restored.feed(&mut map);
    assert_eq!(map.position(80), Some((2, 0, 0)));
    assert!(map.view().unwrap().leads.is_empty());
    // A jump seen at an answer stop still counts at the next command stop.
    let mut asking = At::room(Some("restart"), 80, "Shrine", &[]);
    asking.waiting = Waiting::Answer;
    asking.jumped = true;
    asking.feed(&mut map);
    at(&mut map, Some("y"), 10, "Hall", HALL);
    assert_eq!(
        map.page().iter().filter(|l| l.starts_with("line ")).count(),
        1
    );
}

// The compass's own note where the ways are not known (S§8).
#[test]
fn where_the_ways_are_not_known_the_map_says_so() {
    let mut map = Map::new(true);
    let mut compass = compass_of(&[], &[]);
    compass.known = false;
    compass.note = Some(crate::NOTE_UNKNOWN);
    map.observe(Observation {
        sent: None,
        waiting: Waiting::Command,
        room: Some((10, Some("Hall".to_string()))),
        dark: Some(false),
        compass: &compass,
        jumped: false,
    });
    assert_eq!(map.view().unwrap().note, Some(crate::NOTE_UNKNOWN));
}

// FR-038: a game the layer does not recognise has no map.
#[test]
fn a_game_that_is_not_recognised_has_no_map() {
    let mut map = Map::new(false);
    at(&mut map, None, 10, "Hall", HALL);
    assert!(map.view().is_none());
    assert!(map.page().is_empty());
}

// P§8, S§4: two places are joined once, however many ways were walked
// between them, and the join is straight if any of those ways lies along the
// page; it is bent only when none does.
#[test]
fn two_places_are_joined_once_and_bent_only_if_no_way_lies_along() {
    let mut map = Map::new(true);
    let any: &[WayOf] = &[
        (North, Maybe, None),
        (Northeast, Maybe, None),
        (East, Maybe, None),
        (West, Maybe, None),
        (Southwest, Maybe, None),
    ];
    at(&mut map, None, 1, "A", any);
    at(&mut map, Some("east"), 2, "B", any);
    // North from B arrives back at A, which is not north of B; and northeast
    // from A leads to B too. Still one join, drawn by the way that is true.
    at(&mut map, Some("north"), 1, "A", any);
    at(&mut map, Some("northeast"), 2, "B", any);
    at(&mut map, Some("southwest"), 1, "A", any);
    let view = map.view().unwrap();
    assert_eq!(view.lines.len(), 1, "{:?}", view.lines);
    assert_eq!(view.lines[0].kind, LineKind::Straight);
    assert_eq!((view.lines[0].a, view.lines[0].out.as_str()), (1, "east"));
    // Every way walked is still known where it leads.
    let named: Vec<(&str, Option<&str>)> = view
        .ways
        .iter()
        .map(|w| (w.way.as_str(), w.leads_to.as_deref()))
        .collect();
    assert_eq!(
        named,
        [
            ("north", None),
            ("northeast", Some("B")),
            ("east", Some("B")),
            ("southwest", None),
            ("west", None)
        ]
    );
    assert_eq!(
        map.page().iter().filter(|l| l.starts_with("line")).count(),
        2
    );
    // A third place north of A whose east way leads to B: B is not east of it.
    at(&mut map, Some("north"), 3, "C", any);
    at(&mut map, Some("east"), 2, "B", any);
    let view = map.view().unwrap();
    assert_eq!(view.lines.len(), 3);
    let last = view.lines.last().unwrap();
    assert_eq!((last.a, last.b, last.kind), (3, 2, LineKind::Bent));
    assert_eq!(map.position(2), Some((1, 1, 0)));
    assert_eq!(map.position(3), Some((1, 0, -1)));
}

// P§9: the same turns give the same page.
#[test]
fn the_same_turns_give_the_same_page() {
    let build = || {
        let mut map = Map::new(true);
        at(&mut map, None, 10, "Hall", HALL);
        at(&mut map, Some("north"), 11, "Study", STUDY);
        at(&mut map, Some("xyzzy"), 40, "Cave", &[(Up, Open, None)]);
        at(&mut map, Some("up"), 41, "Ledge", &[]);
        map
    };
    assert_eq!(build().page(), build().page());
    assert_eq!(build().view(), build().view());
    assert!(build().page().len() >= 6);
}
