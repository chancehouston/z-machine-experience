//! The ledger: wrong reports that are known, each with its cause
//! (specs/005-corpus-testing/contracts/formats.md, "The ledger").

use crate::report::{Case, Refusal};

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
    pub fn knows(&self, _story: &str, _case: &Case) -> bool {
        false
    }
}

/// Reads a ledger. Filled in by T023; until then a ledger holds nothing.
pub fn read(_text: &str) -> Result<Ledger, Refusal> {
    Ok(Ledger::default())
}
