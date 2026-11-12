//! Command-line host for the Z-machine core (contracts/cli.md).
//!
//! Everything that touches files, the terminal or the operating system's
//! randomness lives here; the core does none of it.

pub mod ledger;

use std::fs::File;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use zmachine_core::{DiagnosticKind, Event, LoadOptions, Machine};

pub const USAGE: &str = "\
usage: zm [--script FILE] [--seed N] [--width N] [--dir DIR] [--max-instructions N] STORY
       zm --version
       zm --help

  STORY                  a Z-machine version 3 story file
  --script FILE          read input lines from FILE instead of the terminal (- for standard input)
  --seed N               random seed, 0 to 2147483647, for a repeatable run
  --width N              wrap terminal output at N columns; 0 turns wrapping off (default 80)
  --dir DIR              directory for transcript and save files (default: current directory)
  --max-instructions N   stop with a fault after N instructions without a request for input
";

/// Exit status when the game quit or the input ended.
pub const EXIT_OK: i32 = 0;
/// Exit status when the story file faulted or ran past `--max-instructions`.
pub const EXIT_FAULT: i32 = 1;
/// Exit status for a usage error or a story file that could not be loaded.
pub const EXIT_USAGE: i32 = 2;

const MAX_SEED: u32 = 2_147_483_647;

/// How many instructions to run between checks on the host side.
const SLICE: u64 = 100_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub story: PathBuf,
    /// `None` reads from standard input interactively.
    pub script: Option<PathBuf>,
    pub seed: Option<u32>,
    pub width: usize,
    pub dir: PathBuf,
    pub max_instructions: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    Run(Config),
    Help,
    Version,
}

/// Parses the arguments after the program name.
pub fn parse_args(args: &[String]) -> Result<Invocation, String> {
    let mut config = Config {
        story: PathBuf::new(),
        script: None,
        seed: None,
        width: 80,
        dir: PathBuf::from("."),
        max_instructions: None,
    };
    let mut story = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match arg.as_str() {
            "--help" | "-h" => return Ok(Invocation::Help),
            "--version" | "-V" => return Ok(Invocation::Version),
            "--script" => config.script = Some(PathBuf::from(value("--script")?)),
            "--dir" => config.dir = PathBuf::from(value("--dir")?),
            "--seed" => {
                let text = value("--seed")?;
                let seed: u32 = text
                    .parse()
                    .ok()
                    .filter(|&s| s <= MAX_SEED)
                    .ok_or_else(|| format!("--seed must be 0 to {MAX_SEED}, not {text:?}"))?;
                config.seed = Some(seed);
            }
            "--width" => {
                let text = value("--width")?;
                config.width = text
                    .parse()
                    .map_err(|_| format!("--width must be a number of columns, not {text:?}"))?;
            }
            "--max-instructions" => {
                let text = value("--max-instructions")?;
                config.max_instructions =
                    Some(text.parse().map_err(|_| {
                        format!("--max-instructions must be a number, not {text:?}")
                    })?);
            }
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => {
                if story.replace(PathBuf::from(other)).is_some() {
                    return Err("more than one story file given".to_string());
                }
            }
        }
    }
    config.story = story.ok_or("no story file given")?;
    Ok(Invocation::Run(config))
}

/// Wraps a stream of text at word boundaries.
///
/// Text arrives in pieces, so the wrapper carries the current column and any word
/// still being assembled from one piece to the next.
#[derive(Debug)]
pub struct Wrapper {
    width: usize,
    column: usize,
    /// Spaces seen since the last word, not yet written.
    spaces: usize,
    word: String,
    word_len: usize,
}

impl Wrapper {
    /// A width of 0 turns wrapping off.
    pub fn new(width: usize) -> Self {
        Wrapper {
            width,
            column: 0,
            spaces: 0,
            word: String::new(),
            word_len: 0,
        }
    }

