//! The praxix unit test for versions 5 and up (002 spec User Story 4, SC-001;
//! constitution Principle I). praxix checks itself: each group of tests reports how
//! many failed, and the run ends with "All tests passed." or a list of the groups
//! that did not. See `stories/praxix/SOURCE.md`.

use sha2::{Digest, Sha256};
use std::io::Write;
use std::process::{Command, Stdio};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/praxix");
const LEDGER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/divergences.md");

fn read(name: &str) -> Vec<u8> {
    std::fs::read(format!("{DIR}/{name}")).unwrap_or_else(|e| panic!("stories/praxix/{name}: {e}"))
}

/// Runs every praxix test and returns what it printed, unwrapped.
fn run_praxix() -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zm"))
        .args([
            "--seed",
            "1",
            "--width",
            "0",
            "--max-instructions",
            "50000000",
        ])
        .arg(format!("{DIR}/praxix.z5"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zm runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"all\nquit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "praxix did not run to its end: {stderr}"
    );
    assert_eq!(stderr, "", "praxix should cause no faults or diagnostics");
    String::from_utf8(out.stdout).expect("praxix's output is UTF-8")
}

// Principle VI: the committed files are the ones the record describes.
#[test]
fn the_committed_files_match_their_record() {
    let record = String::from_utf8(read("SOURCE.md")).unwrap();
    for name in ["praxix.z5", "praxix.inf"] {
        let digest: String = Sha256::digest(read(name))
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(
            record.contains(&digest),
            "stories/praxix/{name} has digest {digest}"
        );
    }
}

// SC-001: praxix reports no failed test in any group.
#[test]
fn praxix_passes_every_test() {
    let output = run_praxix();
    // A group that fails says so on a line of its own: "Failed 2 tests." or
    // "1 test failed", and the last line names the groups.
    let failures: Vec<&str> = output
        .lines()
        .filter(|l| {
            let l = l.to_ascii_lowercase();
            // One line of praxix's own commentary mentions failures in passing.
            l.contains("fail") && !l.contains("failures are not counted twice")
        })
        .collect();
    assert!(
        failures.is_empty(),
        "praxix reported:\n{}\n\nin:\n{output}",
        failures.join("\n")
    );
    assert!(
        output.contains("\nAll tests passed.\n"),
        "the pass summary; got:\n{output}"
    );
    assert!(
        output.trim_end().ends_with("Goodbye."),
        "praxix ran to its end"
    );
}

// praxix prints one "Passed." for each group that checks itself; a group that
// stopped early would be missing. The count is what the reference interpreter
// gives for this file.
#[test]
fn praxix_runs_every_group() {
    let output = run_praxix();
    for group in [
        "Basic operand values:",
        "Integer arithmetic:",
        "Compound arithmetic expressions:",
        "Bitwise arithmetic:",
        "Bit shifts:",
        "Increment/decrement:",
        "Increment/decrement and branch:",
        "Array loads and stores:",
        "Undo:",
        "Multi-level undo:",
        "Indirect opcodes:",
        "Memory stream round-trip:",
        "Memory stream opcodes:",
        "Z-Machine 1.1 Updates/Clarifications:",
    ] {
        assert!(
            output.contains(&format!("\n{group}\n")),
            "no {group:?} group in:\n{output}"
        );
    }
    // Sixteen groups check themselves and say so; `spec11` and `spec12` only print.
    assert_eq!(output.matches("\nPassed.\n").count(), 16, "{output}");
    assert!(output.contains("Interpreter claims to support undo."));
    assert!(output.contains("Ok, interpreter is version 1.1."));
}

// A praxix test may be excused only by a rule in the ledger; none fails, so the
// ledger must hold no rule naming praxix.
#[test]
fn the_ledger_excuses_no_praxix_test() {
    let ledger = std::fs::read_to_string(LEDGER).unwrap();
    assert!(
        !ledger.to_ascii_lowercase().contains("kind: praxix"),
        "docs/divergences.md excuses a praxix test, but none fails"
    );
}
