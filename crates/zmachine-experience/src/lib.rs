//! The experience layer: what a phone's screen shows of a game, and what a tap
//! sends it.
//!
//! Everything a shell draws is decided here, so that the iPhone and Android apps
//! share one tested implementation of behaviour and contain presentation only.
//! The contract is `specs/004-first-demo/contracts/experience-api.md`.
#![forbid(unsafe_code)]

mod announce;
mod compass;
mod json;
mod kept;
mod light;
pub mod lookahead;
pub mod map;
mod map_place;
mod strip;
mod transcript;

#[cfg(test)]
mod announce_tests;
#[cfg(test)]
mod compass_tests;
#[cfg(test)]
mod json_tests;
#[cfg(test)]
mod kept_tests;
#[cfg(test)]
mod light_tests;
#[cfg(test)]
mod lookahead_tests;
#[cfg(test)]
mod map_place_tests;
#[cfg(test)]
mod map_tests;
#[cfg(test)]
mod strip_tests;

pub use announce::NOTE_ENDED;

/// The layer's words when a kept game could not be brought back.
pub const NOTE_NOT_BROUGHT_BACK: &str =
    "Your earlier game could not be brought back. This is a new game.";
/// The layer's words when a host could not keep the game.
pub const NOTE_NOT_KEPT: &str = "Your place is not being kept.";
/// The layer's question before a game in progress is given up.
pub const ASK_START_OVER: &str = "Start over? The game in progress will be given up.";
pub use compass::{Compass, CompassPlace, Extra, NOTE_NONE, NOTE_UNKNOWN, Place, PlaceState};
pub use light::{Light, NOTE_DARK};
pub use strip::{ItemState, NOTE_NOT_KNOWN, NOTE_NOTHING, Strip, StripItem};
pub use transcript::{Source, Turn};

use zmachine_core::{Event, HostError, Key, LoadOptions, Machine, StatusKind, StatusLine};
use zmachine_introspect::{Family, Introspector};

/// Where a game's randomness comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seed {
    /// A repeatable game: 0 to 2,147,483,647.
    Fixed(u32),
    /// A number from the host's random source. The layer has none of its own.
    Fresh(u32),
}

/// The game and turn a command was made against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stamp {
    pub generation: u64,
    pub turn_count: usize,
}

/// Why a command did not reach the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Refused {
    /// It was made against a turn or a game that is no longer the current one.
    Stale,
    /// The game has ended or faulted.
    NotWaiting,
    /// The compass does not offer that place now.
    NotOffered,
    /// The line holds a line break.
    BadLine,
}

/// Why a game could not be started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartError {
    pub message: String,
}

/// What the game wants next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Waiting {
    /// A command, at the game's command prompt.
    Command,
    /// Some other line of text: the answer to a question.
    Answer,
    /// Nothing: the game has ended.
    Ended,
    /// Nothing: the story file failed.
    Fault,
}

impl Waiting {
    pub fn as_str(self) -> &'static str {
        match self {
            Waiting::Command => "command",
            Waiting::Answer => "answer",
            Waiting::Ended => "ended",
            Waiting::Fault => "fault",
        }
    }
}

/// What became of the kept game a session was opened with
/// (specs/011-resume-game/contracts/kept-game.md, L§1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// No kept game was given: a new game.
    New,
    /// The kept game was brought back.
    BroughtBack,
    /// The kept game could not be brought back: a new game, with a notice.
    CouldNot,
}

/// The game's status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub place: String,
    pub score: Option<i16>,
    pub moves: Option<u16>,
    pub hours: Option<u16>,
    pub minutes: Option<u16>,
}

/// How many instructions a game may run without stopping before the layer gives
/// it up as not responding.
pub const INSTRUCTION_CEILING: u64 = 50_000_000;

/// How many instructions to run between checks on the ceiling.
const SLICE: u64 = 1_000_000;

/// The largest fixed seed the core accepts.
const MAX_SEED: u32 = 2_147_483_647;

