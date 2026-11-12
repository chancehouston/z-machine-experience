//! Undo positions (§15 save_undo, restore_undo; data-model.md "Undo position").

use crate::exec::Frame;

/// A position the game can return to: dynamic memory, the call stack, and the
/// address of the `save_undo` instruction's store byte. Not the random state, the
/// screen or the output streams.
#[derive(Clone, Debug)]
pub(crate) struct Position {
    pub dynamic: Vec<u8>,
    pub frames: Vec<Frame>,
    pub pc: u32,
}

/// The most recent positions, newest last, up to a limit the host sets.
#[derive(Clone, Debug, Default)]
pub(crate) struct History {
    positions: Vec<Position>,
}

impl History {
    /// Records a position, dropping the oldest if `limit` is reached.
    pub fn push(&mut self, position: Position, limit: usize) {
        if self.positions.len() >= limit {
            self.positions.remove(0);
        }
        self.positions.push(position);
    }

    pub fn pop(&mut self) -> Option<Position> {
        self.positions.pop()
    }
}
