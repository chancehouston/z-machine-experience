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
    /// The crawl's budget for this story, when not the default.
    pub rooms: Option<usize>,
    pub trials: Option<usize>,
    /// The line of the manifest the entry starts on.
    pub line: usize,
}

const KEYS: [&str; 14] = [
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
        if key != "opening" && entry.iter().any(|(_, k, _)| *k == key) {
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
            "opening" => story.opening.push(value.to_string()),
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
    Ok(story)
}
