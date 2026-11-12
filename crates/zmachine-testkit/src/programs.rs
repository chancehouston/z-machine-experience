//! Small ready-made programs used to compare interpreters.

use crate::op::{self, Arg};
use crate::{Object, Story, zencode};

/// How many results [`random_probe`] prints after each seeding.
pub const RANDOM_PROBE_COUNTS: [usize; 4] = [200, 20, 20, 20];

/// A program that turns transcription on and prints the results of `random 100`,
/// one per line: 200 with the generator as the interpreter seeded it, then 20 after
/// each of `random -5`, `random -1000` and `random 0` (§15 random).
///
/// Run under two interpreters with the same seed, the transcripts show whether
/// their generators agree.
pub fn random_probe() -> Vec<u8> {
    let mut story = Story::new();
    let mut code = Vec::new();

    // Set Flags 2 bit 0 (§11.1, word at 0x10): transcription on.
    code.extend(op::var(op::STOREW, &[Arg::S(0), Arg::S(8), Arg::S(1)]));

    let reseeds: [Option<u16>; 4] = [None, Some(-5i16 as u16), Some(-1000i16 as u16), Some(0)];
    for (i, (reseed, count)) in reseeds.iter().zip(RANDOM_PROBE_COUNTS).enumerate() {
        if let Some(value) = reseed {
            // random <value> -> (stack); the result, always 0, is thrown away.
            code.extend(op::var(op::RANDOM, &[Arg::L(*value)]));
            code.extend(op::store(0));
            code.extend(op::op0(op::POP));
        }
        // Each loop counts in its own global, which starts at 0.
        let counter = 16 + i as u8;
        let body_start = code.len();
        code.extend(op::var(op::RANDOM, &[Arg::S(100)]));
        code.extend(op::store(0));
        code.extend(op::var(op::PRINT_NUM, &[Arg::V(0)]));
        code.extend(op::op0(op::NEW_LINE));
        // inc_chk counter (count - 1): loop again unless the counter now exceeds it.
        code.extend(op::op2(
            op::INC_CHK,
            Arg::S(counter),
            Arg::S(count as u8 - 1),
        ));
        let after = code.len() + 2;
        let offset = body_start as i16 - after as i16 + 2;
        code.extend(op::branch_long(false, offset));
    }
    code.extend(op::op0(op::QUIT));

    story.main(&code);
    story.build()
}

/// One small program for comparing transcripts between interpreters.
pub struct TranscriptProbe {
    pub name: &'static str,
    /// What the probe is for.
    pub purpose: &'static str,
    pub image: Vec<u8>,
    /// The lines of input the program reads, in order. They do not include the
    /// transcript file name, which a host asks for separately.
    pub input: &'static [&'static str],
}

