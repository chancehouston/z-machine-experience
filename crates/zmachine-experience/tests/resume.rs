//! A game kept and brought back is the game that was left
//! (specs/011-resume-game/contracts/kept-game.md, part L, cited L§).

mod common;

use common::{
    asker, cat, faulter, forever, key_asker, looper, print, print_num, quitter, read, read_text,
    say, story3,
};
use zmachine_experience::{
    ASK_START_OVER, NOTE_NOT_BROUGHT_BACK, NOTE_NOT_KEPT, Outcome, Place, Seed, Session, Source,
    Waiting,
};
use zmachine_testkit::op;

const FRESH: Seed = Seed::Fresh(77);

fn open(story: &[u8], kept: Option<&[u8]>) -> (Session, Outcome) {
    Session::open_kept(story.to_vec(), None, FRESH, kept).expect("the story starts")
}

/// The session a kept game gives, which must be brought back.
fn back(story: &[u8], kept: &[u8]) -> Session {
    let (session, outcome) = open(story, Some(kept));
    assert_eq!(outcome, Outcome::BroughtBack);
    session
}

/// Two sessions show the same game: every turn and the whole snapshot.
fn assert_same(left: &Session, right: &Session, when: &str) {
    assert_eq!(left.turns(), right.turns(), "{when}");
    assert_eq!(left.snapshot_json(0), right.snapshot_json(0), "{when}");
    assert_eq!(left.waiting(), right.waiting(), "{when}");
    assert_eq!(left.announcement(), right.announcement(), "{when}");
}

/// A toy of three rooms: east from the first is decided by a draw, and south
/// from the second leads back.
fn coin_toy() -> Vec<u8> {
    use zmachine_testkit::toy::*;
    let mut toy = Toy::new(3);
    toy.uncertain(0, EAST, COIN)
        .uncertain(0, NORTH, Toy::room(1) as u8);
    toy.way(1, SOUTH, 0).way(1, EAST, 2).way(2, WEST, 1);
    toy.build()
}

/// Reads a command and prints three random numbers, for ever. (The shared
/// `roller` is good for one command only: it stores its draws in the global
/// a version 3 status line reads the place from, so its second read faults.)
fn roller() -> Vec<u8> {
    story3(|read| {
        let mut code = cat(&[print(">"), read()]);
        for _ in 0..3 {
            code.extend(cat(&[
                op::var(op::RANDOM, &[op::Arg::L(1000)]),
                op::store(17),
                print_num(op::Arg::V(17)),
                print(" "),
            ]));
        }
        forever(code)
    })
}

/// Reads a command, prints a random number, and restarts itself.
fn restarter() -> Vec<u8> {
    story3(|read| {
        cat(&[
            print("Again.\n\n>"),
            read(),
            op::var(op::RANDOM, &[op::Arg::L(1000)]),
            op::store(17),
            print_num(op::Arg::V(17)),
            print("\n"),
            read(),
            op::op0(op::RESTART),
        ])
    })
}

// L§1, FR-006: with nothing kept, a new game.
#[test]
fn with_nothing_kept_a_new_game_starts() {
    let (session, outcome) = open(&roller(), None);
    assert_eq!(outcome, Outcome::New);
    assert_eq!(session.turns().len(), 1);
    assert_eq!(session.notice(), None);
}

// L§6, FR-025: the kept game holds the player's inputs, in order.
#[test]
fn the_kept_game_holds_the_inputs_in_order() {
    let story = coin_toy();
    let (mut s, _) = open(&story, None);
    let none = String::from_utf8(s.kept()).unwrap();
    assert!(none.contains("\nchance fresh 77\ninputs 0\n"), "{none}");
    say(&mut s, "look");
    s.tap(s.stamp(), Place::North).expect("north is offered");
    say(&mut s, "");
    let kept = String::from_utf8(s.kept()).unwrap();
    assert!(kept.starts_with("lamplight-kept 1\nstory "), "{kept}");
    assert!(kept.contains("\ninputs 3\n"), "{kept}");
    let lines: Vec<&str> = kept.lines().skip(5).take(3).collect();
    assert_eq!(lines, ["typed look", "tap north", "typed "]);
    // It holds no word of the game: only the inputs and digests.
    for turn in s.turns() {
        for word in turn.text.split_whitespace().filter(|w| w.len() > 5) {
            assert!(!kept.contains(word), "{word}");
        }
    }
}

