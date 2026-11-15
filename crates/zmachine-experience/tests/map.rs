//! The map, kept by a session as a game is played
//! (specs/012-map-first-slice/contracts/map.md; success criteria cited SC).

mod common;

use common::{asker, read, read_text, say};
use std::collections::{BTreeMap, BTreeSet};
use zmachine_experience::map::{LineKind, Map};
use zmachine_experience::{Outcome, Place, Refused, Seed, Session, Source};

/// Four rooms in a ring: east, south, west and north bring the player round.
fn ring() -> Vec<u8> {
    use zmachine_testkit::toy::*;
    let mut toy = Toy::new(4);
    toy.way(0, EAST, 1)
        .way(1, SOUTH, 2)
        .way(2, WEST, 3)
        .way(3, NORTH, 0);
    toy.way(1, WEST, 0)
        .way(2, NORTH, 1)
        .way(3, EAST, 2)
        .way(0, SOUTH, 3);
    toy.build()
}

fn toy_room(index: usize) -> u16 {
    zmachine_testkit::toy::Toy::room(index)
}

// W§5, P§2, P§8: walking a ring closes it with four straight lines.
#[test]
fn a_ring_of_rooms_closes() {
    let mut s = Session::new(ring(), Seed::Fixed(1)).unwrap();
    for line in ["east", "south", "west", "north"] {
        say(&mut s, line);
    }
    let map = s.map();
    let at: Vec<Option<(usize, i32, i32)>> = (0..4).map(|n| map.position(toy_room(n))).collect();
    assert_eq!(
        at,
        [
            Some((1, 0, 0)),
            Some((1, 1, 0)),
            Some((1, 1, 1)),
            Some((1, 0, 1))
        ]
    );
    let view = map.view().expect("the toy is recognised");
    assert_eq!(view.lines.len(), 4);
    assert!(
        view.lines
            .iter()
            .all(|line| line.kind == LineKind::Straight),
        "{:?}",
        view.lines
    );
    assert_eq!(view.here, Some(toy_room(0)));
    // Each room's way back the other way round has not been walked.
    assert_eq!(view.stubs.len(), 3);
    assert!(view.leads.is_empty());
    // Walked both ways round, every line has its way back and is still straight.
    for line in ["south", "east", "north", "west"] {
        say(&mut s, line);
    }
    let view = s.map().view().unwrap();
    assert_eq!(view.lines.len(), 4);
    assert!(
        view.lines
            .iter()
            .all(|l| l.kind == LineKind::Straight && l.back.is_some())
    );
    assert!(view.stubs.is_empty());
}

// P§9: the same game played the same way gives the same map.
#[test]
fn the_same_game_gives_the_same_map() {
    let play = || {
        let mut s = Session::new(ring(), Seed::Fixed(1)).unwrap();
        for line in ["east", "look", "south", "west"] {
            say(&mut s, line);
        }
        (s.map().page(), s.map().view())
    };
    assert_eq!(play(), play());
}

// A touch on a way is a move like a typed one.
#[test]
fn a_tap_is_drawn_as_a_typed_move_is() {
    let mut typed = Session::new(ring(), Seed::Fixed(1)).unwrap();
    say(&mut typed, "east");
    let mut tapped = Session::new(ring(), Seed::Fixed(1)).unwrap();
    tapped
        .tap(tapped.stamp(), Place::East)
        .expect("east is offered");
    assert_eq!(typed.map().page(), tapped.map().page());
}

// FR-037: a new game begins the map again.
#[test]
fn a_new_game_begins_the_map_again() {
    let mut s = Session::new(ring(), Seed::Fixed(1)).unwrap();
    say(&mut s, "east");
    say(&mut s, "south");
    assert_eq!(
        s.map()
            .page()
            .iter()
            .filter(|l| l.starts_with("place"))
            .count(),
        3
    );
    s.new_game(Seed::Fixed(1));
    let page = s.map().page();
    assert_eq!(page.len(), 1, "{page:?}");
    assert_eq!(s.map().view().unwrap().here, Some(toy_room(0)));
}

