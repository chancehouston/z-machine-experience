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
    /// The comparison of what is seen of the inventory with the game's own
    /// listing, for a story whose manifest gives its inventory command.
    pub inventory: Option<Inventory>,
    /// Each difference, with how many times it was met.
    pub inventory_detail: Vec<(Difference, usize)>,
    /// The comparison of what the interface says about light with whether the
    /// game describes the room, for a story whose manifest gives its look
    /// command (specs/008-darkness/research.md R7).
    pub light: Option<Light>,
    pub light_detail: Vec<(LightDifference, usize)>,
    /// What look-ahead found for the ways the interface could not settle, for
    /// a story that has such ways at the places reached
    /// (specs/009-compass-lookahead/contracts/corpus.md).
    pub lookahead: Option<Looked>,
    /// Each finding that a trial apart from it did not bear out, as a case:
    /// `fixed N` stands for "found walked to N" and `none` for "found
    /// refused"; the outcome is the trial's.
    pub lookahead_detail: Vec<(Case, usize)>,
}

/// Look-ahead's figures: of the ways asked, how many were found walked, found
/// refused, and left; and, among the first two, how many findings a trial
/// apart did not bear out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Looked {
    pub walked: usize,
    pub refused: usize,
    pub left: usize,
    pub missed_known: usize,
    pub missed_unexplained: usize,
}

impl Looked {
    pub fn asked(&self) -> usize {
        self.walked + self.refused + self.left
    }
}

/// A finding not borne out, as a detail line and a failure name it, without a
/// count: `lookahead ROOM DIRECTION walked N|refused OUTCOME`.
pub fn missed_text(case: &Case) -> String {
    let found = match case.reported {
        Reported::Fixed(room) => format!("walked {room}"),
        _ => "refused".to_string(),
    };
    format!(
        "lookahead {} {} {found} {}",
        case.room,
        case.direction,
        outcome_text(case.outcome)
    )
}

/// Reads what [`missed_text`] writes, from the words of a line after its first;
/// returns the case and the words left over. A pairing that is borne out, and
/// a trial that was lost, are not of the form.
pub fn missed_read<'a>(words: &[&'a str]) -> Option<(Case, Vec<&'a str>)> {
    let mut as_case: Vec<&str> = Vec::new();
    as_case.extend(words.get(..2)?);
    let rest = match *words.get(2)? {
        "walked" => {
            as_case.push("fixed");
            as_case.push(words.get(3)?);
            &words[4..]
        }
        "refused" => {
            as_case.push("none");
            &words[3..]
        }
        _ => return None,
    };
    as_case.extend(rest);
    let (case, left) = Case::read(&as_case)?;
    if crate::verdict::judge(&case) == crate::verdict::Verdict::Confirmed
        || case.outcome == Outcome::Lost
    {
        return None;
    }
    // What is left over is the end of the words given.
    let over = left.len();
    Some((case, words[words.len() - over..].to_vec()))
}

/// The light comparison's figures. The first group is of places; the second of
/// trials that walked a way to the room it named.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Light {
    pub compared: usize,
    pub agreed: usize,
    pub differed_known: usize,
    pub differed_unexplained: usize,
    pub unjudged: usize,
    /// Places where the interface said dark and the game agreed.
    pub dark: usize,
    /// Rooms seen dark at one place and lit at another: a light, and not the
    /// room, made the difference.
    pub rooms_both: usize,
    /// Trials at a place reported dark whose verdict would have been wrong.
    pub in_the_dark: usize,
    pub ahead_compared: usize,
    pub ahead_agreed: usize,
    pub ahead_differed_known: usize,
    pub ahead_differed_unexplained: usize,
    pub ahead_unjudged: usize,
}

/// A place, or a way walked, where the interface and the game disagree about
/// light. `direction` is `None` for the light of a place, and the way's
/// direction for dark ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LightDifference {
    pub room: u16,
    pub direction: Option<u8>,
    /// What the interface said: dark, or lit.
    pub dark: bool,
    /// Whether the game described the room.
    pub described: bool,
}

