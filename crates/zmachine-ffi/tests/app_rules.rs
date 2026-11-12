//! Rules the iPhone app's sources must keep (FR-028, SC-011; research R12;
//! constitution Principles III, IV and VI): the app names no game, composes no
//! command, and touches the bindings in one file.

use std::path::{Path, PathBuf};

const APP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/ios");

/// What `scripts/build-ios.sh` generates: not the app's source.
fn generated(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.contains("/Frameworks/")
        || text.contains(".xcodeproj")
        || text.contains("/Resources/")
        || text.contains("xcuserdata")
        || text.ends_with(".DS_Store")
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if generated(&path) {
            continue;
        }
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn all_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    files(Path::new(APP), &mut paths);
    paths
        .into_iter()
        .map(|p| {
            let name = p.strip_prefix(APP).unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&p).unwrap())
        })
        .collect()
}

/// The app's own Swift sources, not its tests.
fn app_swift() -> Vec<(String, String)> {
    let found: Vec<_> = all_files()
        .into_iter()
        .filter(|(name, _)| name.starts_with("Lamplight/") && name.ends_with(".swift"))
        .collect();
    assert!(found.len() >= 7, "found the app's sources: {}", found.len());
    found
}

/// The string literals of one line of Swift.
fn literals(line: &str) -> Vec<&str> {
    let code = line.split("//").next().unwrap_or("");
    code.split('"').skip(1).step_by(2).collect()
}

// Constitution Principle VI; FR-028: the game's name appears nowhere in the
// app's own files. (It appears in the bundled story and its license, which are
// generated, and in the game's own text.)
#[test]
fn the_app_names_no_game() {
    for (name, text) in all_files() {
        for (number, line) in text.lines().enumerate() {
            assert!(
                !line.to_ascii_lowercase().contains("zork"),
                "apps/ios/{name}:{}: {line}",
                number + 1
            );
        }
    }
}

// SC-011, constitution Principle III: the app composes no command. A typed line
// is sent from exactly one place, and what is sent is the field's text.
#[test]
fn a_typed_line_is_sent_from_one_place() {
    let sources = app_swift();
    let calls: Vec<(&str, &str)> = sources
        .iter()
        .flat_map(|(name, text)| text.lines().map(move |l| (name.as_str(), l)))
        .filter(|(_, l)| l.contains("zmx_session_send_line"))
        .collect();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].0, "Lamplight/Engine.swift");
    assert!(calls[0].1.contains(", typed)"), "{}", calls[0].1);

    // Followed back: `typed` reaches the engine only from the game model, and
    // the model only from the view's `send`, which passes the field's text.
    let callers: Vec<(&str, &str)> = sources
        .iter()
        .flat_map(|(name, text)| text.lines().map(move |l| (name.as_str(), l)))
        .filter(|(_, l)| l.contains(".send(typed:"))
        .collect();
    assert_eq!(callers.len(), 2, "{callers:?}");
    let view = &sources
        .iter()
        .find(|(n, _)| n == "Lamplight/GameView.swift")
        .unwrap()
        .1;
    assert!(
        view.contains("let typed = line\n"),
        "the line sent is the field's text"
    );
    assert!(view.contains("game.send(typed: typed)"));
    assert!(view.contains("TextField(") && view.contains("text: $line"));
}

// SC-011: no command word is written in the app. The only place the words for
// directions appear is the table of where each place goes on the screen, where
// they are identifiers the layer gave, sent back unchanged.
#[test]
fn no_direction_word_is_written_outside_the_layout() {
    let words = [
        "north",
        "northeast",
        "east",
        "southeast",
        "south",
        "southwest",
        "west",
        "northwest",
        "up",
        "down",
        "in",
        "out",
        "n",
        "s",
        "e",
        "w",
        "ne",
        "nw",
        "se",
        "sw",
        "u",
        "d",
        "enter",
        "exit",
        "land",
        "look",
        "inventory",
    ];
    for (name, text) in app_swift() {
        if name == "Lamplight/CompassLayout.swift" {
            continue;
        }
        for (number, line) in text.lines().enumerate() {
            for literal in literals(line) {
                assert!(
                    !words.contains(&literal.to_ascii_lowercase().as_str()),
                    "apps/ios/{name}:{}: the literal {literal:?}",
                    number + 1
                );
            }
        }
    }
}

// Constitution Principle V, FR-026: one file touches the bindings.
#[test]
fn only_the_engine_touches_the_bindings() {
    for (name, text) in app_swift() {
        if name == "Lamplight/Engine.swift" {
            assert!(text.contains("import ZMX"));
            continue;
        }
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for banned in [
                "import ZMX",
                "zmx_",
                "OpaquePointer",
                "UnsafeMutablePointer",
                "UnsafePointer",
                "UnsafeRawPointer",
                "UnsafeMutableRawPointer",
                "UnsafeBufferPointer",
            ] {
                assert!(
                    !code.contains(banned),
                    "apps/ios/{name}:{}: {banned}",
                    number + 1
                );
            }
        }
    }
}

// Constitution Principle IV: what is said about ways out is the layer's. The
// app writes no text of its own about exits.
#[test]
fn the_app_says_nothing_of_its_own_about_exits() {
    for (name, text) in app_swift() {
        for (number, line) in text.lines().enumerate() {
            for literal in literals(line) {
                // The name of a field of the snapshot is not text of the app's.
                if literal == "leads_to" {
                    continue;
                }
                let lower = literal.to_ascii_lowercase();
                for word in ["exit", "leads", "blocked"] {
                    assert!(
                        !lower.contains(word),
                        "apps/ios/{name}:{}: {word:?} in the literal {literal:?}",
                        number + 1
                    );
                }
            }
        }
    }
}