// FR-038: a game the layer does not recognise has no map.
#[test]
fn an_unrecognised_game_has_no_map() {
    let mut s = Session::new(asker(), Seed::Fixed(1)).unwrap();
    say(&mut s, "look");
    assert!(s.map().view().is_none());
    assert!(
        s.snapshot_json(0).ends_with(r#","map":null}"#),
        "{}",
        s.snapshot_json(0)
    );
}

// ---------------------------------------------------------------------------
// Zork I and Adventure
// ---------------------------------------------------------------------------

fn zork() -> Session {
    let pack = read_text("packs/zork1.pack");
    Session::with_pack(read("stories/zork1/zork1.z3"), Seed::Fixed(11), Some(&pack)).unwrap()
}

fn advent() -> Session {
    Session::new(read("stories/advent/advent.z5"), Seed::Fixed(1)).unwrap()
}

/// The lines of a story's walkthrough, less the two that set up its
/// transcript.
fn walkthrough(story: &str) -> Vec<String> {
    read_text(&format!("stories/{story}/fidelity/walkthrough.txt"))
        .lines()
        .skip(2)
        .map(str::to_string)
        .collect()
}

/// The place with this name on the map, as it is shown.
fn place_named(map: &Map, name: &str) -> Option<String> {
    map.page()
        .into_iter()
        .find(|line| line.starts_with("place") && line.ends_with(&format!(" {name}")))
}

/// What a walk gave, for the measures of SC-003 and SC-004.
struct Walked {
    places: usize,
    sheets: usize,
    largest: usize,
    straight: usize,
    bent: usize,
    crossings: usize,
}

/// Plays a script and checks at every turn that the map holds exactly the
/// places the player has been in, and that none has moved but once, to a new
/// sheet (SC-002).
fn walk(mut session: Session, script: &[String]) -> (Session, Walked) {
    let mut stood: BTreeSet<u16> = BTreeSet::new();
    let mut put: BTreeMap<u16, (usize, i32, i32, bool)> = BTreeMap::new();
    let mut moved: BTreeSet<u16> = BTreeSet::new();
    for (turn, line) in std::iter::once(None)
        .chain(script.iter().map(Some))
        .enumerate()
    {
        if let Some(line) = line
            && session.send_line(session.stamp(), line).is_err()
        {
            break;
        }
        let map = session.map();
        if let Some(here) = map.view().and_then(|view| view.here) {
            stood.insert(here);
        }
        let page = map.page();
        let drawn = page.iter().filter(|l| l.starts_with("place")).count();
        assert_eq!(
            drawn,
            stood.len(),
            "turn {turn}: exactly the places stood in"
        );
        for id in &stood {
            let (sheet, x, y) = map
                .position(*id)
                .unwrap_or_else(|| panic!("turn {turn}: {id} is drawn"));
            let now = (sheet, x, y, map.is_inset(*id));
            match put.get(id) {
                None => {
                    put.insert(*id, now);
                }
                Some(before) if *before == now => {}
                Some(before) => {
                    assert_ne!(before.0, now.0, "turn {turn}: {id} moved within its sheet");
                    assert!(moved.insert(*id), "turn {turn}: {id} moved a second time");
                    put.insert(*id, now);
                }
            }
        }
        // P§7: no two places in one position.
        let positions: BTreeSet<(usize, i32, i32, bool)> = put.values().copied().collect();
        assert_eq!(
            positions.len(),
            put.len(),
            "turn {turn}: two places in one position"
        );
    }
    let measures = session.map().measures();
    let walked = Walked {
        places: stood.len(),
        sheets: measures.len(),
        largest: measures.iter().map(|m| m.0).max().unwrap_or(0),
        straight: measures.iter().map(|m| m.1).sum(),
        bent: measures.iter().map(|m| m.2).sum(),
        crossings: session.map().crossings(),
    };
    (session, walked)
}

// SC-002, SC-003, SC-004 on Zork I. The spike gave 49 straight of 60 on 11
// sheets.
#[test]
fn zork_along_its_walkthrough() {
    let (session, got) = walk(zork(), &walkthrough("zork1"));
    println!(
        "zork1: {} places on {} sheets (largest {}); {} straight, {} bent; {} crossings",
        got.places, got.sheets, got.largest, got.straight, got.bent, got.crossings
    );
    for line in session.map().page().iter().filter(|l| l.ends_with("bent")) {
        println!("  {line}");
    }
    assert!(got.places > 75, "{}", got.places);
    assert!(
        got.straight * 5 >= (got.straight + got.bent) * 4,
        "four lines in five straight"
    );
    assert_eq!(got.crossings, 0);
    assert!(got.sheets <= 12, "{}", got.sheets);
    assert!(got.largest <= 30, "{}", got.largest);
    // The house and what is above ground are one sheet; the cellar is not on it.
    let page = session.map().page();
    let sheet_of = |name: &str| {
        let line = page
            .iter()
            .find(|l| l.starts_with("place") && l.ends_with(&format!(" {name}")))
            .unwrap_or_else(|| panic!("{name} is on the map"));
        line.split_whitespace().nth(3).unwrap().to_string()
    };
    for name in [
        "North of House",
        "Kitchen",
        "Living Room",
        "Attic",
        "Up a Tree",
    ] {
        assert_eq!(sheet_of(name), sheet_of("West of House"), "{name}");
    }
    assert_ne!(sheet_of("Cellar"), sheet_of("West of House"));
}

// The same on Adventure. The spike gave 44 straight of 51 on 7 sheets.
#[test]
fn adventure_along_its_walkthrough() {
    let (_, got) = walk(advent(), &walkthrough("advent"));
    println!(
        "advent: {} places on {} sheets (largest {}); {} straight, {} bent; {} crossings",
        got.places, got.sheets, got.largest, got.straight, got.bent, got.crossings
    );
    assert!(got.places > 55, "{}", got.places);
    assert!(
        got.straight * 5 >= (got.straight + got.bent) * 4,
        "four lines in five straight"
    );
    assert_eq!(got.crossings, 0);
    assert!(got.sheets <= 8, "{}", got.sheets);
    assert!(got.largest <= 30, "{}", got.largest);
}

// SC-005: on Zork I a place entered only in the dark has no name, and no way
// but the way back, until it is stood in lit; and does not move when it is.
#[test]
fn the_attic_has_no_name_until_it_is_seen_lit() {
    let mut s = zork();
    for line in ["north", "east", "open window", "west"] {
        say(&mut s, line);
    }
    assert!(place_named(s.map(), "Kitchen").is_some());
    say(&mut s, "up");
    let view = s.map().view().unwrap();
    let here = view.here.expect("the player is somewhere");
    assert_eq!(view.lamp.as_str(), "out");
    assert!(view.places.iter().any(|p| p.id == here && p.name.is_none()));
    // W§12a: only the way back to the Kitchen, which the player walked from.
    let ways: Vec<(&str, Option<&str>)> = view
        .ways
        .iter()
        .map(|way| (way.way.as_str(), way.leads_to.as_deref()))
        .collect();
    assert_eq!(ways, [("down", None)]);
    assert_eq!(view.spoken, "Map. You are somewhere dark.");
    assert!(place_named(s.map(), "Attic").is_none());
    let dark_at = s.map().position(here);
    for line in ["down", "west", "take lamp", "turn on lamp", "east", "up"] {
        say(&mut s, line);
    }
    let view = s.map().view().unwrap();
    assert_eq!(view.here, Some(here));
    assert_eq!(view.lamp.as_str(), "lit");
    assert!(place_named(s.map(), "Attic").is_some());
    assert!(!view.ways.is_empty());
    assert_eq!(s.map().position(here), dark_at);
}

// W§7 on Zork I: the compass names where a way leads; the map does not until
// it has been walked.
#[test]
fn zorks_ways_are_named_only_once_walked() {
    let mut s = zork();
    let north = |s: &Session| {
        let view = s.map().view().unwrap();
        view.ways
            .into_iter()
            .find(|w| w.way == "north")
            .expect("north is a way")
    };
    assert_eq!(
        s.compass().place(Place::North).unwrap().leads_to.as_deref(),
        Some("North of House")
    );
    assert_eq!(north(&s).leads_to, None);
    assert_eq!(north(&s).spoken, "North");
    assert!(!s.map().page().iter().any(|l| l.contains("North of House")));
    say(&mut s, "north");
    say(&mut s, "west");
    assert_eq!(north(&s).leads_to.as_deref(), Some("North of House"));
    assert_eq!(north(&s).spoken, "North, to North of House");
}

// W§14 through the session: the game's own restart draws no line.
#[test]
fn zorks_restart_draws_no_line() {
    let mut s = zork();
    say(&mut s, "north");
    let lines = |s: &Session| {
        s.map()
            .page()
            .into_iter()
            .filter(|l| l.starts_with("line"))
            .count()
    };
    assert_eq!(lines(&s), 1);
    say(&mut s, "restart");
    say(&mut s, "y");
    let view = s.map().view().unwrap();
    assert!(
        view.places
            .iter()
            .any(|p| Some(p.id) == view.here && p.name.as_deref() == Some("West of House"))
    );
    assert_eq!(lines(&s), 1, "{:?}", s.map().page());
    assert!(view.lines.iter().all(|line| line.back.is_none()));
}

// SC-008, FR-037: a game brought back shows the map that was left.
#[test]
fn a_game_brought_back_has_the_same_map() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let script = walkthrough("zork1");
    let mut never_closed = zork();
    let step = script.len() / 10;
    for (turn, line) in script.iter().enumerate() {
        if never_closed.send_line(never_closed.stamp(), line).is_err() {
            break;
        }
        if turn % step != step - 1 {
            continue;
        }
        let (brought, outcome) = Session::open_kept(
            story.clone(),
            Some(&pack),
            Seed::Fresh(5),
            Some(&never_closed.kept()),
        )
        .unwrap();
        assert_eq!(outcome, Outcome::BroughtBack, "turn {turn}");
        assert_eq!(
            brought.map().page(),
            never_closed.map().page(),
            "turn {turn}"
        );
        assert_eq!(
            brought.map().view(),
            never_closed.map().view(),
            "turn {turn}"
        );
    }
}

