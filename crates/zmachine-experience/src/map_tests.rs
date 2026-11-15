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

// W§12a: in a place seen only in the dark, a way is offered when the layer
// says it leads to a place the player walked here from. Nothing else of the
// place is told, and where the way leads is not named until walked (W§7).
#[test]
fn in_the_dark_the_way_back_is_offered() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Kitchen", &[(Up, Open, Some("Attic"))]);
    let mut attic = At::room(
        Some("up"),
        11,
        "Attic",
        &[(Down, Open, Some("Kitchen")), (East, Open, Some("Loft"))],
    );
    attic.dark = Some(true);
    attic.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.places[1].name, None);
    assert_eq!(view.ways.len(), 1, "only the way back: {:?}", view.ways);
    let back = &view.ways[0];
    assert_eq!(
        (back.way.as_str(), back.command.as_deref(), &back.leads_to),
        ("down", Some("down"), &None)
    );
    assert_eq!(back.spoken, "Down");
    // Walked, it is the way back along the line already drawn.
    at(
        &mut map,
        Some("down"),
        10,
        "Kitchen",
        &[(Up, Open, Some("Attic"))],
    );
    let view = map.view().unwrap();
    assert_eq!(view.lines.len(), 1);
    assert_eq!(view.lines[0].back.as_deref(), Some("down"));
    assert!(view.stubs.is_empty(), "the attic's ways were not seen");
}

// W§12a: a way that does not lead back where the player came from is not
// offered in the dark, whatever the layer knows of it.
#[test]
fn in_the_dark_no_other_way_is_offered() {
    let mut map = Map::new(true);
    at(
        &mut map,
        None,
        10,
        "Kitchen",
        &[(Down, Open, Some("Cellar"))],
    );
    let mut cellar = At::room(
        Some("down"),
        11,
        "Cellar",
        &[(North, Open, Some("Passage")), (Up, Door, None)],
    );
    cellar.dark = Some(true);
    cellar.feed(&mut map);
    assert!(map.view().unwrap().ways.is_empty());
}

// S§8: a place whose ways are known, and that has none, says so; not in the
// dark, where nothing of the place is told.
#[test]
fn a_place_with_no_ways_says_so() {
    let feed = |map: &mut Map, dark: bool| {
        let mut compass = compass_of(&[], &[]);
        compass.note = Some(crate::NOTE_NONE);
        map.observe(Observation {
            sent: None,
            waiting: Waiting::Command,
            room: Some((10, Some("Pit".to_string()))),
            dark: Some(dark),
            compass: &compass,
            jumped: false,
        });
    };
    let mut map = Map::new(true);
    feed(&mut map, false);
    assert_eq!(map.view().unwrap().note, Some(crate::NOTE_NONE));
    let mut map = Map::new(true);
    feed(&mut map, true);
    assert_eq!(map.view().unwrap().note, None);
}

// specs/013-room-look/contracts/look.md, L§2, L§6: the place the player is in
// can be touched to look when the game waits for a command, the place is
// known and the story has the word; in the dark and with no name as anywhere.
#[test]
fn the_players_place_can_be_touched_to_look() {
    let mut map = Map::new(true).with_look(Some("look".to_string()));
    at(&mut map, None, 10, "Hall", HALL);
    let look = map.view().unwrap().look.expect("a look");
    assert_eq!(
        (look.command.as_str(), look.spoken.as_str()),
        ("look", "Look around")
    );
    // In the dark, in a place with no name.
    let mut study = At::room(Some("north"), 11, "Study", STUDY);
    study.dark = Some(true);
    study.feed(&mut map);
    let view = map.view().unwrap();
    assert_eq!(view.places[1].name, None);
    assert!(view.look.is_some());
    // At a question there is nothing to touch.
    study.waiting = Waiting::Answer;
    study.sent = Some("quit");
    study.feed(&mut map);
    assert_eq!(map.view().unwrap().look, None);
}

// L§2: no look where the story has no word, or the place is not known.
#[test]
fn there_is_no_look_without_a_word_or_a_place() {
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", HALL);
    assert_eq!(map.view().unwrap().look, None);
    let mut map = Map::new(true).with_look(Some("look".to_string()));
    at(&mut map, None, 10, "Hall", HALL);
    let mut lost = At::room(Some("north"), 11, "Study", STUDY);
    lost.room = None;
    lost.feed(&mut map);
    assert_eq!(map.view().unwrap().look, None);
}