    /// Adds text and returns what can be written so far. A word cut off at the end
    /// of `text` is held back until it is known to be complete.
    pub fn push(&mut self, text: &str) -> String {
        let mut out = String::new();
        if self.width == 0 {
            out.push_str(text);
            return out;
        }
        for ch in text.chars() {
            match ch {
                '\n' => {
                    self.end_word(&mut out);
                    out.push('\n');
                    self.column = 0;
                    self.spaces = 0;
                }
                ' ' => {
                    self.end_word(&mut out);
                    self.spaces += 1;
                }
                _ => {
                    // A word longer than a whole line is broken where it must be.
                    if self.word_len == self.width {
                        self.end_word(&mut out);
                    }
                    self.word.push(ch);
                    self.word_len += 1;
                }
            }
        }
        out
    }

    /// Writes out a held-back word and any spaces, for when the game stops to wait
    /// for input and its prompt must appear.
    pub fn flush(&mut self) -> String {
        let mut out = String::new();
        self.end_word(&mut out);
        if self.width == 0 || self.column + self.spaces <= self.width {
            out.extend(std::iter::repeat_n(' ', self.spaces));
            self.column += self.spaces;
        }
        self.spaces = 0;
        out
    }

    /// Notes that the cursor went back to the start of a line without the wrapper
    /// writing a line feed: the player pressed return.
    pub fn line_ended(&mut self) {
        self.column = 0;
        self.spaces = 0;
    }

    fn end_word(&mut self, out: &mut String) {
        if self.word_len == 0 {
            return;
        }
        if self.column > 0 && self.column + self.spaces + self.word_len > self.width {
            // The word does not fit: start a new line and drop the spaces before it.
            out.push('\n');
            self.column = 0;
        } else {
            out.extend(std::iter::repeat_n(' ', self.spaces));
            self.column += self.spaces;
        }
        out.push_str(&self.word);
        self.column += self.word_len;
        self.spaces = 0;
        self.word.clear();
        self.word_len = 0;
    }
}

/// A value from the operating system's randomness, for runs with no `--seed`.
fn os_entropy() -> u32 {
    use std::hash::{BuildHasher, Hasher};
    // `RandomState` is keyed from the operating system's random source.
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish() as u32
}

/// Reads one line of input without its line ending. `None` at the end of input.
fn read_line(input: &mut dyn BufRead) -> std::io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    line.truncate(line.trim_end_matches(['\n', '\r']).len());
    Ok(Some(line))
}

/// Resolves a file name the player gave inside `dir`. A name that could lead
/// outside it is refused (contracts/cli.md).
fn file_in_dir(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(format!(
            "refused the file name {name:?}: a file name may not contain a path separator"
        ));
    }
    Ok(dir.join(name))
}

/// Asks for the name of a file, as the reference interpreter does: the next line
/// of input is the answer, and an empty line takes the default. `None` if the
/// input has ended.
fn ask_file_name(
    what: &str,
    default: &str,
    interactive: bool,
    input: &mut dyn BufRead,
    err: &mut dyn Write,
) -> std::io::Result<Option<String>> {
    if interactive {
        write!(err, "{what} file name [{default}]: ")?;
        err.flush()?;
    }
    Ok(read_line(input)?.map(|line| {
        if line.is_empty() {
            default.to_string()
        } else {
            line
        }
    }))
}

