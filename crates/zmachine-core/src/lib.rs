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
mod dictionary;
mod exec;
mod header;
mod inspect;
mod memory;
mod objects;
mod ops_io;
mod ops_math;
mod ops_object;
mod ops_table;
mod quetzal;
mod random;
mod screen;
mod text;
mod undo;
mod version;

#[cfg(test)]
mod testing;

use exec::{Frame, State};
use header::Header;
use memory::{Memory, MemoryError};
use random::Random;
pub use inspect::View;
pub use screen::{Screen, Style, StyledRun};

/// Why a story file was refused at load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The file's version byte is not one this interpreter runs.
    UnsupportedVersion(u8),
    /// The file is too short to hold a header.
    TooShort,
    /// A header field points outside the file or breaks a rule of §11.
    BadHeader(HeaderField),
    /// The file is larger than its version allows (§1.1.4).
    TooLarge,
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
    TerminatingCharacters,
    AlphabetTable,
    HeaderExtension,
    UnicodeTable,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::UnsupportedVersion(v) => write!(
                f,
                "its header says version {v}, and this interpreter runs versions 3, 5 and 8"
            ),
            LoadError::TooLarge => write!(f, "larger than a story file of its version may be"),
            LoadError::TooShort => write!(f, "too short to be a story file"),
            LoadError::BadHeader(field) => {
                write!(
                    f,
                    "damaged story file: the header's {field:?} address is outside the file"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// The host called the machine out of turn. Nothing was changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostError {
    /// `run` was called while an answer to the previous event is owed.
    AnswerOwed,
    /// An answer was given that the machine is not waiting for.
    NotWaitingForThis,
    /// The game has finished or faulted; the machine accepts no further calls.
    Stopped,
    /// The key has no code in the game's character set. The machine still waits.
    UnrepresentableKey,
    /// The key is not one the game listed as ending a line. The machine still waits.
    NotATerminator,
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
    /// `throw` named a call frame that no longer exists (versions 5 and 8).
    InvalidFrame,
    /// Output was redirected into memory more than sixteen deep (§7.1.2.1.1).
    StreamNesting,
}

impl std::fmt::Display for FaultKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FaultKind::UnknownInstruction => "unknown instruction",
            FaultKind::DivisionByZero => "division by zero",
            FaultKind::StackUnderflow => "stack underflow",
            FaultKind::CallStackOverflow => "call stack overflow",
            FaultKind::ReadOutsideFile => "read outside the story file",
            FaultKind::WriteOutsideDynamicMemory => "write outside dynamic memory",
            FaultKind::MissingProperty => "property not present on the object",
            FaultKind::InvalidVariable => "variable does not exist",
            FaultKind::TextOverrun => "text runs past the end of the story file",
            FaultKind::InvalidFrame => "throw to a call frame that does not exist",
            FaultKind::StreamNesting => "output redirected into memory more than sixteen deep",
        })
    }
}

/// An illegal operation by the story file. The game cannot continue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    /// Address of the instruction that caused it.
    pub address: u32,
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:#x}", self.kind, self.address)
    }
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
    /// The game is waiting for a single keypress (versions 5 and 8). Answer with
    /// [`Machine::provide_key`].
    KeyWanted,
}

/// A single keypress for a game that asked for one (§3.8.4, §10.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A character. It must be one the game's character set can represent.
    Char(char),
    Return,
    Delete,
    Escape,
    Up,
    Down,
    Left,
    Right,
    /// A function key, 1 to 12.
    Function(u8),
    /// A keypad digit, 0 to 9.
    Keypad(u8),
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

/// What a host tells a version 5 or 8 game about itself. None of it affects a
/// version 3 game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Screen width in characters.
    pub screen_columns: u16,
    /// Screen height in lines.
    pub screen_rows: u16,
    /// The interpreter number of §11.1.3, which games may read.
    pub interpreter_number: u8,
    /// The interpreter version, a letter games may print.
    pub interpreter_version: u8,
    /// How many undo positions are kept. With 0, games are told undo is unavailable.
    pub undo_slots: usize,
}

impl Default for Settings {
    /// An 80 by 24 screen, interpreter number 1 and version `Z`, and the 25 undo
    /// positions the reference interpreter keeps.
    fn default() -> Self {
        Settings {
            screen_columns: 80,
            screen_rows: 24,
            interpreter_number: 1,
            interpreter_version: b'Z',
            undo_slots: 25,
        }
    }
}

