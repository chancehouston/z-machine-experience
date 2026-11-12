//! SC-007: "A set of at least twenty deliberately malformed or misbehaving story
//! files each produce a diagnostic and none takes the host down." FR-002, FR-018,
//! FR-021.
//!
//! Each case is a story file built to go wrong in one particular way. `zm` must
//! exit with status 2 (refused at load) or 1 (faulted while running), say why on
//! standard error, and never crash.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object, Story, Story5};

struct Case {
    name: &'static str,
    image: Vec<u8>,
    /// The exit status expected: 2 refused, 1 faulted.
    status: i32,
    /// Words the diagnostic must contain.
    says: &'static str,
}

fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

/// A story whose main code is `code`.
fn story(code: Vec<u8>) -> Vec<u8> {
    let mut story = Story::new();
    story.main(&code);
    story.build()
}

fn valid() -> Vec<u8> {
    story(op::op0(op::QUIT))
}

fn with_byte(at: usize, value: u8) -> Vec<u8> {
    let mut image = valid();
    image[at] = value;
    image
}

/// A valid story with the header word at `offset` pointing outside the file.
fn with_address_outside(offset: usize) -> Vec<u8> {
    let mut image = valid();
    image[offset] = 0xFF;
    image[offset + 1] = 0xFE;
    image
}

fn refused() -> Vec<Case> {
    let case = |name, image, says| Case {
        name,
        image,
        status: 2,
        says,
    };
    vec![
        case("an empty file", Vec::new(), "too short"),
        case("one byte", vec![3], "too short"),
        case(
            "a header cut one byte short",
            valid()[..63].to_vec(),
            "too short",
        ),
        case("version 0", with_byte(0, 0), "version 0"),
        case("version 1", with_byte(0, 1), "version 1"),
        case("version 2", with_byte(0, 2), "version 2"),
        case("version 4", with_byte(0, 4), "version 4"),
        case("version 7", with_byte(0, 7), "version 7"),
        case("version 6", with_byte(0, 6), "version 6"),
        case("version 9", with_byte(0, 9), "version 9"),
        case(
            "high memory base outside the file",
            with_address_outside(0x04),
            "HighMemoryBase",
        ),
        case(
            "initial program counter outside the file",
            with_address_outside(0x06),
            "InitialProgramCounter",
        ),
        case(
            "dictionary outside the file",
            with_address_outside(0x08),
            "Dictionary",
        ),
        case(
            "object table outside the file",
            with_address_outside(0x0A),
            "ObjectTable",
        ),
        case(
            "globals outside the file",
            with_address_outside(0x0C),
            "Globals",
        ),
        case(
            "static memory base outside the file",
            with_address_outside(0x0E),
            "StaticMemoryBase",
        ),
        case(
            "abbreviations outside the file",
            with_address_outside(0x18),
            "Abbreviations",
        ),
        case(
            "static memory base inside the header",
            {
                let mut image = valid();
                image[0x0E] = 0;
                image[0x0F] = 0x20;
                image
            },
            "StaticMemoryBase",
        ),
        case(
            "a text file",
            b"This is a plain text file. It is long enough to hold a header but is not a story."
                .to_vec(),
            "version",
        ),
    ]
}

