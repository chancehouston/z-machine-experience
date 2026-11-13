//! Tests of what is to be spoken for a turn
//! (specs/010-screen-reader-announcements/contracts/announcements.md, cited
//! N§).

use crate::announce::{NOTE_ENDED, announcement, reply};

// N§1: the white space at the end goes, then the prompt the game ends a turn
// with when it waits for a line, then the white space at both ends.
#[test]
fn reply_is_the_text_without_its_prompt() {
    assert_eq!(reply("Kitchen\nA table.\n\n>", true), "Kitchen\nA table.");
    assert_eq!(reply("Sure? >", true), "Sure?");
    assert_eq!(reply("\nTaken.\n\n> \n", true), "Taken.");
}

// N§1: where the game is not waiting for a line, nothing is taken for a prompt.
#[test]
fn reply_keeps_a_final_mark_where_no_line_is_waited_for() {
    assert_eq!(reply("Press a key >", false), "Press a key >");
    assert_eq!(reply("The end.\n", false), "The end.");
}

// N§1: one prompt is removed, and no other character of the game's text.
#[test]
fn reply_changes_nothing_else() {
    assert_eq!(reply("a > b\n>", true), "a > b");
    assert_eq!(reply("x >>", true), "x >");
    assert_eq!(
        reply("two  spaces\tand a tab\n>", true),
        "two  spaces\tand a tab"
    );
}

// N§1, 8: a turn that is only a prompt has nothing to say.
#[test]
fn reply_of_a_bare_prompt_is_empty() {
    assert_eq!(reply("\n>", true), "");
    assert_eq!(reply(">", true), "");
    assert_eq!(reply("", true), "");
    assert_eq!(announcement("", None, false), None);
}

// N§2, 3, 6: the parts in order, each on a line of its own.
#[test]
fn the_parts_are_joined_in_order() {
    assert_eq!(announcement("Attic", None, false).as_deref(), Some("Attic"));
    assert_eq!(
        announcement("Pitch black.", Some("It is dark"), false).as_deref(),
        Some("Pitch black.\nIt is dark")
    );
    assert_eq!(
        announcement("Bye.", None, true),
        Some(format!("Bye.\n{NOTE_ENDED}"))
    );
    assert_eq!(
        announcement("", Some("It is dark"), false).as_deref(),
        Some("It is dark")
    );
    assert_eq!(announcement("", None, true).as_deref(), Some(NOTE_ENDED));
}
