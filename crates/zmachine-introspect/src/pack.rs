//! Game packs: the file, and a story file's identity (docs/introspection.md,
//! section 12).

use zmachine_core::View;

/// A game pack as its file states it (specification section 12.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack {
    /// The story file's SHA-256, as 64 lowercase hexadecimal digits.
    pub sha256: String,
    pub release: u16,
    pub serial: [u8; 6],
    /// The attribute set on a door while it is open.
    pub open_attribute: Option<u8>,
    /// Variables whose stored value is not to be relied on, in the file's order.
    pub untrusted_variables: Vec<u8>,
}

/// Why a pack was refused: the line, counted from 1, and the fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackError {
    pub line: usize,
    pub what: &'static str,
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.what)
    }
}

/// What became of the pack a reader was given (specification section 12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackState {
    None,
    InForce,
    NotMatching,
    Refused,
}

impl PackState {
    pub fn as_str(self) -> &'static str {
        match self {
            PackState::None => "none",
            PackState::InForce => "in-force",
            PackState::NotMatching => "not-matching",
            PackState::Refused => "refused",
        }
    }
}

/// The facts of a pack in force, as the adapters use them. Empty when no pack is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Entries {
    pub open_attribute: Option<u8>,
    pub untrusted_variables: Vec<u8>,
}

impl Pack {
    /// Reads a pack's text. Any fault refuses the whole pack.
    pub fn parse(_text: &str) -> Result<Pack, PackError> {
        Err(PackError {
            line: 0,
            what: "not written yet",
        })
    }

    /// Whether the pack states any fact.
    pub fn has_entries(&self) -> bool {
        self.open_attribute.is_some() || !self.untrusted_variables.is_empty()
    }
}

/// Whether a pack's identity is that of the story file as shipped.
pub(crate) fn matches(_pack: &Pack, _view: &View<'_>) -> bool {
    false
}
