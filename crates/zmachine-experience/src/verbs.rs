//! The story's own words for the commands the layer sends of itself: looking
//! about, examining a thing, dropping it
//! (specs/013-room-look/contracts/look.md, L§1;
//! specs/014-inventory-card/contracts/card.md, L§1).
//!
//! Each word is read from the story's dictionary, and where the dictionary
//! has none there is no word. That the word does what its English says is
//! taken on trust: nothing in a story file says what a verb does.

/// What a screen reader is given for what a touch on the player's place does.
pub const SPOKEN_LOOK: &str = "Look around";

/// The first of `candidates` that the dictionary has, where it keeps `kept`
/// characters of each word. The word is given in full: the game cuts what it
/// reads the same way.
fn find(words: &[String], kept: usize, candidates: &[&str]) -> Option<String> {
    candidates.iter().find_map(|candidate| {
        let cut: String = candidate.chars().take(kept).collect();
        words.contains(&cut).then(|| candidate.to_string())
    })
}

/// The word for looking about.
pub(crate) fn look(words: &[String], kept: usize) -> Option<String> {
    find(words, kept, &["look", "l"])
}

/// The word for examining a thing.
pub(crate) fn examine(words: &[String], kept: usize) -> Option<String> {
    find(words, kept, &["examine", "x"])
}

/// The word for dropping a thing.
pub(crate) fn drop(words: &[String], kept: usize) -> Option<String> {
    find(words, kept, &["drop"])
}