/// One game being played.
#[derive(Clone, Debug)]
pub struct Session {
    /// The story file, kept for starting a new game.
    story: Vec<u8>,
    machine: Machine,
    introspector: Introspector,
    turns: Vec<Turn>,
    generation: u64,
    waiting: Waiting,
    /// The game wants a single key, not a line.
    wants_key: bool,
    /// Where the game first asked for a line: its command prompt (research R6).
    command_address: Option<u32>,
    message: Option<String>,
    status: Option<Status>,
    compass: Compass,
    strip: Strip,
    light: Light,
    ceiling: u64,
    /// How far look-ahead's copies may run at one prompt.
    look_ahead: lookahead::Limits,
    /// What look-ahead ran at the last stop: all copies, and the longest one.
    looked: (u64, u64),
    /// Whether the light said dark at the stop before this one.
    was_dark: bool,
    /// What a screen reader is to speak for the newest turn.
    announcement: Option<String>,
    /// The SHA-256 of the story file: which story a kept game is of.
    identity: String,
    /// What the game in progress was started with.
    chance: Seed,
    /// How many inputs have been answered: the turns after the opening that
    /// a kept game holds. A command that brought a fault is not among them.
    answered: usize,
    /// Words to show once: set when a game is opened or keeping fails, and
    /// cleared when the next turn is recorded.
    notice: Option<&'static str>,
    /// Whether the player has been told, in this game, that it is not kept.
    keep_failed_told: bool,
    /// The places the player has stood in and the moves they made
    /// (specs/012-map-first-slice).
    map: map::Map,
    /// The input given since the last stop, for the map.
    sent: Option<String>,
    /// The core's count of jumps at the last stop.
    jumps: u32,
}

fn load(story: &[u8], seed: Seed) -> Result<Machine, StartError> {
    let options = match seed {
        Seed::Fixed(seed) if seed > MAX_SEED => {
            return Err(StartError {
                message: format!("a fixed seed must be 0 to {MAX_SEED}, not {seed}"),
            });
        }
        Seed::Fixed(seed) => LoadOptions {
            seed: Some(seed),
            entropy: 0,
        },
        Seed::Fresh(entropy) => LoadOptions {
            seed: None,
            entropy,
        },
    };
    Machine::load(story.to_vec(), options).map_err(|e| StartError {
        message: format!("The story file is {e}."),
    })
}

impl Session {
    /// Loads the story file and runs it to its first stop.
    pub fn new(story: Vec<u8>, seed: Seed) -> Result<Session, StartError> {
        Session::with_pack(story, seed, None)
    }

    /// As [`Session::new`], with the text of a game pack for the story
    /// (docs/introspection.md, section 12). A pack that cannot be read, or is
    /// another story's, changes nothing.
    pub fn with_pack(
        story: Vec<u8>,
        seed: Seed,
        pack: Option<&str>,
    ) -> Result<Session, StartError> {
        Session::start(story, seed, pack, lookahead::LIMITS, INSTRUCTION_CEILING)
    }

    /// Loads the story file and runs it to its first stop, with these limits
    /// in force from the first instruction.
    fn start(
        story: Vec<u8>,
        seed: Seed,
        pack: Option<&str>,
        look_ahead: lookahead::Limits,
        ceiling: u64,
    ) -> Result<Session, StartError> {
        let machine = load(&story, seed)?;
        let identity = kept::digest(&story);
        // The interface is made once for the story file, so a pack in force
        // stays in force through every new game.
        let introspector = match pack {
            Some(text) => Introspector::with_pack(&machine.view(), text),
            None => Introspector::new(&machine.view()),
        };
        let mut session = Session {
            story,
            machine,
            introspector,
            turns: Vec::new(),
            generation: 1,
            waiting: Waiting::Command,
            wants_key: false,
            command_address: None,
            message: None,
            status: None,
            compass: Compass::idle(),
            strip: Strip::default(),
            light: Light::default(),
            ceiling,
            look_ahead,
            looked: (0, 0),
            was_dark: false,
            announcement: None,
            identity,
            chance: seed,
            answered: 0,
            notice: None,
            keep_failed_told: false,
            map: map::Map::new(false),
            sent: None,
            jumps: 0,
        };
        session.map = session.new_map();
        session.open();
        Ok(session)
    }

