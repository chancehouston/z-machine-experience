//! SC-004, FR-027: every one of the instructions the Standard defines for version 3
//! is covered by at least one test citing the relevant section of the Standard.
//! Constitution Principle V and FR-006: the core has no dependencies, no `unsafe`
//! and no input or output of its own.
//!
//! These tests read the crate's own source. The table below names, for each
//! instruction, a test that exercises it; the check confirms the test exists, that
//! the comment above it cites a section ("§"), and that its body really mentions
//! the instruction.

use std::collections::BTreeSet;
use std::path::Path;

const CRATE: &str = env!("CARGO_MANIFEST_DIR");

fn source(relative: &str) -> String {
    let path = Path::new(CRATE).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// (instruction, file, test function, text the test's body must contain)
#[rustfmt::skip]
const COVERAGE: &[(&str, &str, &str, &str)] = &[
    // 2OP (§14)
    ("je",            "src/exec_tests.rs",     "conditional_branches_follow_their_sense", "op::JE"),
    ("jl",            "src/exec_tests.rs",     "conditional_branches_follow_their_sense", "op::JL"),
    ("jg",            "src/exec_tests.rs",     "conditional_branches_follow_their_sense", "op::JG"),
    ("dec_chk",       "src/exec_tests.rs",     "inc_chk_and_dec_chk", "op::DEC_CHK"),
    ("inc_chk",       "src/exec_tests.rs",     "inc_chk_and_dec_chk", "op::INC_CHK"),
    ("jin",           "src/objects_tests.rs",  "tree_instructions", "op::JIN"),
    ("test",          "src/exec_tests.rs",     "conditional_branches_follow_their_sense", "op::TEST"),
    ("or",            "src/ops_math_tests.rs", "bitwise_operations", "or("),
    ("and",           "src/ops_math_tests.rs", "bitwise_operations", "and("),
    ("test_attr",     "src/objects_tests.rs",  "tree_instructions", "op::TEST_ATTR"),
    ("set_attr",      "src/objects_tests.rs",  "object_changing_instructions", "op::SET_ATTR"),
    ("clear_attr",    "src/objects_tests.rs",  "object_changing_instructions", "op::CLEAR_ATTR"),
    ("store",         "src/exec_tests.rs",     "indirect_references_to_the_stack_work_in_place", "op::STORE"),
    ("insert_obj",    "src/objects_tests.rs",  "object_changing_instructions", "op::INSERT_OBJ"),
    ("loadw",         "src/exec_tests.rs",     "array_reads_and_writes", "op::LOADW"),
    ("loadb",         "src/exec_tests.rs",     "array_reads_and_writes", "op::LOADB"),
    ("get_prop",      "src/objects_tests.rs",  "object_changing_instructions", "op::GET_PROP,"),
    ("get_prop_addr", "src/objects_tests.rs",  "object_changing_instructions", "op::GET_PROP_ADDR"),
    ("get_next_prop", "src/objects_tests.rs",  "object_changing_instructions", "op::GET_NEXT_PROP"),
    ("add",           "src/ops_math_tests.rs", "add_sub_mul_wrap_at_16_bits", "add("),
    ("sub",           "src/ops_math_tests.rs", "add_sub_mul_wrap_at_16_bits", "sub("),
    ("mul",           "src/ops_math_tests.rs", "add_sub_mul_wrap_at_16_bits", "mul("),
    ("div",           "src/ops_math_tests.rs", "div_rounds_toward_zero", "div("),
    ("mod",           "src/ops_math_tests.rs", "mod_takes_the_sign_of_the_dividend", "modulo("),
    // 1OP
    ("jz",            "src/exec_tests.rs",     "conditional_branches_follow_their_sense", "op::JZ"),
    ("get_sibling",   "src/objects_tests.rs",  "tree_instructions", "op::GET_SIBLING"),
    ("get_child",     "src/objects_tests.rs",  "tree_instructions", "op::GET_CHILD"),
    ("get_parent",    "src/objects_tests.rs",  "tree_instructions", "op::GET_PARENT"),
    ("get_prop_len",  "src/objects_tests.rs",  "object_changing_instructions", "op::GET_PROP_LEN"),
    ("inc",           "src/exec_tests.rs",     "inc_and_dec_wrap", "op::INC"),
    ("dec",           "src/exec_tests.rs",     "inc_and_dec_wrap", "op::DEC"),
    ("print_addr",    "tests/streams.rs",      "print_addr_and_print_paddr", "op::PRINT_ADDR"),
    ("remove_obj",    "src/objects_tests.rs",  "object_changing_instructions", "op::REMOVE_OBJ"),
    ("print_obj",     "tests/streams.rs",      "print_obj_prints_the_short_name", "op::PRINT_OBJ"),
    ("ret",           "src/exec_tests.rs",     "call_passes_arguments_over_default_locals", "op::RET"),
    ("jump",          "src/exec_tests.rs",     "jump_skips_forward_and_backward", "op::JUMP"),
    ("print_paddr",   "tests/streams.rs",      "print_addr_and_print_paddr", "op::PRINT_PADDR"),
    ("load",          "src/exec_tests.rs",     "load_reads_a_named_variable", "op::LOAD"),
    ("not",           "src/ops_math_tests.rs", "bitwise_operations", "not("),
    // 0OP
    ("rtrue",         "src/exec_tests.rs",     "the_ways_of_returning", "op::RTRUE"),
    ("rfalse",        "src/exec_tests.rs",     "the_ways_of_returning", "op::RFALSE"),
    ("print",         "tests/streams.rs",      "literal_printing", "print(\"Hello"),
    ("print_ret",     "tests/streams.rs",      "print_ret_prints_a_newline_and_returns_true", "op::PRINT_RET"),
    ("nop",           "src/exec_tests.rs",     "nop_and_pop", "op::NOP"),
    ("save",          "tests/save.rs",         "save_asks_the_host_and_branches_on_its_answer", "save_and_report()"),
    ("restore",       "tests/save.rs",         "restore_resumes_in_the_saved_save_with_state_as_it_was", "save_restore_program()"),
    ("restart",       "tests/streams.rs",      "restart_returns_to_the_opening_state", "op::RESTART"),
    ("ret_popped",    "src/exec_tests.rs",     "the_ways_of_returning", "op::RET_POPPED"),
    ("pop",           "src/exec_tests.rs",     "nop_and_pop", "op::POP"),
    ("quit",          "tests/events.rs",       "quit_finishes_the_game", "quit()"),
    ("new_line",      "tests/streams.rs",      "literal_printing", "op::NEW_LINE"),
    ("show_status",   "tests/streams.rs",      "show_status_updates_when_the_content_changes", "op::SHOW_STATUS"),
    ("verify",        "tests/streams.rs",      "verify_passes_on_an_intact_file", "verify_story()"),
    // VAR
    ("call",          "src/exec_tests.rs",     "call_passes_arguments_over_default_locals", "call(routine"),
    ("storew",        "src/exec_tests.rs",     "array_reads_and_writes", "op::STOREW"),
    ("storeb",        "src/exec_tests.rs",     "array_reads_and_writes", "op::STOREB"),
    ("put_prop",      "src/objects_tests.rs",  "object_changing_instructions", "op::PUT_PROP"),
    ("sread",         "tests/events.rs",       "provide_line_resumes_the_game", "read_program("),
    ("print_char",    "tests/streams.rs",      "literal_printing", "op::PRINT_CHAR"),
    ("print_num",     "tests/streams.rs",      "literal_printing", "print_num("),
    ("random",        "tests/random.rs",       "results_lie_between_1_and_the_range", "sample("),
    ("push",          "src/exec_tests.rs",     "push_and_pull", "push(7)"),
    ("pull",          "src/exec_tests.rs",     "push_and_pull", "op::PULL"),
    ("split_window",  "tests/streams.rs",      "upper_window_text_stays_out_of_the_screen_text", "op::SPLIT_WINDOW"),
    ("set_window",    "tests/streams.rs",      "upper_window_text_stays_out_of_the_screen_text", "op::SET_WINDOW"),
    ("output_stream", "tests/streams.rs",      "deselecting_stream_one_suppresses_screen_text", "op::OUTPUT_STREAM"),
    ("input_stream",  "tests/streams.rs",      "input_stream_and_sound_effect_are_accepted", "op::INPUT_STREAM"),
    ("sound_effect",  "tests/streams.rs",      "input_stream_and_sound_effect_are_accepted", "op::SOUND_EFFECT"),
];

/// `GetPropAddr` -> `get_prop_addr`
fn snake(camel: &str) -> String {
    let mut out = String::new();
    for (i, ch) in camel.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

const MARKER: &str = "// -- versions 5 and 8 only --";

/// The instruction names in the decoder's `Opcode` enum: those of version 3, and
/// those that exist only from version 5, which follow a marker line.
fn decoder_instructions() -> (BTreeSet<String>, BTreeSet<String>) {
    let text = source("src/decode.rs");
    let start = text
        .find("pub(crate) enum Opcode {")
        .expect("the Opcode enum")
        + "pub(crate) enum Opcode {".len();
    let end = start + text[start..].find("\n}").expect("the end of the enum");
    let (v3, later) = text[start..end]
        .split_once(MARKER)
        .expect("the version marker in the enum");
    let names = |block: &str| -> BTreeSet<String> {
        block
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .flat_map(|line| line.split(','))
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(snake)
            .collect()
    };
    (names(v3), names(later))
}

/// The version 3 instructions that version 5 does not have (§14): `sread` is
/// replaced by `aread`, and `pop` and `show_status` are withdrawn.
const NOT_IN_VERSION_5: [&str; 3] = ["sread", "pop", "show_status"];

/// (instruction, file, test function, text the test's body must contain) for the
/// instructions that exist only from version 5. Feature 002.
#[rustfmt::skip]
const COVERAGE_V5: &[(&str, &str, &str, &str)] = &[
    // 2OP
    ("call2s",          "tests/instructions5.rs", "storing_calls_pass_arguments_into_zeroed_locals", "op::CALL_2S"),
    ("call2n",          "tests/instructions5.rs", "discarding_calls_store_nothing", "op::CALL_2N"),
    ("set_colour",      "tests/streams5.rs",      "instructions_accepted_without_effect", "op::SET_COLOUR"),
    ("throw",           "tests/instructions5.rs", "catch_and_throw", "op::THROW"),
    // 1OP
    ("call1s",          "tests/instructions5.rs", "storing_calls_pass_arguments_into_zeroed_locals", "op::CALL_1S"),
    ("call1n",          "tests/instructions5.rs", "discarding_calls_store_nothing", "op::CALL_1N"),
    // 0OP
    ("catch",           "tests/instructions5.rs", "catch_and_throw", "op::CATCH"),
    ("piracy",          "tests/streams5.rs",      "piracy_always_branches", "op::PIRACY"),
    // VAR
    ("aread",           "tests/input5.rs",        "aread_without_a_parse_buffer", "op::AREAD"),
    ("call_vs2",        "tests/instructions5.rs", "seven_arguments", "op::CALL_VS2"),
    ("erase_window",    "tests/screen.rs",        "program_5_the_erases", "erase(-1)"),
    ("erase_line",      "tests/screen.rs",        "program_5_the_erases", "erase_line(1)"),
    ("set_cursor",      "tests/screen.rs",        "program_7_set_cursor_at_and_beyond_the_edges", "cursor(0, 0)"),
    ("get_cursor",      "tests/screen.rs",        "program_10_get_cursor_and_set_font", "op::GET_CURSOR"),
    ("set_text_style",  "tests/streams5.rs",      "styled_runs_carry_the_style_in_force", "style(2)"),
    ("buffer_mode",     "tests/streams5.rs",      "instructions_accepted_without_effect", "op::BUFFER_MODE"),
    ("read_char",       "tests/input5.rs",        "timed_input_behaves_as_untimed", "op::READ_CHAR"),
    ("scan_table",      "tests/instructions5.rs", "scan_table", "op::SCAN_TABLE"),
    ("call_vn",         "tests/instructions5.rs", "discarding_calls_store_nothing", "op::CALL_VN,"),
    ("call_vn2",        "tests/instructions5.rs", "discarding_calls_store_nothing", "op::CALL_VN2"),
    ("tokenise",        "tests/instructions5.rs", "tokenise", "op::TOKENISE"),
    ("encode_text",     "tests/instructions5.rs", "encode_text", "op::ENCODE_TEXT"),
    ("copy_table",      "tests/instructions5.rs", "copy_table", "op::COPY_TABLE"),
    ("print_table",     "tests/instructions5.rs", "print_table", "op::PRINT_TABLE"),
    ("check_arg_count", "tests/instructions5.rs", "check_arg_count_counts_every_argument_passed", "op::CHECK_ARG_COUNT"),
    // EXT
    ("log_shift",       "tests/instructions5.rs", "shifts", "op::LOG_SHIFT"),
    ("art_shift",       "tests/instructions5.rs", "shifts", "op::ART_SHIFT"),
    ("set_font",        "tests/streams5.rs",      "set_font", "op::SET_FONT"),
    ("save_undo",       "tests/screen.rs",        "undo_leaves_the_screen_as_it_is", "op::SAVE_UNDO"),
    ("restore_undo",    "tests/screen.rs",        "undo_leaves_the_screen_as_it_is", "op::RESTORE_UNDO"),
    ("print_unicode",   "tests/streams5.rs",      "print_unicode_and_check_unicode", "op::PRINT_UNICODE"),
    ("check_unicode",   "tests/streams5.rs",      "print_unicode_and_check_unicode", "op::CHECK_UNICODE"),
    ("set_true_colour", "tests/streams5.rs",      "instructions_accepted_without_effect", "op::SET_TRUE_COLOUR"),
];

// §14: version 3 has 68 instructions. The table covers exactly the set the decoder
// implements, so adding an instruction without a test fails here.
#[test]
fn the_table_lists_exactly_the_instructions_the_decoder_implements() {
    let listed: BTreeSet<String> = COVERAGE.iter().map(|row| row.0.to_string()).collect();
    assert_eq!(COVERAGE.len(), 68, "version 3 has 68 instructions");
    assert_eq!(listed.len(), 68, "no instruction is listed twice");
    let (implemented, _) = decoder_instructions();
    assert_eq!(
        implemented.len(),
        68,
        "the decoder's version 3 instructions"
    );
    assert_eq!(listed, implemented);
}

/// The comment lines directly above the `#[test]` attribute of `function`, and the
/// function's text up to the next test.
fn comment_and_body(text: &str, function: &str) -> Option<(String, String)> {
    let at = text.find(&format!("fn {function}("))?;
    let body_end = text[at..].find("\n#[test]").map_or(text.len(), |n| at + n);
    let before: Vec<&str> = text[..at].lines().collect();
    let mut comment = Vec::new();
    // The last line before `fn` is `#[test]`; above it, the comment.
    for line in before.iter().rev().skip(1) {
        if line.trim_start().starts_with("//") {
            comment.push(*line);
        } else {
            break;
        }
    }
    Some((comment.join("\n"), text[at..body_end].to_string()))
}

// §14: version 5 has 98 instructions: version 3's less three, plus 33 that exist
// only from version 5. The second table covers exactly the 33 the decoder adds, so
// adding an instruction without a row fails here. (002 SC-005)
#[test]
fn the_version_5_table_lists_exactly_the_instructions_the_decoder_adds() {
    let (v3, added) = decoder_instructions();
    let listed: BTreeSet<String> = COVERAGE_V5.iter().map(|row| row.0.to_string()).collect();
    assert_eq!(
        listed.len(),
        COVERAGE_V5.len(),
        "no instruction is listed twice"
    );
    assert_eq!(listed, added);
    for name in NOT_IN_VERSION_5 {
        assert!(v3.contains(name), "{name} is a version 3 instruction");
    }
    assert_eq!(
        v3.len() - NOT_IN_VERSION_5.len() + added.len(),
        98,
        "version 5 has 98 instructions"
    );
}

// SC-004, FR-027, 002 SC-005: each instruction's test exists, cites the Standard,
// and uses the instruction.
#[test]
fn every_instruction_has_a_test_that_cites_the_standard() {
    let mut problems = Vec::new();
    for &(instruction, file, function, evidence) in COVERAGE.iter().chain(COVERAGE_V5) {
        let text = source(file);
        let Some((comment, body)) = comment_and_body(&text, function) else {
            problems.push(format!("{instruction}: no test `{function}` in {file}"));
            continue;
        };
        if !text.contains(&format!("#[test]\nfn {function}(")) {
            problems.push(format!(
                "{instruction}: `{function}` in {file} is not a #[test]"
            ));
        }
        if !comment.contains('\u{a7}') {
            problems.push(format!(
                "{instruction}: the comment above `{function}` cites no section"
            ));
        }
        if !body.contains(evidence) {
            problems.push(format!(
                "{instruction}: `{function}` does not contain {evidence:?}"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// The crate's non-test source files.
fn implementation_files() -> Vec<(String, String)> {
    let dir = Path::new(CRATE).join("src");
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.ends_with(".rs") && !name.ends_with("_tests.rs") && name != "testing.rs" {
            files.push((name.clone(), source(&format!("src/{name}"))));
        }
    }
    assert!(files.len() >= 10, "found the source files");
    files
}

// Constitution Principle V: "The core MUST NOT depend on, or know about, any UI,
// platform or the experience layer." plan.md: no dependencies in the core.
#[test]
fn the_core_has_no_dependencies() {
    let manifest = source("Cargo.toml");
    let after = manifest
        .split("\n[dependencies]")
        .nth(1)
        .expect("a [dependencies] table");
    let table = after.split("\n[").next().unwrap();
    let entries: Vec<&str> = table
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert!(
        entries.is_empty(),
        "zmachine-core must have no dependencies, found {entries:?}"
    );
}

// Constitution Principle V: "`unsafe` Rust is permitted only in the C-binding crate".
#[test]
fn the_core_forbids_unsafe_code() {
    assert!(source("src/lib.rs").contains("#![forbid(unsafe_code)]"));
    for (name, text) in implementation_files() {
        let uses_unsafe = text.lines().any(|line| {
            let code = line.split("//").next().unwrap();
            code.contains("unsafe ") || code.contains("unsafe{")
        });
        assert!(!uses_unsafe, "src/{name} uses unsafe");
    }
}

// FR-006, contracts/core-api.md guarantee 1: "No I/O. The crate opens no files,
// reads no clock, prints nothing."
#[test]
fn the_core_performs_no_input_or_output() {
    let forbidden = [
        "std::fs",
        "std::io",
        "std::net",
        "std::time",
        "std::env",
        "std::process",
        "std::thread",
        "println!",
        "eprintln!",
        "print!(",
        "eprint!(",
        "dbg!(",
    ];
    for (name, text) in implementation_files() {
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap();
            for word in forbidden {
                assert!(
                    !code.contains(word),
                    "src/{name}:{}: the core must not use {word}",
                    number + 1
                );
            }
        }
    }
}

// Feature 003, specs/003-introspection/contracts/rust-api.md guarantee 1: the
// introspection interface reads the machine through a shared reference, and that
// proves nothing is altered only while the core has no interior mutability.
#[test]
fn the_core_has_no_interior_mutability() {
    let banned = [
        "Cell<",
        "RefCell",
        "UnsafeCell",
        "static mut",
        "Mutex",
        "RwLock",
        "Atomic",
        "OnceLock",
        "thread_local",
    ];
    for (name, text) in implementation_files() {
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for word in banned {
                assert!(
                    !code.contains(word),
                    "src/{name}:{}: {word} in {line:?}",
                    number + 1
                );
            }
        }
    }
}
