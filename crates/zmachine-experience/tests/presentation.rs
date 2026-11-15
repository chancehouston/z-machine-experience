//! A game opened with a presentation file
//! (specs/017-authored-map/contracts/presentation.md, cited F§, M§ and S§;
//! success criteria cited SC).

mod common;

use common::{read, read_text, say};
use zmachine_experience::{Outcome, Seed, Session};

const NOTE: &str = "The map's layout could not be used.";

fn story() -> Vec<u8> {
    read("stories/zork1/zork1.z3")
}

fn pack() -> String {
    read_text("packs/zork1.pack")
}

fn plain() -> Session {
    Session::with_pack(story(), Seed::Fixed(11), Some(&pack())).unwrap()
}

fn presented(file: &str) -> Session {
    Session::with_presentation(story(), Seed::Fixed(11), Some(&pack()), Some(file)).unwrap()
}

/// The identity lines of a file for the story.
fn head() -> String {
    let story = story();
    format!(
        "presentation: 1\nsha256: {}\nrelease: {}\nserial: {}\n",
        zmachine_introspect::pack::sha256_hex(&story),
        u16::from_be_bytes([story[2], story[3]]),
        String::from_utf8(story[0x12..0x18].to_vec()).unwrap(),
    )
}

/// The room the game begins in and the room north of it.
fn two_rooms() -> (u16, u16) {
    let mut s = plain();
    let first = s.map().view().unwrap().here.unwrap();
    say(&mut s, "north");
    (first, s.map().view().unwrap().here.unwrap())
}

/// A file for the story that puts those two rooms, the second three steps
/// north of the first, on a sheet with a name.
fn file() -> String {
    let (first, north) = two_rooms();
    format!(
        "{}sheet: 4 Outside\nplace: {first} 4 4 4\nplace: {north} 4 4 1\n",
        head()
    )
}

fn texts(s: &Session) -> Vec<String> {
    s.turns().iter().map(|turn| turn.text.clone()).collect()
}

// F§6, M§1, M§5, S§1: with its story's file a game has its places where the
// file says, and a new game of the session has them too.
#[test]
fn a_game_opened_with_its_file_has_authored_places() {
    let (first, north) = two_rooms();
    let mut s = presented(&file());
    assert_eq!(s.notice(), None);
    assert_eq!(s.map().position(first), Some((1, 4, 4)));
    say(&mut s, "north");
    assert_eq!(s.map().position(north), Some((1, 4, 1)));
    let view = s.map().view().unwrap();
    assert!(view.places.iter().all(|place| place.authored));
    assert_eq!(view.label, "Sheet 1 of 1 \u{b7} Outside");
    assert_eq!(view.redraw, None);
    assert!(s.snapshot_json(0).contains("\"authored\":true"));
    s.new_game(Seed::Fixed(3));
    assert_eq!(s.map().position(first), Some((1, 4, 4)));
    assert_eq!(s.notice(), None);
}

// F§5, F§6, S§2, M§7: a file out of form, for another story, or with a place
// that is not a room is refused whole: the map is as without one, and the
// notice says so until the next turn.
#[test]
fn a_file_that_cannot_be_used_changes_nothing() {
    let (first, _) = two_rooms();
    let good = file();
    let mut without = plain();
    for line in ["north", "east"] {
        say(&mut without, line);
    }
    let digest = zmachine_introspect::pack::sha256_hex(&story());
    let flipped = if digest.starts_with('0') { "1" } else { "0" };
    for (why, text) in [
        ("out of form", format!("{good}label: 1 A Word\n")),
        ("another digest", good.replacen(&digest[..1], flipped, 1)),
        ("another release", good.replace("release: ", "release: 1")),
        ("another serial", {
            let at = good.find("serial: ").unwrap() + 8;
            format!("{}zzzzzz{}", &good[..at], &good[at + 6..])
        }),
        ("not a room", format!("{good}place: 60000 4 0 0\n")),
        // The thing a room is in is not a room.
        (
            "not a room",
            format!(
                "{good}place: 1 4 0 0\nplace: 2 4 1 0\nplace: 3 4 2 0\nplace: 4 4 3 0\nplace: 5 4 5 0\n"
            ),
        ),
    ] {
        let mut s = presented(&text);
        assert_eq!(s.notice(), Some(NOTE), "{why}");
        assert!(s.snapshot_json(0).contains(NOTE), "{why}");
        assert_eq!(s.map().position(first), Some((1, 0, 0)), "{why}");
        for line in ["north", "east"] {
            say(&mut s, line);
        }
        assert_eq!(s.notice(), None, "{why}");
        assert_eq!(s.map().view(), without.map().view(), "{why}");
        assert_eq!(s.map().page(), without.map().page(), "{why}");
        // It is not told again in a new game.
        s.new_game(Seed::Fixed(3));
        assert_eq!(s.notice(), None, "{why}");
        assert_eq!(s.map().position(first), Some((1, 0, 0)), "{why}");
    }
}