    /// Makes a session from a story and, perhaps, a kept game
    /// (specs/011-resume-game/contracts/kept-game.md, L§1). With kept bytes
    /// that can be brought back, the game is the one they were made from, and
    /// `chance` is not used. Otherwise it is a new game with `chance`; and if
    /// kept bytes were given, the session has a notice saying they could not
    /// be brought back.
    pub fn open_kept(
        story: Vec<u8>,
        pack: Option<&str>,
        chance: Seed,
        kept: Option<&[u8]>,
    ) -> Result<(Session, Outcome), StartError> {
        Session::open_kept_within(story, pack, chance, kept, INSTRUCTION_CEILING)
    }

    /// As [`Session::open_kept`], with the instruction ceiling to hold while
    /// the kept inputs are given. A host has no need to; tests do.
    pub fn open_kept_within(
        story: Vec<u8>,
        pack: Option<&str>,
        chance: Seed,
        kept: Option<&[u8]>,
        ceiling: u64,
    ) -> Result<(Session, Outcome), StartError> {
        let Some(bytes) = kept else {
            return Session::with_pack(story, chance, pack).map(|s| (s, Outcome::New));
        };
        if let Some(session) = Session::bring_back(&story, pack, bytes, ceiling) {
            return Ok((session, Outcome::BroughtBack));
        }
        let mut session = Session::with_pack(story, chance, pack)?;
        session.notice = Some(NOTE_NOT_BROUGHT_BACK);
        session.announce_notice();
        Ok((session, Outcome::CouldNot))
    }

    /// The game a kept game was made from, or none if these bytes are not
    /// one this story can be brought back from (L§2).
    fn bring_back(story: &[u8], pack: Option<&str>, bytes: &[u8], ceiling: u64) -> Option<Session> {
        let kept = kept::Kept::from_bytes(bytes).ok()?;
        if kept.story != kept::digest(story) {
            return None;
        }
        // Look-ahead runs at every stop, as it did when the game was played:
        // it never touches the game, but the map is made from the compass it
        // settles, and must come back the same (feature 012; this replaces
        // feature 011's leaving it out until the last stop).
        let count = kept.inputs.len();
        let mut session = Session::start(
            story.to_vec(),
            kept.chance,
            pack,
            lookahead::LIMITS,
            ceiling,
        )
        .ok()?;
        for (source, line) in &kept.inputs {
            // Each kept input was answered by a game that was waiting.
            if !matches!(session.waiting, Waiting::Command | Waiting::Answer) {
                return None;
            }
            session.give(line, *source);
        }
        session.ceiling = INSTRUCTION_CEILING;
        if session.waiting == Waiting::Fault || session.answered != count {
            return None;
        }
        let texts = session.turns.iter().map(|turn| turn.text.as_str());
        (kept::transcript(texts) == kept.transcript).then_some(session)
    }

    /// The bytes a host stores so that this game can be brought back (L§6):
    /// every answered input, and what is needed to know the game they give.
    pub fn kept(&self) -> Vec<u8> {
        let turns = &self.turns[..=self.answered.min(self.turns.len().saturating_sub(1))];
        let inputs = turns
            .iter()
            .filter_map(|turn| Some((turn.source?, turn.command.clone()?)))
            .collect();
        kept::Kept {
            story: self.identity.clone(),
            chance: self.chance,
            inputs,
            transcript: kept::transcript(turns.iter().map(|turn| turn.text.as_str())),
        }
        .to_bytes()
    }

    /// A host could not store the kept game. The first time in a game, the
    /// notice says so; after that nothing is said until a new game (L§8).
    pub fn keep_failed(&mut self) {
        if !self.keep_failed_told {
            self.keep_failed_told = true;
            self.notice = Some(NOTE_NOT_KEPT);
        }
    }

    /// Words for the player to be shown once, or none.
    pub fn notice(&self) -> Option<&str> {
        self.notice
    }

