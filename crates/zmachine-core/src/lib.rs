//! A faithful Z-machine interpreter core.
//!
//! The core performs no input or output of its own, uses no `unsafe`, and has no
//! dependencies. A host drives it as a resumable state machine: call
//! [`Machine::run`], receive an [`Event`], answer it, and call `run` again.
//!
//! Section numbers ("§") in this crate refer to the Z-Machine Standards Document
//! 1.1, which defines correct behavior.

#![forbid(unsafe_code)]

mod decode;
mod header;
mod memory;

use header::Header;
use memory::Memory;

/// Why a story file was refused at load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The file's version byte is not one this interpreter runs.
    UnsupportedVersion(u8),
    /// The file is too short to hold a header.
    TooShort,
    /// A header field points outside the file or breaks a rule of §11.
    BadHeader(HeaderField),
}

/// The header field a [`LoadError::BadHeader`] refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderField {
    HighMemoryBase,
    InitialProgramCounter,
    Dictionary,
    ObjectTable,
    Globals,
    StaticMemoryBase,
    Abbreviations,
}

/// The host called the machine out of turn. Nothing was changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostError {
    /// `run` was called while an answer to the previous event is owed.
    AnswerOwed,
    /// An answer was given that the machine is not waiting for.
    NotWaitingForThis,
    /// The game has finished or faulted; the machine accepts no further calls.
    Stopped,
}

/// The kind of illegal operation that stopped a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    UnknownInstruction,
    DivisionByZero,
    StackUnderflow,
    CallStackOverflow,
    ReadOutsideFile,
    WriteOutsideDynamicMemory,
    MissingProperty,
    InvalidVariable,
    TextOverrun,
}

/// An illegal operation by the story file. The game cannot continue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    /// Address of the instruction that caused it.
    pub address: u32,
}

/// The kind of illegal operation the core tolerates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// An instruction was applied to object 0; it did nothing or yielded 0.
    ObjectZero,
}

/// A tolerated illegal operation, reported to the host and not to the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    /// Address of the instruction that caused it.
    pub address: u32,
}

/// What the right-hand side of a version 3 status line shows (§8.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Score { score: i16, turns: u16 },
    Time { hours: u16, minutes: u16 },
}

/// The content of the status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    /// Short name of the player's current location.
    pub location: String,
    pub kind: StatusKind,
}

/// What [`Machine::run`] returns: the one thing the host must deal with next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The game is waiting for a line of input of at most `max_len` characters.
    /// Answer with [`Machine::provide_line`].
    LineWanted { max_len: usize },
    /// The game wants to save; the bytes are a complete Quetzal file.
    /// Answer with [`Machine::save_result`].
    SaveRequested(Vec<u8>),
    /// The game wants to restore. Answer with [`Machine::provide_restore`].
    RestoreRequested,
    /// The game turned transcription on. Answer with
    /// [`Machine::transcript_result`].
    TranscriptRequested,
    /// The game quit. The machine accepts no further calls.
    Finished,
    /// The game performed an illegal operation. The machine accepts no further
    /// calls.
    Fault(Fault),
    /// The instruction budget ran out. Call `run` again to continue.
    BudgetExhausted,
}

/// Options for [`Machine::load`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadOptions {
    /// A fixed random seed, 0 to 2147483647, for repeatable runs.
    pub seed: Option<u32>,
    /// Used when `seed` is `None`: a value from the host's own source of
    /// randomness. The core never reads a clock.
    pub entropy: u32,
}

/// A loaded story file and everything that changes while it runs.
#[derive(Debug)]
pub struct Machine {
    #[allow(dead_code)]
    story: Vec<u8>,
    #[allow(dead_code)]
    header: Header,
    #[allow(dead_code)]
    memory: Memory,
    #[allow(dead_code)]
    options: LoadOptions,
}

impl Machine {
    /// Loads a version 3 story file.
    ///
    /// A header length or checksum that does not match the file is not an error
    /// here; the game's own `verify` reports it.
    pub fn load(story: Vec<u8>, options: LoadOptions) -> Result<Machine, LoadError> {
        let header = header::parse(&story)?;
        let mut memory = Memory::new(story.clone(), header.static_base as usize);
        header::apply_interpreter_fields(memory.dynamic_mut());
        Ok(Machine { story, header, memory, options })
    }

    /// Executes at most `budget` instructions and returns the next event.
    pub fn run(&mut self, _budget: u64) -> Result<Event, HostError> {
        Err(HostError::Stopped)
    }

    /// Answers [`Event::LineWanted`].
    pub fn provide_line(&mut self, _text: &str) -> Result<(), HostError> {
        Err(HostError::NotWaitingForThis)
    }

    /// Answers [`Event::SaveRequested`]: whether the host stored the bytes.
    pub fn save_result(&mut self, _stored: bool) -> Result<(), HostError> {
        Err(HostError::NotWaitingForThis)
    }

    /// Answers [`Event::RestoreRequested`] with a saved position, or `None` to
    /// decline.
    pub fn provide_restore(&mut self, _saved: Option<&[u8]>) -> Result<(), HostError> {
        Err(HostError::NotWaitingForThis)
    }

    /// Answers [`Event::TranscriptRequested`].
    pub fn transcript_result(&mut self, _accepted: bool) -> Result<(), HostError> {
        Err(HostError::NotWaitingForThis)
    }

    /// Screen text printed since the last call, unwrapped.
    pub fn take_output(&mut self) -> String {
        String::new()
    }

    /// Transcript text produced since the last call.
    pub fn take_transcript(&mut self) -> String {
        String::new()
    }

    /// The status line, if it changed since the last call.
    pub fn take_status(&mut self) -> Option<StatusLine> {
        None
    }

    /// Tolerated illegal operations recorded since the last call.
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        Vec::new()
    }

    /// The byte at `address` in the machine's memory, for tests of this crate.
    #[cfg(test)]
    pub(crate) fn peek(&self, address: u32) -> u8 {
        self.memory.read_byte(address).expect("address inside the file")
    }
}