/// Programs that switch transcription on and off around printing and reading, to
/// pin down exactly what reaches a transcript (contracts/transcript-format.md).
///
/// Each was run under dfrotz 2.55; its transcripts are the fixtures in
/// `crates/zmachine-core/tests/fixtures/transcript/`.
pub fn transcript_probes() -> Vec<TranscriptProbe> {
    use Arg::*;
    let print = |s: &str| [op::op0(op::PRINT), zencode(s)].concat();
    // Transcription by writing Flags 2 directly, as Infocom's games do (§11.1).
    let on = || op::var(op::STOREW, &[S(0), S(8), S(1)]);
    let off = || op::var(op::STOREW, &[S(0), S(8), S(0)]);
    // Transcription through output_stream (§15).
    let stream = |n: i16| op::var(op::OUTPUT_STREAM, &[L(n as u16)]);
    let window = |n: u8| op::var(op::SET_WINDOW, &[S(n)]);

    let build = |name, purpose, input, code: &dyn Fn(Vec<u8>) -> Vec<Vec<u8>>| {
        let mut story = Story::new();
        let room = story.object(Object::new("Room"));
        story.global(0, room as u16);
        story.word("hello").word("there");
        let mut text = vec![0u8; 64];
        text[0] = 60;
        let text = story.scratch(&text);
        let mut parse = vec![0u8; 22];
        parse[0] = 5;
        let parse = story.scratch(&parse);
        let read = op::var(op::SREAD, &[L(text), L(parse)]);
        let mut main = code(read).concat();
        main.extend(op::op0(op::QUIT));
        story.main(&main);
        TranscriptProbe {
            name,
            purpose,
            image: story.build(),
            input,
        }
    };

    vec![
        build(
            "quit_mid_line",
            "a line left unfinished at quit is ended",
            &[],
            &|_| vec![on(), print("abc")],
        ),
        build(
            "quit_after_newline",
            "nothing is added after a finished line",
            &[],
            &|_| vec![on(), print("abc\n")],
        ),
        build(
            "quit_after_blank_line",
            "a blank line at the end is kept as it is",
            &[],
            &|_| vec![on(), print("abc\n\n")],
        ),
        build(
            "on_mid_line",
            "turning on after a space: the space belongs to the next word",
            &["Hello There"],
            &|read| vec![print("before "), on(), print("go>"), read],
        ),
        build(
            "stream_toggle",
            "output_stream 2 and -2 between words",
            &[],
            &|_| {
                vec![
                    stream(2),
                    print("one "),
                    stream(-2),
                    print("two "),
                    stream(2),
                    print("three"),
                ]
            },
        ),
        build(
            "off_mid_word",
            "a word is judged when it is complete, not when it starts",
            &[],
            &|_| vec![on(), print("abc"), off(), print("def")],
        ),
        build(
            "on_mid_word",
            "a word begun while off is recorded if on when complete",
            &[],
            &|_| {
                vec![
                    stream(2),
                    print("x "),
                    off(),
                    print("abc"),
                    on(),
                    print("def ghi"),
                ]
            },
        ),
        build(
            "screen_deselected",
            "the transcript does not depend on stream 1",
            &[],
            &|_| {
                vec![
                    on(),
                    print("x"),
                    stream(-1),
                    print("hidden"),
                    stream(1),
                    print("y"),
                ]
            },
        ),
        build(
            "upper_window",
            "upper-window text is not transcribed",
            &[],
            &|_| {
                vec![
                    on(),
                    print("x"),
                    op::var(op::SPLIT_WINDOW, &[S(1)]),
                    window(1),
                    print("UP"),
                    window(0),
                    print("y"),
                ]
            },
        ),
        build(
            "two_reads",
            "input is echoed as typed, after the prompt",
            &["Hello There", "  x  "],
            &|read| {
                vec![
                    on(),
                    print("a>"),
                    read.clone(),
                    print("b>"),
                    read,
                    print("end"),
                ]
            },
        ),
        build("hyphens", "a word also ends after a hyphen", &[], &|_| {
            vec![
                on(),
                print("well-"),
                off(),
                print("known "),
                on(),
                print("a--"),
                off(),
                print("b "),
                on(),
                print("end"),
            ]
        }),
        build(
            "newline_then_off",
            "a new-line completes the word before it",
            &[],
            &|_| {
                vec![
                    on(),
                    print("a\n"),
                    off(),
                    print("b\nc"),
                    on(),
                    print("d\n"),
                    print("e"),
                ]
            },
        ),
        build(
            "numbers_and_chars",
            "print_num and print_char join the same word",
            &[],
            &|_| {
                vec![
                    on(),
                    op::var(op::PRINT_NUM, &[S(12)]),
                    op::var(op::PRINT_CHAR, &[S(b'z')]),
                    off(),
                    op::var(op::PRINT_CHAR, &[S(b' ')]),
                    op::var(op::PRINT_NUM, &[S(7)]),
                    on(),
                    print(" end"),
                ]
            },
        ),
        build(
            "off_then_read",
            "input is echoed only while on",
            &["first", "second"],
            &|read| {
                vec![
                    on(),
                    print("a>"),
                    off(),
                    read.clone(),
                    on(),
                    print("b>"),
                    read,
                ]
            },
        ),
        build(
            "status_between",
            "show_status completes the word before it",
            &[],
            &|_| {
                vec![
                    on(),
                    print("ab"),
                    off(),
                    op::op0(op::SHOW_STATUS),
                    print("cd "),
                    on(),
                    print("ef"),
                ]
            },
        ),
    ]
}
