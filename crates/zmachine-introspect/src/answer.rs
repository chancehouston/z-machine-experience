//! The form of every answer (docs/introspection.md section 2).

/// Why an answer, or a part of one, is not known.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unknown {
    /// A family-dependent query on a story file of neither family.
    UnrecognisedFamily,
    /// The player cannot be found, or is not placed anywhere.
    NoPlayer,
    /// More than one object fits the family's rule for the player.
    AmbiguousPlayer,
    /// Two stored facts that should agree do not.
    Inconsistent,
    /// The story file's contents do not settle it.
    NotDeterminable,
    /// A table could not be read.
    Damaged,
}

impl Unknown {
    pub const ALL: [Unknown; 6] = [
        Unknown::UnrecognisedFamily,
        Unknown::NoPlayer,
        Unknown::AmbiguousPlayer,
        Unknown::Inconsistent,
        Unknown::NotDeterminable,
        Unknown::Damaged,
    ];

    /// The reason as the specification writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Unknown::UnrecognisedFamily => "unrecognised-family",
            Unknown::NoPlayer => "no-player",
            Unknown::AmbiguousPlayer => "ambiguous-player",
            Unknown::Inconsistent => "inconsistent",
            Unknown::NotDeterminable => "not-determinable",
            Unknown::Damaged => "damaged",
        }
    }
}

/// Whether an answer is the fact or an approximation to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Exactness {
    Exact,
    Approximate,
}

impl Exactness {
    pub fn as_str(self) -> &'static str {
        match self {
            Exactness::Exact => "exact",
            Exactness::Approximate => "approximate",
        }
    }
}

/// A value that can be unknown on its own, within an answer.
pub type Part<T> = Result<T, Unknown>;

/// A yes-or-no fact that the family may not record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

impl Tri {
    pub fn as_str(self) -> &'static str {
        match self {
            Tri::Yes => "yes",
            Tri::No => "no",
            Tri::Unknown => "unknown",
        }
    }

    pub fn from_bool(value: bool) -> Tri {
        if value { Tri::Yes } else { Tri::No }
    }
}

/// What a query returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer<T> {
    pub value: Part<T>,
    pub exactness: Exactness,
}

impl<T> Answer<T> {
    pub fn exact(value: T) -> Self {
        Answer { value: Ok(value), exactness: Exactness::Exact }
    }

    pub fn approximate(value: T) -> Self {
        Answer { value: Ok(value), exactness: Exactness::Approximate }
    }

    /// An answer that is not known. `exactness` says what it would have been.
    pub fn unknown(reason: Unknown, exactness: Exactness) -> Self {
        Answer { value: Err(reason), exactness }
    }

    pub fn is_unknown(&self) -> bool {
        self.value.is_err()
    }
}
