//! Game packs in the corpus: finding and checking them, and the proof of each
//! entry by trials (specs/006-game-packs/research.md R9, R11).

use std::path::Path;

use zmachine_introspect::Pack;

use crate::manifest::Story;
use crate::report::Report;

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
pub fn of(_root: &Path, _story: &Story) -> Option<Found> {
    None
}

/// Checks every file under `packs/` (research R9). Returns the failures, each
/// as `file:line: why` or `file: why`, and notes that are not failures.
pub fn check(_root: &Path, _stories: &[Story]) -> (Vec<String>, Vec<String>) {
    (Vec::new(), Vec::new())
}

/// A pack's text, with one untrusted variable left out if asked.
pub fn text(_pack: &Pack, _without: Option<u8>) -> String {
    String::new()
}

/// Whether the run with the pack proves its `open-attribute` entry.
pub fn prove_open(_with: &Report) -> Proof {
    Proof::NotProven
}

/// Whether variable `N` is proven untrusted: `with` is the run with the whole
/// pack, `without` the run with every entry but this one.
pub fn prove_variable(_with: &Report, _without: &Report) -> Proof {
    Proof::NotProven
}
