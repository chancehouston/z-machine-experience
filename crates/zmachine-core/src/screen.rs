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
#[derive(Clone, Debug, Default)]
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
            _ => self.runs.push(StyledRun {
                text: text.to_string(),
                style,
                clears_window: false,
            }),
        }
    }

    pub fn mark_cleared(&mut self) {
        self.runs.push(StyledRun {
            text: String::new(),
            style: Style::default(),
            clears_window: true,
        });
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
        (1..=self.width)
            .map(|c| self.cell(row, c).map_or(' ', |(ch, _)| ch))
            .collect()
    }

    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    fn index(&self, row: u16, column: u16) -> usize {
        (row as usize - 1) * self.width as usize + column as usize - 1
    }

    /// Empties the rows from `first` to `last`, counting from 1.
    fn clear_rows(&mut self, first: u16, last: u16) {
        if first > last {
            return;
        }
        let (from, to) = (self.index(first, 1), self.index(last, self.width) + 1);
        if self.cells[from..to].iter().any(Option::is_some) {
            self.cells[from..to].fill(None);
            self.changed = true;
        }
    }

    fn move_cursor(&mut self, row: u16, column: u16) {
        if self.cursor != (row, column) {
            self.cursor = (row, column);
            self.changed = true;
        }
    }

    /// §8.7.2.3: moves the cursor. A position off the canvas is brought to the
    /// nearest one on it.
    pub(crate) fn set_cursor(&mut self, row: u16, column: u16) {
        self.move_cursor(row.clamp(1, self.rows), column.clamp(1, self.width));
    }

    /// §8.7.2.1: sets the upper window's height, at most the whole screen. Cells
    /// keep their contents; a cursor left below the window goes to its top left.
    pub(crate) fn split(&mut self, height: u16) {
        let height = height.min(self.rows);
        if height != self.height {
            self.height = height;
            self.changed = true;
        }
        if self.cursor.0 > height {
            self.move_cursor(1, 1);
        }
    }

    /// §8.7.2: selecting the upper window puts its cursor at the top left.
    pub(crate) fn select_upper(&mut self) {
        self.move_cursor(1, 1);
    }

    /// §8.7.3.2.1, §8.7.3.3, §15 erase_window.
    pub(crate) fn erase_window(&mut self, which: i16) {
        match which {
            // The lower window: whatever the game drew below its upper window goes.
            0 => self.clear_rows(self.height + 1, self.rows),
            1 => {
                self.clear_rows(1, self.height);
                self.move_cursor(1, 1);
            }
            -1 => {
                self.clear_rows(1, self.rows);
                if self.height != 0 {
                    self.height = 0;
                    self.changed = true;
                }
                self.move_cursor(1, 1);
            }
            -2 => self.clear_rows(1, self.rows),
            _ => {}
        }
    }

    /// §8.7.3.4: empties from the cursor to the end of its row.
    pub(crate) fn erase_line(&mut self) {
        let (row, column) = self.cursor;
        if column > self.width {
            return;
        }
        let (from, to) = (self.index(row, column), self.index(row, self.width) + 1);
        if self.cells[from..to].iter().any(Option::is_some) {
            self.cells[from..to].fill(None);
            self.changed = true;
        }
    }

    /// Writes text at the cursor. A character past the right edge is discarded; a
    /// new-line goes to the start of the next row, or nowhere on the last row.
    pub(crate) fn write(&mut self, text: &str, style: Style) {
        for ch in text.chars() {
            let (row, column) = self.cursor;
            if ch == '\n' {
                if row < self.rows {
                    self.move_cursor(row + 1, 1);
                }
                continue;
            }
            if column > self.width {
                continue;
            }
            let at = self.index(row, column);
            if self.cells[at] != Some((ch, style)) {
                self.cells[at] = Some((ch, style));
            }
            // §8.7.3.1: printing in the last column is legal, and leaves the cursor
            // past the edge.
            self.cursor = (row, column + 1);
            self.changed = true;
        }
    }

    /// Empties whatever the game drew below its upper window (the overlay), which
    /// a terminal would have scrolled away once the player answered.
    pub(crate) fn clear_overlay(&mut self) {
        self.clear_rows(self.height + 1, self.rows);
    }

    /// §8.7.3.3 on restart: blank and unsplit.
    pub(crate) fn reset(&mut self) {
        let blank = Screen::new(self.width, self.rows);
        if self.cells != blank.cells || self.height != 0 || self.cursor != (1, 1) {
            *self = Screen {
                changed: true,
                ..blank
            };
        }
    }
}
