//! What a screen reader is to speak for a turn
//! (specs/010-screen-reader-announcements/contracts/announcements.md, cited
//! N§; research.md R1 to R4). The words are the game's and the layer's; a
//! host posts them and composes none.

/// The layer's words for a game that has ended, shown beneath the transcript
/// and spoken after the last reply.
pub const NOTE_ENDED: &str = "The game has ended.";

/// A turn's text as it is to be spoken: without the white space at its end,
/// without the prompt the game ends it with when it waits for a line, and
/// without white space at either end (N§1). Nothing else is changed.
pub(crate) fn reply(text: &str, waits_for_line: bool) -> &str {
    let text = text.trim_end();
    let text = match text.strip_suffix('>') {
        Some(before) if waits_for_line => before,
        _ => text,
    };
    text.trim()
}

/// The announcement for a turn: the reply, then the note of darkness if the
/// turn fell dark, then the words for an ended game if it ended, each on a
/// line of its own; or nothing (N§2 to 8).
pub(crate) fn announcement(reply: &str, fell_dark: Option<&str>, ended: bool) -> Option<String> {
    let parts: Vec<&str> = [
        Some(reply).filter(|r| !r.is_empty()),
        fell_dark,
        ended.then_some(NOTE_ENDED),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!parts.is_empty()).then(|| parts.join("\n"))
}
