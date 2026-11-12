//! A story's report and the summary across stories
//! (specs/005-corpus-testing/contracts/formats.md; data-model.md "Report").
//!
//! A report holds numbers, digests and the story's id, and nothing a game
//! printed.

use crate::trial::Outcome;
use crate::verdict::Reported;
use zmachine_introspect::PackState;

/// What identifies a trial's result apart from when it happened: the room it
/// started in (0 when unknown), the direction, what was reported, and what the
/// copy did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Case {
    pub room: u16,
    pub direction: u8,
    pub reported: Reported,
    pub outcome: Outcome,
}

/// Which limit of the budget ended the crawl, if either.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reached {
    #[default]
    No,
    Rooms,
    Trials,
}

/// Counts by outcome, for exits reported conditional and for directions
/// reported unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcomes {
    pub moved: usize,
    pub stayed: usize,
    pub question: usize,
    pub ended: usize,
    pub fault: usize,
}

impl Outcomes {
    pub fn total(&self) -> usize {
        self.moved + self.stayed + self.question + self.ended + self.fault
    }
}

/// Trials on answers that rest on a game pack, by what was reported
/// (specs/006-game-packs/research.md R10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marked {
    pub open_confirmed: usize,
    pub open_wrong_known: usize,
    pub open_wrong_unexplained: usize,
    pub closed_confirmed: usize,
    pub closed_wrong_known: usize,
    pub closed_wrong_unexplained: usize,
    /// Reported closed; the game ended, asked a question or faulted.
    pub closed_other: usize,
    pub distrusted: Outcomes,
}

impl Marked {
    pub fn total(&self) -> usize {
        self.open_confirmed
            + self.open_wrong_known
            + self.open_wrong_unexplained
            + self.closed_confirmed
            + self.closed_wrong_known
            + self.closed_wrong_unexplained
            + self.closed_other
            + self.distrusted.total()
    }
}

/// The figures for one story on one run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub id: String,
    pub sha256: String,
    pub version: u8,
    /// `zil`, `inform` or `none`.
    pub family: String,
    pub seed: u32,
    pub places: usize,
    pub walkthrough: usize,
    pub crawl: usize,
    pub rooms_visited: usize,
    pub named_not_reached: usize,
    pub budget_rooms: usize,
    pub budget_trials: usize,
    pub reached: Reached,
    pub trials: usize,
    pub wordless: usize,
    pub fixed_confirmed: usize,
    pub fixed_wrong_known: usize,
    pub fixed_wrong_unexplained: usize,
    pub none_confirmed: usize,
    pub none_wrong_known: usize,
    pub none_wrong_unexplained: usize,
    pub none_other: usize,
    pub conditional: Outcomes,
    pub unknown: Outcomes,
    pub no_verdict: usize,
    /// What became of the story's game pack; and, in force, how many entries
    /// it holds.
    pub pack: PackState,
    pub entries: usize,
    /// The trials on answers that rest on the pack.
    pub marked: Marked,
    /// Each case whose verdict is not "confirmed", with how many trials.
    pub detail: Vec<(Case, usize)>,
}

/// Where a file departs from its format: the line, from 1, and why.
pub type Refusal = (usize, String);

pub const FAMILIES: [&str; 3] = ["zil", "inform", "none"];

fn reported_text(reported: Reported) -> String {
    match reported {
        Reported::Fixed(room) => format!("fixed {room}"),
        Reported::None => "none".to_string(),
        Reported::Conditional => "conditional".to_string(),
        Reported::Unknown => "unknown".to_string(),
        Reported::Open(room) => format!("open {room}"),
        Reported::Closed(room) => format!("closed {room}"),
        Reported::Distrusted => "distrusted".to_string(),
    }
}

fn outcome_text(outcome: Outcome) -> String {
    match outcome {
        Outcome::Moved(room) => format!("moved {room}"),
        Outcome::Stayed => "stayed".to_string(),
        Outcome::Question => "question".to_string(),
        Outcome::Ended => "ended".to_string(),
        Outcome::Fault => "fault".to_string(),
        Outcome::Lost => "lost".to_string(),
    }
}