// L§2, L§5, FR-007, FR-008: brought back after any turn, it is the same game,
// and goes on the same. The story draws three numbers at every command.
#[test]
fn a_game_of_chance_brought_back_at_any_turn_goes_on_the_same() {
    let story = roller();
    let lines = ["a", "b", "", "look", "e", "f"];
    let (mut never_closed, _) = open(&story, None);
    for at in 0..lines.len() {
        let mut brought = back(&story, &never_closed.kept());
        assert_same(&never_closed, &brought, &format!("after {at}"));
        // Brought back a second time, from itself.
        let mut twice = back(&story, &brought.kept());
        assert_same(&never_closed, &twice, &format!("twice after {at}"));
        for line in &lines[at..] {
            say(&mut brought, line);
            say(&mut twice, line);
        }
        say(&mut never_closed, lines[at]);
        let mut whole = never_closed.clone();
        for line in &lines[at + 1..] {
            say(&mut whole, line);
        }
        assert_same(&whole, &brought, &format!("played on from {at}"));
        assert_same(&whole, &twice, &format!("played on twice from {at}"));
    }
    // The draws differ from turn to turn, so the comparison is of something.
    let texts: Vec<&str> = never_closed.turns()[1..]
        .iter()
        .map(|t| t.text.as_str())
        .collect();
    assert_ne!(texts[0], texts[1]);
}

// L§5, L§3: the ways out, settled by look-ahead, are the same after coming
// back; and a tap is kept as a tap.
#[test]
fn the_ways_out_and_a_tap_come_back() {
    let story = coin_toy();
    let (mut never_closed, _) = open(&story, None);
    for step in 0..6 {
        let brought = back(&story, &never_closed.kept());
        assert_same(&never_closed, &brought, &format!("step {step}"));
        let place = if step % 2 == 0 {
            Place::North
        } else {
            Place::South
        };
        never_closed
            .tap(never_closed.stamp(), place)
            .expect("the way is offered");
    }
    assert_eq!(never_closed.turns()[1].source, Some(Source::Tap));
    let brought = back(&story, &never_closed.kept());
    assert_eq!(brought.turns()[1].source, Some(Source::Tap));
}

// FR-004: a game waiting for an answer comes back waiting for it.
#[test]
fn a_game_asking_a_question_comes_back_asking_it() {
    let story = asker();
    let (mut s, _) = open(&story, None);
    say(&mut s, "look");
    assert_eq!(s.waiting(), Waiting::Answer);
    let mut brought = back(&story, &s.kept());
    assert_same(&s, &brought, "asking");
    assert_eq!(brought.start_over(), Some(ASK_START_OVER));
    say(&mut s, "yes");
    say(&mut brought, "yes");
    assert_same(&s, &brought, "answered");
}

// FR-004: a game waiting for a key comes back waiting for a key.
#[test]
fn a_game_waiting_for_a_key_comes_back_waiting_for_it() {
    let story = key_asker();
    let (mut s, _) = open(&story, None);
    say(&mut s, "look");
    let mut brought = back(&story, &s.kept());
    assert_same(&s, &brought, "at the key");
    say(&mut s, "x");
    say(&mut brought, "x");
    assert_same(&s, &brought, "after the key");
    assert!(
        brought.turns()[2].text.contains("120"),
        "{:?}",
        brought.turns()[2]
    );
}

// Edge case: a game that ended comes back ended.
#[test]
fn an_ended_game_comes_back_ended() {
    let story = quitter();
    let (mut s, _) = open(&story, None);
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Ended);
    let mut brought = back(&story, &s.kept());
    assert_same(&s, &brought, "ended");
    assert_eq!(brought.message(), Some("The game has ended."));
    assert!(brought.send_line(brought.stamp(), "more").is_err());
    // L§10: nothing would be given up, so nothing is asked.
    assert_eq!(brought.start_over(), None);
}