/// Plays a story. `echo_input` writes each input line to `out`, for when the
/// terminal is not doing so itself.
pub fn play(
    config: &Config,
    input: &mut dyn BufRead,
    echo_input: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<i32> {
    let story = match std::fs::read(&config.story) {
        Ok(bytes) => bytes,
        Err(e) => {
            writeln!(err, "zm: cannot read {}: {e}", config.story.display())?;
            return Ok(EXIT_USAGE);
        }
    };
    let options = LoadOptions {
        seed: config.seed,
        entropy: os_entropy(),
    };
    let mut machine = match Machine::load(story, options) {
        Ok(machine) => machine,
        Err(e) => {
            writeln!(err, "zm: {}: {e}", config.story.display())?;
            return Ok(EXIT_USAGE);
        }
    };

    let stem = config
        .story
        .file_stem()
        .map_or("story".into(), |s| s.to_string_lossy().into_owned());
    // Once the game starts a transcript, the file stays open: if the game turns
    // transcription off and on again, writing continues where it left off.
    let mut transcript: Option<File> = None;
    let mut wrapper = Wrapper::new(config.width);
    // Instructions run since the game last asked for input.
    let mut since_input: u64 = 0;
    loop {
        let slice = match config.max_instructions {
            Some(max) => SLICE.min(max - since_input),
            None => SLICE,
        };
        let event = machine
            .run(slice)
            .expect("the driver answers every event in turn");
        out.write_all(wrapper.push(&machine.take_output()).as_bytes())?;
        let recorded = machine.take_transcript();
        if let Some(file) = transcript.as_mut() {
            file.write_all(recorded.as_bytes())?;
        }
        for diagnostic in machine.take_diagnostics() {
            match diagnostic.kind {
                DiagnosticKind::ObjectZero => writeln!(
                    err,
                    "zm: warning: the instruction at {:#x} was applied to object 0 and was ignored",
                    diagnostic.address
                )?,
            }
        }
        match event {
            Event::BudgetExhausted => {
                since_input += slice;
                if let Some(max) = config.max_instructions
                    && since_input >= max
                {
                    out.write_all(wrapper.flush().as_bytes())?;
                    out.flush()?;
                    writeln!(
                        err,
                        "zm: stopped: {max} instructions ran without a request for input"
                    )?;
                    return Ok(EXIT_FAULT);
                }
            }
            Event::LineWanted { .. } => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                let Some(line) = read_line(input)? else {
                    // The input ended while the game was waiting: a normal end.
                    return Ok(EXIT_OK);
                };
                if echo_input {
                    writeln!(out, "{line}")?;
                }
                wrapper.line_ended();
                machine.provide_line(&line).expect("a line was wanted");
                since_input = 0;
            }
            Event::Finished => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                return Ok(EXIT_OK);
            }
            Event::Fault(fault) => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                writeln!(err, "zm: the story file stopped: {fault}")?;
                return Ok(EXIT_FAULT);
            }
            Event::TranscriptRequested => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                let default = format!("{stem}.scr");
                let name = ask_file_name("Transcript", &default, !echo_input, input, err)?;
                let opened = match name {
                    // The input ended at the prompt.
                    None => None,
                    Some(name) => match file_in_dir(&config.dir, &name) {
                        Err(reason) => {
                            writeln!(err, "zm: {reason}")?;
                            None
                        }
                        Ok(path) => match File::create(&path) {
                            Ok(file) => Some(file),
                            Err(e) => {
                                writeln!(err, "zm: cannot write {}: {e}", path.display())?;
                                None
                            }
                        },
                    },
                };
                let accepted = opened.is_some();
                transcript = opened;
                machine
                    .transcript_result(accepted)
                    .expect("a transcript was requested");
            }
            Event::SaveRequested(bytes) => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                let default = format!("{stem}.qzl");
                let name = ask_file_name("Save", &default, !echo_input, input, err)?;
                let stored = match name.map(|name| file_in_dir(&config.dir, &name)) {
                    // The input ended at the prompt.
                    None => false,
                    Some(Err(reason)) => {
                        writeln!(err, "zm: {reason}")?;
                        false
                    }
                    Some(Ok(path)) => match std::fs::write(&path, &bytes) {
                        Ok(()) => true,
                        Err(e) => {
                            writeln!(err, "zm: cannot write {}: {e}", path.display())?;
                            false
                        }
                    },
                };
                machine.save_result(stored).expect("a save was requested");
            }
            Event::RestoreRequested => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                let default = format!("{stem}.qzl");
                let name = ask_file_name("Restore", &default, !echo_input, input, err)?;
                let saved = match name.map(|name| file_in_dir(&config.dir, &name)) {
                    None => None,
                    Some(Err(reason)) => {
                        writeln!(err, "zm: {reason}")?;
                        None
                    }
                    Some(Ok(path)) => match std::fs::read(&path) {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            writeln!(err, "zm: cannot read {}: {e}", path.display())?;
                            None
                        }
                    },
                };
                // If the file is not a save of this story, the core refuses it and
                // the game reports the failure itself.
                machine
                    .provide_restore(saved.as_deref())
                    .expect("a restore was requested");
            }
        }
    }
}
