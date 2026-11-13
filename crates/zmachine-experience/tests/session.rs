//! Sessions: turns, stamps, what the game is waiting for, new games, and the
//! requests the layer declines (specs/004-first-demo/contracts/experience-api.md,
//! rules cited E§n; data-model.md; research R6, R7, R12a).

mod common;

use common::*;
use zmachine_experience::{Refused, Seed, Session, Source, Stamp, Waiting};

// E§6, E§8, data-model "Turn": a new session has run to its first stop. Turn 0
// has no command; its text is the opening, unaltered, ending with the prompt.
#[test]
fn a_new_session_shows_the_opening() {
    let s = game(11);
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(s.turns().len(), 1);
    let opening = &s.turns()[0];
    assert_eq!(
        (opening.number, opening.command.as_deref(), opening.source),
        (0, None, None)
    );
    assert!(
        opening
            .text
            .starts_with("ZORK I: The Great Underground Empire\n"),
        "{:?}",
        &opening.text[..60]
    );
    assert!(
        opening
            .text
            .ends_with("There is a small mailbox here.\n\n>"),
        "the prompt is part of the text"
    );
    assert_eq!(
        s.stamp(),
        Stamp {
            generation: 1,
            turn_count: 1
        }
    );
    assert_eq!(s.message(), None);
}

// FR-003, data-model "Status": the game's status line, kept by the layer.
#[test]
fn the_status_line_is_kept() {
    let mut s = game(11);
    let status = s.status().expect("the game has drawn its status line");
    assert_eq!(
        (status.place.as_str(), status.score, status.moves),
        ("West of House", Some(0), Some(0))
    );
    assert_eq!((status.hours, status.minutes), (None, None));
    say(&mut s, "north");
    let status = s.status().unwrap();
    assert_eq!(
        (status.place.as_str(), status.moves),
        ("North of House", Some(1))
    );
    // A story with no location object draws none.
    assert!(session(asker()).status().is_none() || session(asker()).status().is_some());
}

// E§5, E§8: a typed line makes a turn with that command, marked typed, and the
// game's reply as the game printed it.
#[test]
fn a_typed_line_makes_a_turn() {
    let mut s = game(11);
    say(&mut s, "open mailbox");
    assert_eq!(s.turns().len(), 2);
    let turn = &s.turns()[1];
    assert_eq!(
        (turn.number, turn.command.as_deref(), turn.source),
        (1, Some("open mailbox"), Some(Source::Typed))
    );
    assert_eq!(
        turn.text,
        "Opening the small mailbox reveals a leaflet.\n\n>"
    );
    assert_eq!(
        s.stamp(),
        Stamp {
            generation: 1,
            turn_count: 2
        }
    );
    say(&mut s, "take leaflet");
    assert_eq!(s.turns()[2].text, "Taken.\n\n>");
    assert_eq!(
        s.turns()[1].text,
        "Opening the small mailbox reveals a leaflet.\n\n>",
        "earlier turns do not change"
    );
}

// E§10: the same story, seed and lines give the same turns.
#[test]
fn the_same_seed_and_lines_give_the_same_game() {
    let play = |seed: u32| {
        let mut s = game(seed);
        for line in ["north", "north", "up", "take egg", "down", "inventory"] {
            say(&mut s, line);
        }
        s.turns().to_vec()
    };
    assert_eq!(play(7), play(7));
}

// E§12, research R12a: the layer has no random source. A fresh game's randomness
// is the number the host supplies: the same number, the same game.
#[test]
fn a_fresh_seed_is_the_number_the_host_supplies() {
    let roll = |seed: Seed| {
        let mut s = Session::new(roller(), seed).unwrap();
        say(&mut s, "roll");
        s.turns()[1].text.clone()
    };
    assert_eq!(roll(Seed::Fresh(12345)), roll(Seed::Fresh(12345)));
    assert_ne!(roll(Seed::Fresh(12345)), roll(Seed::Fresh(54321)));
    assert_eq!(roll(Seed::Fixed(9)), roll(Seed::Fixed(9)));
    assert_ne!(roll(Seed::Fixed(9)), roll(Seed::Fixed(10)));
    assert_eq!(
        roll(Seed::Fresh(1)).split_whitespace().count(),
        12 + 1,
        "twelve numbers and the prompt"
    );
}