// ---------------------------------------------------------------------------
// Recorded pages (FR-042, FR-046; research R12)
// ---------------------------------------------------------------------------

/// Compares the map at the end of a story's walkthrough with the page kept
/// beside the story. With `UPDATE_PAGES=1` the page is written instead.
fn check_recorded_page(story: &str, session: Session) {
    let (session, _) = walk(session, &walkthrough(story));
    let page = session.map().page().join("\n") + "\n";
    let path = format!("{}/stories/{story}/map/walkthrough.txt", common::ROOT);
    if std::env::var("UPDATE_PAGES").is_ok_and(|v| v == "1") {
        std::fs::create_dir_all(format!("{}/stories/{story}/map", common::ROOT)).unwrap();
        std::fs::write(&path, &page).unwrap();
    }
    let kept = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{path}: {e}. Run with UPDATE_PAGES=1 to write it"));
    assert!(
        kept == page,
        "the map of {story}'s walkthrough is not the page kept in {path}.\n\
         If the rules were meant to change, run with UPDATE_PAGES=1 and read the difference."
    );
}

#[test]
fn zorks_page_is_the_one_recorded() {
    check_recorded_page("zork1", zork());
}

#[test]
fn adventures_page_is_the_one_recorded() {
    check_recorded_page("advent", advent());
}