// ---------------------------------------------------------------------------
// Feature 015: the other sheets
// (specs/015-map-sheets/contracts/sheets.md, cited L§)
// ---------------------------------------------------------------------------

/// Three sheets: the Hall and the Study; a Cave reached by a word, with a
/// Grotto east of it; and a Vault reached by another word from the Hall. The
/// player ends in the Hall.
fn three_sheets() -> Map {
    let cave: &[WayOf] = &[(East, Open, Some("Grotto"))];
    let grotto: &[WayOf] = &[(West, Open, Some("Cave")), (South, Open, None)];
    let mut map = Map::new(true).with_look(Some("look".to_string()));
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    at(&mut map, Some("south"), 10, "Hall", HALL);
    at(&mut map, Some("xyzzy"), 20, "Cave", cave);
    at(&mut map, Some("east"), 21, "Grotto", grotto);
    at(&mut map, Some("west"), 20, "Cave", cave);
    at(&mut map, Some("xyzzy"), 10, "Hall", HALL);
    at(&mut map, Some("plugh"), 30, "Vault", &[]);
    at(&mut map, Some("plugh"), 10, "Hall", HALL);
    map
}

// L§1, L§2, L§3: any sheet can be had; the player's is the view; another has
// no place of the player's, no ways and no look, and every place's stubs.
#[test]
fn any_sheet_of_the_map_can_be_had() {
    let map = three_sheets();
    assert_eq!(map.sheet_count(), 3);
    assert_eq!(map.sheet(1), map.view());
    assert_eq!(map.sheet(0), None);
    assert_eq!(map.sheet(4), None);
    let own = map.sheet(1).unwrap();
    assert!(own.own);
    assert_eq!(own.here, Some(10));
    assert!(!own.ways.is_empty() && own.look.is_some());
    let cave = map.sheet(2).unwrap();
    assert!(!cave.own);
    assert_eq!((cave.sheet, cave.sheets, cave.here), (2, 3, None));
    assert!(cave.ways.is_empty());
    assert_eq!(cave.look, None);
    assert_eq!(cave.note, None);
    assert_eq!(cave.lamp, Lamp::Unknown);
    let ids: Vec<u16> = cave.places.iter().map(|p| p.id).collect();
    assert_eq!(ids, [20, 21]);
    assert_eq!(cave.lines.len(), 1);
    // The Grotto's way south was seen and never walked.
    let stubs: Vec<(u16, &str)> = cave.stubs.iter().map(|s| (s.of, s.way.as_str())).collect();
    assert_eq!(stubs, [(21, "south")]);
    // Every place of the map is on one sheet and one only.
    let mut all: Vec<u16> = (1..=3)
        .flat_map(|n| map.sheet(n).unwrap().places)
        .map(|p| p.id)
        .collect();
    all.sort_unstable();
    assert_eq!(all, [10, 11, 20, 21, 30]);
}

