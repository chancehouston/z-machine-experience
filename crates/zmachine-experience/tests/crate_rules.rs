//! Rules the crate itself must keep (specs/004-first-demo/contracts/
//! experience-api.md rule 12, cited E§12; constitution Principles IV and V): only
//! the layers beneath as dependencies, no `unsafe`, no input or output, no random
//! source or clock of its own, and no knowledge of any one game.

use std::path::Path;

const CRATE: &str = env!("CARGO_MANIFEST_DIR");

fn sources() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(Path::new(CRATE).join("src")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
            files.push((name, std::fs::read_to_string(&path).unwrap()));
        }
    }
    assert!(files.len() >= 4, "found the source files: {}", files.len());
    files
}

// E§12: the layer sits on the core and the introspection interface, and on
// nothing else.
#[test]
fn the_crate_depends_only_on_the_layers_beneath() {
    let manifest = std::fs::read_to_string(Path::new(CRATE).join("Cargo.toml")).unwrap();
    let section = manifest.split("\n[dependencies]").nth(1).expect("a dependencies section");
    let section = section.split("\n[").next().unwrap();
    let names: Vec<&str> =
        section.lines().filter_map(|l| l.split('=').next()).map(str::trim).filter(|n| !n.is_empty()).collect();
    assert_eq!(names, ["zmachine-core", "zmachine-introspect"]);
}

// E§12: no `unsafe`, no input or output, no clock and no random source.
#[test]
fn the_crate_has_no_unsafe_io_clock_or_random_source() {
    for (name, text) in sources() {
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for banned in ["unsafe ", "std::fs", "std::io", "std::net", "std::env", "std::time", "std::process", "rand::", "RandomState", "println!", "eprintln!", "print!("] {
                assert!(!code.contains(banned), "src/{name}:{}: {banned} in {line:?}", number + 1);
            }
        }
    }
    let lib = std::fs::read_to_string(Path::new(CRATE).join("src/lib.rs")).unwrap();
    assert!(lib.contains("#![forbid(unsafe_code)]"));
}

// E§12, constitution Principle IV: no source file names a game or holds a
// serial number. The layer knows what a compass is, not what any game is.
#[test]
fn the_crate_knows_no_game() {
    for (name, text) in sources() {
        for (number, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            for game in ["zork", "advent", "czech", "praxix", "colossal"] {
                assert!(!lower.contains(game), "src/{name}:{}: names a game: {line:?}", number + 1);
            }
            let digits = line.split(|c: char| !c.is_ascii_digit()).any(|run| run.len() == 6);
            assert!(!digits, "src/{name}:{}: a six-digit literal: {line:?}", number + 1);
        }
    }
}