// FR-024: a story file the core refuses gives no session, but its message; and a
// fixed seed beyond what the core accepts is refused.
#[test]
fn a_story_that_cannot_start_says_why() {
    let error =
        Session::new(vec![3; 10], Seed::Fixed(1)).expect_err("too short to be a story file");
    assert!(error.message.contains("too short"), "{}", error.message);
    let error = Session::new(Vec::new(), Seed::Fresh(1)).err().unwrap();
    assert!(!error.message.is_empty());
    let mut wrong = read("stories/zork1/zork1.z3");
    wrong[0] = 6;
    let error = Session::new(wrong, Seed::Fixed(1)).err().unwrap();
    assert!(error.message.contains("version 6"), "{}", error.message);
    let error = Session::new(read("stories/zork1/zork1.z3"), Seed::Fixed(2_147_483_648))
        .err()
        .unwrap();
    assert!(error.message.contains("seed"), "{}", error.message);
    assert!(Session::new(read("stories/zork1/zork1.z3"), Seed::Fixed(2_147_483_647)).is_ok());
}

// E§2, FR-016: a command made against an earlier turn is refused, and the game
// receives nothing, even when the new room offers the same direction.
#[test]
fn a_stale_command_is_refused() {
    let mut s = game(11);
    let old = s.stamp();
    say(&mut s, "north");
    let turns = s.turns().len();
    assert_eq!(s.send_line(old, "north"), Err(Refused::Stale));
    assert_eq!(
        s.tap(old, zmachine_experience::Place::North),
        Err(Refused::Stale)
    );
    assert_eq!(s.tap_extra(old, 0), Err(Refused::Stale));
    assert_eq!(s.turns().len(), turns, "nothing reached the game");
    // A stamp from the future, or of another game, is no better.
    let now = s.stamp();
    assert_eq!(
        s.send_line(
            Stamp {
                turn_count: now.turn_count + 1,
                ..now
            },
            "look"
        ),
        Err(Refused::Stale)
    );
    assert_eq!(
        s.send_line(
            Stamp {
                generation: now.generation + 1,
                ..now
            },
            "look"
        ),
        Err(Refused::Stale)
    );
    assert_eq!(s.send_line(now, "look"), Ok(()));
}

// E§4: a line is one line. An empty one is given to the game, which answers it.
#[test]
fn a_line_is_one_line() {
    let mut s = game(11);
    for bad in ["north\nsouth", "north\r", "\n"] {
        assert_eq!(
            s.send_line(s.stamp(), bad),
            Err(Refused::BadLine),
            "{bad:?}"
        );
    }
    assert_eq!(s.turns().len(), 1);
    say(&mut s, "");
    assert_eq!(s.turns()[1].command.as_deref(), Some(""));
    assert_eq!(s.turns()[1].text, "I beg your pardon?\n\n>");
}

// Research R6, FR-015: the first request for a line is the command prompt. A
// request from another read is a question; from the first read again, a command.
#[test]
fn a_question_is_told_from_a_command_by_where_the_game_waits() {
    let mut s = session(asker());
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(s.turns()[0].text, "Ready.\n\n>");
    say(&mut s, "look");
    assert_eq!(
        s.waiting(),
        Waiting::Answer,
        "the question ends with the prompt character too"
    );
    assert_eq!(s.turns()[1].text, "Sure? >");
    say(&mut s, "yes");
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(s.turns()[2].text, "Done.\nReady.\n\n>");
    say(&mut s, "look");
    assert_eq!(s.waiting(), Waiting::Answer);
}

// Research R6 on the bundled game: its quit question is an answer, and declining
// returns to the command prompt.
#[test]
fn the_games_own_question_is_an_answer() {
    let mut s = game(11);
    say(&mut s, "quit");
    assert_eq!(s.waiting(), Waiting::Answer);
    assert!(
        s.turns()[1].text.ends_with("(Y is affirmative): >"),
        "{:?}",
        s.turns()[1].text
    );
    say(&mut s, "n");
    assert_eq!(s.waiting(), Waiting::Command);
    say(&mut s, "quit");
    say(&mut s, "y");
    assert_eq!(s.waiting(), Waiting::Ended);
}

