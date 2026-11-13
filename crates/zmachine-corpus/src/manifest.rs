//! The manifest: the stories of the corpus
//! (specs/005-corpus-testing/contracts/formats.md, "The manifest").

use crate::report::Refusal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Committed,
    Fetched,
}

/// One story file in the corpus (data-model.md "Story").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Story {
    pub id: String,
    pub sha256: String,
    pub size: u64,
    pub version: u8,
    /// `zil`, `inform` or `none`.
    pub family: String,
    pub source: String,
    pub standing: String,
    pub tier: Tier,
    /// Relative to the repository's root.
    pub path: String,
    pub seed: u32,
    pub walkthrough: Option<String>,
    /// Answers given before the first command prompt.
    pub opening: Vec<String>,
    /// Commands given at the first command prompts, each of which is a place,
    /// before the crawl starts (specs/006-game-packs/research.md R12).
    pub first: Vec<String>,
    /// The command that asks the game what the player carries
    /// (specs/007-inventory-panel/research.md R3).
    pub inventory: Option<String>,
    /// Short walks from the start, at whose prompts the inventory is compared
    /// and no exit is tried (research R2).
    pub asides: Vec<Vec<String>>,
    /// The command that asks the game to describe where the player is
    /// (specs/008-darkness/research.md R6).
    pub look: Option<String>,
    /// The crawl's budget for this story, when not the default.
    pub rooms: Option<usize>,
    pub trials: Option<usize>,
    /// The line of the manifest the entry starts on.
    pub line: usize,
}

const KEYS: [&str; 18] = [
    "id",
    "sha256",
    "size",
    "version",
    "family",
    "source",
    "standing",
    "tier",
    "path",
    "seed",
    "walkthrough",
    "opening",
    "first",
    "inventory",
    "aside",
    "look",
    "rooms",
    "trials",
];

/// The keys every entry must have.
const REQUIRED: [&str; 10] = [
    "id", "sha256", "size", "version", "family", "source", "standing", "tier", "path", "seed",
];

/// Where fetched stories live; the repository ignores it.
pub const LOCAL: &str = "stories/local/corpus/";

/// Reads a manifest's entries, refusing the first line that departs from the
/// format. Lines starting with `#` are comments; a blank line ends an entry.
pub fn read(text: &str) -> Result<Vec<Story>, Refusal> {
    let mut stories = Vec::new();
    let mut entry: Vec<(usize, &str, &str)> = Vec::new();
    let lines = text.lines().chain(std::iter::once(""));
    for (number, line) in lines.enumerate().map(|(i, l)| (i + 1, l)) {
        if line.starts_with('#') {
            continue;
        }
        if line.trim().is_empty() {
            if !entry.is_empty() {
                stories.push(story(&entry)?);
                entry.clear();
            }
            continue;
        }
        let Some((key, value)) = line.split_once(": ") else {
            return Err((number, "expected `key: value`".to_string()));
        };
        if !KEYS.contains(&key) {
            return Err((number, format!("`{key}` is not a key of the manifest")));
        }
        if !["opening", "first", "aside"].contains(&key) && entry.iter().any(|(_, k, _)| *k == key)
        {
            return Err((number, format!("`{key}` is given twice")));
        }
        entry.push((number, key, value));
    }
    Ok(stories)
}

fn story(entry: &[(usize, &str, &str)]) -> Result<Story, Refusal> {
    let first = entry[0].0;
    for key in REQUIRED {
        if !entry.iter().any(|(_, k, _)| *k == key) {
            return Err((first, format!("the entry has no `{key}`")));
        }
    }
    let mut story = Story {
        id: String::new(),
        sha256: String::new(),
        size: 0,
        version: 0,
        family: String::new(),
        source: String::new(),
        standing: String::new(),
        tier: Tier::Committed,
        path: String::new(),
        seed: 0,
        walkthrough: None,
        opening: Vec::new(),
        first: Vec::new(),
        inventory: None,
        asides: Vec::new(),
        look: None,
        rooms: None,
        trials: None,
        line: first,
    };
    let number = |value: &str| -> Option<u64> {
        (!value.is_empty() && value.len() <= 12 && value.bytes().all(|b| b.is_ascii_digit()))
            .then(|| value.parse().ok())
            .flatten()
    };
    for &(line, key, value) in entry {
        let refuse = |why: &str| Err((line, why.to_string()));
        match key {
            "id" if crate::report::is_id(value) => story.id = value.to_string(),
            "id" => return refuse("an id is lowercase letters, digits and hyphens"),
            "sha256" if crate::report::is_digest(value) => story.sha256 = value.to_string(),
            "sha256" => return refuse("a digest is 64 lowercase hexadecimal digits"),
            "size" => match number(value) {
                Some(size) => story.size = size,
                None => return refuse("the size is a number of bytes"),
            },
            "version" => match value {
                "3" | "5" | "8" => story.version = value.parse().unwrap_or(0),
                _ => return refuse("the version is 3, 5 or 8"),
            },
            "family" if crate::report::FAMILIES.contains(&value) => {
                story.family = value.to_string()
            }
            "family" => return refuse("the family is zil, inform or none"),
            "source" => story.source = value.to_string(),
            "standing" => story.standing = value.to_string(),
            "tier" => match value {
                "committed" => story.tier = Tier::Committed,
                "fetched" => story.tier = Tier::Fetched,
                _ => return refuse("the tier is committed or fetched"),
            },
            "path" => story.path = value.to_string(),
            "seed" => match number(value) {
                Some(seed) if seed <= 2_147_483_647 => story.seed = seed as u32,
                _ => return refuse("the seed is 0 to 2147483647"),
            },
            "walkthrough" => story.walkthrough = Some(value.to_string()),
            "opening" => story.opening.push(match value {
                "(space)" => " ".to_string(),
                "(return)" => String::new(),
                other => other.to_string(),
            }),
            "first" => story.first.push(value.to_string()),
            "inventory" => story.inventory = Some(value.to_string()),
            "look" => story.look = Some(value.to_string()),
            "aside" => story
                .asides
                .push(value.split(" / ").map(str::to_string).collect()),
            "rooms" | "trials" => match number(value) {
                Some(n) if key == "rooms" => story.rooms = Some(n as usize),
                Some(n) => story.trials = Some(n as usize),
                None => return refuse("a budget is a number"),
            },
            _ => unreachable!("the key was checked when the line was read"),
        }
    }
    // What one key requires of another, refused at the line that breaks it.
    let line_of = |key: &str| {
        entry
            .iter()
            .find(|(_, k, _)| *k == key)
            .map_or(first, |e| e.0)
    };
    let local = story.path.starts_with(LOCAL);
    match story.tier {
        Tier::Fetched if !local => {
            return Err((
                line_of("path"),
                format!("a fetched story's path is under {LOCAL}"),
            ));
        }
        Tier::Committed if local => {
            return Err((
                line_of("path"),
                format!("a committed story's path is not under {LOCAL}"),
            ));
        }
        Tier::Fetched if story.walkthrough.is_some() => {
            return Err((
                line_of("walkthrough"),
                "only a committed story has a walkthrough".to_string(),
            ));
        }
        _ => {}
    }
    if story.inventory.is_none() && story.look.is_none() && !story.asides.is_empty() {
        return Err((
            line_of("aside"),
            "an aside compares the inventory or the light, and needs `inventory` or `look`"
                .to_string(),
        ));
    }
    if story.walkthrough.is_some() && !story.first.is_empty() {
        return Err((
            line_of("first"),
            "a story with a walkthrough has no first commands".to_string(),
        ));
    }
    Ok(story)
}

