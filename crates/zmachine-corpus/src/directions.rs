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

/// The directions of a story, in ascending order of number: the distinct
/// direction numbers among its dictionary's words, each with its longest stored
/// word, and among words equally long the first in alphabetical order. A story
/// whose dictionary cannot be read as a family's has none.
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
        let better = match best {
            None => true,
            Some(held) => {
                (word.word.len(), std::cmp::Reverse(&word.word))
                    > (held.len(), std::cmp::Reverse(&*held))
            }
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