// L§4, L§5: a sheet's label and spoken words; the sheets before and after,
// going round; and the way back to the player's own.
#[test]
fn a_sheet_says_which_it_is_and_which_are_beside_it() {
    let map = three_sheets();
    let own = map.sheet(1).unwrap();
    assert_eq!(own.label, "Sheet 1 of 3 · Hall");
    assert_eq!(own.spoken, "Map. You are in Hall.");
    assert_eq!(own.first, Some(10));
    assert_eq!(own.back, None);
    let refs = |view: &crate::map::MapView| {
        (
            view.before.clone().map(|r| (r.number, r.label)),
            view.after.clone().map(|r| (r.number, r.label)),
        )
    };
    assert_eq!(
        refs(&own),
        (
            Some((3, "Sheet 3 of 3 · Vault".to_string())),
            Some((2, "Sheet 2 of 3 · Cave".to_string()))
        )
    );
    let cave = map.sheet(2).unwrap();
    assert_eq!(cave.label, "Sheet 2 of 3 · Cave");
    assert_eq!(
        cave.spoken,
        "Map. Sheet 2 of 3, Cave. You are not on this sheet."
    );
    assert_eq!(cave.first, Some(20));
    assert_eq!(
        cave.back.clone().map(|r| (r.number, r.label)),
        Some((1, "Sheet 1 of 3 · Hall".to_string()))
    );
    let vault = map.sheet(3).unwrap();
    assert_eq!(refs(&vault).1, Some((1, "Sheet 1 of 3 · Hall".to_string())));

    // One sheet: nothing before or after it.
    let mut one = Map::new(true);
    at(&mut one, None, 10, "Hall", HALL);
    let view = one.view().unwrap();
    assert_eq!(view.label, "Sheet 1 of 1 · Hall");
    assert_eq!((view.before, view.after, view.back), (None, None, None));

    // A sheet whose places have no name yet is called by its numbers.
    let mut dark = Map::new(true);
    at(&mut dark, None, 10, "Hall", HALL);
    let mut pit = At::room(Some("xyzzy"), 40, "Pit", &[]);
    pit.dark = Some(true);
    pit.feed(&mut dark);
    at(&mut dark, Some("xyzzy"), 10, "Hall", HALL);
    let pit = dark.sheet(2).unwrap();
    assert_eq!(pit.label, "Sheet 2 of 2");
    assert_eq!(pit.spoken, "Map. Sheet 2 of 2. You are not on this sheet.");
}

// L§6: a way that leaves a sheet says which place it leads to, and that place
// is on the sheet it names.
#[test]
fn a_way_off_the_sheet_names_the_place_it_leads_to() {
    let map = three_sheets();
    let own = map.sheet(1).unwrap();
    let leads: Vec<(u16, &str, usize, u16)> = own
        .leads
        .iter()
        .map(|l| (l.from, l.label.as_str(), l.sheet, l.place))
        .collect();
    assert_eq!(leads, [(10, "xyzzy", 2, 20), (10, "plugh", 3, 30)]);
    for lead in &own.leads {
        let there = map.sheet(lead.sheet).unwrap();
        assert!(there.places.iter().any(|p| p.id == lead.place));
    }
    // And back from the cave's sheet to the player's.
    let cave = map.sheet(2).unwrap();
    assert_eq!(cave.leads.len(), 1);
    assert_eq!((cave.leads[0].sheet, cave.leads[0].place), (1, 10));
}

// ---------------------------------------------------------------------------
// Feature 016: the sheet redrawn around the player
// (specs/016-map-untangle/contracts/untangle.md, cited R§ and S§)
// ---------------------------------------------------------------------------

/// A sheet with a place out of place. The Pit, found first, has the position
/// south of the Hall; the Yard, which is south of the Hall, was pushed past
/// it. The player ends in the Yard.
fn out_of_place() -> Map {
    let hall: &[WayOf] = &[(East, Open, Some("Den")), (South, Open, Some("Yard"))];
    let den: &[WayOf] = &[(West, Open, Some("Hall")), (Southwest, Open, Some("Pit"))];
    let pit: &[WayOf] = &[(Northeast, Open, Some("Den"))];
    let mut map = Map::new(true);
    at(&mut map, None, 10, "Hall", hall);
    at(&mut map, Some("east"), 11, "Den", den);
    at(&mut map, Some("southwest"), 13, "Pit", pit);
    at(&mut map, Some("northeast"), 11, "Den", den);
    at(&mut map, Some("west"), 10, "Hall", hall);
    at(
        &mut map,
        Some("south"),
        12,
        "Yard",
        &[(North, Open, Some("Hall"))],
    );
    map
}

fn kind_of(map: &Map, a: u16, b: u16) -> LineKind {
    map.view()
        .unwrap()
        .lines
        .iter()
        .find(|l| (l.a, l.b) == (a, b) || (l.a, l.b) == (b, a))
        .unwrap_or_else(|| panic!("no line between {a} and {b}"))
        .kind
}

