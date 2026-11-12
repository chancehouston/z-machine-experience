//! Rules the crate itself must keep (specs/003-introspection/contracts/rust-api.md
//! guarantees 4 and 6; constitution Principles IV and V): no dependency but the
//! core, no `unsafe`, no input or output, and no knowledge of any one game.

use std::path::Path;

const CRATE: &str = env!("CARGO_MANIFEST_DIR");

fn sources() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(Path::new(CRATE).join("src")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        // Unit tests may name the games they were checked against in comments.
        if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
            files.push((name, std::fs::read_to_string(&path).unwrap()));
        }
    }
    assert!(files.len() >= 6, "found the source files: {}", files.len());
    files
}

// Guarantee 6: "No dependencies beyond `zmachine-core`".
#[test]
fn the_crate_depends_only_on_the_core() {
    let manifest = std::fs::read_to_string(Path::new(CRATE).join("Cargo.toml")).unwrap();
    let section = manifest.split("\n[dependencies]").nth(1).expect("a dependencies section");
    let section = section.split("\n[").next().unwrap();
    let names: Vec<&str> = section
        .lines()
        .filter_map(|l| l.split('=').next())
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .collect();
    assert_eq!(names, ["zmachine-core"]);
}

// Guarantee 6: no `unsafe`, no input or output.
#[test]
fn the_crate_has_no_unsafe_and_no_io() {
    for (name, text) in sources() {
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for banned in ["unsafe ", "std::fs", "std::io", "std::net", "std::env", "println!", "eprintln!", "print!(", "std::process"] {
                assert!(!code.contains(banned), "src/{name}:{}: {banned} in {line:?}", number + 1);
            }
        }
    }
    let lib = std::fs::read_to_string(Path::new(CRATE).join("src/lib.rs")).unwrap();
    assert!(lib.contains("#![forbid(unsafe_code)]"));
}

// Guarantee 4, constitution Principle IV: "Per-game code and per-game data files
// are prohibited." No source file names a game or holds a serial number.
#[test]
fn the_crate_knows_no_game() {
    for (name, text) in sources() {
        for (number, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            for game in ["zork", "advent", "czech", "praxix", "colossal"] {
                assert!(!lower.contains(game), "src/{name}:{}: names a game: {line:?}", number + 1);
            }
            // A serial number is six digits (§11.1); so is a date written that way.
            let digits = line.as_bytes().windows(6).any(|w| w.iter().all(u8::is_ascii_digit));
            assert!(!digits, "src/{name}:{}: a six-digit literal: {line:?}", number + 1);
        }
    }
}
