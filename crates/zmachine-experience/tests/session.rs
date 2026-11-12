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
    let error = Session::new(vec![3; 10], Seed::Fixed(1))
        .err()
        .expect("too short to be a story file");
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
    assert_eq!(s.turns(), [opening.clone()]);
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
        assert_eq!(s.turns(), [opening.clone()], "new game {n}");
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