impl Case {
    /// The case as a detail line gives it, without the count.
    pub fn text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.room,
            self.direction,
            reported_text(self.reported),
            outcome_text(self.outcome)
        )
    }

    /// Reads what [`Case::text`] writes, from the words of a line; returns the
    /// case and the words left over.
    pub fn read<'a>(words: &'a [&'a str]) -> Option<(Case, &'a [&'a str])> {
        let number = |word: &str| -> Option<u16> {
            (!word.is_empty() && word.bytes().all(|b| b.is_ascii_digit()))
                .then(|| word.parse().ok())
                .flatten()
        };
        let room = number(words.first()?)?;
        let direction = u8::try_from(number(words.get(1)?)?).ok()?;
        let (reported, rest) = match *words.get(2)? {
            "fixed" => (Reported::Fixed(number(words.get(3)?)?), &words[4..]),
            "none" => (Reported::None, &words[3..]),
            "conditional" => (Reported::Conditional, &words[3..]),
            "unknown" => (Reported::Unknown, &words[3..]),
            "open" => (Reported::Open(number(words.get(3)?)?), &words[4..]),
            "closed" => (Reported::Closed(number(words.get(3)?)?), &words[4..]),
            "distrusted" => (Reported::Distrusted, &words[3..]),
            _ => return None,
        };
        let (outcome, rest) = match *rest.first()? {
            "moved" => (Outcome::Moved(number(rest.get(1)?)?), &rest[2..]),
            "stayed" => (Outcome::Stayed, &rest[1..]),
            "question" => (Outcome::Question, &rest[1..]),
            "ended" => (Outcome::Ended, &rest[1..]),
            "fault" => (Outcome::Fault, &rest[1..]),
            "lost" => (Outcome::Lost, &rest[1..]),
            _ => return None,
        };
        Some((
            Case {
                room,
                direction,
                reported,
                outcome,
            },
            rest,
        ))
    }
}

/// Matches a line against a form: the form's words must appear as they are,
/// except `#`, which stands for a decimal number. Returns the numbers.
pub(crate) fn numbers(line: &str, form: &str) -> Option<Vec<usize>> {
    let words: Vec<&str> = line.split(' ').collect();
    let shape: Vec<&str> = form.split(' ').collect();
    if words.len() != shape.len() {
        return None;
    }
    let mut found = Vec::new();
    for (word, expected) in words.iter().zip(shape) {
        if expected == "#" {
            if word.is_empty() || word.len() > 12 || !word.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            found.push(word.parse().ok()?);
        } else if *word != expected {
            return None;
        }
    }
    Some(found)
}