    /// The question to put before this game is given up for a new one, or
    /// none where nothing would be given up: the game has ended or faulted.
    pub fn start_over(&self) -> Option<&str> {
        match self.waiting {
            Waiting::Command | Waiting::Answer => Some(ASK_START_OVER),
            Waiting::Ended | Waiting::Fault => None,
        }
    }

    /// Puts the notice before what is to be spoken for the newest turn (L§9).
    fn announce_notice(&mut self) {
        if let Some(notice) = self.notice {
            self.announcement = Some(match self.announcement.take() {
                Some(rest) => format!("{notice}\n{rest}"),
                None => notice.to_string(),
            });
        }
    }

    /// A map with nothing on it, for the game about to be played. A story of
    /// neither family has none: the layer cannot say where the player is.
    fn new_map(&self) -> map::Map {
        map::Map::new(self.introspector.family() != Family::Unrecognised)
    }

    /// The map of the game in progress.
    pub fn map(&self) -> &map::Map {
        &self.map
    }

    /// Lowers or raises the instruction ceiling. A host has no need to; tests do.
    pub fn set_instruction_ceiling(&mut self, instructions: u64) {
        self.ceiling = instructions;
    }

    /// Sets how far look-ahead's copies may run. A host has no need to; tests
    /// do. It takes effect at the next stop.
    pub fn set_look_ahead_limits(&mut self, limits: lookahead::Limits) {
        self.look_ahead = limits;
    }

    /// What a screen reader is to speak for the newest turn: the game's reply
    /// and the layer's own words, whole. None where there is nothing to say.
    pub fn announcement(&self) -> Option<&str> {
        self.announcement.as_deref()
    }

    /// How many instructions look-ahead's copies ran at the last stop, all
    /// together, and the most any one of them ran; each counted in whole
    /// steps of a thousand. For measuring the limits.
    pub fn look_ahead_cost(&self) -> (u64, u64) {
        self.looked
    }

    /// Runs a freshly loaded game to its first stop and records the opening.
    fn open(&mut self) {
        let text = self.advance();
        self.turns = vec![Turn {
            number: 0,
            command: None,
            source: None,
            text,
        }];
        self.announce();
    }

    /// Makes the announcement for the turn just recorded
    /// (specs/010-screen-reader-announcements/contracts/announcements.md).
    fn announce(&mut self) {
        // A stop that is not a command prompt counts as not dark: nothing is
        // said of light there, and the note is said again after it.
        let dark = self.waiting == Waiting::Command && self.light.dark == Some(true);
        let fell_dark = (dark && !self.was_dark).then_some(light::NOTE_DARK);
        self.was_dark = dark;
        if self.waiting == Waiting::Fault {
            self.announcement = self.message.clone();
            return;
        }
        let waits_for_line =
            matches!(self.waiting, Waiting::Command | Waiting::Answer) && !self.wants_key;
        let text = self.turns.last().map_or("", |turn| turn.text.as_str());
        self.announcement = announce::announcement(
            announce::reply(text, waits_for_line),
            fell_dark,
            self.waiting == Waiting::Ended,
        );
    }

