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

/// The instructions the decoder knows, read from its `Opcode` enum.
fn decoder_instructions() -> BTreeSet<String> {
    let text = source("src/decode.rs");
    let start = text
        .find("pub(crate) enum Opcode {")
        .expect("the Opcode enum")
        + "pub(crate) enum Opcode {".len();
    let end = start + text[start..].find("\n}").expect("the end of the enum");
    text[start..end]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(snake)
        .collect()
}

// §14: version 3 has 68 instructions. The table covers exactly the set the decoder
// implements, so adding an instruction without a test fails here.
#[test]
fn the_table_lists_exactly_the_instructions_the_decoder_implements() {
    let listed: BTreeSet<String> = COVERAGE.iter().map(|row| row.0.to_string()).collect();
    assert_eq!(COVERAGE.len(), 68, "version 3 has 68 instructions");
    assert_eq!(listed.len(), 68, "no instruction is listed twice");
    let implemented = decoder_instructions();
    assert_eq!(implemented.len(), 68, "the decoder's Opcode enum");
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

// SC-004, FR-027: each instruction's test exists, cites the Standard, and uses the
// instruction.
#[test]
fn every_instruction_has_a_test_that_cites_the_standard() {
    let mut problems = Vec::new();
    for &(instruction, file, function, evidence) in COVERAGE {
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