// R§1, R§3, R§6, R§8: the player's place stays; the place it was walked into
// from is put opposite that way; the join is straight; no two places share a
// position; and a second redraw changes nothing.
#[test]
fn a_redraw_puts_the_places_round_the_player_where_the_ways_say() {
    let mut map = out_of_place();
    assert_eq!(map.position(12), Some((1, 0, 2)), "pushed past the Pit");
    assert_eq!(kind_of(&map, 10, 12), LineKind::Bent);
    assert!(map.redraw());
    assert_eq!(
        map.position(12),
        Some((1, 0, 2)),
        "the player's place has not moved"
    );
    assert_eq!(map.position(10), Some((1, 0, 1)), "the Hall is north of it");
    assert_eq!(kind_of(&map, 10, 12), LineKind::Straight);
    let mut cells: Vec<(usize, i32, i32)> = [10, 11, 12, 13]
        .iter()
        .map(|id| map.position(*id).unwrap())
        .collect();
    cells.sort_unstable();
    cells.dedup();
    assert_eq!(cells.len(), 4);
    assert_eq!(map.view().unwrap().here, Some(12));
    // Again, with nothing walked between: nothing changes.
    let view = map.view();
    assert!(!map.redraw());
    assert_eq!(map.view(), view);
}

// S§2: a redraw is on offer while the game waits for a command, the player's
// place is known and their sheet has two places; and not on another sheet.
#[test]
fn a_redraw_is_offered_where_it_can_be_made() {
    let map = out_of_place();
    assert_eq!(
        map.view().unwrap().redraw,
        Some("Redraw the map around here")
    );
    let mut one = Map::new(true);
    at(&mut one, None, 10, "Hall", HALL);
    assert_eq!(one.view().unwrap().redraw, None);
    assert!(!one.redraw());
    // At a question.
    let mut asked = out_of_place();
    let mut yard = At::room(Some("quit"), 12, "Yard", &[]);
    yard.waiting = Waiting::Answer;
    yard.feed(&mut asked);
    assert_eq!(asked.view().unwrap().redraw, None);
    // Another sheet than the player's.
    let sheets = three_sheets();
    assert!(sheets.sheet(1).unwrap().redraw.is_some());
    assert_eq!(sheets.sheet(2).unwrap().redraw, None);
}

// R§5: a place attached to another keeps its corner of it; and where the
// player is in an attached place, it and the place it hangs from stay.
#[test]
fn attached_places_go_with_the_places_they_hang_from() {
    let hall: &[WayOf] = &[
        (East, Open, Some("Den")),
        (South, Open, Some("Yard")),
        (Up, Open, Some("Loft")),
    ];
    let den: &[WayOf] = &[(West, Open, Some("Hall")), (Southwest, Open, Some("Pit"))];
    let pit: &[WayOf] = &[(Northeast, Open, Some("Den"))];
    let build = || {
        let mut map = Map::new(true);
        at(&mut map, None, 10, "Hall", hall);
        at(
            &mut map,
            Some("up"),
            14,
            "Loft",
            &[(Down, Open, Some("Hall"))],
        );
        at(&mut map, Some("down"), 10, "Hall", hall);
        at(&mut map, Some("east"), 11, "Den", den);
        at(&mut map, Some("southwest"), 13, "Pit", pit);
        at(&mut map, Some("northeast"), 11, "Den", den);
        at(&mut map, Some("west"), 10, "Hall", hall);
        map
    };
    let mut map = build();
    at(
        &mut map,
        Some("south"),
        12,
        "Yard",
        &[(North, Open, Some("Hall"))],
    );
    let (hall_before, loft_before) = (map.position(10).unwrap(), map.position(14).unwrap());
    assert!(map.is_inset(14));
    assert!(map.redraw());
    let (hall_after, loft_after) = (map.position(10).unwrap(), map.position(14).unwrap());
    assert_ne!(hall_before, hall_after, "the Hall moved");
    assert_eq!(
        (loft_after.1 - loft_before.1, loft_after.2 - loft_before.2),
        (hall_after.1 - hall_before.1, hall_after.2 - hall_before.2),
        "the Loft went with it"
    );
    assert!(map.is_inset(14));
    assert_eq!(kind_of(&map, 10, 14), LineKind::Ladder);

    // In the Loft: neither it nor the Hall moves.
    let mut map = build();
    at(
        &mut map,
        Some("south"),
        12,
        "Yard",
        &[(North, Open, Some("Hall"))],
    );
    at(&mut map, Some("north"), 10, "Hall", hall);
    at(
        &mut map,
        Some("up"),
        14,
        "Loft",
        &[(Down, Open, Some("Hall"))],
    );
    let (hall, loft) = (map.position(10), map.position(14));
    map.redraw();
    assert_eq!((map.position(10), map.position(14)), (hall, loft));
    assert_eq!(
        kind_of(&map, 10, 12),
        LineKind::Straight,
        "the Yard is south of the Hall now"
    );
}

