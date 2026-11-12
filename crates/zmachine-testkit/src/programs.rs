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

/// One small version 5 program for comparing transcripts between interpreters.
pub struct TranscriptProbe5 {
    pub name: &'static str,
    /// What the probe is for, with the rule of contracts/transcript-format.md.
    pub purpose: &'static str,
    pub image: Vec<u8>,
    /// The lines of input the program consumes, in order: a whole line for a read,
    /// and a line of one character for a keypress. They do not include the
    /// transcript file name.
    pub input: &'static [&'static str],
    /// How many times the program starts transcription. Frotz asks a version 5
    /// game for the file name each time and appends to the file; this interpreter
    /// asks once (§7.1.1.2). Probes that start it more than once read no input, so
    /// a script for Frotz is the name repeated.
    pub starts: usize,
}

/// Version 5 programs that pin down rules 13 to 21 of
/// specs/002-v5-v8/contracts/transcript-format.md.
///
/// Each was run under dfrotz 2.55; its transcripts are the fixtures in
/// `crates/zmachine-core/tests/fixtures/transcript5/`.
pub fn transcript5_probes() -> Vec<TranscriptProbe5> {
    use crate::Story5;
    use Arg::*;
    let print = |s: &str| [op::op0(op::PRINT), zencode(s)].concat();
    let on = || op::var(op::STOREW, &[S(0), S(8), S(1)]);
    let off = || op::var(op::STOREW, &[S(0), S(8), S(0)]);
    let stream = |n: i16| op::var(op::OUTPUT_STREAM, &[L(n as u16)]);
    let window = |n: u8| op::var(op::SET_WINDOW, &[S(n)]);
    let split = |n: u8| op::var(op::SPLIT_WINDOW, &[S(n)]);
    let cursor = |row: u8, column: u8| op::var(op::SET_CURSOR, &[S(row), S(column)]);
    let style = |n: u8| op::var(op::SET_TEXT_STYLE, &[S(n)]);
    let buffer = |n: u8| op::var(op::BUFFER_MODE, &[S(n)]);
    let erase = |n: i16| op::var(op::ERASE_WINDOW, &[L(n as u16)]);
    let key = || [op::var(op::READ_CHAR, &[S(1)]), op::store(16)].concat();
    let chr = |n: u8| op::var(op::PRINT_CHAR, &[S(n)]);

    /// What a probe's code may refer to.
    struct Parts {
        read: Vec<u8>,
        timed_read: Vec<u8>,
        timed_key: Vec<u8>,
        table: u16,
        text: u16,
    }

    let build = |name, purpose, preloaded: &str, input, code: &dyn Fn(&Parts) -> Vec<Vec<u8>>| {
        let mut story = Story5::new(5);
        story.word("hello").word("there");
        let text = story.text_buffer(60, preloaded);
        let parse = story.parse_buffer(5);
        let table = story.reserve(40);
        let tick = story.routine(0, &op::op0(op::RTRUE));
        let parts = Parts {
            read: [op::var(op::AREAD, &[L(text), L(parse)]), op::store(16)].concat(),
            timed_read: [op::var(op::AREAD, &[L(text), L(parse), S(50), L(tick)]), op::store(16)]
                .concat(),
            timed_key: [op::var(op::READ_CHAR, &[S(1), S(50), L(tick)]), op::store(16)].concat(),
            table,
            text,
        };
        let mut main = code(&parts).concat();
        main.extend(op::op0(op::QUIT));
        story.main(&main);
        let starts = match name {
            "buffer_off_toggle" | "buffer_on_toggle" | "buffer_off_then_on_mid_word" | "stream_2_mid_line" => 2,
            _ => 1,
        };
        TranscriptProbe5 { name, purpose, image: story.build(), input, starts }
    };

    vec![
        build("v5_quit_mid_line", "rule 11 holds in version 5: an unfinished last line is ended", "", &[], &|_| {
            vec![on(), print("abc")]
        }),
        build("v5_read_echo", "rules 4 and 8 hold in version 5: the typed line is written as typed", "", &["Hello There"], &|p| {
            vec![on(), print("go>"), p.read.clone(), print("after\n")]
        }),
        build("key_not_written", "rule 13: a keypress writes neither the key nor a line feed", "", &["x"], &|_| {
            vec![on(), print("press"), key(), print("done\n"), print("next\n")]
        }),
        build("two_keys_then_read", "rule 13: keys between prompts leave the line as it was", "", &["a", "b", "hello"], &|p| {
            vec![on(), print("k1 "), key(), print("k2 "), key(), print("\n>"), p.read.clone(), print("end")]
        }),
        build("memory_stream", "rule 14: text redirected into memory is not written; the word before is completed", "", &[], &|p| {
            vec![
                on(),
                print("be"),
                op::var(op::OUTPUT_STREAM, &[S(3), L(p.table)]),
                print("hidden words"),
                stream(-3),
                print("fore after"),
            ]
        }),
        build("upper_window_mid_word", "rule 15: upper-window text is not written and does not break a word", "", &[], &|_| {
            vec![on(), split(1), print("pend"), window(1), print("UPPER"), window(0), print("ing done")]
        }),
        build("screen_off", "rule 16: text printed with the screen stream off is written", "", &[], &|_| {
            vec![on(), print("a "), stream(-1), print("[quoted] "), stream(1), print("b")]
        }),
        build("quotation_box", "rule 16: a boxed quotation the veneer's way: bracketed copy to the transcript alone, box to the upper window", "", &["look"], &|p| {
            vec![
                on(),
                print("Before the box.\n"),
                stream(-1),
                print("[ "),
                print("A quotation"),
                print("]\n"),
                stream(1),
                split(4),
                window(1),
                cursor(2, 10),
                style(1),
                print(" A quotation "),
                style(0),
                window(0),
                split(1),
                print("\n>"),
                p.read.clone(),
                print("After.\n"),
            ]
        }),
        build("erase_windows", "rule 17: clearing a window writes nothing", "", &[], &|_| {
            vec![on(), print("one "), erase(0), print("two "), erase(-1), print("three\n"), erase(-2), print("four")]
        }),
        build("preloaded_input", "rule 18: pre-loaded input is written with what was typed", "abc", &["DEF"], &|p| {
            vec![
                on(),
                print("pre>abc"),
                p.read.clone(),
                [op::op2(op::LOADB, L(p.text), S(1)), op::store(17)].concat(),
                op::var(op::PRINT_NUM, &[V(17)]),
                print(" end\n"),
            ]
        }),
        build("buffer_off_toggle", "rule 19: with buffering off each character is judged as it is printed", "", &[], &|_| {
            vec![on(), buffer(0), print("abc"), off(), print("def"), on(), print("ghi jkl"), buffer(1), print(" mno\n")]
        }),
        build("buffer_on_toggle", "rule 12 holds in version 5: with buffering on a word is judged when complete", "", &[], &|_| {
            vec![on(), print("abc"), off(), print("def"), on(), print("ghi jkl\n")]
        }),
        build("buffer_off_then_on_mid_word", "rule 19: turning buffering back on mid-word", "", &[], &|_| {
            vec![on(), buffer(0), print("ab"), buffer(1), print("cd"), off(), print("ef gh"), on(), print(" ij\n")]
        }),
        build("non_ascii", "rule 20: characters outside ASCII are written as UTF-8", "", &[], &|_| {
            vec![on(), print("a"), chr(155), chr(163), chr(220), print("z "), op::ext(op::PRINT_UNICODE, &[L(0x00E9)]), print("\n")]
        }),
        build("timed_input", "rule 21: a timed request is recorded as an untimed one", "", &["k", "hello"], &|p| {
            vec![on(), print("key "), p.timed_key.clone(), print("line>"), p.timed_read.clone(), print("done\n")]
        }),
        build("styles_not_written", "styles leave no trace in a transcript", "", &[], &|_| {
            vec![on(), print("plain "), style(2), print("bold "), style(4), print("both"), style(0), print(" plain\n")]
        }),
        build("stream_2_mid_line", "output_stream 2 and -2 between words in version 5", "", &[], &|_| {
            vec![stream(2), print("one "), stream(-2), print("two "), stream(2), print("three")]
        }),
    ]
}