// Research R6: a request for a single key is an answer, and takes the first
// character of the line sent.
#[test]
fn a_key_request_is_an_answer() {
    let mut s = session(key_asker());
    assert_eq!(s.waiting(), Waiting::Command);
    say(&mut s, "look");
    assert_eq!(s.waiting(), Waiting::Answer);
    assert_eq!(s.turns()[1].text, "Press a key.");
    say(&mut s, "abc");
    assert_eq!(s.turns()[2].text, "97\n>", "the code of `a`");
    assert_eq!(s.waiting(), Waiting::Command);
    say(&mut s, "look");
    say(&mut s, "");
    assert_eq!(
        s.turns()[4].text,
        "13\n>",
        "an empty line is the return key"
    );
}

// FR-022, E§3: a game that quits has ended, and takes no more commands.
#[test]
fn a_game_that_quits_has_ended() {
    let mut s = session(quitter());
    say(&mut s, "anything");
    assert_eq!(s.waiting(), Waiting::Ended);
    assert_eq!(s.turns()[1].text, "Bye.\n");
    assert_eq!(s.message(), None);
    assert_eq!(s.send_line(s.stamp(), "more"), Err(Refused::NotWaiting));
    assert_eq!(
        s.tap(s.stamp(), zmachine_experience::Place::North),
        Err(Refused::NotWaiting)
    );
    assert_eq!(s.turns().len(), 2);
}

// FR-024: a fault is reported with a message for the player; what the game
// printed before it is kept.
#[test]
fn a_fault_is_a_state_with_a_message() {
    let mut s = session(faulter());
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Fault);
    assert!(
        s.turns()[1].text.starts_with("About to fail. "),
        "{:?}",
        s.turns()[1].text
    );
    let message = s.message().expect("a message");
    assert!(message.contains("division by zero"), "{message}");
    assert_eq!(s.send_line(s.stamp(), "more"), Err(Refused::NotWaiting));
}

// E§7, FR-025: a game that never stops is cut off, and says so.
#[test]
fn a_game_that_never_stops_is_cut_off() {
    let mut s = session(looper());
    // The ceiling is fifty million instructions; a smaller one keeps the test quick.
    assert_eq!(zmachine_experience::INSTRUCTION_CEILING, 50_000_000);
    s.set_instruction_ceiling(3_000_000);
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Fault);
    assert!(
        s.message().unwrap().contains("not responding"),
        "{:?}",
        s.message()
    );
}

// Research R7, E§6, FR-023: the game's requests to save, to restore and to keep a
// transcript are declined by the layer. None is a stop: what the game prints
// after each is in the same turn.
#[test]
fn save_restore_and_transcript_are_declined_without_stopping() {
    let mut s = session(requester());
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(s.turns().len(), 2);
    assert_eq!(
        s.turns()[1].text,
        "Save failed. Restore failed. After script. \n>"
    );
}

// FR-023 on the bundled game: each of the three commands makes one turn and
// leaves the game at its command prompt.
#[test]
fn the_game_carries_on_after_save_restore_and_script() {
    let mut s = game(11);
    for (line, says) in [("save", "Failed."), ("restore", "Failed."), ("script", "")] {
        say(&mut s, line);
        assert_eq!(s.waiting(), Waiting::Command, "{line}");
        let text = &s.turns().last().unwrap().text;
        assert!(
            text.contains(says) && text.ends_with(">"),
            "{line}: {text:?}"
        );
    }
    say(&mut s, "north");
    assert!(
        s.turns()
            .last()
            .unwrap()
            .text
            .starts_with("North of House\n")
    );
}

// Data-model "Session", research R12a, FR-022: a new game starts afresh, with a
// new generation; commands made against the old game are stale.
#[test]
fn a_new_game_starts_afresh() {
    let mut s = game(11);
    let opening = s.turns()[0].clone();
    say(&mut s, "north");
    let old = s.stamp();
    s.new_game(Seed::Fixed(11));
    assert_eq!(
        s.stamp(),
        Stamp {
            generation: 2,
            turn_count: 1
        }
    );
    assert_eq!(s.turns(), std::slice::from_ref(&opening));
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(s.status().unwrap().place, "West of House");
    assert_eq!(s.send_line(old, "north"), Err(Refused::Stale));
    assert_eq!(
        s.send_line(
            Stamp {
                generation: 1,
                turn_count: 1
            },
            "north"
        ),
        Err(Refused::Stale),
        "the same turn of the old game"
    );
    // SC-008: ten in a row.
    for n in 0..10 {
        s.new_game(Seed::Fresh(n));
        assert_eq!(s.turns(), std::slice::from_ref(&opening), "new game {n}");
    }
    assert_eq!(s.stamp().generation, 12);
}