// ---------------------------------------------------------------------------
// A second order of exploring (FR-043; research R13)
// ---------------------------------------------------------------------------

/// Wanders for `moves` moves: at each command prompt one of the ways the map
/// offers, by a seeded draw. Returns the session and how many moves were made.
fn wander(mut session: Session, seed: u32, moves: usize) -> (Session, usize) {
    let mut state = seed;
    let mut made = 0;
    for _ in 0..moves {
        let ways: Vec<String> = session
            .map()
            .view()
            .map(|view| {
                view.ways
                    .into_iter()
                    .filter_map(|way| way.command)
                    .collect()
            })
            .unwrap_or_default();
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let line = match ways.len() {
            0 => "look".to_string(),
            n => ways[(state >> 16) as usize % n].clone(),
        };
        if session.send_line(session.stamp(), &line).is_err() {
            break;
        }
        made += 1;
    }
    (session, made)
}

// The map of a wander obeys the same rules as the map of a walkthrough. The
// measures are printed and recorded in tasks.md; no bar is set on them.
#[test]
fn a_wander_gives_a_sound_map() {
    for (name, start) in [("zork1", zork as fn() -> Session), ("advent", advent)] {
        for seed in [1, 2, 3] {
            let (session, made) = wander(start(), seed, 400);
            let map = session.map();
            let measures = map.measures();
            let places: usize = measures.iter().map(|m| m.0).sum();
            let (straight, bent): (usize, usize) = measures
                .iter()
                .fold((0, 0), |sum, m| (sum.0 + m.1, sum.1 + m.2));
            println!(
                "{name} wander {seed}: {made} moves, {places} places on {} sheets (largest {}); \
                 {straight} straight, {bent} bent; {} crossings",
                measures.len(),
                measures.iter().map(|m| m.0).max().unwrap_or(0),
                map.crossings()
            );
            // P§7: no two places in one position.
            let mut seen = BTreeSet::new();
            for line in map.page().iter().filter(|l| l.starts_with("place")) {
                let words: Vec<&str> = line.split_whitespace().collect();
                // The sheet, which grid, and the position.
                let position = (words[3], words[4], words[5], words[6]);
                assert!(seen.insert(position), "{name} {seed}: {line}");
            }
            assert!(places >= 2, "{name} {seed}: the wander went somewhere");
        }
    }
}

