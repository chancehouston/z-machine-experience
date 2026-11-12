//! A story's directions and the word a trial sends for each
//! (specs/005-corpus-testing/research.md R4).

use zmachine_core::View;
use zmachine_introspect::Introspector;

/// One direction a story has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Direction {
    /// The direction's number, as the introspection interface gives it.
    pub number: u8,
    /// The word a trial sends: none when the dictionary has no word for it.
    pub word: Option<String>,
}

/// The directions of a story, in ascending order of number.
pub fn directions(_introspector: &Introspector, _view: &View<'_>) -> Vec<Direction> {
    Vec::new()
}