fn faulting() -> Vec<Case> {
    let case = |name, image, says| Case {
        name,
        image,
        status: 1,
        says,
    };
    let quit = || op::op0(op::QUIT);
    let call = |routine: u16| cat(&[op::var(op::CALL, &[L(routine)]), op::store(16)]);

    let recursion = {
        let mut s = Story::new();
        let at = (s.next_code_address() / 2) as u16;
        s.routine(
            &[],
            &cat(&[
                cat(&[op::var(op::CALL, &[L(at)]), op::store(0)]),
                op::op0(op::RTRUE),
            ]),
        );
        s.main(&cat(&[call(at), quit()]));
        s.build()
    };
    let too_many_locals = {
        let mut s = Story::new();
        let bogus = s.raw_string(&[16, 0, 0]);
        s.main(&cat(&[call(bogus), quit()]));
        s.build()
    };
    let write_to_static = {
        let mut s = Story::new();
        let base = s.static_base();
        s.main(&cat(&[op::var(op::STOREB, &[L(base), S(0), S(1)]), quit()]));
        s.build()
    };
    let missing_property = {
        let mut s = Story::new();
        s.object(Object::new("thing"));
        s.main(&cat(&[op::var(op::PUT_PROP, &[S(1), S(5), S(1)]), quit()]));
        s.build()
    };
    let next_missing_property = {
        let mut s = Story::new();
        s.object(Object::new("thing"));
        s.main(&cat(&[
            op::op2(op::GET_NEXT_PROP, S(1), S(9)),
            op::store(16),
            quit(),
        ]));
        s.build()
    };
    let unterminated_text = {
        // A string with no end bit, placed last in the file.
        let mut s = Story::new();
        let at = s.next_code_address();
        let code = cat(&[op::op1(op::PRINT_PADDR, L(0)), quit()]);
        let string_at = (at as usize + code.len()).div_ceil(2) * 2;
        let code = cat(&[op::op1(op::PRINT_PADDR, L((string_at / 2) as u16)), quit()]);
        s.main(&code);
        let packed = s.raw_string(&[0x35, 0x51, 0x35, 0x51]);
        assert_eq!(packed as usize * 2, string_at);
        s.build()
    };
    let read_into_static = {
        let mut s = Story::new();
        let room = s.object(Object::new("Room"));
        s.global(0, room as u16);
        let mut parse = vec![0u8; 22];
        parse[0] = 5;
        let parse = s.scratch(&parse);
        let base = s.static_base();
        s.main(&cat(&[op::var(op::SREAD, &[L(base), L(parse)]), quit()]));
        s.build()
    };

    vec![
        case(
            "an opcode version 3 does not have",
            story(vec![0xBE]),
            "unknown instruction",
        ),
        case(
            "a version 5 extended opcode",
            story(vec![0xBE, 0x00, 0xFF]),
            "unknown instruction",
        ),
        case(
            "2OP number 0",
            story(vec![0x00, 0x00, 0x00]),
            "unknown instruction",
        ),
        case(
            "VAR number 31",
            story(vec![0xFF, 0xFF]),
            "unknown instruction",
        ),
        case(
            "division by zero",
            story(cat(&[op::op2(op::DIV, S(1), S(0)), op::store(16), quit()])),
            "division by zero",
        ),
        case(
            "remainder by zero",
            story(cat(&[op::op2(op::MOD, S(1), S(0)), op::store(16), quit()])),
            "division by zero",
        ),
        case(
            "popping an empty stack",
            story(cat(&[op::op0(op::POP), quit()])),
            "stack underflow",
        ),
        case(
            "an operand from an empty stack",
            story(cat(&[op::op2(op::ADD, V(0), S(1)), op::store(16), quit()])),
            "stack underflow",
        ),
        case(
            "returning from the main code",
            story(op::op0(op::RTRUE)),
            "stack underflow",
        ),
        case("runaway recursion", recursion, "call stack overflow"),
        case(
            "a local the main code does not have",
            story(cat(&[op::op1(op::INC, S(1)), quit()])),
            "variable does not exist",
        ),
        case(
            "a routine with sixteen locals",
            too_many_locals,
            "variable does not exist",
        ),
        case(
            "reading past the end of the file",
            story(cat(&[
                op::op2(op::LOADB, L(0xFFFF), S(0)),
                op::store(16),
                quit(),
            ])),
            "read outside",
        ),
        case(
            "writing to static memory",
            write_to_static,
            "write outside dynamic memory",
        ),
        case(
            "writing into the header's neighbour beyond dynamic memory",
            story(cat(&[
                op::var(op::STOREW, &[L(0xFFF0), S(0), S(1)]),
                quit(),
            ])),
            "write outside dynamic memory",
        ),
        case(
            "put_prop on a property the object lacks",
            missing_property,
            "property not present",
        ),
        case(
            "get_next_prop after a property the object lacks",
            next_missing_property,
            "property not present",
        ),
        case(
            "text with no end",
            unterminated_text,
            "text runs past the end",
        ),
        case(
            "running off the end of the code",
            story(op::op0(op::NOP)),
            "",
        ),
        case(
            "a jump far outside the file",
            story(op::op1(op::JUMP, L(0x7FFF))),
            "",
        ),
        case(
            "a call to an address outside the file",
            story(cat(&[call(0xFFFF), quit()])),
            "read outside",
        ),
        case(
            "a text buffer in static memory",
            read_into_static,
            "write outside dynamic memory",
        ),
        case(
            "an endless loop",
            story(cat(&[
                op::op1(op::INC, S(16)),
                op::op1(op::JUMP, L(-3i16 as u16)),
            ])),
            "instructions ran without a request for input",
        ),
    ]
}

// ---- Versions 5 and 8 (feature 002, SC-008) ----

/// A version 5 or 8 story whose main code is `code`.
fn story5(version: u8, code: Vec<u8>) -> Vec<u8> {
    let mut story = Story5::new(version);
    story.main(&code);
    story.build()
}