// Writes pages under `target/map-pages/` for looking at: the walkthroughs and
// a wander of each game. They hold rooms' names and are not to be committed.
#[test]
#[ignore = "writes pages to look at; run with -- --ignored"]
fn write_pages_to_look_at() {
    let folder = format!("{}/target/map-pages", common::ROOT);
    std::fs::create_dir_all(&folder).unwrap();
    let write = |name: &str, session: &Session| {
        std::fs::write(
            format!("{folder}/{name}.txt"),
            session.map().page().join("\n"),
        )
        .unwrap();
    };
    write("zork1-walkthrough", &walk(zork(), &walkthrough("zork1")).0);
    write(
        "advent-walkthrough",
        &walk(advent(), &walkthrough("advent")).0,
    );
    write("zork1-wander", &wander(zork(), 1, 400).0);
    write("advent-wander", &wander(advent(), 1, 400).0);
}

// Contract A§2 and Principle III: a touch on a way of the map sends that
// way's command, as a tap, and is refused where there is nothing to send.
#[test]
fn a_way_of_the_map_is_tapped_by_its_place_in_the_list() {
    use zmachine_experience::{Refused, Source};
    let mut s = Session::new(ring(), Seed::Fixed(1)).unwrap();
    let ways = s.map().view().unwrap().ways;
    let east = ways
        .iter()
        .position(|way| way.way == "east")
        .expect("east is a way");
    s.tap_way(s.stamp(), east).expect("the way can be tapped");
    let turn = s.turns().last().unwrap();
    assert_eq!(
        (turn.command.as_deref(), turn.source),
        (Some("east"), Some(Source::Tap))
    );
    assert_eq!(s.map().view().unwrap().here, Some(toy_room(1)));
    // Beyond the list, and against a turn that has passed: refused, and
    // nothing is sent.
    let turns = s.turns().len();
    assert_eq!(s.tap_way(s.stamp(), 99), Err(Refused::NotOffered));
    let mut stale = s.stamp();
    stale.turn_count -= 1;
    assert_eq!(s.tap_way(stale, 0), Err(Refused::Stale));
    assert_eq!(s.turns().len(), turns);
    // A game with no map offers no way.
    let mut none = Session::new(asker(), Seed::Fixed(1)).unwrap();
    assert_eq!(none.tap_way(none.stamp(), 0), Err(Refused::NotOffered));
}

// ---------------------------------------------------------------------------
// Feature 013: touching the place the player is in, to look
// (specs/013-room-look/contracts/look.md, cited L§)
// ---------------------------------------------------------------------------

/// Walks out of the first place and back, touches it, and returns the reply.
fn looked_at(mut s: Session, out: &str, back: &str) -> (Session, String) {
    say(&mut s, out);
    say(&mut s, back);
    let page = s.map().page();
    let view = s.map().view().unwrap();
    let look = view.look.clone().expect("the place can be touched");
    s.tap_place(s.stamp()).expect("the look is sent");
    let turn = s.turns().last().unwrap().clone();
    assert_eq!(
        (turn.command.as_deref(), turn.source),
        (Some(look.command.as_str()), Some(Source::Tap))
    );
    // L§4: the map is as it was.
    assert_eq!(s.map().page(), page);
    assert_eq!(s.map().view().unwrap().places, view.places);
    (s, turn.text)
}

// L§3, L§4; SC-001: on Zork I one touch brings the full description.
#[test]
fn touching_the_place_looks_on_zork() {
    let (mut s, text) = looked_at(zork(), "north", "west");
    assert_eq!(s.map().view().unwrap().look.unwrap().command, "look");
    assert!(text.contains("open field west of a white house"), "{text}");
    // A stale stamp is refused, and nothing is sent.
    let turns = s.turns().len();
    let mut stale = s.stamp();
    stale.turn_count -= 1;
    assert_eq!(s.tap_place(stale), Err(Refused::Stale));
    assert_eq!(s.turns().len(), turns);
}

// SC-001 on Adventure.
#[test]
fn touching_the_place_looks_on_adventure() {
    let (_, text) = looked_at(advent(), "east", "west");
    assert!(text.contains("end of a road"), "{text}");
}

