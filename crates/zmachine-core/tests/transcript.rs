//! The transcript stream, output stream 2 (§7.1.1.2, §7.4;
//! contracts/transcript-format.md).

mod common;

use common::*;
use zmachine_core::{Event, HostError, Machine};
use zmachine_testkit::Story;
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::programs::{TranscriptProbe, transcript_probes};

/// What happened in a run driven to its end.
struct Run {
    transcript: String,
    screen: String,
    /// How many times the game asked for a transcript.
    requests: usize,
}

/// Runs an image to the end, feeding `input` to reads and giving `answers` in turn
/// to transcript requests (accepting once they run out).
fn drive(image: Vec<u8>, input: &[&str], answers: &[bool]) -> Run {
    let mut m = Machine::load(image, OPTIONS).unwrap();
    let mut run = Run {
        transcript: String::new(),
        screen: String::new(),
        requests: 0,
    };
    let mut input = input.iter();
    loop {
        let event = m.run(BIG).unwrap();
        run.screen.push_str(&m.take_output());
        run.transcript.push_str(&m.take_transcript());
        match event {
            Event::Finished => return run,
            Event::TranscriptRequested => {
                let accept = answers.get(run.requests).copied().unwrap_or(true);
                run.requests += 1;
                m.transcript_result(accept).unwrap();
            }
            Event::LineWanted { .. } => match input.next() {
                Some(line) => m.provide_line(line).unwrap(),
                None => return run,
            },
            other => panic!("unexpected {other:?}"),
        }
    }
}

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/transcript/{name}.scr",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

// contracts/transcript-format.md: for each probe program the transcript equals,
// byte for byte, the one dfrotz 2.55 wrote (tests/fixtures/README.md). Between them
// the probes cover rules 1 to 9 of the contract and the way text is sent to the
// transcript a word at a time.
#[test]
fn every_probe_transcript_matches_the_reference() {
    let probes = transcript_probes();
    assert!(probes.len() >= 15);
    for TranscriptProbe {
        name,
        purpose,
        image,
        input,
    } in probes
    {
        let run = drive(image, input, &[]);
        assert_eq!(run.transcript, fixture(name), "{name}: {purpose}");
    }
}

fn program(code: Vec<Vec<u8>>) -> Vec<u8> {
    let mut story = Story::new();
    story.main(&cat(&[cat(&code), quit()]));
    story.build()
}

fn flags2_on() -> Vec<u8> {
    op::var(op::STOREW, &[S(0), S(8), S(1)])
}

fn stream(n: i16) -> Vec<u8> {
    op::var(op::OUTPUT_STREAM, &[L(n as u16)])
}

/// Prints Flags 2 bit 0 as "0" or "1".
fn print_bit() -> Vec<u8> {
    cat(&[
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        cat(&[op::op2(op::AND, V(0), S(1)), op::store(0)]),
        print_num(V(0)),
    ])
}

// §7.1.1.2: "Stream 2 is selected by setting bit 0 of Flags 2" or by output_stream.
// research.md R6: the host is asked "before the next output or input".
#[test]
fn the_host_is_asked_as_soon_as_the_bit_is_set() {
    for turn_on in [flags2_on(), stream(2)] {
        let image = program(vec![print("before"), turn_on, print(" after")]);
        let mut m = Machine::load(image, OPTIONS).unwrap();
        assert_eq!(m.run(BIG), Ok(Event::TranscriptRequested));
        assert_eq!(
            m.take_output(),
            "before",
            "nothing after the request has run yet"
        );
        assert_eq!(m.run(BIG), Err(HostError::AnswerOwed));
        m.transcript_result(true).unwrap();
        assert_eq!(m.run(BIG), Ok(Event::Finished));
        assert_eq!(m.take_output(), " after\n");
    }
}

// research.md R6: "If the host declines, the core clears the bit", so the game can
// see that transcription is off.
#[test]
fn declining_clears_the_bit_and_records_nothing() {
    let image = program(vec![flags2_on(), print("text "), print_bit()]);
    let run = drive(image, &[], &[false]);
    assert_eq!(run.requests, 1);
    assert_eq!(run.transcript, "");
    assert_eq!(run.screen, "text 0\n");
}