// R§6: a redraw is of the player's sheet; every other is as it was, and
// every place is on the sheet it was on.
#[test]
fn a_redraw_leaves_other_sheets_alone() {
    let mut map = three_sheets();
    let (cave, vault) = (map.sheet(2), map.sheet(3));
    let count = map.sheet_count();
    map.redraw();
    assert_eq!(map.sheet_count(), count);
    assert_eq!(map.sheet(2), cave);
    assert_eq!(map.sheet(3), vault);
    let ids: Vec<u16> = map.sheet(1).unwrap().places.iter().map(|p| p.id).collect();
    assert_eq!(ids, [10, 11]);
}

// ---------------------------------------------------------------------------
// Feature 017: places put where a presentation file says
// (specs/017-authored-map/contracts/presentation.md, parts M and S).
// ---------------------------------------------------------------------------

/// A map with a file made by hand: sheet 1 is named and has 10, 11 (north of
/// 10, two steps), 13, and 20, which no test visits; sheet 2 has 12.
fn authored() -> Map {
    let file = crate::presentation::Presentation::parse(&format!(
        "presentation: 1\nsha256: {}\nrelease: 1\nserial: 000000\n\
         sheet: 1 Upstairs\nsheet: 2\n\
         place: 10 1 5 5\nplace: 11 1 5 3\nplace: 13 1 7 5\nplace: 20 1 6 5\nplace: 12 2 0 0\n",
        "0".repeat(64)
    ))
    .unwrap();
    Map::new(true).with_presentation(Some(file))
}

fn place_of(map: &Map, id: u16) -> crate::map::PlaceView {
    let (sheet, ..) = map.position(id).expect("a place");
    let view = map.sheet(sheet).unwrap();
    view.places
        .into_iter()
        .find(|place| place.id == id)
        .unwrap()
}

const ATTIC: &[WayOf] = &[(Down, Open, Some("Study"))];

// M§1, S§1: a place the file has is put at its sheet and position, not
// inset, whatever way it was reached by.
#[test]
fn an_authored_place_is_put_where_the_file_says() {
    // The first place of a game.
    let mut map = authored();
    at(&mut map, None, 10, "Hall", HALL);
    assert_eq!(map.position(10), Some((1, 5, 5)));
    assert!(place_of(&map, 10).authored);
    // By a compass way: where the file says, not one step along.
    at(
        &mut map,
        Some("north"),
        11,
        "Study",
        &[(South, Open, Some("Hall")), (Up, Open, None)],
    );
    assert_eq!(map.position(11), Some((1, 5, 3)));
    // Up: on the sheet's positions, not attached.
    at(&mut map, Some("up"), 13, "Attic", ATTIC);
    assert_eq!(map.position(13), Some((1, 7, 5)));
    let attic = place_of(&map, 13);
    assert!(attic.authored && !attic.inset && attic.attached_to.is_none());
    // By a word: on the file's sheet for it, which is the map's second.
    at(&mut map, Some("xyzzy"), 12, "Cave", &[]);
    assert_eq!(map.position(12), Some((2, 0, 0)));
    assert!(place_of(&map, 12).authored);

    // After a jump: on the sheet its file's sheet already has, not a new one.
    let mut map = authored();
    at(&mut map, None, 10, "Hall", HALL);
    At {
        jumped: true,
        ..At::room(Some("wait"), 11, "Study", STUDY)
    }
    .feed(&mut map);
    assert_eq!(map.position(11), Some((1, 5, 3)));
    assert_eq!(map.sheet_count(), 1);
}