/// A loaded story file and everything that changes while it runs.
#[derive(Debug)]
pub struct Machine {
    /// The story file as loaded, kept for restart, `verify` and save compression.
    story: Vec<u8>,
    header: Header,
    settings: Settings,
    memory: Memory,
    /// The character tables in force: the defaults, or the game's own.
    charset: text::Charset,
    /// Call frames, innermost last. The first is the main code, which is not a
    /// routine and cannot be returned from.
    frames: Vec<Frame>,
    /// Address of the next instruction.
    pc: u32,
    /// Address of the instruction being executed, for faults and diagnostics.
    current: u32,
    state: State,
    /// A fault found while handling an answer, reported by the next `run`.
    pending_fault: Option<Fault>,
    /// The `save` or `restore` instruction waiting for the host's answer.
    pending: Option<decode::Instruction>,
    random: Random,
    /// Output stream 1 is selected (§7.1).
    screen_selected: bool,
    /// The window text goes to: 0 lower, 1 upper (§8.5).
    window: u16,
    output: screen::OutputBuffer,
    /// The style in force for text the game prints (§8.7.1).
    style: Style,
    /// The canvas a version 5 or 8 game draws its upper window on.
    screen: Screen,
    transcript: String,
    /// Text the game put in its input buffer before asking for a line.
    preloaded: String,
    /// The host has agreed to keep a transcript (contracts/core-api.md).
    transcript_accepted: bool,
    /// Text printed but not yet sent to the transcript: the word in progress, with
    /// the space before it. See `Machine::emit`.
    chunk: String,
    /// Something has been printed since the last new-line, so a game that quits
    /// now leaves a line unfinished.
    line_open: bool,
    /// Tables that output is being redirected into, innermost last (§7.1.2).
    memory_streams: Vec<(u16, Vec<u8>)>,
    /// Whether the game has buffering on (`buffer_mode`); it starts on.
    buffering: bool,
    undo: undo::History,
    status: Option<StatusLine>,
    status_changed: bool,
    diagnostics: Vec<Diagnostic>,
}

impl From<MemoryError> for FaultKind {
    fn from(e: MemoryError) -> Self {
        match e {
            MemoryError::ReadOutsideFile(_) => FaultKind::ReadOutsideFile,
            MemoryError::WriteOutsideDynamicMemory(_) => FaultKind::WriteOutsideDynamicMemory,
        }
    }
}

// A deliberately wrong stand-in, so that the tests of `Clone` are seen to fail.
impl Clone for Machine {
    fn clone(&self) -> Self {
        let options = LoadOptions { seed: Some(1), entropy: 0 };
        Machine::load_with(self.story.clone(), options, self.settings).expect("it loaded before")
    }
}

impl Machine {
    /// Loads a story file of version 3, 5 or 8 with default [`Settings`].
    ///
    /// A header length or checksum that does not match the file is not an error
    /// here; the game's own `verify` reports it.
    pub fn load(story: Vec<u8>, options: LoadOptions) -> Result<Machine, LoadError> {
        Self::load_with(story, options, Settings::default())
    }

    /// Loads a story file of version 3, 5 or 8.
    pub fn load_with(
        story: Vec<u8>,
        options: LoadOptions,
        settings: Settings,
    ) -> Result<Machine, LoadError> {
        let header = header::parse(&story)?;
        let mut memory = Memory::new(story.clone(), header.static_base as usize);
        header::apply_interpreter_fields(memory.dynamic_mut(), &header, &settings);
        let pc = header.initial_pc as u32;
        let charset = text::Charset::read(&memory, header.alphabet_table, header.unicode_table);
        Ok(Machine {
            story,
            header,
            settings,
            charset,
            memory,
            frames: vec![Frame::main()],
            pc,
            current: pc,
            state: State::Running,
            pending_fault: None,
            pending: None,
            random: Random::new(options.seed.unwrap_or(options.entropy)),
            screen_selected: true,
            window: 0,
            output: screen::OutputBuffer::default(),
            style: Style::default(),
            screen: Screen::new(settings.screen_columns, settings.screen_rows),
            transcript: String::new(),
            preloaded: String::new(),
            transcript_accepted: false,
            chunk: String::new(),
            line_open: false,
            memory_streams: Vec::new(),
            buffering: true,
            undo: undo::History::default(),
            status: None,
            status_changed: false,
            diagnostics: Vec::new(),
        })
    }

    /// Executes at most `budget` instructions and returns the next event.
    ///
    /// Returns an error, changing nothing, if an answer to the previous event is
    /// owed or the game has stopped.
    pub fn run(&mut self, budget: u64) -> Result<Event, HostError> {
        match self.state {
            State::Running => Ok(self.execute(budget)),
            State::Finished | State::Faulted => Err(HostError::Stopped),
            _ => Err(HostError::AnswerOwed),
        }
    }

    /// The error for an answer the machine is not waiting for.
    fn not_waiting(&self) -> HostError {
        match self.state {
            State::Finished | State::Faulted => HostError::Stopped,
            _ => HostError::NotWaitingForThis,
        }
    }

