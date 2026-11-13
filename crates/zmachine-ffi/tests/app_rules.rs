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
    // the model only from the view's `send`, which is given the field's text by
    // the send button and by the field itself on Return.
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
    assert!(view.contains("CommandField(") && view.contains("text: $line"));
    assert!(view.contains("onReturn: send"));
    // On Return the field hands over its own text, as it was typed.
    let field = &sources
        .iter()
        .find(|(n, _)| n == "Lamplight/CommandField.swift")
        .unwrap()
        .1;
    assert!(field.contains("let typed = field.text ?? \"\"\n"));
    assert!(field.contains("parent.onReturn(typed)"));
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

// Feature 006, FR-032: the app carries the pack of the story it carries, and
// applies it only by the identity rule, which is the layer's. The build copies
// the pack beside the story after checking that the pack is that story's; the
// app reads it by the story's resource name and hands its text over unread.
#[test]
fn the_app_carries_its_storys_pack_and_reads_none_of_it() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let script = std::fs::read_to_string(format!("{root}/scripts/build-ios.sh")).unwrap();
    assert!(
        script.contains(r#"cp "$pack" "$resources/story.pack""#),
        "the build copies the pack beside the story"
    );
    assert!(
        script.contains(r#"grep -qx "sha256: $actual" "$pack""#),
        "and only if the pack names the story it is copied beside"
    );
    let project = std::fs::read_to_string(format!("{APP}/project.yml")).unwrap();
    assert!(
        project.contains("Lamplight/Resources/story.pack"),
        "the pack is a resource"
    );
    let ignore = std::fs::read_to_string(format!("{root}/.gitignore")).unwrap();
    assert!(
        ignore.contains("/apps/ios/Lamplight/Resources/story.pack"),
        "the copy is generated, not committed"
    );

    let sources = app_swift();
    let source = |name: &str| &sources.iter().find(|(n, _)| n == name).unwrap().1;
    let game = source("Lamplight/Game.swift");
    assert!(
        game.contains(r#"Bundle.main.url(forResource: name, withExtension: "pack")"#),
        "the pack is found by the story's resource name"
    );
    let engine = source("Lamplight/Engine.swift");
    assert!(engine.contains("zmx_session_new_with_pack("));
    assert!(
        !engine.contains("zmx_session_new("),
        "one way of starting a session"
    );
    // No source of the app reads a pack's contents: no key of the format is
    // written anywhere in it.
    for (name, text) in &sources {
        for key in [
            "sha256",
            "open-attribute",
            "untrusted-variable",
            "transparent-attribute",
            "lit-attribute",
            "invisible-attribute",
        ] {
            assert!(!text.contains(key), "apps/ios/{name} holds {key:?}");
        }
    }
}

// Feature 007, FR-023, FR-027: the strip of what is carried shows the layer's
// words and nothing of the app's own, and nothing in it sends anything.
#[test]
fn the_strip_shows_the_layers_words_and_sends_nothing() {
    let sources = app_swift();
    let strip = &sources
        .iter()
        .find(|(n, _)| n == "Lamplight/InventoryStrip.swift")
        .expect("the strip's view")
        .1;
    // Its code, without its comments.
    let code: String = strip
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in ["Button", "onTapGesture", "zmx_", "game.", "Task {"] {
        assert!(!code.contains(forbidden), "the strip holds {forbidden:?}");
    }
    // Every text it draws is handed to it: a note, a name, or what is spoken.
    let texts: Vec<&str> = strip
        .lines()
        .filter(|l| l.contains("Text("))
        .map(str::trim)
        .collect();
    assert_eq!(
        texts,
        [
            "Text(verbatim: note)",
            ".accessibilityLabel(Text(verbatim: strip.spoken))",
            "Text(verbatim: item.name)"
        ]
    );
    // Its only string literals are identifiers for tests.
    let literals: Vec<&str> = strip.lines().flat_map(literals).collect();
    assert_eq!(literals, ["inventory.note", "inventory.strip"]);
}

// Feature 008, FR-022: what is said of darkness is the layer's. The app holds
// no words of its own for the note or the mark: the note's text is the
// snapshot's, and a way into darkness is told to a screen reader by the place's
// own spoken words.
#[test]
fn the_app_says_nothing_of_its_own_about_darkness() {
    let sources = app_swift();
    for (name, text) in &sources {
        for (number, line) in text.lines().enumerate() {
            for literal in literals(line) {
                // A field of the snapshot, and an identifier for tests.
                if literal == "dark_ahead" || literal == "light.note" {
                    continue;
                }
                assert!(
                    !literal.to_ascii_lowercase().contains("dark"),
                    "apps/ios/{name}:{}: the literal {literal:?}",
                    number + 1
                );
            }
        }
    }
    let source = |name: &str| &sources.iter().find(|(n, _)| n == name).unwrap().1;
    let snapshot = source("Lamplight/Snapshot.swift");
    assert!(snapshot.contains(r#"case darkAhead = "dark_ahead""#));
    assert!(snapshot.contains("let light: Light"));
    // The note is drawn from the snapshot's words, in the game's view.
    let view = source("Lamplight/GameView.swift");
    let at = view
        .find(r#".accessibilityIdentifier("light.note")"#)
        .expect("the note is drawn");
    let before = &view[..at];
    let drawn = before
        .rfind("Text(verbatim: note)")
        .expect("the layer's words");
    assert!(at - drawn < 400, "the identifier is the note's own");
    assert!(before[..drawn].contains("showing.light.note"));
    // The mark is drawn where the layer says, and nowhere else.
    let compass = source("Lamplight/CompassView.swift");
    assert!(compass.contains("place.darkAhead"));
}