fn valid5(version: u8) -> Vec<u8> {
    story5(version, op::op0(op::QUIT))
}

fn cases5() -> Vec<Case> {
    let refused = |name, image, says| Case {
        name,
        image,
        status: 2,
        says,
    };
    let fault = |name, image, says| Case {
        name,
        image,
        status: 1,
        says,
    };
    let quit = || op::op0(op::QUIT);
    let to = |instruction: Vec<u8>, variable: u8| cat(&[instruction, op::store(variable)]);
    let outside = |version: u8, offset: usize| {
        let mut image = valid5(version);
        image[offset] = 0xFF;
        image[offset + 1] = 0xFE;
        image
    };
    let sized = |version: u8, len: usize| {
        let mut image = valid5(version);
        image.resize(len, 0);
        image
    };
    // A story built by `build`, which is given the address where static memory
    // starts: somewhere a story may read but not write.
    let with_static = |build: &dyn Fn(u16, u16, u16) -> Vec<u8>| {
        let mut story = Story5::new(5);
        story.word("look");
        let text = story.text_buffer(20, "");
        let parse = story.parse_buffer(4);
        let base = story.static_base();
        story.main(&cat(&[build(base, text, parse), quit()]));
        story.build()
    };
    let extension_table = |words: &[u16]| {
        let mut story = Story5::new(5);
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
        let table = story.scratch(&bytes);
        story.header_word(0x36, table);
        story.main(&quit());
        story.build()
    };
    let recursion = |version: u8| {
        let mut story = Story5::new(version);
        let at = (story.next_code_address() as usize).next_multiple_of(story.packed_multiplier());
        let packed = (at / story.packed_multiplier()) as u16;
        let routine = story.routine(
            0,
            &cat(&[op::op1(op::CALL_1N, L(packed)), op::op0(op::RTRUE)]),
        );
        assert_eq!(routine, packed);
        story.main(&cat(&[op::op1(op::CALL_1N, L(routine)), quit()]));
        story.build()
    };
    let sixteen_locals = {
        let mut story = Story5::new(5);
        let bogus = story.raw_string(&[16, 0, 0, 0]);
        story.main(&cat(&[op::op1(op::CALL_1N, L(bogus)), quit()]));
        story.build()
    };
    let stream3 = |table: u16| op::var(op::OUTPUT_STREAM, &[S(3), L(table)]);

    vec![
        // Refused at load (002 FR-002, FR-021).
        refused(
            "version 5 larger than 256K",
            sized(5, 256 * 1024 + 4),
            "larger than",
        ),
        refused(
            "version 8 larger than 512K",
            sized(8, 512 * 1024 + 8),
            "larger than",
        ),
        refused(
            "version 5 header cut short",
            valid5(5)[..63].to_vec(),
            "too short",
        ),
        refused(
            "version 5 high memory base outside the file",
            outside(5, 0x04),
            "HighMemoryBase",
        ),
        refused(
            "version 5 initial program counter outside the file",
            outside(5, 0x06),
            "InitialProgramCounter",
        ),
        refused(
            "version 5 dictionary outside the file",
            outside(5, 0x08),
            "Dictionary",
        ),
        refused(
            "version 5 object table outside the file",
            outside(5, 0x0A),
            "ObjectTable",
        ),
        refused(
            "version 5 globals outside the file",
            outside(5, 0x0C),
            "Globals",
        ),
        refused(
            "version 5 static memory base outside the file",
            outside(5, 0x0E),
            "StaticMemoryBase",
        ),
        refused(
            "version 5 abbreviations outside the file",
            outside(5, 0x18),
            "Abbreviations",
        ),
        refused(
            "version 8 dictionary outside the file",
            outside(8, 0x08),
            "Dictionary",
        ),
        refused(
            "version 8 static memory base outside the file",
            outside(8, 0x0E),
            "StaticMemoryBase",
        ),
        refused(
            "terminating characters table outside the file",
            outside(5, 0x2E),
            "TerminatingCharacters",
        ),
        refused(
            "alphabet table outside the file",
            outside(5, 0x34),
            "AlphabetTable",
        ),
        refused(
            "header extension table outside the file",
            outside(5, 0x36),
            "HeaderExtension",
        ),
        refused(
            "Unicode table outside the file",
            extension_table(&[3, 0, 0, 0xFFFE]),
            "UnicodeTable",
        ),
        // Faults while running (002 FR-018): the new fault kinds.
        fault(
            "throw to a frame that does not exist",
            story5(5, cat(&[op::op2(op::THROW, S(1), S(9)), quit()])),
            "throw to a call frame",
        ),
        fault(
            "throw to frame 65535",
            story5(5, cat(&[op::op2(op::THROW, S(1), L(0xFFFF)), quit()])),
            "throw to a call frame",
        ),
        fault(
            "seventeen nested memory streams",
            {
                let mut story = Story5::new(5);
                let table = story.reserve(8);
                let mut code = vec![];
                for _ in 0..17 {
                    code.extend(stream3(table));
                }
                story.main(&cat(&[code, quit()]));
                story.build()
            },
            "sixteen deep",
        ),
        // The old fault kinds, reached through version 5 instructions.
        fault(
            "a memory stream into static memory",
            with_static(&|base, _, _| {
                cat(&[
                    stream3(base),
                    op::var(op::PRINT_NUM, &[S(7)]),
                    op::var(op::OUTPUT_STREAM, &[L(0xFFFD)]),
                ])
            }),
            "write outside dynamic memory",
        ),
        // The buffer's second byte says how much text is already there; in static
        // memory it is whatever the dictionary holds, and reading that much text
        // runs off the end of this small file.
        fault(
            "aread with its text buffer in static memory",
            with_static(&|base, _, parse| to(op::var(op::AREAD, &[L(base), L(parse)]), 16)),
            "outside",
        ),
        fault(
            "aread with its parse buffer in static memory",
            with_static(&|base, text, _| to(op::var(op::AREAD, &[L(text), L(base)]), 16)),
            "write outside dynamic memory",
        ),
        fault(
            "tokenise with its parse buffer in static memory",
            with_static(&|base, text, _| op::var(op::TOKENISE, &[L(text), L(base)])),
            "write outside dynamic memory",
        ),
        fault(
            "copy_table into static memory",
            with_static(&|base, text, _| op::var(op::COPY_TABLE, &[L(text), L(base), S(4)])),
            "write outside dynamic memory",
        ),
        fault(
            "copy_table zeroing static memory",
            with_static(&|base, _, _| op::var(op::COPY_TABLE, &[L(base), S(0), S(4)])),
            "write outside dynamic memory",
        ),
        fault(
            "encode_text into static memory",
            with_static(&|base, text, _| op::var(op::ENCODE_TEXT, &[L(text), S(2), S(2), L(base)])),
            "write outside dynamic memory",
        ),
        fault(
            "get_cursor into static memory",
            with_static(&|base, _, _| op::var(op::GET_CURSOR, &[L(base)])),
            "write outside dynamic memory",
        ),
        fault(
            "storew into static memory in version 8",
            {
                let mut story = Story5::new(8);
                let base = story.static_base();
                story.main(&cat(&[op::var(op::STOREW, &[L(base), S(0), S(1)]), quit()]));
                story.build()
            },
            "write outside dynamic memory",
        ),
        fault(
            "print_table running past the end of the file",
            story5(
                5,
                cat(&[
                    op::var(op::PRINT_TABLE, &[L(0xFFF0), L(200), L(200)]),
                    quit(),
                ]),
            ),
            "read outside the story file",
        ),
        fault(
            "scan_table running past the end of the file",
            story5(
                5,
                cat(&[
                    to(
                        op::var(op::SCAN_TABLE, &[L(0x1234), L(0xFFF0), L(0x7FFF)]),
                        16,
                    ),
                    op::branch(true, 2),
                    quit(),
                ]),
            ),
            "read outside the story file",
        ),
        fault(
            "a routine the version 5 multiplier puts outside the file",
            story5(5, cat(&[op::op1(op::CALL_1N, L(0xFFFF)), quit()])),
            "read outside the story file",
        ),
        fault(
            "a routine the version 8 multiplier puts outside the file",
            story5(8, cat(&[op::op1(op::CALL_1N, L(0xFFFF)), quit()])),
            "read outside the story file",
        ),
        fault(
            "print_paddr of a string outside the file",
            story5(8, cat(&[op::op1(op::PRINT_PADDR, L(0xFFF0)), quit()])),
            "text runs past the end of the story file",
        ),
        fault(
            "a routine with sixteen locals",
            sixteen_locals,
            "variable does not exist",
        ),
        fault(
            "a local that the main code does not have",
            story5(5, cat(&[op::op2(op::STORE, S(3), S(1)), quit()])),
            "variable does not exist",
        ),
        fault(
            "endless recursion in version 5",
            recursion(5),
            "call stack overflow",
        ),
        fault(
            "endless recursion in version 8",
            recursion(8),
            "call stack overflow",
        ),
        fault(
            "division by zero in version 5",
            story5(5, cat(&[to(op::op2(op::DIV, S(1), S(0)), 16), quit()])),
            "division by zero",
        ),
        fault(
            "catch result thrown away and the stack emptied",
            story5(5, cat(&[op::op0(op::RET_POPPED), quit()])),
            "stack underflow",
        ),
        fault(
            "an extended instruction that only version 6 has",
            story5(5, cat(&[op::ext(5, &[S(1), S(1), S(1)]), quit()])),
            "unknown instruction",
        ),
        fault(
            "an extended instruction below 29 that no version defines",
            story5(5, cat(&[op::ext(14, &[S(1)]), quit()])),
            "unknown instruction",
        ),
        fault(
            "a two-operand instruction numbered 0",
            story5(8, vec![0x00, 0x01, 0x01, 0xBA]),
            "unknown instruction",
        ),
        // Guarantee 7: undo saved without end stays within its limit.
        fault(
            "saving undo positions for ever",
            story5(
                5,
                cat(&[
                    to(op::ext(op::SAVE_UNDO, &[]), 16),
                    op::op1(op::JUMP, L(0xFFFB)),
                ]),
            ),
            "instructions ran",
        ),
    ]
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-malformed-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn every_malformed_story_gets_a_diagnostic_and_none_crashes_the_host() {
    let cases: Vec<Case> = refused().into_iter().chain(faulting()).collect();
    assert!(
        cases.len() >= 20,
        "SC-007 asks for at least twenty; there are {}",
        cases.len()
    );
    check(&cases, "z3");
}

// 002 SC-008: at least thirty more for versions 5 and 8.
#[test]
fn every_malformed_version_5_or_8_story_gets_a_diagnostic_and_none_crashes_the_host() {
    let cases = cases5();
    assert!(
        cases.len() >= 30,
        "SC-008 asks for at least thirty; there are {}",
        cases.len()
    );
    let mut names: Vec<&str> = cases.iter().map(|c| c.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), cases.len(), "each case is named once");
    check(&cases, "z5");
}