// Edge case: after the game's own restart, what comes back is that game.
#[test]
fn the_games_own_restart_is_followed() {
    let story = restarter();
    let (mut s, _) = open(&story, None);
    for line in ["one", "two", "three"] {
        say(&mut s, line);
    }
    assert!(s.turns()[2].text.contains("Again."), "{:?}", s.turns()[2]);
    let mut brought = back(&story, &s.kept());
    assert_same(&s, &brought, "after a restart");
    say(&mut s, "four");
    say(&mut brought, "four");
    assert_same(&s, &brought, "and on");
}

// L§6, R6: a command that brought a fault is not kept.
#[test]
fn a_fault_is_not_kept() {
    let story = faulter();
    let (mut s, _) = open(&story, None);
    let before = s.kept();
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Fault);
    assert_eq!(s.turns().len(), 2);
    assert_eq!(s.kept(), before);
    let brought = back(&story, &before);
    assert_eq!(brought.waiting(), Waiting::Command);
    assert_eq!(brought.turns().len(), 1);

    // The same for a game given up as not responding.
    let story = looper();
    let (mut s, _) = open(&story, None);
    s.set_instruction_ceiling(2_000_000);
    let before = s.kept();
    say(&mut s, "go");
    assert_eq!(s.waiting(), Waiting::Fault);
    assert_eq!(s.kept(), before);
}

// L§4, FR-009: bringing back gives the game the kept inputs and nothing else.
#[test]
fn bringing_back_gives_only_the_kept_inputs() {
    let story = roller();
    let (mut s, _) = open(&story, None);
    for line in ["a", "b", "c"] {
        say(&mut s, line);
    }
    let brought = back(&story, &s.kept());
    let commands: Vec<Option<&str>> = brought
        .turns()
        .iter()
        .map(|t| t.command.as_deref())
        .collect();
    assert_eq!(commands, [None, Some("a"), Some("b"), Some("c")]);
}

// FR-011: what is kept after any turn is a whole game, of that turn.
#[test]
fn what_is_kept_after_each_turn_is_whole() {
    let story = roller();
    let (mut s, _) = open(&story, None);
    for turn in 1..=20 {
        say(&mut s, "roll");
        let brought = back(&story, &s.kept());
        assert_eq!(brought.turns().len(), turn + 1);
    }
}

// L§11: a new game is kept as a new game, with its new chance.
#[test]
fn a_new_game_is_kept_with_no_inputs() {
    let story = roller();
    let (mut s, _) = open(&story, None);
    say(&mut s, "a");
    s.new_game(Seed::Fixed(5));
    let kept = String::from_utf8(s.kept()).unwrap();
    assert!(kept.contains("\nchance fixed 5\ninputs 0\n"), "{kept}");
    let (brought, outcome) = open(&story, Some(kept.as_bytes()));
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(brought.turns(), s.turns());
}

// ---------------------------------------------------------------------------
// A kept game that cannot be brought back (User Story 5)
// ---------------------------------------------------------------------------

/// A kept game of twenty turns of the roller.
fn twenty() -> (Vec<u8>, Vec<u8>) {
    let story = roller();
    let (mut s, _) = open(&story, None);
    for _ in 0..20 {
        say(&mut s, "roll");
    }
    let kept = s.kept();
    (story, kept)
}

fn assert_could_not(story: &[u8], kept: &[u8], why: &str) {
    let (session, outcome) = open(story, Some(kept));
    assert_eq!(outcome, Outcome::CouldNot, "{why}");
    assert_eq!(session.turns().len(), 1, "{why}");
    assert_eq!(session.waiting(), Waiting::Command, "{why}");
    assert_eq!(session.notice(), Some(NOTE_NOT_BROUGHT_BACK), "{why}");
}

// L§2, L§12, FR-018, FR-029, SC-006: damaged in more than a thousand ways,
// a kept game gives a new game and the notice, and never a fault.
#[test]
fn a_damaged_kept_game_gives_a_new_game() {
    let (story, kept) = twenty();
    let mut ways = 0;
    assert_could_not(&story, &[], "empty");
    for length in 0..kept.len() {
        assert_could_not(&story, &kept[..length], &format!("cut at {length}"));
        ways += 1;
    }
    for at in 0..kept.len() {
        for change in [1u8, 0x20, 0x80] {
            let mut bad = kept.clone();
            bad[at] ^= change;
            assert_could_not(&story, &bad, &format!("byte {at} xor {change}"));
            ways += 1;
        }
    }
    assert!(ways >= 1000, "{ways}");
}