/// Checks the manifest and the ledger's form, and the manifest against the
/// files it names (contracts/formats.md, "The manifest check"). Returns each
/// failure as `file:line: why`.
pub fn check(root: &std::path::Path) -> Vec<String> {
    use crate::corpus::{MANIFEST, load, sha256};
    let (stories, _) = match load(root) {
        Ok(loaded) => loaded,
        Err(why) => return vec![why],
    };
    let mut failures = Vec::new();
    for (i, story) in stories.iter().enumerate() {
        let mut fail = |why: String| failures.push(format!("{MANIFEST}:{}: {why}", story.line));
        if stories[..i].iter().any(|s| s.id == story.id) {
            fail(format!("`{}` is the id of another entry", story.id));
        }
        if stories[..i].iter().any(|s| s.sha256 == story.sha256) {
            fail("the digest is the digest of another entry".to_string());
        }
        let path = root.join(&story.path);
        let bytes = std::fs::read(&path).ok();
        match story.tier {
            Tier::Fetched => {
                if tracked(root, &story.path) == Some(true) {
                    fail("a fetched story's file is tracked by the repository".to_string());
                }
                // Absent, or some other file: the story is skipped, not failed.
                let Some(bytes) = bytes.filter(|b| sha256(b) == story.sha256) else {
                    continue;
                };
                describes(story, &bytes, &mut fail);
            }
            Tier::Committed => {
                let Some(bytes) = bytes else {
                    fail("the file is absent".to_string());
                    continue;
                };
                let digest = sha256(&bytes);
                if digest != story.sha256 {
                    fail(format!("the file's digest is {digest}"));
                    continue;
                }
                describes(story, &bytes, &mut fail);
                let beside = path.parent().unwrap_or(root);
                match std::fs::read_to_string(beside.join("SOURCE.md")) {
                    Err(_) => fail("there is no SOURCE.md beside the file".to_string()),
                    Ok(record) if !record.contains(&story.sha256) => {
                        fail("SOURCE.md does not hold the digest".to_string())
                    }
                    Ok(_) => {}
                }
                let licensed = std::fs::read_dir(beside).is_ok_and(|entries| {
                    entries.flatten().any(|entry| {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        name == "LICENSE" || name.starts_with("COPYING")
                    })
                });
                if !licensed {
                    fail("there is no license text beside the file".to_string());
                }
            }
        }
    }
    failures
}

/// Whether the file is what the entry says of it: its size, version and family.
fn describes(story: &Story, bytes: &[u8], fail: &mut impl FnMut(String)) {
    if bytes.len() as u64 != story.size {
        fail(format!("the file's size is {}", bytes.len()));
    }
    let version = bytes.first().copied().unwrap_or(0);
    if version != story.version {
        fail(format!("the file's version is {version}"));
    }
    let family = match crate::walk::load(bytes.to_vec(), story.seed) {
        Ok(machine) => {
            let view = machine.view();
            match zmachine_introspect::Introspector::new(&view).family() {
                zmachine_introspect::Family::Zil => "zil",
                zmachine_introspect::Family::Inform6 => "inform",
                zmachine_introspect::Family::Unrecognised => "none",
            }
        }
        Err(_) => {
            fail("the file does not load".to_string());
            return;
        }
    };
    if family != story.family {
        fail(format!("introspection finds the family {family}"));
    }
}

/// Whether the repository tracks the path; `None` where that cannot be asked
/// (no `git`, or not a repository).
fn tracked(root: &std::path::Path, path: &str) -> Option<bool> {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--error-unmatch", "--", path])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    match status.code() {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
}