/// A version 5 program for comparing how interpreters take keys from a script
/// (specs/002-v5-v8/contracts/cli.md, "Keypresses").
///
/// It turns transcription on, reads twelve keys printing each code, reads a line
/// and prints the terminator and the line's length, then reads three more keys.
/// Keys are not written to a transcript, but the numbers are.
pub fn key_probe() -> Vec<u8> {
    use crate::Story5;
    use Arg::*;
    let print = |s: &str| [op::op0(op::PRINT), zencode(s)].concat();
    let mut story = Story5::new(5);
    story.word("hello").word("there");
    let text = story.text_buffer(60, "");
    let parse = story.parse_buffer(5);
    let key = || {
        [
            op::var(op::READ_CHAR, &[S(1)]),
            op::store(16),
            op::var(op::PRINT_NUM, &[V(16)]),
            print(" "),
        ]
        .concat()
    };
    let mut main = op::var(op::STOREW, &[S(0), S(8), S(1)]);
    main.extend(print("keys: "));
    for _ in 0..12 {
        main.extend(key());
    }
    main.extend(print("\n>"));
    main.extend([op::var(op::AREAD, &[L(text), L(parse)]), op::store(16)].concat());
    main.extend(op::var(op::PRINT_NUM, &[V(16)]));
    main.extend(print(" "));
    main.extend([op::op2(op::LOADB, L(text), S(1)), op::store(17)].concat());
    main.extend(op::var(op::PRINT_NUM, &[V(17)]));
    main.extend(print("\nmore: "));
    for _ in 0..3 {
        main.extend(key());
    }
    main.extend(print("\n"));
    main.extend(op::op0(op::QUIT));
    story.main(&main);
    story.build()
}