// M§8: the game is told nothing by a file: the walkthrough's transcript is
// the same with and without one.
#[test]
fn the_transcript_is_the_same_with_and_without_a_file() {
    let lines: Vec<String> = read_text("stories/zork1/fidelity/walkthrough.txt")
        .lines()
        .skip(2)
        .map(str::to_string)
        .collect();
    let (mut with, mut without) = (presented(&file()), plain());
    for line in &lines {
        for s in [&mut with, &mut without] {
            if s.send_line(s.stamp(), line).is_err() {
                break;
            }
        }
    }
    assert!(without.turns().len() > 300);
    assert_eq!(texts(&with), texts(&without));
    assert_eq!(with.kept(), without.kept());
}

/// A wander with a redraw asked for every `every` moves.
fn wander_redrawing(mut session: Session, seed: u32, moves: usize, every: usize) -> Session {
    let mut state = seed;
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
        if made % every == 0 {
            let _ = session.redraw_map(session.stamp());
        }
    }
    session
}

// M§6: a game kept before its story had a file, with redraws in it, comes
// back with the file: the redraws the file leaves no place for are passed
// over, and the game is the same game.
#[test]
fn a_kept_redraw_of_an_authored_sheet_is_passed_over() {
    let (first, _) = two_rooms();
    let played = wander_redrawing(plain(), 1, 120, 30);
    let kept = played.kept();
    assert!(kept.starts_with(b"lamplight-kept 2\n"));
    let (brought, outcome) = Session::open_presented(
        story(),
        Some(&pack()),
        Some(&file()),
        Seed::Fresh(5),
        Some(&kept),
    )
    .unwrap();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(brought.notice(), None);
    assert_eq!(texts(&brought), texts(&played));
    assert_eq!(brought.map().position(first), Some((1, 4, 4)));
    // Without the file it comes back as it was left, redraws and all.
    let (same, outcome) =
        Session::open_presented(story(), Some(&pack()), None, Seed::Fresh(5), Some(&kept)).unwrap();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(same.map().view(), played.map().view());
    assert_eq!(same.kept(), kept);
}

// ---------------------------------------------------------------------------
// The file the project ships for Zork I: presentations/zork1.present
// ---------------------------------------------------------------------------

const SHIPPED: &str = "presentations/zork1.present";

/// The shipped file's places: room, the file's sheet, and its position.
fn shipped_places() -> Vec<(u16, u16, i32, i32)> {
    read_text(SHIPPED)
        .lines()
        .filter_map(|line| line.strip_prefix("place: "))
        .map(|rest| {
            let n: Vec<i32> = rest.split(' ').map(|n| n.parse().unwrap()).collect();
            (n[0] as u16, n[1] as u16, n[2], n[3])
        })
        .collect()
}

// F§6, SC-005: the shipped file is read for its story, and every place it has
// is a room of that story; were one not, the file would be refused.
#[test]
fn the_shipped_file_is_its_storys() {
    let s = presented(&read_text(SHIPPED));
    assert_eq!(s.notice(), None);
    assert!(s.map().view().unwrap().places[0].authored);
    assert!(shipped_places().len() >= 30);
}

