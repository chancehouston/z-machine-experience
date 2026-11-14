//! The story's word for looking about
//! (specs/013-room-look/contracts/look.md, L§1).
//!
//! The word is read from the story's own dictionary, and where the dictionary
//! has none there is no word. That the word looks about is taken on trust:
//! nothing in a story file says what a verb does.

/// The words looked for, in order: the first the dictionary has is used.
const CANDIDATES: [&str; 2] = ["look", "l"];

/// What a screen reader is given for what a touch on the player's place does.
pub const SPOKEN_LOOK: &str = "Look around";

/// The word for looking among a dictionary's `words`, which keeps `kept`
/// characters of each.
pub(crate) fn word(words: &[String], kept: usize) -> Option<String> {
    CANDIDATES.iter().find_map(|candidate| {
        let cut: String = candidate.chars().take(kept).collect();
        words.contains(&cut).then_some(cut)
    })
}