/// A version 5 program that draws on the screen, for checking the screen model
/// (specs/002-v5-v8/contracts/screen-model.md, test programs 1 to 3).
pub struct ScreenProgram {
    pub name: &'static str,
    pub purpose: &'static str,
    pub image: Vec<u8>,
    /// One line for each time the program waits: a whole line for a read, a line
    /// of one character for a keypress.
    pub input: &'static [&'static str],
}

/// The three screen programs that are also run under the reference interpreter.
/// They draw only within the first 40 columns and 8 rows, so the same program
/// serves a small canvas in unit tests and dfrotz's 80 by 24.
///
/// dfrotz's screens for them are in `crates/zmachine-core/tests/fixtures/screen/`.
pub fn screen_programs() -> Vec<ScreenProgram> {
    use crate::Story5;
    use Arg::*;
    let print = |s: &str| [op::op0(op::PRINT), zencode(s)].concat();
    let window = |n: u8| op::var(op::SET_WINDOW, &[S(n)]);
    let split = |n: u8| op::var(op::SPLIT_WINDOW, &[S(n)]);
    let cursor = |row: u8, column: u8| op::var(op::SET_CURSOR, &[S(row), S(column)]);
    let style = |n: u8| op::var(op::SET_TEXT_STYLE, &[S(n)]);
    let erase = |n: i16| op::var(op::ERASE_WINDOW, &[L(n as u16)]);
    let key = || [op::var(op::READ_CHAR, &[S(1)]), op::store(16)].concat();
    // A status line the way the Inform library draws one.
    let status = |place: &str, score: &str| {
        [
            split(1),
            window(1),
            cursor(1, 1),
            style(1),
            print(&" ".repeat(40)),
            cursor(1, 2),
            print(place),
            cursor(1, 22),
            print(score),
            style(0),
            window(0),
        ]
        .concat()
    };

    let build = |name, purpose, input, code: &dyn Fn(&[u8]) -> Vec<Vec<u8>>| {
        let mut story = Story5::new(5);
        story.word("look");
        let text = story.text_buffer(40, "");
        let parse = story.parse_buffer(5);
        let read = [op::var(op::AREAD, &[L(text), L(parse)]), op::store(16)].concat();
        let mut main = code(&read).concat();
        main.extend(op::op0(op::QUIT));
        story.main(&main);
        ScreenProgram { name, purpose, image: story.build(), input }
    };

    vec![
        build(
            "status_line",
            "1: a status line drawn the Inform library's way, redrawn after a move",
            &["north", "look"],
            &|read| {
                vec![
                    print("At the road.\n\n"),
                    status("End Of Road", "Score: 0  Moves: 0"),
                    print(">"),
                    read.to_vec(),
                    print("In the building.\n\n"),
                    status("Inside Building", "Score: 5  Moves: 1"),
                    print(">"),
                    read.to_vec(),
                ]
            },
        ),
        build(
            "quotation_box",
            "2: a boxed quotation the veneer's way: enlarge, draw, shrink to one row",
            &["look", "look"],
            &|read| {
                vec![
                    print("Some text first.\n\n"),
                    split(6),
                    window(1),
                    style(1),
                    cursor(3, 6),
                    print("                 "),
                    cursor(4, 6),
                    print("  Plugh, he said "),
                    cursor(5, 6),
                    print("                 "),
                    style(0),
                    window(0),
                    status("Y2", "Score: 0  Moves: 0"),
                    print(">"),
                    read.to_vec(),
                    print("After the box.\n\n"),
                    status("Y2", "Score: 0  Moves: 1"),
                    print(">"),
                    read.to_vec(),
                ]
            },
        ),
        build(
            "menu",
            "3: a full-screen menu: clear everything, split, draw, wait for a key",
            &["n", "q"],
            &|_| {
                vec![
                    print("Before the menu.\n"),
                    erase(-1),
                    split(8),
                    window(1),
                    cursor(1, 1),
                    style(1),
                    print(" Menu "),
                    style(0),
                    cursor(3, 3),
                    print("> Instructions"),
                    cursor(4, 5),
                    print("History"),
                    cursor(8, 1),
                    print("N = next  Q = quit"),
                    key(),
                    cursor(3, 3),
                    print(" "),
                    cursor(4, 3),
                    print(">"),
                    key(),
                    erase(-1),
                    print("Back in the game.\n"),
                ]
            },
        ),
    ]
}
