//! Game packs in the corpus: finding and checking them, and the proof of each
//! entry by trials (specs/006-game-packs/research.md R9, R11).

use std::path::Path;

use std::collections::BTreeSet;

use zmachine_introspect::Pack;

use crate::ledger::Ledger;
use crate::manifest::Story;
use crate::report::Report;
use crate::verdict::{Reported, Verdict, judge};

pub const PACKS: &str = "packs";

/// The outcome of testing one entry of a pack by trials.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proof {
    Proven,
    Contradicted,
    NotProven,
    Unneeded,
}

impl Proof {
    pub fn as_str(self) -> &'static str {
        match self {
            Proof::Proven => "proven",
            Proof::Contradicted => "contradicted",
            Proof::NotProven => "not-proven",
            Proof::Unneeded => "unneeded",
        }
    }
}

/// A pack found under `packs/`.
#[derive(Clone, Debug)]
pub struct Found {
    /// The file's name without `.pack`: its story's id.
    pub id: String,
    pub text: String,
    pub pack: Pack,
}

/// The pack for a story, if `packs/` holds one that reads.
pub fn of(root: &Path, story: &Story) -> Option<Found> {
    let text = std::fs::read_to_string(root.join(PACKS).join(format!("{}.pack", story.id))).ok()?;
    let pack = Pack::parse(&text).ok()?;
    Some(Found {
        id: story.id.clone(),
        text,
        pack,
    })
}

/// Checks every file under `packs/` whose name ends in `.pack` (research R9).
/// Returns the failures, each as `file:line: why` or `file: why`, and notes
/// that are not failures: the ledger's entries on answers that rest on a pack,
/// for a person to read their causes (FR-019).
pub fn check(root: &Path, stories: &[Story], ledger: &Ledger) -> (Vec<String>, Vec<String>) {
    let mut names: Vec<String> = std::fs::read_dir(root.join(PACKS))
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                .filter_map(|name| name.strip_suffix(".pack").map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let mut failures = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for name in names {
        let file = format!("{PACKS}/{name}.pack");
        let text = match std::fs::read_to_string(root.join(&file)) {
            Ok(text) => text,
            Err(e) => {
                failures.push(format!("{file}: {e}"));
                continue;
            }
        };
        let pack = match Pack::parse(&text) {
            Ok(pack) => pack,
            Err(error) => {
                failures.push(format!("{file}:{}: {}", error.line, error.what));
                continue;
            }
        };
        let mut fail = |why: String| failures.push(format!("{file}: {why}"));
        if seen.contains(&pack.sha256) {
            fail("another pack names the same story".to_string());
        }
        seen.push(pack.sha256.clone());
        let Some(story) = stories.iter().find(|s| s.sha256 == pack.sha256) else {
            fail("its digest is no story's in the manifest".to_string());
            continue;
        };
        if story.id != name {
            fail(format!("its story is `{}`, which names its pack", story.id));
        }
        if pack.has_entries() && story.family != "zil" {
            fail(format!(
                "an entry, for a story of family {}: both kinds are for zil",
                story.family
            ));
        }
        if let Some(bytes) = crate::corpus::present(root, story) {
            let release = u16::from_be_bytes([bytes[2], bytes[3]]);
            if release != pack.release {
                fail(format!("the story's release is {release}"));
            }
            if bytes[0x12..0x18] != pack.serial {
                fail(format!(
                    "the story's serial is {}",
                    String::from_utf8_lossy(&bytes[0x12..0x18])
                ));
            }
        }
    }
    let notes = ledger
        .entries
        .iter()
        .filter(|entry| entry.case.reported.marked())
        .map(|entry| {
            format!(
                "{}:{}: on an answer that rests on a pack: {} {}: {}",
                crate::corpus::LEDGER,
                entry.line,
                entry.story,
                entry.case.text(),
                entry.cause
            )
        })
        .collect();
    (failures, notes)
}

/// A pack's text, with one untrusted variable left out if asked.
pub fn text(pack: &Pack, without: Option<u8>) -> String {
    let mut out = format!(
        "pack: 1\nsha256: {}\nrelease: {}\nserial: {}\n",
        pack.sha256,
        pack.release,
        String::from_utf8_lossy(&pack.serial)
    );
    if let Some(attribute) = pack.open_attribute {
        out.push_str(&format!("open-attribute: {attribute}\n"));
    }
    for variable in &pack.untrusted_variables {
        if Some(*variable) != without {
            out.push_str(&format!("untrusted-variable: {variable}\n"));
        }
    }
    out
}

/// Whether the run with the pack proves its `open-attribute` entry: no wrong
/// report on a door without a cause in the ledger, and a door seen both open
/// and closed (FR-017).
pub fn prove_open(with: &Report) -> Proof {
    let m = &with.marked;
    if m.open_wrong_unexplained + m.closed_wrong_unexplained > 0 {
        Proof::Contradicted
    } else if m.open_confirmed > 0 && m.closed_confirmed > 0 {
        Proof::Proven
    } else {
        Proof::NotProven
    }
}

/// Whether a variable is proven untrusted (FR-018): `with` is the run with the
/// whole pack, `without` the run with every entry but this one. The exits that
/// hang on the variable are those distrusted in the one and not in the other;
/// the entry is needed if any of them is reported wrongly without it.
pub fn prove_variable(with: &Report, without: &Report) -> Proof {
    let distrusted = |report: &Report| -> BTreeSet<(u16, u8)> {
        report
            .detail
            .iter()
            .filter(|(case, _)| case.reported == Reported::Distrusted)
            .map(|(case, _)| (case.room, case.direction))
            .collect()
    };
    let (all, others) = (distrusted(with), distrusted(without));
    let needed = without.detail.iter().any(|(case, _)| {
        let exit = (case.room, case.direction);
        all.contains(&exit) && !others.contains(&exit) && judge(case) == Verdict::Wrong
    });
    if needed {
        Proof::Proven
    } else {
        Proof::Unneeded
    }
}