    /// Runs the game until it wants input, ends or fails, and returns what it
    /// printed. Requests to save, restore or keep a transcript are declined here
    /// and are not stops (research R7).
    fn advance(&mut self) -> String {
        let mut text = String::new();
        let mut run = 0u64;
        self.wants_key = false;
        loop {
            let event = match self.machine.run(SLICE) {
                Ok(event) => event,
                // The layer answers every event in turn, so this cannot happen;
                // if it does, the game cannot go on.
                Err(_) => {
                    self.fail("The game cannot continue.".to_string());
                    break;
                }
            };
            text.push_str(&self.machine.take_output());
            self.machine.take_transcript();
            self.machine.take_diagnostics();
            if let Some(line) = self.machine.take_status() {
                self.status = Some(status_of(line));
            }
            // An answer the layer gives is one the core's interface offers every
            // host; none can be refused here, since each matches its event.
            let _ = match event {
                Event::BudgetExhausted => {
                    run += SLICE;
                    if run >= self.ceiling {
                        self.fail("The game is not responding.".to_string());
                        break;
                    }
                    continue;
                }
                Event::SaveRequested(_) => self.machine.save_result(false),
                Event::RestoreRequested => self.machine.provide_restore(None),
                Event::TranscriptRequested => self.machine.transcript_result(false),
                Event::LineWanted { .. } => {
                    let here = self.machine.waiting_at();
                    let command = *self.command_address.get_or_insert(here.unwrap_or(0));
                    self.waiting = if here == Some(command) {
                        Waiting::Command
                    } else {
                        Waiting::Answer
                    };
                    break;
                }
                Event::KeyWanted => {
                    self.wants_key = true;
                    self.waiting = Waiting::Answer;
                    break;
                }
                Event::Finished => {
                    self.waiting = Waiting::Ended;
                    // The layer's words for it, for a host to show and speak.
                    self.message = Some(announce::NOTE_ENDED.to_string());
                    break;
                }
                Event::Fault(fault) => {
                    self.fail(format!("The story file stopped: {fault}."));
                    break;
                }
            };
            run = 0;
        }
        self.looked = (0, 0);
        self.compass = match self.waiting {
            Waiting::Command => {
                let view = self.machine.view();
                // A version 3 dictionary keeps six characters of a word; later
                // versions nine (Standard §13.3, §13.4).
                let word_len = if view.version() <= 3 { 6 } else { 9 };
                let mut compass = Compass::of(&self.introspector.exits(&view), word_len);
                // Each way the interface could not settle is tried on copies
                // of the game, which is itself only read (feature 009).
                if let Some(address) = self.command_address {
                    let mut look = lookahead::LookAhead::new(
                        &self.machine,
                        &self.introspector,
                        address,
                        self.turns.len() as u64,
                        self.look_ahead,
                    );
                    compass.settle(|command| look.finding(command));
                    self.looked = (look.spent(), look.most());
                }
                compass
            }
            _ => Compass::idle(),
        };
        // What is carried is asked for at every stop, whatever the game is
        // waiting for; a host shows it as not current when it is not a command.
        self.strip = Strip::of(&self.introspector.inventory(&self.machine.view()));
        // Light is said only while the game waits for a command: at any other
        // stop nothing is claimed.
        self.light = match self.waiting {
            Waiting::Command => Light::of(&self.introspector.location(&self.machine.view())),
            _ => Light::default(),
        };
        // The map is told of every stop: what was sent, where the player now
        // is, and the ways the compass gives. It reads nothing of the game
        // itself (specs/012-map-first-slice/research.md R2).
        let jumps = self.machine.jumps();
        let room = match self.waiting {
            Waiting::Command => self
                .introspector
                .location(&self.machine.view())
                .value
                .ok()
                .map(|location| (location.room, location.name.ok())),
            _ => None,
        };
        self.map.observe(map::Observation {
            sent: self.sent.take().as_deref(),
            waiting: self.waiting,
            room,
            dark: self.light.dark,
            compass: &self.compass,
            jumped: jumps != self.jumps,
        });
        self.jumps = jumps;
        text
    }

    fn fail(&mut self, message: String) {
        self.waiting = Waiting::Fault;
        self.message = Some(message);
    }

    /// The one way a line reaches the game (constitution Principle III).
    fn give(&mut self, line: &str, source: Source) {
        let given = if self.wants_key {
            // The game wants one key: the first character, or return.
            let key = line.chars().next().map_or(Key::Return, Key::Char);
            match self.machine.provide_key(key) {
                Err(HostError::UnrepresentableKey) => self.machine.provide_key(Key::Char('?')),
                other => other,
            }
        } else {
            self.machine.provide_line(line)
        };
        if given.is_err() {
            self.fail("The game cannot continue.".to_string());
            self.announce();
            return;
        }
        self.notice = None;
        self.sent = Some(line.to_string());
        let text = self.advance();
        self.turns.push(Turn {
            number: self.turns.len(),
            command: Some(line.to_string()),
            source: Some(source),
            text,
        });
        // A command that brought a fault is not kept: the game brought back
        // is the one before it (specs/011-resume-game/research.md, R6).
        if self.waiting != Waiting::Fault && self.answered + 2 == self.turns.len() {
            self.answered += 1;
        }
        self.announce();
    }