pub(crate) fn is_digest(word: &str) -> bool {
    word.len() == 64
        && word
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(crate) fn is_id(word: &str) -> bool {
    !word.is_empty()
        && word
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Report {
    /// The number of trials the verdict lines account for.
    pub fn verdicts(&self) -> usize {
        self.fixed_confirmed
            + self.fixed_wrong_known
            + self.fixed_wrong_unexplained
            + self.none_confirmed
            + self.none_wrong_known
            + self.none_wrong_unexplained
            + self.none_other
            + self.conditional.total()
            + self.unknown.total()
            + self.no_verdict
            + self.marked.total()
    }

    pub fn write(&self) -> String {
        let outcomes = |o: &Outcomes| {
            format!(
                "{} moved {} stayed {} question {} ended {} fault {}",
                o.total(),
                o.moved,
                o.stayed,
                o.question,
                o.ended,
                o.fault
            )
        };
        let reached = match self.reached {
            Reached::No => "no",
            Reached::Rooms => "rooms",
            Reached::Trials => "trials",
        };
        let mut out = String::new();
        let mut line = |text: String| {
            out.push_str(&text);
            out.push('\n');
        };
        line(format!("story {}", self.id));
        line(format!("sha256 {}", self.sha256));
        line(format!(
            "version {} family {} seed {}",
            self.version, self.family, self.seed
        ));
        line(format!(
            "places {} walkthrough {} crawl {}",
            self.places, self.walkthrough, self.crawl
        ));
        line(format!(
            "rooms visited {} named-not-reached {}",
            self.rooms_visited, self.named_not_reached
        ));
        line(format!(
            "budget rooms {} trials {} reached {reached}",
            self.budget_rooms, self.budget_trials
        ));
        line(format!("trials {} wordless {}", self.trials, self.wordless));
        line(format!(
            "fixed confirmed {} wrong-known {} wrong-unexplained {}",
            self.fixed_confirmed, self.fixed_wrong_known, self.fixed_wrong_unexplained
        ));
        line(format!(
            "none confirmed {} wrong-known {} wrong-unexplained {} other {}",
            self.none_confirmed,
            self.none_wrong_known,
            self.none_wrong_unexplained,
            self.none_other
        ));
        line(format!("conditional {}", outcomes(&self.conditional)));
        line(format!("unknown {}", outcomes(&self.unknown)));
        line(format!("no-verdict {}", self.no_verdict));
        // Only a run that was given a pack says so: a report without these lines
        // is a run with none, and is what it was before there were packs.
        match self.pack {
            PackState::None => {}
            PackState::InForce => {
                let m = &self.marked;
                line(format!("pack in-force entries {}", self.entries));
                line(format!(
                    "open confirmed {} wrong-known {} wrong-unexplained {}",
                    m.open_confirmed, m.open_wrong_known, m.open_wrong_unexplained
                ));
                line(format!(
                    "closed confirmed {} wrong-known {} wrong-unexplained {} other {}",
                    m.closed_confirmed,
                    m.closed_wrong_known,
                    m.closed_wrong_unexplained,
                    m.closed_other
                ));
                line(format!("distrusted {}", outcomes(&m.distrusted)));
            }
            state => line(format!("pack {}", state.as_str())),
        }
        line("detail".to_string());
        // By room, then direction, then the rest as text.
        let mut detail: Vec<(u16, u8, String, usize)> = self
            .detail
            .iter()
            .map(|(case, count)| {
                let rest = format!(
                    "{} {}",
                    reported_text(case.reported),
                    outcome_text(case.outcome)
                );
                (case.room, case.direction, rest, *count)
            })
            .collect();
        detail.sort();
        for (room, direction, rest, count) in detail {
            line(format!("{room} {direction} {rest} {count}"));
        }
        out
    }

    /// Reads what [`Report::write`] writes, and nothing else: a line that is not
    /// of its form is refused, which is what keeps a game's text out of a report.
    pub fn read(text: &str) -> Result<Report, Refusal> {
        let lines: Vec<&str> = text.lines().collect();
        let refuse = |line: usize, why: &str| Err((line, why.to_string()));
        let get = |line: usize, form: &str| -> Result<Vec<usize>, Refusal> {
            let text = lines.get(line - 1).copied().unwrap_or("");
            numbers(text, form).ok_or((line, format!("expected `{form}`")))
        };
        let mut report = Report::default();

        let words: Vec<&str> = lines.first().copied().unwrap_or("").split(' ').collect();
        match words.as_slice() {
            ["story", id] if is_id(id) => report.id = id.to_string(),
            _ => return refuse(1, "expected `story <id>`"),
        }
        let words: Vec<&str> = lines.get(1).copied().unwrap_or("").split(' ').collect();
        match words.as_slice() {
            ["sha256", digest] if is_digest(digest) => report.sha256 = digest.to_string(),
            _ => return refuse(2, "expected `sha256` and 64 hexadecimal digits"),
        }
        let words: Vec<&str> = lines.get(2).copied().unwrap_or("").split(' ').collect();
        match words.as_slice() {
            [
                "version",
                version @ ("3" | "5" | "8"),
                "family",
                family,
                "seed",
                seed,
            ] if FAMILIES.contains(family) && numbers(seed, "#").is_some() => {
                report.version = version.parse().unwrap_or(0);
                report.family = family.to_string();
                match seed.parse() {
                    Ok(seed) => report.seed = seed,
                    Err(_) => return refuse(3, "the seed is too large"),
                }
            }
            _ => {
                return refuse(
                    3,
                    "expected `version <3|5|8> family <zil|inform|none> seed N`",
                );
            }
        }
        let n = get(4, "places # walkthrough # crawl #")?;
        (report.places, report.walkthrough, report.crawl) = (n[0], n[1], n[2]);
        if report.places != report.walkthrough + report.crawl {
            return refuse(4, "places is not walkthrough plus crawl");
        }
        let n = get(5, "rooms visited # named-not-reached #")?;
        (report.rooms_visited, report.named_not_reached) = (n[0], n[1]);
        let sixth = lines.get(5).copied().unwrap_or("");
        let (budget, reached) = [
            ("no", Reached::No),
            ("rooms", Reached::Rooms),
            ("trials", Reached::Trials),
        ]
        .iter()
        .find_map(|(word, reached)| {
            numbers(sixth, &format!("budget rooms # trials # reached {word}"))
                .map(|n| (n, *reached))
        })
        .ok_or((
            6,
            "expected `budget rooms N trials N reached <no|rooms|trials>`".to_string(),
        ))?;
        (report.budget_rooms, report.budget_trials, report.reached) =
            (budget[0], budget[1], reached);
        let n = get(7, "trials # wordless #")?;
        (report.trials, report.wordless) = (n[0], n[1]);
        let n = get(8, "fixed confirmed # wrong-known # wrong-unexplained #")?;
        (
            report.fixed_confirmed,
            report.fixed_wrong_known,
            report.fixed_wrong_unexplained,
        ) = (n[0], n[1], n[2]);
        let n = get(
            9,
            "none confirmed # wrong-known # wrong-unexplained # other #",
        )?;
        (
            report.none_confirmed,
            report.none_wrong_known,
            report.none_wrong_unexplained,
            report.none_other,
        ) = (n[0], n[1], n[2], n[3]);
        for (line, word) in [(10, "conditional"), (11, "unknown")] {
            let n = get(
                line,
                &format!("{word} # moved # stayed # question # ended # fault #"),
            )?;
            let outcomes = Outcomes {
                moved: n[1],
                stayed: n[2],
                question: n[3],
                ended: n[4],
                fault: n[5],
            };
            if outcomes.total() != n[0] {
                return refuse(line, "the outcomes do not add up to the total");
            }
            if line == 10 {
                report.conditional = outcomes;
            } else {
                report.unknown = outcomes;
            }
        }
        report.no_verdict = get(12, "no-verdict #")?[0];
        // The lines of a run that was given a pack, if it was.
        let mut at = 13;
        match lines.get(12).copied().unwrap_or("") {
            "pack not-matching" => (report.pack, at) = (PackState::NotMatching, 14),
            "pack refused" => (report.pack, at) = (PackState::Refused, 14),
            line if line.starts_with("pack ") => {
                report.pack = PackState::InForce;
                report.entries = get(13, "pack in-force entries #")?[0];
                let n = get(14, "open confirmed # wrong-known # wrong-unexplained #")?;
                let m = &mut report.marked;
                (m.open_confirmed, m.open_wrong_known, m.open_wrong_unexplained) =
                    (n[0], n[1], n[2]);
                let n = get(
                    15,
                    "closed confirmed # wrong-known # wrong-unexplained # other #",
                )?;
                (
                    m.closed_confirmed,
                    m.closed_wrong_known,
                    m.closed_wrong_unexplained,
                    m.closed_other,
                ) = (n[0], n[1], n[2], n[3]);
                let n = get(16, "distrusted # moved # stayed # question # ended # fault #")?;
                m.distrusted = Outcomes {
                    moved: n[1],
                    stayed: n[2],
                    question: n[3],
                    ended: n[4],
                    fault: n[5],
                };
                if m.distrusted.total() != n[0] {
                    return refuse(16, "the outcomes do not add up to the total");
                }
                at = 17;
            }
            _ => {}
        }
        if report.verdicts() != report.trials {
            return refuse(7, "the verdicts do not add up to the trials");
        }
        if lines.get(at - 1).copied() != Some("detail") {
            return refuse(at, "expected `detail`");
        }
        for (i, line) in lines.iter().enumerate().skip(at) {
            let words: Vec<&str> = line.split(' ').collect();
            let Some((case, rest)) = Case::read(&words) else {
                return refuse(i + 1, "expected `room direction reported outcome count`");
            };
            let count = match rest {
                [count] => numbers(count, "#").map(|n| n[0]),
                _ => None,
            };
            let Some(count) = count else {
                return refuse(i + 1, "expected `room direction reported outcome count`");
            };
            if case.reported.marked() && report.pack != PackState::InForce {
                return refuse(i + 1, "an answer that rests on a pack, with no pack in force");
            }
            report.detail.push((case, count));
        }
        if !text.ends_with('\n') {
            return refuse(lines.len(), "the last line does not end");
        }
        Ok(report)
    }
}

/// The figures of one line of the summary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Figures {
    pub places: usize,
    pub rooms: usize,
    pub trials: usize,
    /// Confirmed, of those reported as leading to a room.
    pub fixed: (usize, usize),
    /// Confirmed, of those reported as not an exit.
    pub none: (usize, usize),
    /// That moved the player, of those reported conditional.
    pub conditional: (usize, usize),
    pub unknown: usize,
    pub wrong_known: usize,
    pub wrong_unexplained: usize,
}

/// One story's line of the summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryLine {
    pub id: String,
    pub family: String,
    pub figures: Figures,
}

