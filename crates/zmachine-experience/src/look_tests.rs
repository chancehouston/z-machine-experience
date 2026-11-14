//! The word for looking (specs/013-room-look/contracts/look.md, L§1).

use crate::look;

fn dictionary(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

#[test]
fn the_long_word_is_found_first() {
    let words = dictionary(&["l", "lamp", "look", "north"]);
    assert_eq!(look::word(&words, 6).as_deref(), Some("look"));
}

#[test]
fn the_short_word_is_found_where_the_long_is_not() {
    let words = dictionary(&["l", "lamp", "north"]);
    assert_eq!(look::word(&words, 6).as_deref(), Some("l"));
}

#[test]
fn a_dictionary_with_neither_has_no_word() {
    assert_eq!(look::word(&dictionary(&["lamp", "north"]), 6), None);
    assert_eq!(look::word(&[], 9), None);
}

// A word is looked for as the dictionary keeps it: cut to its length.
#[test]
fn a_word_is_found_cut_to_what_the_dictionary_keeps() {
    assert_eq!(look::word(&dictionary(&["loo"]), 3).as_deref(), Some("loo"));
    assert_eq!(look::word(&dictionary(&["loo"]), 6), None);
}