// FR-017: another story's kept game, and another form.
#[test]
fn another_storys_kept_game_is_not_brought_back() {
    let (story, kept) = twenty();
    assert_could_not(&asker(), &kept, "another story");
    back(&story, &kept);
    let text = String::from_utf8(kept).unwrap();
    let other_form = text.replace("lamplight-kept 1", "lamplight-kept 2");
    assert_could_not(&story, other_form.as_bytes(), "another form");
}

/// A kept game with `change` made to its lines and its check made again, so
/// that it is well formed and only what it says is at fault.
fn reworked(kept: &[u8], change: impl Fn(&str) -> String) -> Vec<u8> {
    let text = String::from_utf8(kept.to_vec()).unwrap();
    let body = change(&text[..text.rfind("check ").unwrap()]);
    let check = zmachine_introspect::pack::sha256_hex(body.as_bytes());
    format!("{body}check {check}\n").into_bytes()
}

// R3: a well-formed kept game whose inputs do not give its transcript.
#[test]
fn a_kept_game_that_plays_differently_is_not_brought_back() {
    let (story, kept) = twenty();
    // One input fewer, and one more, than the transcript was made from.
    let fewer = reworked(&kept, |body| {
        body.replace("inputs 20", "inputs 19")
            .replacen("typed roll\n", "", 1)
    });
    assert_could_not(&story, &fewer, "one input fewer");
    let more = reworked(&kept, |body| {
        format!("{}typed roll\n", body.replace("inputs 20", "inputs 21"))
    });
    assert_could_not(&story, &more, "one input more");
    // Another chance: the same inputs, other draws.
    let chance = reworked(&kept, |body| {
        body.replace("chance fresh 77", "chance fresh 78")
    });
    assert_could_not(&story, &chance, "another chance");
    // The reworking itself is sound: unchanged, it is brought back.
    back(&story, &reworked(&kept, |body| body.to_string()));
}

// L§2, L§12: inputs that end the game early, fault it, or never stop.
#[test]
fn a_kept_game_that_cannot_be_played_is_not_brought_back() {
    let add = |kept: &[u8], lines: usize| {
        reworked(kept, |body| {
            format!(
                "{}{}",
                body.replace("inputs 0", &format!("inputs {lines}")),
                "typed go\n".repeat(lines)
            )
        })
    };
    for (story, lines, why) in [
        (quitter(), 2, "an input after the end"),
        (faulter(), 1, "an input that faults"),
    ] {
        let (s, _) = open(&story, None);
        assert_could_not(&story, &add(&s.kept(), lines), why);
    }
    let story = looper();
    let (s, _) = open(&story, None);
    let kept = add(&s.kept(), 1);
    let (session, outcome) =
        Session::open_kept_within(story, None, FRESH, Some(&kept), 2_000_000).unwrap();
    assert_eq!(outcome, Outcome::CouldNot);
    assert_eq!(session.waiting(), Waiting::Command);
}