// L§3: at a question there is no look, and a touch is refused.
#[test]
fn a_touch_at_a_question_is_refused() {
    let mut s = zork();
    say(&mut s, "quit");
    assert_eq!(s.map().view().unwrap().look, None);
    let turns = s.turns().len();
    assert_eq!(s.tap_place(s.stamp()), Err(Refused::NotOffered));
    assert_eq!(s.turns().len(), turns);
    // A game with no map offers no look.
    let mut none = Session::new(asker(), Seed::Fixed(1)).unwrap();
    assert_eq!(none.tap_place(none.stamp()), Err(Refused::NotOffered));
}

// L§5; SC-003: a kept game holding touches on the place comes back the same.
#[test]
fn a_game_with_looks_comes_back_the_same() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let mut s = zork();
    for line in ["north", "east"] {
        say(&mut s, line);
        s.tap_place(s.stamp()).unwrap();
    }
    for _ in 0..100 {
        let (brought, outcome) =
            Session::open_kept(story.clone(), Some(&pack), Seed::Fresh(5), Some(&s.kept()))
                .unwrap();
        assert_eq!(outcome, Outcome::BroughtBack);
        assert_eq!(brought.map().view(), s.map().view());
        assert_eq!(brought.turns().len(), s.turns().len());
        assert_eq!(brought.turns().last(), s.turns().last());
    }
}

// ---------------------------------------------------------------------------
// Feature 015: the other sheets
// (specs/015-map-sheets/contracts/sheets.md; success criteria cited SC)
// ---------------------------------------------------------------------------

/// SC-001 to SC-003, L§7: every sheet can be had; together they hold every
/// place once; each way off a sheet leads to a place on the sheet it names;
/// and asking changes nothing.
fn every_sheet_can_be_had(mut session: Session, script: &[String]) -> usize {
    for line in script {
        if session.send_line(session.stamp(), line).is_err() {
            break;
        }
    }
    let (kept, snapshot, page) = (
        session.kept(),
        session.snapshot_json(0),
        session.map().page(),
    );
    let count = session.map().sheet_count();
    let mut places = BTreeSet::new();
    let mut total = 0;
    for number in 1..=count {
        let sheet = session
            .sheet(session.stamp(), number)
            .expect("the sheet is given");
        assert_eq!((sheet.sheet, sheet.sheets), (number, count));
        assert!(
            sheet
                .label
                .starts_with(&format!("Sheet {number} of {count}")),
            "{}",
            sheet.label
        );
        assert_eq!(
            sheet.before.as_ref().map(|r| r.number),
            Some(if number == 1 { count } else { number - 1 })
        );
        assert_eq!(
            sheet.after.as_ref().map(|r| r.number),
            Some(number % count + 1)
        );
        assert_eq!(sheet.own, sheet.back.is_none());
        if !sheet.own {
            assert!(sheet.here.is_none() && sheet.ways.is_empty() && sheet.look.is_none());
        }
        for place in &sheet.places {
            total += 1;
            assert!(
                places.insert(place.id),
                "place {} is on two sheets",
                place.id
            );
        }
        for lead in &sheet.leads {
            let there = session
                .sheet(session.stamp(), lead.sheet)
                .expect("a lead's sheet");
            assert!(
                there.places.iter().any(|p| p.id == lead.place),
                "sheet {number}: a lead's place is not on sheet {}",
                lead.sheet
            );
        }
    }
    assert_eq!(
        total,
        session
            .map()
            .page()
            .iter()
            .filter(|l| l.starts_with("place"))
            .count()
    );
    // Asking changed nothing.
    assert_eq!(session.kept(), kept);
    assert_eq!(session.snapshot_json(0), snapshot);
    assert_eq!(session.map().page(), page);
    // A turn that has passed, and a number with no sheet, are refused.
    let mut stale = session.stamp();
    stale.turn_count -= 1;
    assert_eq!(session.sheet(stale, 1).err(), Some(Refused::Stale));
    assert_eq!(
        session.sheet(session.stamp(), count + 1).err(),
        Some(Refused::NotOffered)
    );
    assert_eq!(
        session.sheet(session.stamp(), 0).err(),
        Some(Refused::NotOffered)
    );
    count
}

#[test]
fn every_sheet_of_zorks_map_can_be_had() {
    assert_eq!(every_sheet_can_be_had(zork(), &walkthrough("zork1")), 11);
}

#[test]
fn every_sheet_of_adventures_map_can_be_had() {
    assert_eq!(every_sheet_can_be_had(advent(), &walkthrough("advent")), 7);
}

// ---------------------------------------------------------------------------
// Feature 016: the sheet redrawn around the player
// (specs/016-map-untangle/contracts/untangle.md, cited R§ and K§)
// ---------------------------------------------------------------------------

