//! What a version 5 or 8 game shows: styled text for the lower window, and a
//! canvas the size of the screen on which the game draws its upper window
//! (specs/002-v5-v8/contracts/screen-model.md).

/// The styles a game can apply to text (§8.7.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub reverse: bool,
    pub bold: bool,
    pub italic: bool,
    pub fixed_pitch: bool,
}

impl Style {
    /// Adds the styles named by a `set_text_style` operand: 1 reverse, 2 bold,
    /// 4 italic, 8 fixed pitch. An operand of 0 clears them all (§15).
    pub(crate) fn apply(&mut self, operand: u16) {
        if operand == 0 {
            *self = Style::default();
            return;
        }
        self.reverse |= operand & 1 != 0;
        self.bold |= operand & 2 != 0;
        self.italic |= operand & 4 != 0;
        self.fixed_pitch |= operand & 8 != 0;
    }
}

/// A piece of lower-window output: text in one style, or a marker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyledRun {
    pub text: String,
    pub style: Style,
    /// The game cleared the lower window at this point. Such a run has no text.
    pub clears_window: bool,
}

/// The lower window's output since it was last drained.
#[derive(Debug, Default)]
pub(crate) struct OutputBuffer {
    runs: Vec<StyledRun>,
}

impl OutputBuffer {
    /// Adds text, joining it to the last run when the style is the same.
    pub fn push(&mut self, text: &str, style: Style) {
        if text.is_empty() {
            return;
        }
        match self.runs.last_mut() {
            Some(last) if last.style == style && !last.clears_window => last.text.push_str(text),
            _ => self.runs.push(StyledRun { text: text.to_string(), style, clears_window: false }),
        }
    }

    pub fn mark_cleared(&mut self) {
        self.runs.push(StyledRun { text: String::new(), style: Style::default(), clears_window: true });
    }

    pub fn take_runs(&mut self) -> Vec<StyledRun> {
        std::mem::take(&mut self.runs)
    }

    /// The text alone, as version 3's host received it.
    pub fn take_text(&mut self) -> String {
        self.take_runs().into_iter().map(|run| run.text).collect()
    }
}

/// The canvas a game draws its upper window on: `rows` by `width` cells, of which
/// the top `upper_height` are the upper window. Cells the game wrote below that are
/// an overlay, emptied when the game is next given input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    width: u16,
    rows: u16,
    height: u16,
    cells: Vec<Option<(char, Style)>>,
    /// Row and column, from 1.
    cursor: (u16, u16),
    changed: bool,
}

impl Screen {
    pub(crate) fn new(width: u16, rows: u16) -> Screen {
        let (width, rows) = (width.max(1), rows.max(1));
        Screen {
            width,
            rows,
            height: 0,
            cells: vec![None; width as usize * rows as usize],
            cursor: (1, 1),
            changed: false,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// How many rows at the top are the upper window.
    pub fn upper_height(&self) -> u16 {
        self.height
    }

    /// The cursor's row and column, counting from 1.
    pub fn cursor(&self) -> (u16, u16) {
        self.cursor
    }

    /// What the game wrote at a row and column, counting from 1, or `None` if it has
    /// written nothing there.
    pub fn cell(&self, row: u16, column: u16) -> Option<(char, Style)> {
        if row == 0 || column == 0 || row > self.rows || column > self.width {
            return None;
        }
        self.cells[(row as usize - 1) * self.width as usize + column as usize - 1]
    }

    /// The text of a row, with spaces for cells the game has not written.
    pub fn row_text(&self, row: u16) -> String {
        (1..=self.width).map(|c| self.cell(row, c).map_or(' ', |(ch, _)| ch)).collect()
    }

    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    // The canvas operations arrive with User Story 3; until then they do nothing
    // beyond keeping the cursor, which `print_table` reads.

    pub(crate) fn set_cursor(&mut self, row: u16, column: u16) {
        self.cursor = (row, column);
    }

    pub(crate) fn split(&mut self, _height: u16) {}

    pub(crate) fn select_upper(&mut self) {}

    pub(crate) fn erase_window(&mut self, _which: i16) {}

    pub(crate) fn erase_line(&mut self) {}

    pub(crate) fn write(&mut self, _text: &str, _style: Style) {}

    pub(crate) fn clear_overlay(&mut self) {}

    pub(crate) fn reset(&mut self) {}
}