/// The figures across the corpus (contracts/formats.md, "The summary").
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// In the order of the manifest.
    pub stories: Vec<StoryLine>,
    pub skipped: Vec<String>,
}

impl Report {
    /// The story's line of the summary.
    pub fn line(&self) -> StoryLine {
        // A door reported open counts with the exits reported as leading to a
        // room; one reported closed with those reported as not an exit; a
        // distrusted exit with the conditional. The summary's form is unchanged.
        let m = &self.marked;
        let fixed_wrong = self.fixed_wrong_known
            + self.fixed_wrong_unexplained
            + m.open_wrong_known
            + m.open_wrong_unexplained;
        let none_wrong = self.none_wrong_known
            + self.none_wrong_unexplained
            + m.closed_wrong_known
            + m.closed_wrong_unexplained;
        let fixed_confirmed = self.fixed_confirmed + m.open_confirmed;
        let none_confirmed = self.none_confirmed + m.closed_confirmed;
        StoryLine {
            id: self.id.clone(),
            family: self.family.clone(),
            figures: Figures {
                places: self.places,
                rooms: self.rooms_visited,
                trials: self.trials,
                fixed: (fixed_confirmed, fixed_confirmed + fixed_wrong),
                none: (
                    none_confirmed,
                    none_confirmed + none_wrong + self.none_other + m.closed_other,
                ),
                conditional: (
                    self.conditional.moved + m.distrusted.moved,
                    self.conditional.total() + m.distrusted.total(),
                ),
                unknown: self.unknown.total(),
                wrong_known: self.fixed_wrong_known
                    + self.none_wrong_known
                    + m.open_wrong_known
                    + m.closed_wrong_known,
                wrong_unexplained: self.fixed_wrong_unexplained
                    + self.none_wrong_unexplained
                    + m.open_wrong_unexplained
                    + m.closed_wrong_unexplained,
            },
        }
    }
}