/// A wander through Zork I with a redraw asked for every `every` moves.
fn wander_redrawing(
    mut session: Session,
    seed: u32,
    moves: usize,
    every: usize,
) -> (Session, usize) {
    let mut state = seed;
    let mut redrawn = 0;
    for made in 1..=moves {
        let ways: Vec<String> = session
            .map()
            .view()
            .map(|view| {
                view.ways
                    .into_iter()
                    .filter_map(|way| way.command)
                    .collect()
            })
            .unwrap_or_default();
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let line = match ways.len() {
            0 => "look".to_string(),
            n => ways[(state >> 16) as usize % n].clone(),
        };
        if session.send_line(session.stamp(), &line).is_err() {
            break;
        }
        if made % every == 0 && session.redraw_map(session.stamp()).is_ok() {
            redrawn += 1;
        }
    }
    (session, redrawn)
}

/// A kept game's bytes with its lines changed by `change`, and its check
/// made again so that only the lines differ.
fn with_lines(kept: &[u8], change: impl FnOnce(&mut Vec<String>)) -> Vec<u8> {
    let text = String::from_utf8(kept.to_vec()).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines.pop();
    change(&mut lines);
    let body = format!("{}\n", lines.join("\n"));
    let check = zmachine_introspect::pack::sha256_hex(body.as_bytes());
    format!("{body}check {check}\n").into_bytes()
}

// R§10, and the refusals: a redraw sends nothing and adds no turn; it is
// refused for a turn that has passed and where there is nothing to redraw.
#[test]
fn a_redraw_sends_the_game_nothing() {
    let mut s = zork();
    assert_eq!(s.map().view().unwrap().redraw, None);
    assert_eq!(s.redraw_map(s.stamp()), Err(Refused::NotOffered));
    say(&mut s, "north");
    assert!(s.map().view().unwrap().redraw.is_some());
    let (turns, stamp, moves) = (
        s.turns().len(),
        s.stamp(),
        s.status().and_then(|st| st.moves),
    );
    let mut stale = stamp;
    stale.turn_count -= 1;
    assert_eq!(s.redraw_map(stale), Err(Refused::Stale));
    assert_eq!(s.redraw_map(stamp), Ok(()));
    assert_eq!(s.turns().len(), turns);
    assert_eq!(s.stamp(), stamp);
    assert_eq!(s.status().and_then(|st| st.moves), moves);
    // At a question there is none.
    say(&mut s, "quit");
    assert_eq!(s.redraw_map(s.stamp()), Err(Refused::NotOffered));
}

// K§1, K§2; SC-003: a game with redraws at several turns is kept in form 2 and
// comes back with the map it was left with, every time.
#[test]
fn a_redrawn_map_comes_back_as_it_was_left() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let (s, redrawn) = wander_redrawing(zork(), 1, 120, 30);
    assert_eq!(redrawn, 4);
    let kept = s.kept();
    assert!(kept.starts_with(b"lamplight-kept 2\n"));
    let lines = String::from_utf8(kept.clone()).unwrap();
    assert!(
        lines.lines().filter(|l| *l == "redraw").count() >= 1,
        "{lines}"
    );
    // A wander with no redraw differs: the redraws did move places.
    let plain = wander(zork(), 1, 120).0;
    assert!(plain.kept().starts_with(b"lamplight-kept 1\n"));
    assert_ne!(plain.map().page(), s.map().page());
    for _ in 0..100 {
        let (brought, outcome) =
            Session::open_kept(story.clone(), Some(&pack), Seed::Fresh(5), Some(&kept)).unwrap();
        assert_eq!(outcome, Outcome::BroughtBack);
        assert_eq!(brought.map().page(), s.map().page());
        assert_eq!(brought.map().view(), s.map().view());
        assert_eq!(brought.kept(), kept);
    }
}

// K§2: a redraw belongs where it was made. Moved one input later in the
// kept game, the map that comes back is another.
#[test]
fn a_redraw_one_input_late_gives_another_map() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let (s, _) = wander_redrawing(zork(), 1, 120, 30);
    let mut differed = 0;
    let redraws = String::from_utf8(s.kept())
        .unwrap()
        .lines()
        .filter(|l| *l == "redraw")
        .count();
    for which in 0..redraws {
        let late = with_lines(&s.kept(), |lines| {
            let at = lines
                .iter()
                .enumerate()
                .filter(|(_, l)| *l == "redraw")
                .nth(which)
                .unwrap()
                .0;
            if at + 1 < lines.len() {
                lines.swap(at, at + 1);
            }
        });
        let (brought, outcome) =
            Session::open_kept(story.clone(), Some(&pack), Seed::Fresh(5), Some(&late)).unwrap();
        if outcome == Outcome::BroughtBack && brought.map().page() != s.map().page() {
            differed += 1;
        }
    }
    assert!(differed > 0, "moving a redraw never changed the map");
}