/// Runs `zm` on each case and checks its exit status and diagnostic.
fn check(cases: &[Case], extension: &str) {
    let dir = scratch_dir(extension);
    for (n, case) in cases.iter().enumerate() {
        let path = dir.join(format!("case{n}.{extension}"));
        std::fs::write(&path, &case.image).unwrap();
        // One line of input is on offer, for the case that reads before faulting.
        let mut child = Command::new(env!("CARGO_BIN_EXE_zm"))
            .args(["--max-instructions", "100000", "--seed", "1"])
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("zm starts");
        // zm may exit before reading; a closed pipe is then not an error here.
        let _ = child.stdin.take().unwrap().write_all(b"look\nx\n");
        let out = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        let name = case.name;

        // A panic exits with 101 and a signal gives no code at all.
        assert_eq!(out.status.code(), Some(case.status), "{name}: {stderr}");
        assert!(!stderr.contains("panicked"), "{name}: {stderr}");
        assert!(
            stderr.starts_with("zm: "),
            "{name}: the diagnostic is zm's own: {stderr:?}"
        );
        assert!(
            stderr.contains(case.says),
            "{name}: expected {:?} in {stderr:?}",
            case.says
        );
        if case.status == 1 && !case.says.contains("instructions ran") {
            assert!(
                stderr.contains(" at 0x"),
                "{name}: a fault says where: {stderr:?}"
            );
        }
    }
}