// M§3, S§1: a place the file does not have is put by the present rules, and
// never on a position the file gives, whether that place is drawn yet or not.
#[test]
fn an_unauthored_place_never_takes_an_authored_position() {
    let mut map = authored();
    at(&mut map, None, 10, "Hall", HALL);
    // East of 10 is 20's position and then 13's; neither has been visited.
    at(
        &mut map,
        Some("east"),
        30,
        "Den",
        &[(West, Open, Some("Hall"))],
    );
    assert_eq!(map.position(30), Some((1, 8, 5)));
    assert!(!place_of(&map, 30).authored);
    // South of 10 is free: one step along, as ever.
    at(&mut map, Some("west"), 10, "Hall", HALL);
    at(&mut map, Some("south"), 31, "Yard", &[]);
    assert_eq!(map.position(31), Some((1, 5, 6)));
    // M§2: nothing is drawn that was not visited.
    let ids: Vec<u16> = map.view().unwrap().places.iter().map(|p| p.id).collect();
    assert_eq!(ids, [10, 30, 31]);
}

// M§4: another order of discovery, the same authored positions.
#[test]
fn authored_positions_do_not_depend_on_the_order_found() {
    let mut one = authored();
    at(&mut one, None, 10, "Hall", HALL);
    at(&mut one, Some("north"), 11, "Study", STUDY);
    at(&mut one, Some("south"), 10, "Hall", HALL);
    at(&mut one, Some("east"), 13, "Attic", ATTIC);
    let mut other = authored();
    at(&mut other, None, 13, "Attic", ATTIC);
    at(&mut other, Some("xyzzy"), 11, "Study", STUDY);
    at(&mut other, Some("south"), 10, "Hall", HALL);
    for id in [10, 11, 13] {
        assert_eq!(one.position(id), other.position(id), "{id}");
    }
    assert_eq!((one.sheet_count(), other.sheet_count()), (1, 1));
}

// M§5: a sheet the file names has that name; sheets are numbered in the
// order reached, whatever the file calls them. A way walked between two of
// the file's sheets is a lead.
#[test]
fn a_named_sheet_is_labelled_with_its_name() {
    let mut map = authored();
    at(&mut map, None, 12, "Cave", &[(East, Open, Some("Hall"))]);
    assert_eq!(map.view().unwrap().label, "Sheet 1 of 1 \u{b7} Cave");
    at(&mut map, Some("east"), 10, "Hall", HALL);
    let view = map.view().unwrap();
    assert_eq!(map.position(10), Some((2, 5, 5)));
    assert_eq!(view.label, "Sheet 2 of 2 \u{b7} Upstairs");
    assert_eq!(
        view.before.as_ref().unwrap().label,
        "Sheet 1 of 2 \u{b7} Cave"
    );
    let cave = map.sheet(1).unwrap();
    assert_eq!(
        cave.spoken,
        "Map. Sheet 1 of 2, Cave. You are not on this sheet."
    );
    assert_eq!(
        cave.back.as_ref().unwrap().label,
        "Sheet 2 of 2 \u{b7} Upstairs"
    );
    let lead = &cave.leads[0];
    assert_eq!(
        (lead.from, lead.way.as_str(), lead.sheet, lead.place),
        (12, "east", 2, 10)
    );
    assert!(cave.lines.is_empty() && view.lines.is_empty());
}

// M§6: no redraw is offered or made on a sheet with an authored place; one
// is on a sheet with none.
#[test]
fn an_authored_sheet_is_not_redrawn() {
    let mut map = authored();
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    at(&mut map, Some("south"), 10, "Hall", HALL);
    at(&mut map, Some("south"), 31, "Yard", &[]);
    assert_eq!(map.view().unwrap().redraw, None);
    let before = map.clone();
    assert!(!map.redraw());
    assert_eq!(map, before);
    // A sheet reached by a word, with nothing of the file on it.
    at(&mut map, Some("plugh"), 40, "Pit", &[(North, Open, None)]);
    at(&mut map, Some("north"), 41, "Ledge", &[]);
    assert_eq!(map.view().unwrap().redraw, Some(crate::map::SPOKEN_REDRAW));
}

// M§7: a map with no file is as it was: nothing is authored.
#[test]
fn without_a_file_nothing_is_authored() {
    let mut map = Map::new(true).with_presentation(None);
    at(&mut map, None, 10, "Hall", HALL);
    at(&mut map, Some("north"), 11, "Study", STUDY);
    assert_eq!(map.position(11), Some((1, 0, -1)));
    assert!(
        map.view()
            .unwrap()
            .places
            .iter()
            .all(|place| !place.authored)
    );
}
