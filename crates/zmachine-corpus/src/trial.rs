//! One direction tried on a copy of a game
//! (specs/005-corpus-testing/research.md R1, R2).

use zmachine_core::Machine;
use zmachine_introspect::Introspector;

/// The most instructions a copy may run before it is counted as a fault.
pub const LIMIT: u64 = 50_000_000;

/// What a copy did when given a direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Outcome {
    /// At a command prompt, in this room, which is not the room it started in.
    Moved(u16),
    /// At a command prompt, in the same room.
    Stayed,
    /// Waiting for something that is not a command.
    Question,
    /// The game quit.
    Ended,
    /// The machine faulted, or ran past the limit without stopping.
    Fault,
    /// At a command prompt, but the location is unknown before or after.
    Lost,
}

/// Tries `word` on a copy of `machine`, which must be waiting at a command
/// prompt, and returns what the copy did, and the copy.
pub fn trial(
    machine: &Machine,
    _introspector: &Introspector,
    _command_address: u32,
    _word: &str,
    _limit: u64,
) -> (Outcome, Machine) {
    (Outcome::Lost, machine.clone())
}