// K§4: a redraw where none can be made is no kept game of this story.
#[test]
fn a_redraw_that_cannot_be_made_is_not_brought_back() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let mut s = zork();
    say(&mut s, "north");
    // Before the first input the map has one place.
    let bad = with_lines(&s.kept(), |lines| {
        lines[0] = "lamplight-kept 2".to_string();
        let first = lines.iter().position(|l| l.starts_with("typed ")).unwrap();
        lines.insert(first, "redraw".to_string());
    });
    let (_, outcome) = Session::open_kept(story, Some(&pack), Seed::Fresh(5), Some(&bad)).unwrap();
    assert_eq!(outcome, Outcome::CouldNot);
}

/// Of the joins of the place the player is in: how many are straight, and
/// how many there are; and how many joins of the whole sheet are bent.
fn clarity(session: &Session) -> (usize, usize, usize) {
    let view = session.map().view().unwrap();
    let here = view.here.unwrap();
    let compass = |l: &&zmachine_experience::map::LineView| {
        matches!(l.kind, LineKind::Straight | LineKind::Bent)
    };
    let own: Vec<_> = view
        .lines
        .iter()
        .filter(compass)
        .filter(|l| l.a == here || l.b == here)
        .collect();
    (
        own.iter().filter(|l| l.kind == LineKind::Straight).count(),
        own.len(),
        view.lines
            .iter()
            .filter(|l| l.kind == LineKind::Bent)
            .count(),
    )
}

// SC-001, SC-002: along a wander through Zork I, at every tenth move a copy
// of the game is redrawn around the room the player is in. The room's own
// joins are never fewer straight than they were; in at least four cases of
// five every one of them is straight; and the sheet has no more bent joins
// than it had. The figures are printed.
#[test]
fn a_redraw_makes_the_players_room_clear() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let mut session = zork();
    let mut state = 1u32;
    let (mut cases, mut clear_before, mut clear_after) = (0, 0, 0);
    let (mut bent_before, mut bent_after, mut worse) = (0, 0, 0);
    for made in 1..=400 {
        let ways: Vec<String> = session
            .map()
            .view()
            .map(|view| {
                view.ways
                    .into_iter()
                    .filter_map(|way| way.command)
                    .collect()
            })
            .unwrap_or_default();
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let line = match ways.len() {
            0 => "look".to_string(),
            n => ways[(state >> 16) as usize % n].clone(),
        };
        if session.send_line(session.stamp(), &line).is_err() {
            break;
        }
        if made % 10 != 0 || session.map().view().is_none_or(|v| v.redraw.is_none()) {
            continue;
        }
        let (mut copy, _) = Session::open_kept(
            story.clone(),
            Some(&pack),
            Seed::Fresh(5),
            Some(&session.kept()),
        )
        .unwrap();
        let (straight, own, bent) = clarity(&copy);
        if own == 0 {
            continue;
        }
        let here = copy
            .map()
            .position(copy.map().view().unwrap().here.unwrap());
        copy.redraw_map(copy.stamp()).unwrap();
        assert_eq!(
            copy.map()
                .position(copy.map().view().unwrap().here.unwrap()),
            here,
            "the player's place moved at move {made}"
        );
        let (straight_after, own_after, bent_now) = clarity(&copy);
        assert_eq!(own_after, own);
        assert!(
            straight_after >= straight,
            "move {made}: {straight} of {own} straight before, {straight_after} after"
        );
        cases += 1;
        clear_before += usize::from(straight == own);
        clear_after += usize::from(straight_after == own);
        bent_before += bent;
        bent_after += bent_now;
        worse += usize::from(bent_now > bent);
    }
    println!(
        "redraw on a wander: {cases} cases; the room's own joins all straight in {clear_before} before \
         and {clear_after} after; bent joins on the sheet, summed, {bent_before} before and \
         {bent_after} after; the sheet had more bent joins after in {worse} cases"
    );
    assert!(cases >= 20, "{cases}");
    assert!(clear_after * 5 >= cases * 4, "{clear_after} of {cases}");
    assert_eq!(worse, 0);
}