    /// Checks that a command was made against the current turn of a game that is
    /// waiting.
    fn check(&self, at: Stamp) -> Result<(), Refused> {
        if at != self.stamp() {
            return Err(Refused::Stale);
        }
        match self.waiting {
            Waiting::Command | Waiting::Answer => Ok(()),
            Waiting::Ended | Waiting::Fault => Err(Refused::NotWaiting),
        }
    }

    /// Gives the game a line the player typed.
    pub fn send_line(&mut self, at: Stamp, line: &str) -> Result<(), Refused> {
        self.check(at)?;
        if line.contains(['\n', '\r']) {
            return Err(Refused::BadLine);
        }
        self.give(line, Source::Typed);
        Ok(())
    }

    /// The player tapped a place on the compass.
    pub fn tap(&mut self, at: Stamp, place: Place) -> Result<(), Refused> {
        self.check(at)?;
        let command = self
            .compass
            .place(place)
            .and_then(|p| p.command.clone())
            .ok_or(Refused::NotOffered)?;
        self.give(&command, Source::Tap);
        Ok(())
    }

    /// The player tapped an extra, by its index.
    pub fn tap_extra(&mut self, at: Stamp, index: usize) -> Result<(), Refused> {
        self.check(at)?;
        let command = self
            .compass
            .extras
            .get(index)
            .map(|e| e.command.clone())
            .ok_or(Refused::NotOffered)?;
        self.give(&command, Source::Tap);
        Ok(())
    }

    /// Abandons the game and starts a new one from the same story file.
    pub fn new_game(&mut self, seed: Seed) {
        self.generation += 1;
        self.message = None;
        self.status = None;
        self.was_dark = false;
        self.command_address = None;
        self.chance = seed;
        self.answered = 0;
        self.notice = None;
        self.keep_failed_told = false;
        self.map = self.new_map();
        self.sent = None;
        self.jumps = 0;
        match load(&self.story, seed) {
            Ok(machine) => {
                self.machine = machine;
                self.open();
            }
            // The story file loaded before; only a bad seed can bring this.
            Err(error) => {
                self.turns = vec![Turn {
                    number: 0,
                    command: None,
                    source: None,
                    text: String::new(),
                }];
                self.compass = Compass::idle();
                self.fail(error.message);
                self.announce();
            }
        }
    }

    /// Marks the session as broken: it reports a fault with this message,
    /// offers nothing and refuses commands until a new game. For a caller that
    /// could not let a call into the session finish.
    pub fn mark_failed(&mut self, message: &str) {
        self.compass = Compass::idle();
        self.fail(message.to_string());
        self.announce();
    }

    pub fn stamp(&self) -> Stamp {
        Stamp {
            generation: self.generation,
            turn_count: self.turns.len(),
        }
    }

    pub fn waiting(&self) -> Waiting {
        self.waiting
    }

    /// The message for the player when the story file has failed.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn compass(&self) -> &Compass {
        &self.compass
    }

    /// Whether the player is in the dark (specs/008-darkness).
    pub fn light(&self) -> Light {
        self.light
    }

    /// What to show of the inventory (specs/007-inventory-panel).
    pub fn inventory(&self) -> &Strip {
        &self.strip
    }

    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    /// Everything the screen needs, with the turns from `since` onward, as JSON.
    pub fn snapshot_json(&self, since: usize) -> String {
        json::snapshot(self, since)
    }
}

fn status_of(line: StatusLine) -> Status {
    let (score, moves, hours, minutes) = match line.kind {
        StatusKind::Score { score, turns } => (Some(score), Some(turns), None, None),
        StatusKind::Time { hours, minutes } => (None, None, Some(hours), Some(minutes)),
    };
    Status {
        place: line.location,
        score,
        moves,
        hours,
        minutes,
    }
}
