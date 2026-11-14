//! The story's own words for the commands the layer sends
//! (specs/013-room-look/contracts/look.md, L§1;
//! specs/014-inventory-card/contracts/card.md, L§1).

use crate::verbs;

fn dictionary(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

#[test]
fn the_long_word_is_found_first() {
    let words = dictionary(&["l", "lamp", "look", "north"]);
    assert_eq!(verbs::look(&words, 6).as_deref(), Some("look"));
}

#[test]
fn the_short_word_is_found_where_the_long_is_not() {
    let words = dictionary(&["l", "lamp", "north"]);
    assert_eq!(verbs::look(&words, 6).as_deref(), Some("l"));
}

#[test]
fn a_dictionary_with_neither_has_no_word() {
    assert_eq!(verbs::look(&dictionary(&["lamp", "north"]), 6), None);
    assert_eq!(verbs::look(&[], 9), None);
}

// A word is looked for as the dictionary keeps it, cut to its length, and is
// sent in full: the game cuts it the same way.
#[test]
fn a_word_is_found_cut_to_what_the_dictionary_keeps() {
    assert_eq!(
        verbs::look(&dictionary(&["loo"]), 3).as_deref(),
        Some("look")
    );
    assert_eq!(verbs::look(&dictionary(&["loo"]), 6), None);
    let words = dictionary(&["examin", "x"]);
    assert_eq!(verbs::examine(&words, 6).as_deref(), Some("examine"));
}

// Feature 014, L§1: examining is `examine`, then `x`; dropping is `drop`.
#[test]
fn the_words_for_examining_and_dropping() {
    let words = dictionary(&["drop", "examine", "x"]);
    assert_eq!(verbs::examine(&words, 9).as_deref(), Some("examine"));
    assert_eq!(verbs::Acts::of(&words, 9).drop.as_deref(), Some("drop"));
    let words = dictionary(&["x"]);
    assert_eq!(verbs::examine(&words, 9).as_deref(), Some("x"));
    assert_eq!(verbs::Acts::of(&words, 9).drop, None);
}
