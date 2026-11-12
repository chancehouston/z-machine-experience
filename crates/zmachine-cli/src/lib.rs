//! Command-line host for the Z-machine core (contracts/cli.md).
//!
//! Everything that touches files, the terminal or the operating system's
//! randomness lives here; the core does none of it.

pub mod ledger;

use std::fs::File;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;
use zmachine_core::{DiagnosticKind, Event, HostError, Key, LoadOptions, Machine, Settings};

pub const USAGE: &str = "\
usage: zm [--script FILE] [--seed N] [--width N] [--dir DIR] [--max-instructions N]
          [--upper] [--style MODE] [--interpreter-version C] STORY
       zm --version
       zm --help

  STORY                  a Z-machine version 3, 5 or 8 story file
  --script FILE          read input lines from FILE instead of the terminal (- for standard input)
  --seed N               random seed, 0 to 2147483647, for a repeatable run
  --width N              wrap terminal output at N columns; 0 turns wrapping off (default 80)
  --dir DIR              directory for transcript and save files (default: current directory)
  --max-instructions N   stop with a fault after N instructions without a request for input
  --upper                show what the game draws outside its main text, above each prompt
  --style MODE           ansi, plain or auto: how bold, italic and reverse text is shown (default auto)
  --interpreter-version C  the letter a game may print as the interpreter's version (default Z)
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
    /// Print the game's canvas above each prompt.
    pub upper: bool,
    pub style: StyleMode,
    /// The letter the game is told is the interpreter's version (§11.1.3).
    pub interpreter_version: u8,
}

/// How styled text is written to the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleMode {
    /// Escape codes when standard output is a terminal.
    Auto,
    Ansi,
    Plain,
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
        upper: false,
        style: StyleMode::Auto,
        interpreter_version: b'Z',
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
            "--upper" => config.upper = true,
            "--style" => {
                let text = value("--style")?;
                config.style = match text.as_str() {
                    "auto" => StyleMode::Auto,
                    "ansi" => StyleMode::Ansi,
                    "plain" => StyleMode::Plain,
                    _ => return Err(format!("--style must be ansi, plain or auto, not {text:?}")),
                };
            }
            "--interpreter-version" => {
                let text = value("--interpreter-version")?;
                config.interpreter_version = match text.as_bytes() {
                    [letter] if letter.is_ascii_graphic() => *letter,
                    _ => {
                        return Err(format!(
                            "--interpreter-version must be one printable character, not {text:?}"
                        ));
                    }
                };
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

/// Whether a line is a command to dfrotz itself rather than input: a backslash
/// followed by a lower-case letter (contracts/cli.md).
fn is_dfrotz_command(line: &str) -> bool {
    let mut chars = line.chars();
    chars.next() == Some('\\') && chars.next().is_some_and(|c| c.is_ascii_lowercase())
}

/// Reads the next line that is input for the game, refusing dfrotz's own commands.
fn read_input_line(input: &mut dyn BufRead, err: &mut dyn Write) -> std::io::Result<Option<String>> {
    loop {
        match read_line(input)? {
            Some(line) if is_dfrotz_command(&line) => writeln!(
                err,
                "zm: skipped the line {line:?}: a backslash and a lower-case letter is a command to dfrotz, which zm does not take"
            )?,
            other => return Ok(other),
        }
    }
}

/// Turns dfrotz's escapes in an input line into keys: `\\_` return, `\\[` escape,
/// `\\?` delete, `\\^` `\\.` `\\<` `\\>` the arrows, `\\1` to `\\9` and `\\0`
/// function keys 1 to 10, `\\\\` a backslash. An escape dfrotz does not know is
/// reported and dropped, as dfrotz does.
fn keys_of_line(line: &str, err: &mut dyn Write) -> std::io::Result<Vec<Key>> {
    let mut keys = Vec::new();
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            keys.push(Key::Char(ch));
            continue;
        }
        keys.push(match chars.next() {
            // A backslash at the end of the line stands for the return that ends it.
            None | Some('_') => Key::Return,
            Some('\\') => Key::Char('\\'),
            Some('?') => Key::Delete,
            Some('[') => Key::Escape,
            Some('^') => Key::Up,
            Some('.') => Key::Down,
            Some('<') => Key::Left,
            Some('>') => Key::Right,
            Some('0') => Key::Function(10),
            Some(digit @ '1'..='9') => Key::Function(digit as u8 - b'0'),
            Some(other) => {
                writeln!(err, "zm: unknown escape \\{other} in an input line")?;
                continue;
            }
        });
    }
    Ok(keys)
}

/// The text of a line of input: its characters, with `\\\\` read as one backslash.
/// A line ends with return, so any other special key in it is reported and dropped.
fn text_of_line(line: &str, err: &mut dyn Write) -> std::io::Result<String> {
    if !line.contains('\\') {
        return Ok(line.to_string());
    }
    let mut text = String::new();
    for key in keys_of_line(line, err)? {
        match key {
            Key::Char(ch) => text.push(ch),
            _ => writeln!(err, "zm: a special key inside a line of input was dropped")?,
        }
    }
    Ok(text)
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
    // §8.4: the game is told the screen's size. With wrapping off it is told 80.
    let columns = if config.width == 0 { 80 } else { config.width.min(255) as u16 };
    let settings = Settings {
        screen_columns: columns,
        interpreter_version: config.interpreter_version,
        ..Settings::default()
    };
    let mut machine = match Machine::load_with(story, options, settings) {
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
    // Keys from the current input line that the game has not yet asked for.
    let mut keys: VecDeque<Key> = VecDeque::new();
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
                // Keys left over from a line read for single keys are dropped.
                keys.clear();
                let Some(line) = read_input_line(input, err)? else {
                    // The input ended while the game was waiting: a normal end.
                    return Ok(EXIT_OK);
                };
                if echo_input {
                    writeln!(out, "{line}")?;
                }
                wrapper.line_ended();
                let line = text_of_line(&line, err)?;
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
            Event::KeyWanted => {
                out.write_all(wrapper.flush().as_bytes())?;
                out.flush()?;
                // The reference's rule: the next unused character of the current
                // line, and an empty line is the return key.
                if keys.is_empty() {
                    let Some(line) = read_input_line(input, err)? else {
                        return Ok(EXIT_OK);
                    };
                    keys.extend(keys_of_line(&line, err)?);
                    if keys.is_empty() {
                        keys.push_back(Key::Return);
                    }
                }
                let key = keys.pop_front().expect("a key was just queued");
                match machine.provide_key(key) {
                    // A character the game cannot be given reads as a question mark.
                    Err(HostError::UnrepresentableKey) => machine.provide_key(Key::Char('?')),
                    other => other,
                }
                .expect("a key was wanted");
                since_input = 0;
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