const FIGURES: &str = "places # rooms # trials # fixed #/# none #/# conditional #/# unknown # wrong-known # wrong-unexplained #";

impl Figures {
    fn text(&self) -> String {
        format!(
            "places {} rooms {} trials {} fixed {}/{} none {}/{} conditional {}/{} unknown {} \
             wrong-known {} wrong-unexplained {}",
            self.places,
            self.rooms,
            self.trials,
            self.fixed.0,
            self.fixed.1,
            self.none.0,
            self.none.1,
            self.conditional.0,
            self.conditional.1,
            self.unknown,
            self.wrong_known,
            self.wrong_unexplained
        )
    }

    /// Reads what [`Figures::text`] writes.
    fn read(text: &str) -> Option<Figures> {
        // `8/11` is two numbers: read the line with each stroke as a space.
        let spaced = text.replace('/', " ");
        let form = FIGURES.replace('/', " ");
        if text.matches('/').count() != 3 {
            return None;
        }
        let n = numbers(&spaced, &form)?;
        Some(Figures {
            places: n[0],
            rooms: n[1],
            trials: n[2],
            fixed: (n[3], n[4]),
            none: (n[5], n[6]),
            conditional: (n[7], n[8]),
            unknown: n[9],
            wrong_known: n[10],
            wrong_unexplained: n[11],
        })
    }

