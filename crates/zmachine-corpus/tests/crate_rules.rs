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

// FR-015: this feature does not change introspection, and introspection knows
// nothing of the corpus or its ledger.
#[test]
fn introspection_knows_nothing_of_the_corpus() {
    let mut sources = vec![std::path::PathBuf::from(repo(
        "crates/zmachine-introspect/Cargo.toml",
    ))];
    files(&repo("crates/zmachine-introspect/src"), &mut sources);
    for path in sources {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("corpus"), "{}", path.display());
        assert!(!text.contains("ledger.txt"), "{}", path.display());
    }
}

// FR-020: fetching is a step a person runs. No CI job fetches a story.
#[test]
fn ci_does_not_fetch_stories() {
    let ci = read_text(".github/workflows/ci.yml");
    assert!(!ci.contains("fetch-corpus"));
    assert!(!ci.contains("ifarchive.org/if-archive/games"));
}

// Feature 006, FR-035; constitution Principle IV ("Per-game code is
// prohibited"): what differs between games is in a pack. No source file of the
// interface, the experience layer, the C interface or `zm` holds any story's
// digest, its id as a word, or its serial.
#[test]
fn no_source_knows_any_one_story() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = std::fs::read_to_string(root.join("stories/corpus/manifest.txt")).unwrap();
    let stories = zmachine_corpus::manifest::read(&manifest).unwrap();
    let mut marks: Vec<String> = Vec::new();
    for story in &stories {
        marks.push(story.sha256.clone());
        marks.push(story.id.clone());
        if let Ok(bytes) = std::fs::read(root.join(&story.path)) {
            marks.push(String::from_utf8_lossy(&bytes[0x12..0x18]).into_owned());
        }
    }
    assert!(
        marks.len() > 2 * stories.len(),
        "the committed stories' serials are read"
    );
    let mut read = 0;
    for name in ["introspect", "experience", "ffi", "cli"] {
        let directory = root.join(format!("crates/zmachine-{name}/src"));
        for entry in std::fs::read_dir(directory).unwrap().flatten() {
            let path = entry.path();
            let file = path.file_name().unwrap().to_string_lossy().into_owned();
            if !file.ends_with(".rs") || file.ends_with("_tests.rs") || file == "fixtures.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
                assert!(
                    !marks.iter().any(|mark| mark == word),
                    "{}: {word:?} is a story's digest, id or serial",
                    path.display()
                );
            }
            read += 1;
        }
    }
    assert!(read >= 12, "{read} source files read");
}
