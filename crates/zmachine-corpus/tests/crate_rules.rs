//! Rules the crate keeps (specs/005-corpus-testing/spec.md FR-005; plan.md,
//! "Constitution Check" V).

mod common;

use common::*;

fn files(dir: &str, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(path.to_str().unwrap(), out);
        } else {
            out.push(path);
        }
    }
}

// FR-005: trials exist only in the testing tools. No crate may depend on this one.
#[test]
fn no_crate_depends_on_the_corpus_tools() {
    let mut checked = 0;
    for entry in std::fs::read_dir(repo("crates")).unwrap() {
        let dir = entry.unwrap().path();
        if dir.file_name().unwrap() == "zmachine-corpus" {
            continue;
        }
        let manifest = dir.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest).unwrap();
        assert!(!text.contains("zmachine-corpus"), "{}", manifest.display());
        checked += 1;
    }
    assert!(checked >= 6, "every other crate was read: {checked}");
}

#[test]
fn the_tools_use_no_unsafe_code() {
    let mut sources = Vec::new();
    files(&repo("crates/zmachine-corpus/src"), &mut sources);
    assert!(!sources.is_empty());
    for path in sources {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("unsafe"), "{}", path.display());
    }
}
