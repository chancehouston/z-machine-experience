//! A story's directions and the word a trial sends for each
//! (specs/005-corpus-testing/research.md R4).

use std::collections::BTreeMap;

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

/// The ordinary English words for the twelve directions of a compass.
const ORDINARY: [&str; 12] = [
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
];

/// Whether a stored word is an ordinary direction word: the word itself, or its
/// start as a dictionary keeps it (six letters in version 3, nine later).
fn ordinary(word: &str) -> bool {
    ORDINARY
        .iter()
        .any(|name| *name == word || (word.len() >= 6 && name.starts_with(word)))
}

/// The directions of a story, in ascending order of number: the distinct
/// direction numbers among its dictionary's words. The word for each is its
/// ordinary English word where it has one, which is what the compass sends;
/// otherwise its longest stored word. Among words equally good, the longest,
/// then the first in alphabetical order. A story whose dictionary cannot be
/// read as a family's has none.
pub fn directions(introspector: &Introspector, view: &View<'_>) -> Vec<Direction> {
    let Ok(words) = introspector.dictionary(view).value else {
        return Vec::new();
    };
    let mut found: BTreeMap<u8, Option<String>> = BTreeMap::new();
    for word in words {
        let Some(number) = word.direction else {
            continue;
        };
        let best = found.entry(number).or_default();
        let rank = |w: &str| (ordinary(w), w.len(), std::cmp::Reverse(w.to_string()));
        let better = match best {
            None => true,
            Some(held) => rank(&word.word) > rank(held),
        };
        if better && !word.word.is_empty() {
            *best = Some(word.word);
        }
    }
    found
        .into_iter()
        .map(|(number, word)| Direction { number, word })
        .collect()
}