    fn add(&mut self, other: &Figures) {
        self.places += other.places;
        self.rooms += other.rooms;
        self.trials += other.trials;
        self.fixed = (self.fixed.0 + other.fixed.0, self.fixed.1 + other.fixed.1);
        self.none = (self.none.0 + other.none.0, self.none.1 + other.none.1);
        self.conditional = (
            self.conditional.0 + other.conditional.0,
            self.conditional.1 + other.conditional.1,
        );
        self.unknown += other.unknown;
        self.wrong_known += other.wrong_known;
        self.wrong_unexplained += other.wrong_unexplained;
    }
}

impl StoryLine {
    pub fn text(&self) -> String {
        format!("story {} {} {}", self.id, self.family, self.figures.text())
    }
}

impl Summary {
    /// The line of totals for a family, or for all with `None`.
    fn total(&self, family: Option<&str>) -> String {
        let mut figures = Figures::default();
        let mut stories = 0;
        for story in &self.stories {
            if family.is_none_or(|f| f == story.family) {
                figures.add(&story.figures);
                stories += 1;
            }
        }
        match family {
            Some(family) => format!("family {family} stories {stories} {}", figures.text()),
            None => format!("all stories {stories} {}", figures.text()),
        }
    }

    pub fn write(&self) -> String {
        let mut lines: Vec<String> = self.stories.iter().map(StoryLine::text).collect();
        lines.extend(FAMILIES.iter().map(|family| self.total(Some(family))));
        lines.push(self.total(None));
        lines.extend(self.skipped.iter().map(|id| format!("skipped {id}")));
        lines.join("\n") + "\n"
    }

    /// Reads what [`Summary::write`] writes, and nothing else. The totals must
    /// be the sums of the stories' lines.
    pub fn read(text: &str) -> Result<Summary, Refusal> {
        let mut summary = Summary::default();
        let mut totals: Vec<(usize, &str)> = Vec::new();
        for (number, line) in text.lines().enumerate().map(|(i, l)| (i + 1, l)) {
            let refuse = |why: &str| Err((number, why.to_string()));
            let words: Vec<&str> = line.splitn(4, ' ').collect();
            match words.as_slice() {
                ["story", id, family, figures]
                    if totals.is_empty() && is_id(id) && FAMILIES.contains(family) =>
                {
                    let Some(figures) = Figures::read(figures) else {
                        return refuse("a story's figures are not of the summary's form");
                    };
                    summary.stories.push(StoryLine {
                        id: id.to_string(),
                        family: family.to_string(),
                        figures,
                    });
                }
                ["family" | "all", ..] if summary.skipped.is_empty() => totals.push((number, line)),
                ["skipped", id] if is_id(id) && totals.len() == FAMILIES.len() + 1 => {
                    summary.skipped.push(id.to_string())
                }
                _ => return refuse("not a line of the summary"),
            }
        }
        let expected: Vec<String> = FAMILIES
            .iter()
            .map(|family| summary.total(Some(family)))
            .chain(std::iter::once(summary.total(None)))
            .collect();
        for (i, expected) in expected.iter().enumerate() {
            match totals.get(i) {
                Some((_, line)) if line == expected => {}
                Some((number, _)) => {
                    return Err((
                        *number,
                        "the total is not the sum of the stories".to_string(),
                    ));
                }
                None => {
                    let last = text.lines().count();
                    return Err((last + 1, "a line of totals is missing".to_string()));
                }
            }
        }
        if totals.len() > expected.len() {
            return Err((
                totals[expected.len()].0,
                "a line of totals too many".to_string(),
            ));
        }
        Ok(summary)
    }
}