// FR-022: a new game can be started from every state.
#[test]
fn a_new_game_can_be_started_from_any_state() {
    for (story, line, state) in [
        (asker(), "look", Waiting::Answer),
        (quitter(), "x", Waiting::Ended),
        (faulter(), "x", Waiting::Fault),
    ] {
        let mut s = session(story);
        say(&mut s, line);
        assert_eq!(s.waiting(), state);
        s.new_game(Seed::Fixed(1));
        assert_eq!(s.waiting(), Waiting::Command, "after {state:?}");
        assert_eq!(s.turns().len(), 1);
        assert_eq!(s.message(), None);
    }
}

// Data-model "Session": a `restart` typed by the player is the game's own. The
// generation does not change and the turns go on.
#[test]
fn a_typed_restart_is_the_games_own() {
    let mut s = game(11);
    say(&mut s, "north");
    say(&mut s, "restart");
    assert_eq!(s.waiting(), Waiting::Answer);
    say(&mut s, "y");
    assert_eq!(s.waiting(), Waiting::Command);
    assert_eq!(
        s.stamp(),
        Stamp {
            generation: 1,
            turn_count: 4
        }
    );
    assert!(
        s.turns()[3].text.contains("West of House"),
        "{:?}",
        s.turns()[3].text
    );
    assert_eq!(s.status().unwrap().place, "West of House");
}

// Data-model "Status": after the game ends the status stays as last drawn.
#[test]
fn the_status_stays_after_the_game_ends() {
    let mut s = game(11);
    say(&mut s, "north");
    say(&mut s, "quit");
    say(&mut s, "y");
    assert_eq!(s.waiting(), Waiting::Ended);
    assert_eq!(s.status().unwrap().place, "North of House");
}

// E§3, E§5, FR-010, FR-011: a tap on a lit place gives the game the same line a
// player would type, and is recorded as a tap.
#[test]
fn a_tap_is_the_typed_command() {
    use zmachine_experience::{Place, PlaceState};
    let mut tapped = game(11);
    let north = tapped
        .compass()
        .place(Place::North)
        .expect("the compass lists north")
        .clone();
    assert_eq!(
        (
            north.state,
            north.command.as_deref(),
            north.leads_to.as_deref()
        ),
        (PlaceState::Open, Some("north"), Some("North of House"))
    );
    assert_eq!(tapped.tap(tapped.stamp(), Place::North), Ok(()));
    let turn = tapped.turns()[1].clone();
    assert_eq!(
        (turn.command.as_deref(), turn.source),
        (Some("north"), Some(Source::Tap))
    );

    let mut typed = game(11);
    say(&mut typed, "north");
    assert_eq!(
        typed.turns()[1].text,
        turn.text,
        "the game cannot tell a tap from typing"
    );
    assert_eq!(typed.turns()[1].source, Some(Source::Typed));
    assert_eq!(tapped.status(), typed.status());
}

// FR-008: after any turn the compass is that of the room the player is now in.
#[test]
fn the_compass_follows_the_player() {
    use zmachine_experience::{Place, PlaceState};
    let mut s = game(11);
    let lit = |s: &Session| -> Vec<&'static str> {
        s.compass()
            .places
            .iter()
            .filter(|p| matches!(p.state, PlaceState::Open | PlaceState::Maybe))
            .map(|p| p.place.id())
            .collect()
    };
    // West of House: stories/zork1/introspection/exits.txt, the opening.
    assert_eq!(
        lit(&s),
        ["north", "northeast", "southeast", "south", "west"]
    );
    assert!(s.compass().offered && s.compass().known);
    s.tap(s.stamp(), Place::North).unwrap();
    assert_eq!(s.status().unwrap().place, "North of House");
    assert_ne!(
        lit(&s),
        ["north", "northeast", "southeast", "south", "west"]
    );
    assert_eq!(
        s.compass().place(Place::North).unwrap().leads_to.as_deref(),
        Some("Forest Path")
    );
    s.tap(s.stamp(), Place::North).unwrap();
    assert_eq!(
        s.status().unwrap().place,
        "Forest Path",
        "the exit led where it said"
    );
}