// L§7, L§9, US5 scenario 4: the notice is shown once and spoken first.
#[test]
fn the_notice_is_given_once() {
    let story = asker();
    let (mut s, outcome) = open(&story, Some(b"not a kept game"));
    assert_eq!(outcome, Outcome::CouldNot);
    let json = s.snapshot_json(0);
    assert!(
        json.contains(&format!(
            r#","notice":"{NOTE_NOT_BROUGHT_BACK}","start_over":"#
        )),
        "{json}"
    );
    assert_eq!(
        s.announcement(),
        Some(format!("{NOTE_NOT_BROUGHT_BACK}\nReady.").as_str())
    );
    // The new game is kept like any other, and comes back with no notice.
    let brought = back(&story, &s.kept());
    assert_eq!(brought.notice(), None);
    assert_eq!(brought.announcement(), Some("Ready."));
    say(&mut s, "look");
    assert_eq!(s.notice(), None);
    assert!(
        s.snapshot_json(0)
            .contains(r#","notice":null,"start_over":"#)
    );
}

// FR-019: a game pack, or none, does not stop a game coming back.
#[test]
fn a_changed_pack_does_not_stop_a_game_coming_back() {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let (mut s, _) = Session::open_kept(story.clone(), Some(&pack), Seed::Fixed(11), None).unwrap();
    for line in ["open mailbox", "north"] {
        say(&mut s, line);
    }
    let kept = s.kept();
    for other in [None, Some("pack 1\n"), Some(pack.as_str())] {
        let (brought, outcome) =
            Session::open_kept(story.clone(), other, FRESH, Some(&kept)).unwrap();
        assert_eq!(outcome, Outcome::BroughtBack, "{other:?}");
        assert_eq!(brought.turns(), s.turns());
    }
}

// L§8, FR-013: that the game is not being kept is said once in a game.
#[test]
fn that_the_place_is_not_kept_is_said_once() {
    let (mut s, _) = open(&asker(), None);
    assert_eq!(s.notice(), None);
    s.keep_failed();
    assert_eq!(s.notice(), Some(NOTE_NOT_KEPT));
    // It is shown, and what is spoken for the turn is not said again.
    assert_eq!(s.announcement(), Some("Ready."));
    say(&mut s, "look");
    assert_eq!(s.notice(), None);
    s.keep_failed();
    assert_eq!(s.notice(), None);
    s.new_game(FRESH);
    s.keep_failed();
    assert_eq!(s.notice(), Some(NOTE_NOT_KEPT));
}

// L§10: the question before starting over, only where something is given up.
#[test]
fn starting_over_asks_only_while_a_game_is_in_progress() {
    let (s, _) = open(&asker(), None);
    assert_eq!(s.start_over(), Some(ASK_START_OVER));
    assert!(s.snapshot_json(0).ends_with(&format!(
        r#","notice":null,"start_over":"{ASK_START_OVER}"}}"#
    )));
    let (mut s, _) = open(&faulter(), None);
    say(&mut s, "go");
    assert_eq!(s.start_over(), None);
    assert!(
        s.snapshot_json(0)
            .ends_with(r#","notice":null,"start_over":null}"#)
    );
}

// ---------------------------------------------------------------------------
// Zork I (SC-002, FR-028)
// ---------------------------------------------------------------------------

/// Plays Zork I's walkthrough with its pack; at each command `every` apart
/// brings the game back and checks it against the session never closed, then
/// gives both the next command and checks again.
fn zork_walkthrough_brought_back(every: usize) {
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let lines: Vec<&str> = script.lines().skip(2).collect();
    let (mut never_closed, _) =
        Session::open_kept(story.clone(), Some(&pack), Seed::Fixed(11), None).unwrap();
    let mut checked = 0;
    for (index, line) in lines.iter().enumerate() {
        let brought = (index % every == 0).then(|| {
            let (brought, outcome) = Session::open_kept(
                story.clone(),
                Some(&pack),
                FRESH,
                Some(&never_closed.kept()),
            )
            .unwrap();
            assert_eq!(outcome, Outcome::BroughtBack, "before line {index}");
            assert_same(&never_closed, &brought, &format!("before line {index}"));
            brought
        });
        if never_closed.send_line(never_closed.stamp(), line).is_err() {
            break;
        }
        if let Some(mut brought) = brought {
            say(&mut brought, line);
            assert_same(
                &never_closed,
                &brought,
                &format!("after line {index}: {line}"),
            );
            checked += 1;
        }
    }
    assert!(never_closed.turns().len() > 300);
    assert!(checked >= 300 / every, "{checked}");
    // The game as the walkthrough leaves it comes back too.
    let (last, outcome) =
        Session::open_kept(story, Some(&pack), FRESH, Some(&never_closed.kept())).unwrap();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_same(&never_closed, &last, "at the end");
}

#[test]
fn zork_brought_back_along_its_walkthrough_is_the_same_game() {
    zork_walkthrough_brought_back(16);
}

#[test]
#[ignore = "slow in a debug build; run with --release -- --ignored"]
fn zork_brought_back_after_every_command_is_the_same_game() {
    zork_walkthrough_brought_back(1);
}