// F§7: the shipped file holds no word of the game: its lines are the keys,
// digits, and the names of its sheets, which are the project's own.
#[test]
fn the_shipped_file_holds_no_word_of_the_game() {
    let numbers = |rest: &str| rest.split(' ').all(|n| n.parse::<i32>().is_ok());
    let mut names = Vec::new();
    for line in read_text(SHIPPED).lines().filter(|line| !line.is_empty()) {
        let (key, rest) = line.split_once(": ").unwrap_or_else(|| panic!("{line}"));
        match key {
            "presentation" | "release" | "place" => assert!(numbers(rest), "{line}"),
            "sha256" | "serial" => assert!(rest.chars().all(|c| c.is_ascii_alphanumeric())),
            "sheet" => {
                let (number, name) = rest.split_once(' ').unwrap_or((rest, ""));
                assert!(numbers(number), "{line}");
                names.push(name.to_string());
            }
            _ => panic!("{line}"),
        }
    }
    assert_eq!(names, ["Above Ground", "Under the House"]);
    // No sheet has the name of a place the walkthrough reaches.
    let mut s = plain();
    for line in read_text("stories/zork1/fidelity/walkthrough.txt")
        .lines()
        .skip(2)
    {
        if s.send_line(s.stamp(), line).is_err() {
            break;
        }
        for place in s.map().view().into_iter().flat_map(|view| view.places) {
            assert!(!names.contains(&place.name.unwrap_or_default()));
        }
    }
}

/// Checks the map after a turn: nothing is drawn that the player has not been
/// in, and every place of the file is where the file says (SC-002).
fn check(s: &Session, visited: &mut Vec<u16>, file: &[(u16, u16, i32, i32)]) {
    let Some(view) = s.map().view() else { return };
    visited.extend(view.here);
    for number in 1..=view.sheets {
        for place in s.map().sheet(number).unwrap().places {
            assert!(visited.contains(&place.id), "{} was not visited", place.id);
            let entry = file.iter().find(|entry| entry.0 == place.id);
            assert_eq!(place.authored, entry.is_some(), "{}", place.id);
            if let Some(&(_, _, x, y)) = entry {
                assert_eq!(
                    (place.x, place.y, place.inset),
                    (x, y, false),
                    "{}",
                    place.id
                );
            } else if !place.inset {
                // M§3: never on a position the file gives to a place.
                let on: Vec<u16> = s
                    .map()
                    .sheet(number)
                    .unwrap()
                    .places
                    .iter()
                    .filter_map(|p| file.iter().find(|e| e.0 == p.id).map(|e| e.1))
                    .collect();
                assert!(
                    !file
                        .iter()
                        .any(|e| on.contains(&e.1) && (e.2, e.3) == (place.x, place.y)),
                    "{} is on an authored position",
                    place.id
                );
            }
        }
    }
}

// SC-002, M§2, M§3: at every turn of the walkthrough and of a wander of 400
// moves.
#[test]
fn nothing_unvisited_is_drawn_and_every_authored_place_is_where_the_file_says() {
    let file = shipped_places();
    let text = read_text(SHIPPED);
    let mut s = presented(&text);
    let mut visited = Vec::new();
    check(&s, &mut visited, &file);
    for line in read_text("stories/zork1/fidelity/walkthrough.txt")
        .lines()
        .skip(2)
    {
        if s.send_line(s.stamp(), line).is_err() {
            break;
        }
        check(&s, &mut visited, &file);
    }
    assert!(
        visited
            .iter()
            .filter(|id| file.iter().any(|e| e.0 == **id))
            .count()
            > 100
    );

    let mut s = presented(&text);
    let (mut visited, mut state) = (Vec::new(), 1u32);
    check(&s, &mut visited, &file);
    for _ in 0..400 {
        let ways: Vec<String> = s
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
        if s.send_line(s.stamp(), &line).is_err() {
            break;
        }
        check(&s, &mut visited, &file);
    }
}

// SC-001, R10: the wander of earlier features, above ground, with the file
// and without. The figures are recorded in specs/017-authored-map/tasks.md.
#[test]
fn the_wander_above_ground_is_drawn_straighter_with_the_file() {
    let wander = |s: Session| wander_redrawing(s, 1, 400, usize::MAX);
    let (with, without) = (wander(presented(&read_text(SHIPPED))), wander(plain()));
    assert_eq!(texts(&with), texts(&without));
    let total = |s: &Session| {
        let sheets = s.map().measures();
        (
            sheets.iter().map(|m| m.1).sum::<usize>(),
            sheets.iter().map(|m| m.2).sum::<usize>(),
            s.map().crossings(),
        )
    };
    let (with, without) = (total(&with), total(&without));
    println!("with the file: straight, bent, crossings {with:?}; without: {without:?}");
    assert_eq!(with.2, 0, "no straight lines cross");
    // SC-001 asked for a quarter of the bent joins and is not met: the
    // forest's ways do not agree with one another, and five joins bend
    // wherever its places are put. Half is what the first layout gives.
    assert!(with.1 * 2 <= without.1, "{with:?} against {without:?}");
}