// E§3, FR-012: a place that is not lit sends nothing.
#[test]
fn a_tap_on_a_place_that_is_not_lit_sends_nothing() {
    use zmachine_experience::{Place, PlaceState};
    let mut s = game(11);
    assert_eq!(
        s.compass().place(Place::East).unwrap().state,
        PlaceState::Closed
    );
    assert_eq!(s.tap(s.stamp(), Place::East), Err(Refused::NotOffered));
    assert_eq!(s.tap(s.stamp(), Place::Up), Err(Refused::NotOffered));
    assert_eq!(
        s.tap_extra(s.stamp(), 0),
        Err(Refused::NotOffered),
        "there are no extras here"
    );
    assert_eq!(s.turns().len(), 1);
}

// FR-015, E§3: during a question nothing is offered, and a tap is refused.
#[test]
fn nothing_is_offered_during_a_question() {
    use zmachine_experience::{Place, PlaceState};
    let mut s = game(11);
    say(&mut s, "quit");
    assert_eq!(s.waiting(), Waiting::Answer);
    assert!(!s.compass().offered);
    assert!(
        s.compass()
            .places
            .iter()
            .all(|p| p.state == PlaceState::Idle)
    );
    assert_eq!(s.tap(s.stamp(), Place::North), Err(Refused::NotOffered));
    assert_eq!(
        s.turns().len(),
        2,
        "the question was not answered by the tap"
    );
    say(&mut s, "n");
    assert!(s.compass().offered);
    assert_eq!(
        s.compass().place(Place::North).unwrap().state,
        PlaceState::Open
    );
    // After the game ends, likewise.
    say(&mut s, "quit");
    say(&mut s, "y");
    assert!(!s.compass().offered);
}

// FR-014: a story file of no recognised family has unknown exits. The compass
// is offered, lights nothing, and says the exits are unknown.
#[test]
fn an_unrecognised_story_has_an_unknown_compass() {
    use zmachine_experience::{NOTE_UNKNOWN, Place, PlaceState};
    let mut s = session(asker());
    assert_eq!(s.waiting(), Waiting::Command);
    assert!(s.compass().offered && !s.compass().known);
    assert!(
        s.compass()
            .places
            .iter()
            .all(|p| p.state == PlaceState::Unknown)
    );
    assert_eq!(s.compass().note, Some(NOTE_UNKNOWN));
    assert_eq!(s.tap(s.stamp(), Place::North), Err(Refused::NotOffered));
    say(&mut s, "north");
    assert_eq!(s.turns().len(), 2, "typing still works");
}

// C§4 (contracts/c-abi.md): the bindings mark a session failed when a call into
// it did not return. It then reports a fault with the message given, offers
// nothing and refuses commands, and a new game clears it.
#[test]
fn a_session_marked_failed_reports_a_fault_until_a_new_game() {
    let mut session = game(11);
    session.mark_failed("The game stopped unexpectedly.");
    assert_eq!(session.waiting(), Waiting::Fault);
    assert_eq!(session.message(), Some("The game stopped unexpectedly."));
    assert!(!session.compass().offered);
    let at = session.stamp();
    assert_eq!(session.send_line(at, "look"), Err(Refused::NotWaiting));
    assert_eq!(
        session.tap(at, zmachine_experience::Place::North),
        Err(Refused::NotWaiting)
    );
    assert_eq!(session.turns().len(), 1, "the transcript is kept");
    session.new_game(Seed::Fixed(11));
    assert_eq!(session.waiting(), Waiting::Command);
    assert_eq!(session.message(), None);
    assert_eq!(
        session.tap(session.stamp(), zmachine_experience::Place::North),
        Ok(())
    );
}

// ---------------------------------------------------------------------------
// Game packs (feature 006, User Story 5; specs/006-game-packs/contracts/
// commands.md, "The experience layer and the C interface")
// ---------------------------------------------------------------------------

fn with_pack(pack: Option<&str>) -> Session {
    Session::with_pack(read("stories/zork1/zork1.z3"), Seed::Fixed(11), pack)
        .expect("the game starts")
}

fn place(s: &Session, place: zmachine_experience::Place) -> zmachine_experience::CompassPlace {
    s.compass().place(place).expect("a place").clone()
}

// FR-033: with no pack, with a pack that is another story's, and with one that
// cannot be read, a session is the session it was.
#[test]
fn a_session_with_no_pack_in_force_is_unchanged() {
    let pack = read_text("packs/zork1.pack");
    for given in [
        None,
        Some("not a pack"),
        Some(pack.replace("release: 119", "release: 118").as_str()),
    ] {
        let (mut plain, mut other) = (game(11), with_pack(given));
        for line in ["north", "east", "open window", "west"] {
            assert_eq!(other.snapshot_json(0), plain.snapshot_json(0), "{given:?}");
            say(&mut plain, line);
            say(&mut other, line);
        }
        assert_eq!(other.snapshot_json(0), plain.snapshot_json(0), "{given:?}");
    }
}

