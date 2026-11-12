//! Writing styles and the game's canvas to a terminal
//! (specs/002-v5-v8/contracts/cli.md, `--style` and `--upper`).
//!
//! Styled text has to be wrapped like any other, and an escape code has no width.
//! So a change of style travels through the wrapper as a single private-use
//! character, which the wrapper counts as taking no room, and is turned into an
//! escape code only when the text is written.

use zmachine_core::{Screen, Style};

/// The first of sixteen private-use characters, one for each combination of styles.
const MARK_BASE: u32 = 0xE000;

fn bits(style: Style) -> u32 {
    style.reverse as u32 | (style.bold as u32) << 1 | (style.italic as u32) << 2
}

/// The character that stands for a change to `style`. Fixed pitch has no escape
/// code: a terminal is fixed pitch already.
pub fn marker(style: Style) -> char {
    char::from_u32(MARK_BASE + bits(style)).expect("a private-use character")
}

pub fn is_marker(ch: char) -> bool {
    (MARK_BASE..MARK_BASE + 8).contains(&(ch as u32))
}

/// The escape code that sets exactly these styles.
fn code(bits: u32) -> String {
    let mut code = String::from("\x1b[0");
    for (bit, number) in [(2, 1), (4, 3), (1, 7)] {
        if bits & bit != 0 {
            code.push_str(&format!(";{number}"));
        }
    }
    code.push('m');
    code
}

/// Turns each marker in wrapped text into its escape code.
pub fn render(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if is_marker(ch) {
            out.push_str(&code(ch as u32 - MARK_BASE));
        } else {
            out.push(ch);
        }
    }
    out
}

/// The escape code that turns every style off.
pub const RESET: &str = "\x1b[0m";

/// The rows of the canvas the game has written on, each cut after its last written
/// cell, for printing above the prompt. Empty if the game has drawn nothing.
pub fn canvas(screen: &Screen, ansi: bool) -> String {
    let mut out = String::new();
    for row in 1..=screen.rows() {
        let Some(last) = (1..=screen.width()).rev().find(|&c| screen.cell(row, c).is_some()) else {
            continue;
        };
        let mut current = 0;
        for column in 1..=last {
            let (ch, style) = screen.cell(row, column).unwrap_or((' ', Style::default()));
            if ansi && bits(style) != current {
                current = bits(style);
                out.push_str(&code(current));
            }
            out.push(ch);
        }
        if current != 0 {
            out.push_str(RESET);
        }
        out.push('\n');
    }
    out
}