    /// Answers [`Event::LineWanted`] with what the player typed.
    ///
    /// Letters are reduced to lower case, characters the game cannot represent
    /// become question marks, and anything beyond the length the game asked for is
    /// dropped.
    pub fn provide_line(&mut self, text: &str) -> Result<(), HostError> {
        let State::AwaitingLine {
            text: buffer,
            parse,
            max_len,
        } = self.state
        else {
            return Err(self.not_waiting());
        };
        self.state = State::Running;
        self.accept_line(text, buffer, parse, max_len, 13);
        Ok(())
    }

    /// Answers [`Event::SaveRequested`]: whether the host stored the bytes. The
    /// game is told the save succeeded or failed accordingly.
    pub fn save_result(&mut self, stored: bool) -> Result<(), HostError> {
        if self.state != State::AwaitingSaveResult {
            return Err(self.not_waiting());
        }
        self.state = State::Running;
        self.finish_save(stored);
        Ok(())
    }

    /// Answers [`Event::RestoreRequested`] with a saved position, or `None` to
    /// decline.
    ///
    /// If the bytes are not a save of this story, the game is told the restore
    /// failed and nothing else changes. Otherwise play resumes from the saved
    /// position.
    pub fn provide_restore(&mut self, saved: Option<&[u8]>) -> Result<(), HostError> {
        if self.state != State::AwaitingRestoreData {
            return Err(self.not_waiting());
        }
        self.state = State::Running;
        self.finish_restore(saved);
        Ok(())
    }

    /// Answers [`Event::TranscriptRequested`]. If the host declines, the game's
    /// transcription bit is cleared so that it can tell.
    pub fn transcript_result(&mut self, accepted: bool) -> Result<(), HostError> {
        if self.state != State::AwaitingTranscriptAnswer {
            return Err(self.not_waiting());
        }
        self.state = State::Running;
        self.answer_transcript_request(accepted);
        Ok(())
    }

    /// Screen text printed since the last call, unwrapped. It does not include
    /// what the player typed.
    pub fn take_output(&mut self) -> String {
        self.output.take_text()
    }

    /// The same text as [`Machine::take_output`], as runs that carry the style each
    /// was printed in. The two calls drain one buffer; a host uses one or the other.
    pub fn take_styled_output(&mut self) -> Vec<StyledRun> {
        self.output.take_runs()
    }

    /// A read-only view of memory as it is now and of the story file as loaded.
    pub fn view(&self) -> View<'_> {
        View {
            version: self.header.version,
            now: self.memory.bytes(),
            shipped: &self.story,
        }
    }

    /// What a version 5 or 8 game has drawn outside its main text: the canvas, the
    /// height of the upper window and the cursor.
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// True once after anything alters the screen.
    pub fn take_screen_changed(&mut self) -> bool {
        self.screen.take_changed()
    }

    /// Answers [`Event::KeyWanted`]. A character the game's character set cannot
    /// represent is refused, and the machine keeps waiting.
    pub fn provide_key(&mut self, key: Key) -> Result<(), HostError> {
        if self.state != State::AwaitingKey {
            return Err(self.not_waiting());
        }
        let code = self.key_code(key).ok_or(HostError::UnrepresentableKey)?;
        self.state = State::Running;
        self.finish_read_char(code);
        Ok(())
    }

    /// During [`Event::LineWanted`], the text the game placed in its input buffer
    /// beforehand; what the player types follows it. Empty at any other time.
    pub fn preloaded_input(&self) -> &str {
        match self.state {
            State::AwaitingLine { .. } => &self.preloaded,
            _ => "",
        }
    }

    /// Answers [`Event::LineWanted`] with a line ended by `key`, which must be
    /// return or one the game listed in its terminating characters table
    /// (§10.5.2.1). Otherwise the key is refused and the machine keeps waiting.
    pub fn provide_line_ended_by(&mut self, text: &str, key: Key) -> Result<(), HostError> {
        let State::AwaitingLine {
            text: buffer,
            parse,
            max_len,
        } = self.state
        else {
            return Err(self.not_waiting());
        };
        let code = self
            .key_code(key)
            .filter(|&c| c == 13 || self.ends_a_line(c));
        let code = code.ok_or(HostError::NotATerminator)?;
        self.state = State::Running;
        self.accept_line(text, buffer, parse, max_len, code);
        Ok(())
    }

    /// Transcript text produced since the last call.
    pub fn take_transcript(&mut self) -> String {
        std::mem::take(&mut self.transcript)
    }

    /// The status line, if it changed since the last call.
    pub fn take_status(&mut self) -> Option<StatusLine> {
        if self.status_changed {
            self.status_changed = false;
            self.status.clone()
        } else {
            None
        }
    }

    /// Tolerated illegal operations recorded since the last call.
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// The byte at `address` in the machine's memory, for tests of this crate.
    #[cfg(test)]
    pub(crate) fn peek(&self, address: u32) -> u8 {
        self.memory
            .read_byte(address)
            .expect("address inside the file")
    }
}