impl LightDifference {
    /// The difference as a detail line and a failure name it, without a count.
    pub fn text(&self) -> String {
        let said = if self.dark { "dark" } else { "lit" };
        let game = if self.described {
            "described"
        } else {
            "undescribed"
        };
        match self.direction {
            None => format!("light {} {said} {game}", self.room),
            Some(direction) => format!("ahead {} {direction} {said} {game}", self.room),
        }
    }
}

/// The inventory comparison's figures (specs/007-inventory-panel/research.md
/// R9). The first five count objects; the next three count places.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub compared: usize,
    pub agreed: usize,
    pub differed_known: usize,
    pub differed_unexplained: usize,
    /// Objects the interface could not say were seen or not.
    pub unjudged: usize,
    pub no_listing: usize,
    pub empty: usize,
    pub unknown: usize,
    /// Objects seen through a closed thing, and listed: what proves a
    /// `transparent-attribute`.
    pub through_closed: usize,
}

/// A carried object the interface and the game's listing disagree about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Difference {
    pub room: u16,
    pub object: u16,
    /// What the interface said: seen, or kept back.
    pub seen: bool,
    pub listed: bool,
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
            + self.light.map_or(0, |light| light.in_the_dark)
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
        // Only a story whose manifest gives its inventory command has this line.
        if let Some(i) = &self.inventory {
            line(format!(
                "inventory compared {} agreed {} differed-known {} differed-unexplained {} \
                 unjudged {} no-listing {} empty {} unknown {} through-closed {}",
                i.compared,
                i.agreed,
                i.differed_known,
                i.differed_unexplained,
                i.unjudged,
                i.no_listing,
                i.empty,
                i.unknown,
                i.through_closed
            ));
        }
        // Only a story whose manifest gives its look command has these lines.
        if let Some(l) = &self.light {
            line(format!(
                "light compared {} agreed {} differed-known {} differed-unexplained {} \
                 unjudged {} dark {} rooms-both {} in-the-dark {}",
                l.compared,
                l.agreed,
                l.differed_known,
                l.differed_unexplained,
                l.unjudged,
                l.dark,
                l.rooms_both,
                l.in_the_dark
            ));
            line(format!(
                "ahead compared {} agreed {} differed-known {} differed-unexplained {} unjudged {}",
                l.ahead_compared,
                l.ahead_agreed,
                l.ahead_differed_known,
                l.ahead_differed_unexplained,
                l.ahead_unjudged
            ));
        }
        // Only a story with ways the interface could not settle has this line.
        if let Some(l) = &self.lookahead {
            line(format!(
                "lookahead walked {} refused {} left {} not-borne-out-known {} \
                 not-borne-out-unexplained {}",
                l.walked, l.refused, l.left, l.missed_known, l.missed_unexplained
            ));
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
        let mut differences = self.inventory_detail.clone();
        differences.sort();
        for (d, count) in differences {
            line(format!(
                "inventory {} {} {} {} {count}",
                d.room,
                d.object,
                if d.seen { "seen" } else { "kept" },
                if d.listed { "listed" } else { "unlisted" }
            ));
        }
        let mut lights = self.light_detail.clone();
        lights.sort();
        for (d, count) in lights {
            line(format!("{} {count}", d.text()));
        }
        let mut missed = self.lookahead_detail.clone();
        missed.sort();
        for (case, count) in missed {
            line(format!("{} {count}", missed_text(&case)));
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
                (
                    m.open_confirmed,
                    m.open_wrong_known,
                    m.open_wrong_unexplained,
                ) = (n[0], n[1], n[2]);
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
                let n = get(
                    16,
                    "distrusted # moved # stayed # question # ended # fault #",
                )?;
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
        if lines
            .get(at - 1)
            .is_some_and(|line| line.starts_with("inventory "))
        {
            let n = get(
                at,
                "inventory compared # agreed # differed-known # differed-unexplained # unjudged # \
                 no-listing # empty # unknown # through-closed #",
            )?;
            let inventory = Inventory {
                compared: n[0],
                agreed: n[1],
                differed_known: n[2],
                differed_unexplained: n[3],
                unjudged: n[4],
                no_listing: n[5],
                empty: n[6],
                unknown: n[7],
                through_closed: n[8],
            };
            if inventory.compared
                != inventory.agreed
                    + inventory.differed_known
                    + inventory.differed_unexplained
                    + inventory.unjudged
            {
                return refuse(at, "the objects compared do not add up");
            }
            report.inventory = Some(inventory);
            at += 1;
        }
        if lines
            .get(at - 1)
            .is_some_and(|line| line.starts_with("light "))
        {
            let n = get(
                at,
                "light compared # agreed # differed-known # differed-unexplained # unjudged # \
                 dark # rooms-both # in-the-dark #",
            )?;
            let m = get(
                at + 1,
                "ahead compared # agreed # differed-known # differed-unexplained # unjudged #",
            )?;
            if n[0] != n[1] + n[2] + n[3] + n[4] {
                return refuse(at, "the places compared do not add up");
            }
            if m[0] != m[1] + m[2] + m[3] + m[4] {
                return refuse(at + 1, "the ways compared do not add up");
            }
            report.light = Some(Light {
                compared: n[0],
                agreed: n[1],
                differed_known: n[2],
                differed_unexplained: n[3],
                unjudged: n[4],
                dark: n[5],
                rooms_both: n[6],
                in_the_dark: n[7],
                ahead_compared: m[0],
                ahead_agreed: m[1],
                ahead_differed_known: m[2],
                ahead_differed_unexplained: m[3],
                ahead_unjudged: m[4],
            });
            at += 2;
        }
        if lines
            .get(at - 1)
            .is_some_and(|line| line.starts_with("lookahead "))
        {
            let n = get(
                at,
                "lookahead walked # refused # left # not-borne-out-known # \
                 not-borne-out-unexplained #",
            )?;
            if n[3] + n[4] > n[0] + n[1] {
                return refuse(at, "more findings not borne out than findings");
            }
            report.lookahead = Some(Looked {
                walked: n[0],
                refused: n[1],
                left: n[2],
                missed_known: n[3],
                missed_unexplained: n[4],
            });
            at += 1;
        }
        if report.verdicts() != report.trials {
            return refuse(7, "the verdicts do not add up to the trials");
        }
        if lines.get(at - 1).copied() != Some("detail") {
            return refuse(at, "expected `detail`");
        }
        for (i, line) in lines.iter().enumerate().skip(at) {
            let words: Vec<&str> = line.split(' ').collect();
            // The inventory's differences come after every exit's detail.
            if words.first() == Some(&"inventory") {
                let number = |word: &str| numbers(word, "#").and_then(|n| u16::try_from(n[0]).ok());
                let difference = match words.as_slice() {
                    [
                        _,
                        room,
                        object,
                        seen @ ("seen" | "kept"),
                        listed @ ("listed" | "unlisted"),
                        count,
                    ] if report.inventory.is_some() => {
                        match (number(room), number(object), numbers(count, "#")) {
                            (Some(room), Some(object), Some(count)) => Some((
                                Difference {
                                    room,
                                    object,
                                    seen: *seen == "seen",
                                    listed: *listed == "listed",
                                },
                                count[0],
                            )),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                match difference {
                    Some((d, _)) if d.seen == d.listed => {
                        return refuse(i + 1, "seen and listed do not differ");
                    }
                    Some(found) => report.inventory_detail.push(found),
                    None => {
                        return refuse(
                            i + 1,
                            "expected `inventory room object seen|kept listed|unlisted count`",
                        );
                    }
                }
                continue;
            }
            // And after the inventory's, the differences about light.
            if matches!(words.first(), Some(&"light") | Some(&"ahead")) {
                let number = |word: &str| numbers(word, "#").and_then(|n| u16::try_from(n[0]).ok());
                let found = match words.as_slice() {
                    [
                        "light",
                        room,
                        said @ ("lit" | "dark"),
                        game @ ("described" | "undescribed"),
                        count,
                    ] if report.light.is_some() => {
                        number(room).zip(numbers(count, "#")).map(|(room, count)| {
                            (
                                LightDifference {
                                    room,
                                    direction: None,
                                    dark: *said == "dark",
                                    described: *game == "described",
                                },
                                count[0],
                            )
                        })
                    }
                    [
                        "ahead",
                        room,
                        direction,
                        said @ ("lit" | "dark"),
                        game @ ("described" | "undescribed"),
                        count,
                    ] if report.light.is_some() => {
                        match (
                            number(room),
                            number(direction).and_then(|d| u8::try_from(d).ok()),
                            numbers(count, "#"),
                        ) {
                            (Some(room), Some(direction), Some(count)) => Some((
                                LightDifference {
                                    room,
                                    direction: Some(direction),
                                    dark: *said == "dark",
                                    described: *game == "described",
                                },
                                count[0],
                            )),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                match found {
                    // Lit and described, or dark and not, is no difference.
                    Some((d, _)) if d.dark != d.described => {
                        return refuse(i + 1, "what was said and what the game did do not differ");
                    }
                    Some(found) => report.light_detail.push(found),
                    None => {
                        return refuse(i + 1, "not a line about light of the report's form");
                    }
                }
                continue;
            }
            // And last, look-ahead's findings that were not borne out.
            if words.first() == Some(&"lookahead") {
                let found = missed_read(&words[1..]).and_then(|(case, rest)| match rest[..] {
                    [count] => numbers(count, "#").map(|n| (case, n[0])),
                    _ => None,
                });
                match found {
                    Some(found) if report.lookahead.is_some() => {
                        report.lookahead_detail.push(found)
                    }
                    _ => {
                        return refuse(
                            i + 1,
                            "expected `lookahead room direction walked N|refused outcome count`",
                        );
                    }
                }
                continue;
            }
            if !report.lookahead_detail.is_empty() {
                return refuse(i + 1, "an earlier kind of detail after look-ahead's");
            }
            if !report.light_detail.is_empty() {
                return refuse(i + 1, "an earlier kind of detail after light's");
            }
            if !report.inventory_detail.is_empty() {
                return refuse(i + 1, "an exit's detail after the inventory's");
            }
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
                return refuse(
                    i + 1,
                    "an answer that rests on a pack, with no pack in force",
                );
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
    /// Look-ahead: found walked, found refused, left, and not borne out.
    pub looked: [usize; 4],
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
                looked: self.lookahead.map_or([0; 4], |l| {
                    [
                        l.walked,
                        l.refused,
                        l.left,
                        l.missed_known + l.missed_unexplained,
                    ]
                }),
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
        ) + &self.looked_text()
    }

    /// Look-ahead's figures, for a line that has any; nothing otherwise, so
    /// that a line with none is what it was before there was look-ahead.
    fn looked_text(&self) -> String {
        if self.looked == [0; 4] {
            return String::new();
        }
        let [walked, refused, left, missed] = self.looked;
        format!(" lookahead walked {walked} refused {refused} left {left} not-borne-out {missed}")
    }

    /// Reads what [`Figures::text`] writes.
    fn read(text: &str) -> Option<Figures> {
        // `8/11` is two numbers: read the line with each stroke as a space.
        let (text, looked) = match text.split_once(" lookahead ") {
            Some((before, after)) => {
                let n = numbers(after, "walked # refused # left # not-borne-out #")?;
                let looked = [n[0], n[1], n[2], n[3]];
                // A line with none says nothing of look-ahead.
                if looked == [0; 4] {
                    return None;
                }
                (before, looked)
            }
            None => (text, [0; 4]),
        };
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
            looked,
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
        for (mine, theirs) in self.looked.iter_mut().zip(other.looked) {
            *mine += theirs;
        }
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