// FR-018: "The host MUST remain running and able to load another game." One
// process loads every case in turn through the library, then plays a real game.
#[test]
fn one_host_survives_every_case_and_then_plays_a_real_game() {
    use zmachine_core::{Event, LoadOptions, Machine};
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut refused_count = 0;
    let mut faulted = 0;
    for case in refused().into_iter().chain(faulting()) {
        match Machine::load(case.image, options) {
            Err(_) => refused_count += 1,
            Ok(mut machine) => {
                for _ in 0..10 {
                    match machine.run(10_000).unwrap() {
                        Event::Fault(_) => {
                            faulted += 1;
                            break;
                        }
                        Event::LineWanted { .. } => machine.provide_line("x").unwrap(),
                        Event::Finished => break,
                        _ => {}
                    }
                }
            }
        }
    }
    assert_eq!(refused_count, refused().len());
    assert!(
        faulted >= faulting().len() - 1,
        "all but the endless loop fault: {faulted}"
    );

    let zork = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../stories/zork1/zork1.z3"
    ))
    .unwrap();
    let mut machine = Machine::load(zork, options).unwrap();
    assert!(matches!(
        machine.run(1_000_000),
        Ok(Event::LineWanted { .. })
    ));
    assert!(machine.take_output().contains("West of House"));
}