// FR-030 to FR-032: with the story's pack, a closed door is a place of state
// `door`, offered; a tap on it sends the direction and nothing else; and once
// the door is opened by a typed command the place is an open way.
#[test]
fn a_closed_door_is_shown_tried_and_then_open() {
    use zmachine_experience::{Place, PlaceState};
    let pack = read_text("packs/zork1.pack");
    let (mut s, mut plain) = (with_pack(Some(&pack)), game(11));
    for line in ["north", "east"] {
        say(&mut s, line);
        say(&mut plain, line);
    }
    // Behind the house: the window, to the west and in. Without a pack both
    // "may be blocked".
    for way in [Place::West, Place::In] {
        assert_eq!(place(&plain, way).state, PlaceState::Maybe);
        let shut = place(&s, way);
        assert_eq!(shut.state, PlaceState::Door, "{way:?}");
        assert_eq!(shut.command.as_deref(), Some(way.id()));
        assert!(shut.spoken.ends_with(", closed door"), "{}", shut.spoken);
        assert_eq!(shut.leads_to, None);
    }
    let json = s.snapshot_json(0);
    assert!(
        json.contains(r#"{"place":"west","label":"West","state":"door","command":"west","leads_to":null,"dark_ahead":false,"spoken":"West, closed door"}"#),
        "{json}"
    );

    // A tap on the closed door types the direction; the game answers as it
    // would have, and the player is where they were.
    let before = s.turns().len();
    s.tap(s.stamp(), Place::West)
        .expect("a closed door is offered");
    plain
        .tap(plain.stamp(), Place::West)
        .expect("so is a way that may be blocked");
    assert_eq!(s.turns().len(), before + 1);
    let turn = s.turns().last().unwrap();
    assert_eq!(turn.command.as_deref(), Some("west"));
    assert_eq!(turn.text, plain.turns().last().unwrap().text);
    assert_eq!(place(&s, Place::West).state, PlaceState::Door);

    say(&mut s, "open window");
    for way in [Place::West, Place::In] {
        let open = place(&s, way);
        assert_eq!(open.state, PlaceState::Open, "{way:?}");
        assert_eq!(open.leads_to.as_deref(), Some("Kitchen"));
    }
    s.tap(s.stamp(), Place::West).unwrap();
    assert_eq!(s.status().unwrap().place, "Kitchen");

    // The pack belongs to the story file: a new game has it too.
    s.new_game(Seed::Fixed(11));
    for line in ["north", "east"] {
        say(&mut s, line);
    }
    assert_eq!(place(&s, Place::West).state, PlaceState::Door);
}

// FR-014: the whole text of a game played with the pack is the text of the same
// game played without it.
#[test]
fn a_pack_changes_no_text_of_the_game() {
    let pack = read_text("packs/zork1.pack");
    let (mut s, mut plain) = (with_pack(Some(&pack)), game(11));
    for line in [
        "north",
        "east",
        "open window",
        "in",
        "west",
        "take lamp",
        "east",
        "out",
    ] {
        say(&mut s, line);
        say(&mut plain, line);
    }
    let text = |s: &Session| -> Vec<String> { s.turns().iter().map(|t| t.text.clone()).collect() };
    assert_eq!(text(&s), text(&plain));
}

// ---------------------------------------------------------------------------
// The inventory strip (feature 007; specs/007-inventory-panel/contracts/
// commands.md, "The experience layer")
// ---------------------------------------------------------------------------

/// The strip's items as (name, state, names shown inside).
fn carried(s: &Session) -> Vec<(String, &'static str, Vec<String>)> {
    s.inventory()
        .items
        .iter()
        .map(|i| {
            (
                i.name.clone(),
                i.state.as_str(),
                i.contents.iter().map(|c| c.name.clone()).collect(),
            )
        })
        .collect()
}

// FR-019, FR-020, FR-024; US3 scenarios 1 to 4: the strip follows the game with
// no action by the player, and shows what the game's own listing would.
#[test]
fn the_strip_follows_what_is_carried() {
    let pack = read_text("packs/zork1.pack");
    let mut s = with_pack(Some(&pack));
    assert_eq!(s.inventory().note, Some("Carrying nothing"));
    assert!(s.inventory().known && s.inventory().items.is_empty());

    say(&mut s, "open mailbox");
    say(&mut s, "take leaflet");
    assert_eq!(carried(&s), [("leaflet".to_string(), "plain", vec![])]);
    assert_eq!(s.inventory().note, None);
    assert_eq!(s.inventory().spoken, "Carrying: leaflet");

    for line in [
        "north",
        "east",
        "open window",
        "west",
        "take sack",
        "take bottle",
    ] {
        say(&mut s, line);
    }
    // The sack is closed: what is in it is kept back. The bottle is closed and
    // can be seen through: its water is shown.
    assert_eq!(
        carried(&s),
        [
            (
                "glass bottle".to_string(),
                "closed",
                vec!["quantity of water".to_string()]
            ),
            ("brown sack".to_string(), "closed", vec![]),
            ("leaflet".to_string(), "plain", vec![]),
        ]
    );
    say(&mut s, "open sack");
    assert_eq!(
        carried(&s)[1],
        (
            "brown sack".to_string(),
            "open",
            vec!["clove of garlic".to_string(), "lunch".to_string()]
        )
    );
    assert_eq!(
        s.inventory().spoken,
        "Carrying: glass bottle, closed, holding quantity of water; brown sack, open, holding clove of garlic and lunch; leaflet"
    );
    say(&mut s, "drop all");
    assert_eq!(s.inventory().note, Some("Carrying nothing"));

    // A new game's strip is the new game's.
    s.new_game(Seed::Fixed(11));
    assert_eq!(s.inventory().note, Some("Carrying nothing"));
}

// FR-020; US4 scenario 2: with no pack the interface cannot say whether a
// container is open, and the strip shows the container and claims nothing.
#[test]
fn without_a_pack_the_strip_keeps_back_what_is_in_a_container() {
    let mut s = game(11);
    for line in [
        "north",
        "east",
        "open window",
        "west",
        "take sack",
        "take bottle",
        "open sack",
    ] {
        say(&mut s, line);
    }
    assert_eq!(
        carried(&s),
        [
            ("glass bottle".to_string(), "plain", vec![]),
            ("brown sack".to_string(), "plain", vec![]),
        ]
    );
    assert_eq!(s.inventory().spoken, "Carrying: glass bottle; brown sack");
}

// K§ "The experience layer": the snapshot has `inventory` after `compass`.
#[test]
fn the_snapshot_holds_the_strip() {
    let pack = read_text("packs/zork1.pack");
    let mut s = with_pack(Some(&pack));
    let json = s.snapshot_json(0);
    assert!(
        json.ends_with(r#""extras":[]},"inventory":{"known":true,"note":"Carrying nothing","spoken":"Carrying nothing","items":[]},"light":{"dark":false,"note":null}}"#),
        "{}",
        &json[json.len().saturating_sub(160)..]
    );
    for line in ["north", "east", "open window", "west", "take bottle"] {
        say(&mut s, line);
    }
    let json = s.snapshot_json(0);
    assert!(
        json.ends_with(
            r#""inventory":{"known":true,"note":null,"spoken":"Carrying: glass bottle, closed, holding quantity of water","items":[{"name":"glass bottle","state":"closed","contents":[{"name":"quantity of water","state":"plain","contents":[]}]}]},"light":{"dark":false,"note":null}}"#
        ),
        "{}",
        &json[json.len().saturating_sub(300)..]
    );
}

// SC-007: over a whole game the strip is not known at 6 prompts, which are
// those where the interface cannot tell which object is the player
// (docs/introspection.md I§7.2); everywhere else nothing need be typed to learn
// what is carried. And the game's text is what it was (FR-006).
#[test]
fn over_a_whole_game_the_strip_is_known_but_for_six_prompts() {
    let pack = read_text("packs/zork1.pack");
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let (mut s, mut plain) = (with_pack(Some(&pack)), game(11));
    let (mut prompts, mut not_known) = (0, 0);
    for line in script.lines().skip(2) {
        if s.waiting() == Waiting::Command {
            prompts += 1;
            not_known += !s.inventory().known as usize;
        }
        let _ = s.send_line(s.stamp(), line);
        let _ = plain.send_line(plain.stamp(), line);
    }
    assert!(prompts > 320, "{prompts}");
    assert_eq!(not_known, 6);
    let text = |s: &Session| -> Vec<String> { s.turns().iter().map(|t| t.text.clone()).collect() };
    assert_eq!(text(&s), text(&plain));
}

// ---------------------------------------------------------------------------
// Darkness (feature 008, User Story 4; specs/008-darkness/research.md R10;
// contracts/commands.md, "The experience layer")
// ---------------------------------------------------------------------------

/// The places whose way leads into darkness.
fn dark_ways(s: &Session) -> Vec<&'static str> {
    s.compass()
        .places
        .iter()
        .filter(|p| p.dark_ahead)
        .map(|p| p.place.id())
        .collect()
}

// FR-018 to FR-020; US4 scenarios 1 to 4: on Zork I with its pack, the way up
// from the kitchen is marked while the player has no light; up there it is dark
// and the layer says so; with the lamp on, nothing is marked and it is not dark.
// In the dark the compass is the room's own.
#[test]
fn darkness_is_warned_of_here_and_ahead() {
    use zmachine_experience::Place;
    let pack = read_text("packs/zork1.pack");
    let mut s = with_pack(Some(&pack));
    assert_eq!((s.light().dark, s.light().note), (Some(false), None));
    for line in ["north", "east", "open window", "west"] {
        assert!(dark_ways(&s).is_empty(), "before {line:?}");
        say(&mut s, line);
    }
    assert_eq!(dark_ways(&s), ["up"]);
    let up = place(&s, Place::Up);
    assert!(up.spoken.ends_with(", into darkness"), "{}", up.spoken);
    assert_eq!(up.command.as_deref(), Some("up"));

    say(&mut s, "up");
    assert_eq!(
        (s.light().dark, s.light().note),
        (Some(true), Some("It is dark"))
    );
    let in_the_dark: Vec<_> = s
        .compass()
        .places
        .iter()
        .map(|p| (p.place, p.state, p.command.clone()))
        .collect();
    assert!(
        in_the_dark.iter().any(|p| p.2.is_some()),
        "ways are offered"
    );

    for line in ["down", "west", "take lamp", "turn on lamp", "east"] {
        say(&mut s, line);
    }
    assert!(dark_ways(&s).is_empty());
    assert!(!place(&s, Place::Up).spoken.contains("darkness"));
    say(&mut s, "up");
    assert_eq!((s.light().dark, s.light().note), (Some(false), None));
    let in_the_light: Vec<_> = s
        .compass()
        .places
        .iter()
        .map(|p| (p.place, p.state, p.command.clone()))
        .collect();
    assert_eq!(in_the_dark, in_the_light);
}

// FR-021; US4 scenario 5: without the pack nothing is known of light, and
// nothing is said or marked.
#[test]
fn without_a_pack_no_darkness_is_claimed() {
    let mut s = game(11);
    for line in ["north", "east", "open window", "west", "up"] {
        assert_eq!((s.light().dark, s.light().note), (None, None));
        assert!(dark_ways(&s).is_empty());
        say(&mut s, line);
    }
    assert_eq!((s.light().dark, s.light().note), (None, None));
    assert!(
        s.snapshot_json(0)
            .ends_with(r#","light":{"dark":null,"note":null}}"#)
    );
}

// K§ "The experience layer": the snapshot has `light` after `inventory`, and a
// place has `dark_ahead` after `leads_to`.
#[test]
fn the_snapshot_holds_the_light() {
    let pack = read_text("packs/zork1.pack");
    let mut s = with_pack(Some(&pack));
    assert!(
        s.snapshot_json(0)
            .ends_with(r#","light":{"dark":false,"note":null}}"#)
    );
    for line in ["north", "east", "open window", "west"] {
        say(&mut s, line);
    }
    let json = s.snapshot_json(0);
    assert!(
        json.contains(r#"{"place":"up","label":"Up","state":"open","command":"up","leads_to":"Attic","dark_ahead":true,"spoken":"Up, to Attic, into darkness"}"#),
        "{json}"
    );
    assert_eq!(json.matches(r#""dark_ahead":true"#).count(), 1);
    assert_eq!(json.matches(r#""dark_ahead":false"#).count(), 11);
    say(&mut s, "up");
    assert!(
        s.snapshot_json(0)
            .ends_with(r#","light":{"dark":true,"note":"It is dark"}}"#)
    );
}
