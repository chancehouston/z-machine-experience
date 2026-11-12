//! Helpers and small story files shared by the experience layer's tests.
#![allow(dead_code)]

use zmachine_experience::{Seed, Session};
use zmachine_testkit::op::{self, Arg, Arg::*};
use zmachine_testkit::{Story, Story5, zencode};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn read(relative: &str) -> Vec<u8> {
    std::fs::read(format!("{ROOT}/{relative}")).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

pub fn read_text(relative: &str) -> String {
    String::from_utf8(read(relative)).unwrap()
}

/// The game the demo bundles, as a session with a fixed seed.
pub fn game(seed: u32) -> Session {
    Session::new(read("stories/zork1/zork1.z3"), Seed::Fixed(seed)).expect("the game starts")
}

pub fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

pub fn print(text: &str) -> Vec<u8> {
    cat(&[op::op0(op::PRINT), zencode(text)])
}

pub fn print_num(value: Arg) -> Vec<u8> {
    op::var(op::PRINT_NUM, &[value])
}

pub fn quit() -> Vec<u8> {
    op::op0(op::QUIT)
}

/// `instruction` with branch data that skips `skipped` when the condition equals
/// `on_true`, then `skipped`.
pub fn skip_if(instruction: Vec<u8>, on_true: bool, skipped: &[u8]) -> Vec<u8> {
    cat(&[
        instruction,
        op::branch(on_true, skipped.len() as i16 + 2),
        skipped.to_vec(),
    ])
}

/// A jump back over `code` to its own start, placed after it.
pub fn forever(code: Vec<u8>) -> Vec<u8> {
    let back = -(code.len() as i16 + 3) + 2;
    cat(&[code, op::op1(op::JUMP, L(back as u16))])
}

/// A version 3 story with a text buffer and a parse buffer; `build` is given a
/// function that makes a read instruction, and returns the main code.
pub fn story3(build: impl FnOnce(&dyn Fn() -> Vec<u8>) -> Vec<u8>) -> Vec<u8> {
    let mut story = Story::new();
    story.word("look").word("yes");
    let mut text = vec![0u8; 64];
    text[0] = 60;
    let text = story.scratch(&text);
    let mut parse = vec![0u8; 22];
    parse[0] = 5;
    let parse = story.scratch(&parse);
    let read = move || op::var(op::SREAD, &[L(text), L(parse)]);
    story.main(&build(&read));
    story.build()
}

/// Asks for a command, then asks a question from a second read, for ever:
/// "> " read A; "Sure? >" read B; "Done." and round again.
pub fn asker() -> Vec<u8> {
    story3(|read| {
        forever(cat(&[
            print("Ready.\n\n>"),
            read(),
            print("Sure? >"),
            read(),
            print("Done.\n"),
        ]))
    })
}

/// Reads a command, then quits.
pub fn quitter() -> Vec<u8> {
    story3(|read| cat(&[print("One command.\n\n>"), read(), print("Bye."), quit()]))
}

/// Reads a command, then divides by zero.
pub fn faulter() -> Vec<u8> {
    story3(|read| {
        cat(&[
            print(">"),
            read(),
            print("About to fail. "),
            op::op2(op::DIV, S(1), S(0)),
            op::store(16),
            quit(),
        ])
    })
}

/// Reads a command, then loops without ever stopping.
pub fn looper() -> Vec<u8> {
    story3(|read| cat(&[print(">"), read(), forever(op::op0(op::NOP))]))
}

/// Reads a command; asks to save, to restore and to keep a transcript, saying
/// what came of each; then reads again, for ever.
pub fn requester() -> Vec<u8> {
    story3(|read| {
        forever(cat(&[
            print("\n>"),
            read(),
            skip_if(op::op0(op::SAVE), true, &print("Save failed. ")),
            skip_if(op::op0(op::RESTORE), true, &print("Restore failed. ")),
            // Flags 2 bit 0: transcription on (§11.1).
            op::var(op::STOREW, &[S(0), S(8), S(1)]),
            print("After script. "),
            op::var(op::STOREW, &[S(0), S(8), S(0)]),
        ]))
    })
}

/// Reads a command, then prints twelve random numbers and reads again.
pub fn roller() -> Vec<u8> {
    story3(|read| {
        let mut code = cat(&[print(">"), read()]);
        for _ in 0..12 {
            code.extend(cat(&[
                op::var(op::RANDOM, &[L(1000)]),
                op::store(16),
                print_num(V(16)),
                print(" "),
            ]));
        }
        forever(code)
    })
}

/// A version 5 story: reads a command, then one key, printing its code.
pub fn key_asker() -> Vec<u8> {
    let mut story = Story5::new(5);
    let text = story.text_buffer(40, "");
    let parse = story.parse_buffer(5);
    story.main(&forever(cat(&[
        print(">"),
        cat(&[op::var(op::AREAD, &[L(text), L(parse)]), op::store(16)]),
        print("Press a key."),
        cat(&[op::var(op::READ_CHAR, &[S(1)]), op::store(17)]),
        print_num(V(17)),
        print("\n"),
    ])));
    story.build()
}

pub fn session(story: Vec<u8>) -> Session {
    Session::new(story, Seed::Fixed(1)).expect("the story starts")
}

/// Sends a typed line against the current stamp.
pub fn say(session: &mut Session, line: &str) {
    session
        .send_line(session.stamp(), line)
        .unwrap_or_else(|e| panic!("{line:?} was refused: {e:?}"));
}
