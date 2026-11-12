//! The ledger: wrong reports that are known, each with its cause
//! (specs/005-corpus-testing/contracts/formats.md, "The ledger").

use crate::report::{Case, Refusal};
use crate::verdict::{Verdict, judge};

/// One known wrong report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub story: String,
    pub case: Case,
    pub cause: String,
    /// The line of the ledger the entry starts on.
    pub line: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub entries: Vec<Entry>,
}

impl Ledger {
    /// Whether the ledger holds this case of this story.
    pub fn knows(&self, story: &str, case: &Case) -> bool {
        self.entries
            .iter()
            .any(|e| e.story == story && e.case == *case)
    }
}

const KEYS: [&str; 6] = ["story", "room", "direction", "reported", "outcome", "cause"];

/// Reads a ledger, refusing the first line that departs from the format. Lines
/// starting with `#` are comments; a blank line ends an entry; a line starting
/// with two spaces continues the line before it.
pub fn read(text: &str) -> Result<Ledger, Refusal> {
    let mut ledger = Ledger::default();
    let mut entry: Vec<(usize, &str, String)> = Vec::new();
    let lines = text.lines().chain(std::iter::once(""));
    for (number, line) in lines.enumerate().map(|(i, l)| (i + 1, l)) {
        if line.starts_with('#') {
            continue;
        }
        if line.trim().is_empty() {
            if !entry.is_empty() {
                let made = make(&entry)?;
                if ledger.knows(&made.story, &made.case) {
                    return Err((made.line, "the same case is entered twice".to_string()));
                }
                ledger.entries.push(made);
                entry.clear();
            }
            continue;
        }
        if let Some(more) = line.strip_prefix("  ") {
            match entry.last_mut() {
                Some((_, _, value)) => {
                    value.push(' ');
                    value.push_str(more.trim());
                }
                None => return Err((number, "nothing to continue".to_string())),
            }
            continue;
        }
        let (key, value) = match line.split_once(':') {
            Some((key, value)) => (key, value.trim()),
            None => return Err((number, "expected `key: value`".to_string())),
        };
        if !KEYS.contains(&key) {
            return Err((number, format!("`{key}` is not a key of the ledger")));
        }
        if entry.iter().any(|(_, k, _)| *k == key) {
            return Err((number, format!("`{key}` is given twice")));
        }
        entry.push((number, key, value.to_string()));
    }
    Ok(ledger)
}

fn make(entry: &[(usize, &str, String)]) -> Result<Entry, Refusal> {
    let first = entry[0].0;
    let get = |key: &str| -> Result<(usize, &str), Refusal> {
        entry
            .iter()
            .find(|(_, k, _)| *k == key)
            .map(|(line, _, value)| (*line, value.as_str()))
            .ok_or((first, format!("the entry has no `{key}`")))
    };
    let (_, story) = get("story")?;
    let (room_line, room) = get("room")?;
    let (direction_line, direction) = get("direction")?;
    let (reported_line, reported) = get("reported")?;
    let (outcome_line, outcome) = get("outcome")?;
    let (cause_line, cause) = get("cause")?;
    if cause.trim().is_empty() {
        return Err((cause_line, "a cause must be written".to_string()));
    }
    // A case reads as a report's detail line does.
    let text = format!("{room} {direction} {reported} {outcome}");
    let words: Vec<&str> = text.split(' ').collect();
    let case = match Case::read(&words) {
        Some((case, [])) => case,
        _ => {
            // Say which of the four is not of its form.
            let alone = |value: &str, around: (&str, &str)| {
                let text = format!("{}{value}{}", around.0, around.1);
                let words: Vec<&str> = text.split(' ').collect();
                matches!(Case::read(&words), Some((_, [])))
            };
            let line = if !alone(room, ("", " 1 none stayed")) {
                room_line
            } else if !alone(direction, ("1 ", " none stayed")) {
                direction_line
            } else if !alone(reported, ("1 1 ", " stayed")) {
                reported_line
            } else {
                outcome_line
            };
            return Err((line, "not of the ledger's form".to_string()));
        }
    };
    if matches!(
        case.reported,
        crate::verdict::Reported::Conditional | crate::verdict::Reported::Unknown
    ) {
        return Err((reported_line, "reported is `fixed N` or `none`".to_string()));
    }
    if case.outcome == crate::trial::Outcome::Lost {
        return Err((outcome_line, "a lost trial has no verdict".to_string()));
    }
    if judge(&case) != Verdict::Wrong {
        return Err((
            first,
            "the report and the outcome do not make a wrong verdict".to_string(),
        ));
    }
    Ok(Entry {
        story: story.to_string(),
        case,
        cause: cause.to_string(),
        line: first,
    })
}