// §7.4: Flags 2 bit 0 tracks the state of stream 2, whichever way it was changed.
#[test]
fn flags_2_bit_0_tracks_the_stream() {
    let image = program(vec![
        print_bit(),
        stream(2),
        print_bit(),
        stream(-2),
        print_bit(),
        flags2_on(),
        print_bit(),
    ]);
    let run = drive(image, &[], &[]);
    assert_eq!(run.screen, "0101\n");
}

// contracts/transcript-format.md rule 10: the host is asked only the first time;
// turning transcription off and on again continues without a new request.
#[test]
fn an_accepted_transcript_is_not_asked_for_again() {
    let image = program(vec![
        stream(2),
        print("a "),
        stream(-2),
        print("b "),
        stream(2),
        print("c"),
    ]);
    let run = drive(image, &[], &[]);
    assert_eq!(run.requests, 1);
    assert_eq!(run.transcript, "a c\n");
}

// A declined request is not remembered: the next attempt asks again.
#[test]
fn a_declined_transcript_is_asked_for_again() {
    let image = program(vec![stream(2), print("a "), stream(2), print("b")]);
    let run = drive(image, &[], &[false, true]);
    assert_eq!(run.requests, 2);
    // The space after "a" was completed, and dropped, while transcription was off.
    assert_eq!(run.transcript, "b\n");
}

// The transcript is separate from the screen text: each gets what belongs to it.
#[test]
fn screen_and_transcript_are_independent() {
    let (mut story, text, parse) = story_with_buffers(20);
    story.main(&cat(&[
        print("hidden "),
        flags2_on(),
        print("shown>"),
        sread(text, parse),
        quit(),
    ]));
    let run = drive(story.build(), &["LOOK"], &[]);
    assert_eq!(
        run.screen, "hidden shown>\n",
        "the screen has no echo of the input"
    );
    assert_eq!(
        run.transcript, " shown>LOOK\n\n",
        "the transcript has it as typed"
    );
}

// contracts/transcript-format.md rule 6: if input ends while the game is waiting,
// the transcript ends with the prompt and no line feed.
#[test]
fn a_transcript_cut_off_at_a_prompt_ends_with_the_prompt() {
    let (mut story, text, parse) = story_with_buffers(20);
    story.main(&cat(&[
        flags2_on(),
        print("Hello.\n\n>"),
        sread(text, parse),
        quit(),
    ]));
    let run = drive(story.build(), &[], &[]);
    assert_eq!(run.transcript, "Hello.\n\n>");
}

// The input line is recorded whole, even when the game's buffer takes only part of
// it (contracts/transcript-format.md, "A limit of the reference").
#[test]
fn input_is_recorded_as_given_even_when_the_game_truncates_it() {
    let (mut story, text, parse) = story_with_buffers(4);
    story.main(&cat(&[flags2_on(), print(">"), sread(text, parse), quit()]));
    let run = drive(story.build(), &["Look Around"], &[]);
    assert_eq!(run.transcript, ">Look Around\n\n");
}

// §6.1.3: Flags 2 survives a restart, so a transcript in progress continues.
#[test]
fn a_transcript_continues_across_restart() {
    // First pass (Flags 2 bit 1 clear): start the transcript, set bit 1, restart.
    let first_pass = cat(&[
        op::var(op::STOREB, &[S(0), S(0x11), S(3)]),
        print("one\n"),
        op::op0(op::RESTART),
    ]);
    let image = program(vec![
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        skip_if(op::op2(op::TEST, V(0), S(2)), true, &first_pass),
        print("two\n"),
    ]);
    let run = drive(image, &[], &[]);
    assert_eq!(run.requests, 1);
    assert_eq!(run.transcript, "one\ntwo\n");
}

// Answering a request that was not made is refused.
#[test]
fn transcript_result_out_of_turn_is_refused() {
    let mut m = Machine::load(program(vec![print("x")]), OPTIONS).unwrap();
    assert_eq!(m.transcript_result(true), Err(HostError::NotWaitingForThis));
}

// research.md R1: Frotz ends an unfinished last line when a game quits, on the
// screen as in the transcript; a finished line is left alone.
#[test]
fn quit_ends_an_unfinished_line_on_the_screen_too() {
    assert_eq!(output_of(&print("abc")), "abc\n");
    assert_eq!(output_of(&print("abc\n")), "abc\n");
    assert_eq!(output_of(&[]), "");
}
