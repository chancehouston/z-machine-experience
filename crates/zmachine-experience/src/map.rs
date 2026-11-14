//! The map: the places the player has stood in and the moves they made, and
//! the page they are drawn on
//! (specs/012-map-first-slice/contracts/map.md).
//!
//! It is fed once at every stop of the game with what was sent and what the
//! layer then reports, and it is a function of those stops alone: it keeps no
//! clock, reads no chance, and asks the game nothing. So a game brought back
//! by giving it its inputs again brings its map back with it.
//!
//! It holds only what the player has witnessed. The layer can read every room
//! of a game and every way out of each; none of that is here until the player
//! has stood in the room and seen it.

use crate::compass::{self, Compass, SPOKEN_DARK};
use crate::map_place::{self, Cell, Sheet};
use crate::{Place, PlaceState, Waiting};
use std::collections::{BTreeMap, BTreeSet};

/// The layer's words where the player's place is not known.
pub const NOTE_PLACE_UNKNOWN: &str = "Where you are is not known.";
/// The layer's words for a game it cannot make a map of.
pub const NOTE_NO_MAP: &str = "No map for this game.";

/// What the map is given at a stop of the game.
pub(crate) struct Observation<'a> {
    /// The input given just before this stop, if any.
    pub sent: Option<&'a str>,
    pub waiting: Waiting,
    /// The room the player is in and its printed name, or none where the
    /// layer cannot say.
    pub room: Option<(u16, Option<String>)>,
    /// Whether it is dark there; none where that is not known.
    pub dark: Option<bool>,
    /// The compass as settled for this stop.
    pub compass: &'a Compass,
    /// Whether the game restarted, restored or undid a turn since the last
    /// stop.
    pub jumped: bool,
}

/// How a move was made, or which way a stub points.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    /// One of the compass's twelve places.
    Way(Place),
    /// Any other command, as it was sent.
    Word(String),
}

impl Step {
    /// Whether two steps are the same way: words are compared without case.
    fn same(&self, other: &Step) -> bool {
        match (self, other) {
            (Step::Way(a), Step::Way(b)) => a == b,
            (Step::Word(a), Step::Word(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }

    /// The name a snapshot gives the way.
    fn id(&self) -> &str {
        match self {
            Step::Way(place) => place.id(),
            Step::Word(_) => "word",
        }
    }

    /// The command, for a move by a word.
    fn command(&self) -> Option<&str> {
        match self {
            Step::Way(_) => None,
            Step::Word(command) => Some(command),
        }
    }

    /// The step on the page, for a compass way.
    fn delta(&self) -> Option<Cell> {
        match self {
            Step::Way(place) => map_place::delta(*place),
            Step::Word(_) => None,
        }
    }
}

/// A way out of a place, as last seen from it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Seen {
    step: Step,
    label: String,
    state: PlaceState,
    command: String,
    dark_ahead: bool,
    /// Where the layer said it leads. Kept to judge a move by (W§9, W§16);
    /// never shown.
    said: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Spot {
    /// The printed name, once seen not dark.
    name: Option<String>,
    sheet: usize,
    cell: Cell,
    /// Whether it is on the sheet's grid of insets: attached, and drawn small
    /// in the gap where four positions meet (P§3).
    inset: bool,
    attached_to: Option<u16>,
    group: Option<usize>,
    /// Whether a move by a word put it where it is (P§6).
    by_word: bool,
    /// The ways last seen here; none until the place is seen not dark.
    ways: Option<Vec<Seen>>,
    /// Ways found to lead back here (W§16).
    returns: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Line {
    a: u16,
    b: u16,
    out: Step,
    back: Option<Step>,
}

/// Two places joined by one or more walked ways.
struct Join<'a> {
    a: u16,
    b: u16,
    /// The ways walked from `a` to `b`, and from `b` to `a`, in the order
    /// first walked.
    out: Vec<&'a Step>,
    back: Vec<&'a Step>,
}

/// How a line between two places of one sheet is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineKind {
    /// A compass line that lies along its way.
    Straight,
    /// A compass line that cannot (P§8).
    Bent,
    /// Up or down.
    Ladder,
    /// In, out, or a move by a word.
    Word,
}

impl LineKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LineKind::Straight => "straight",
            LineKind::Bent => "bent",
            LineKind::Ladder => "ladder",
            LineKind::Word => "word",
        }
    }
}

/// The marker's light.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lamp {
    Lit,
    /// The layer reports the place dark.
    Out,
    Unknown,
}

impl Lamp {
    pub fn as_str(self) -> &'static str {
        match self {
            Lamp::Lit => "lit",
            Lamp::Out => "out",
            Lamp::Unknown => "unknown",
        }
    }
}

/// A place as a snapshot gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceView {
    pub id: u16,
    pub x: i32,
    pub y: i32,
    /// Whether `x` and `y` count insets: the place is drawn small, centred
    /// half a step right of and below that position.
    pub inset: bool,
    /// None until the place has been seen not dark. Numbered where another
    /// place has the same name.
    pub name: Option<String>,
    pub attached_to: Option<u16>,
}

/// A line between two places of the sheet shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineView {
    pub a: u16,
    pub b: u16,
    pub out: String,
    pub back: Option<String>,
    pub kind: LineKind,
    /// The command, for a move by a word.
    pub label: Option<String>,
}

/// A walked way that leaves the sheet shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeadView {
    pub from: u16,
    pub way: String,
    /// What to show for it: a direction's name, or the command.
    pub label: String,
    /// The far place's name, if it has one.
    pub to: Option<String>,
    pub sheet: usize,
}

/// A way seen and not walked, of a place other than the one the player is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StubView {
    pub of: u16,
    pub way: String,
    pub state: String,
    pub dark_ahead: bool,
}

/// A way of the place the player is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WayView {
    pub way: String,
    pub label: String,
    pub state: String,
    /// The line that walks it, where it can be sent now.
    pub command: Option<String>,
    /// Where it leads, once walked (W§7).
    pub leads_to: Option<String>,
    pub dark_ahead: bool,
    pub refused: bool,
    pub varies: bool,
    pub returns: bool,
    pub spoken: String,
}

/// The sheet the player is on, as a snapshot gives it (contract part S).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapView {
    pub note: Option<&'static str>,
    pub spoken: String,
    /// From 1.
    pub sheet: usize,
    pub sheets: usize,
    pub here: Option<u16>,
    pub lamp: Lamp,
    pub places: Vec<PlaceView>,
    pub lines: Vec<LineView>,
    pub leads: Vec<LeadView>,
    pub stubs: Vec<StubView>,
    pub ways: Vec<WayView>,
}

/// The map of one game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    /// False for a game the layer does not recognise: there is no map.
    recognised: bool,
    places: BTreeMap<u16, Spot>,
    /// Places in the order first reached.
    order: Vec<u16>,
    lines: Vec<Line>,
    /// Ways tried and refused (W§8).
    refusals: BTreeSet<(u16, Step)>,
    sheets: Vec<Sheet>,
    /// The places of each group still attached (P§4), first attached first.
    groups: Vec<Vec<u16>>,
    here: Option<u16>,
    /// The place at the last command stop, for judging the next move.
    last: Option<u16>,
    /// The sheet shown: the one the player is on, or was last known on.
    shown: usize,
    /// The command sent at the last command stop, until the next.
    pending: Option<String>,
    /// Whether the last stop was a command stop.
    at_command: bool,
    /// Whether the game has jumped since the last command stop.
    jumped: bool,
    dark: Option<bool>,
    /// The compass's note where the ways here are not known.
    ways_note: Option<&'static str>,
}

/// The direction a word names. Six characters are kept, as a version 3
/// dictionary keeps them.
fn direction(word: &str) -> Option<Place> {
    let short: String = word.chars().take(6).collect();
    Some(match short.as_str() {
        "n" | "north" => Place::North,
        "ne" | "northe" => Place::Northeast,
        "e" | "east" => Place::East,
        "se" | "southe" => Place::Southeast,
        "s" | "south" => Place::South,
        "sw" | "southw" => Place::Southwest,
        "w" | "west" => Place::West,
        "nw" | "northw" => Place::Northwest,
        "u" | "up" => Place::Up,
        "d" | "down" => Place::Down,
        "in" | "inside" | "enter" => Place::In,
        "out" | "outsid" | "exit" | "leave" => Place::Out,
        _ => return None,
    })
}

impl Map {
    pub(crate) fn new(recognised: bool) -> Map {
        Map {
            recognised,
            places: BTreeMap::new(),
            order: Vec::new(),
            lines: Vec::new(),
            refusals: BTreeSet::new(),
            sheets: Vec::new(),
            groups: Vec::new(),
            here: None,
            last: None,
            shown: 0,
            pending: None,
            at_command: false,
            jumped: false,
            dark: None,
            ways_note: None,
        }
    }

    /// Takes in one stop of the game (contract part W).
    pub(crate) fn observe(&mut self, seen: Observation<'_>) {
        if !self.recognised {
            return;
        }
        // The command is the one sent at a command stop; an answer to a
        // question between two command stops is not a command (W§17).
        if self.at_command
            && let Some(sent) = seen.sent
        {
            self.pending = Some(sent.trim().to_string());
        }
        self.jumped |= seen.jumped;
        self.at_command = seen.waiting == Waiting::Command;
        if !self.at_command {
            return;
        }
        let command = self.pending.take();
        let jumped = std::mem::take(&mut self.jumped);
        self.dark = seen.dark;
        self.ways_note = (!seen.compass.known).then_some(seen.compass.note).flatten();
        let Some((id, name)) = seen.room else {
            // W§13: nothing is added, and the next arrival is joined to
            // nothing.
            self.here = None;
            self.last = None;
            return;
        };
        let lit = seen.dark != Some(true);
        let from = self.last.filter(|_| !jumped);
        match (from, command) {
            (Some(from), Some(command)) if from != id => {
                let step = self.step_taken(from, &command, name.as_deref());
                if !self.places.contains_key(&id) {
                    self.put(id, from, &step);
                }
                self.join(from, id, step);
            }
            (Some(from), Some(command)) => self.stayed(from, &command),
            (Some(_), None) => {}
            // The first place; or a jump; or the place before was not known.
            (None, _) => {
                if !self.places.contains_key(&id) {
                    self.put_on_new_sheet(id);
                }
            }
        }
        if lit && let Some(spot) = self.places.get_mut(&id) {
            if name.is_some() {
                spot.name = name;
            }
            spot.ways = Some(ways_of(seen.compass));
        }
        self.here = Some(id);
        self.last = Some(id);
        self.shown = self.places[&id].sheet;
    }

    /// How the move from `from` by `command` is to be recorded, given the
    /// name of the place reached.
    fn step_taken(&self, from: u16, command: &str, reached: Option<&str>) -> Step {
        let lower = command.to_ascii_lowercase();
        let ways = self.places[&from].ways.as_deref().unwrap_or(&[]);
        // A way the place offered, by the line that walks it.
        let offered = ways
            .iter()
            .find(|way| way.command.eq_ignore_ascii_case(&lower));
        let word = lower.strip_prefix("go ").unwrap_or(&lower).trim();
        let step = match offered {
            Some(way) => way.step.clone(),
            None => match direction(word).filter(|_| !word.contains(' ')) {
                Some(place) => Step::Way(place),
                None => return Step::Word(command.to_string()),
            },
        };
        // W§9: the way said it led to one place and the player is in another.
        let said = ways
            .iter()
            .find(|way| way.step.same(&step))
            .and_then(|w| w.said.as_deref());
        match (said, reached) {
            (Some(said), Some(reached)) if said != reached => Step::Word(command.to_string()),
            _ => step,
        }
    }

    /// Puts a new place, reached from `from` by `step` (contract part P).
    fn put(&mut self, id: u16, from: u16, step: &Step) {
        let origin = self.places[&from].clone();
        let sheet = &self.sheets[origin.sheet];
        // An attached place and what is reached from it live on the insets,
        // which are a grid like the other.
        let layer = if origin.inset {
            Sheet {
                cells: sheet.insets.clone(),
                ..Sheet::default()
            }
        } else {
            sheet.clone()
        };
        let mut spot = Spot {
            name: None,
            sheet: origin.sheet,
            cell: (0, 0),
            inset: origin.inset,
            attached_to: None,
            group: origin.group,
            by_word: false,
            ways: None,
            returns: Vec::new(),
        };
        match step {
            Step::Way(Place::Up | Place::Down) => {
                let up = *step == Step::Way(Place::Up);
                let rays: Vec<Cell> = origin
                    .ways
                    .iter()
                    .flatten()
                    .filter_map(|way| way.step.delta())
                    .collect();
                spot.cell = if origin.inset {
                    map_place::ladder(&layer, origin.cell, up, &rays)
                } else {
                    map_place::corner(sheet, origin.cell, up, &rays)
                };
                spot.inset = true;
                spot.attached_to = Some(from);
                if spot.group.is_none() {
                    self.groups.push(Vec::new());
                    spot.group = Some(self.groups.len() - 1);
                }
            }
            Step::Way(Place::In | Place::Out) => {
                spot.cell = map_place::beside(&layer, origin.cell);
            }
            Step::Way(place) => {
                let d = map_place::delta(*place).unwrap_or((0, 1));
                spot.cell = map_place::along(&layer, origin.cell, d);
            }
            Step::Word(_) => {
                // P§6: on from a place itself put by a word; else a new sheet.
                if origin.by_word && sheet.by_word && !origin.inset {
                    spot.cell = map_place::below(sheet, origin.cell);
                    spot.by_word = true;
                    spot.group = None;
                } else {
                    self.put_on_new_sheet(id);
                    return;
                }
            }
        }
        let sheet = &mut self.sheets[spot.sheet];
        if spot.inset {
            sheet.insets.insert(spot.cell, id);
        } else {
            sheet.cells.insert(spot.cell, id);
        }
        let group = spot.group;
        self.places.insert(id, spot);
        self.order.push(id);
        if let Some(group) = group {
            self.groups[group].push(id);
            if self.groups[group].len() == 4 {
                self.move_group(group);
            }
        }
    }

    fn put_on_new_sheet(&mut self, id: u16) {
        // The first sheet is where the game begins; every other is reached by
        // a word, a jump, or from a place that was not known.
        let mut sheet = Sheet {
            by_word: !self.sheets.is_empty(),
            ..Sheet::default()
        };
        sheet.cells.insert((0, 0), id);
        self.sheets.push(sheet);
        let spot = Spot {
            name: None,
            sheet: self.sheets.len() - 1,
            cell: (0, 0),
            inset: false,
            attached_to: None,
            group: None,
            by_word: self.sheets.len() > 1,
            ways: None,
            returns: Vec::new(),
        };
        self.places.insert(id, spot);
        self.order.push(id);
    }

    /// P§4: a group of four moves to a sheet of its own, once, keeping its
    /// shape, with the place first attached at (0, 0). It is then no group.
    fn move_group(&mut self, group: usize) {
        let members = std::mem::take(&mut self.groups[group]);
        let Some(base) = members.first().map(|first| self.places[first].cell) else {
            return;
        };
        let mut sheet = Sheet::default();
        let number = self.sheets.len();
        for id in &members {
            let Some(spot) = self.places.get_mut(id) else {
                continue;
            };
            let old = &mut self.sheets[spot.sheet];
            if spot.inset {
                old.insets.remove(&spot.cell);
            } else {
                old.cells.remove(&spot.cell);
            }
            spot.cell = (spot.cell.0 - base.0, spot.cell.1 - base.1);
            spot.inset = false;
            spot.sheet = number;
            spot.attached_to = None;
            spot.group = None;
            sheet.cells.insert(spot.cell, *id);
        }
        self.sheets.push(sheet);
    }

    /// Records the move from `from` to `to` by `step` as a line, or as the
    /// way back along one already drawn.
    fn join(&mut self, from: u16, to: u16, step: Step) {
        let drawn = |l: &Line| {
            (l.a == from && l.b == to && l.out.same(&step))
                || (l.a == to && l.b == from && l.back.as_ref().is_some_and(|b| b.same(&step)))
        };
        if self.lines.iter().any(drawn) {
            return;
        }
        // The way back along a line walked the other way and not yet back.
        let unreturned = |l: &&mut Line| l.a == to && l.b == from && l.back.is_none();
        match self.lines.iter_mut().find(unreturned) {
            Some(line) => line.back = Some(step),
            None => self.lines.push(Line {
                a: from,
                b: to,
                out: step,
                back: None,
            }),
        }
    }

    /// The player sent `command` in `place` and is still there.
    fn stayed(&mut self, place: u16, command: &str) {
        let lower = command.to_ascii_lowercase();
        let word = lower.strip_prefix("go ").unwrap_or(&lower).trim();
        let Some(spot) = self.places.get_mut(&place) else {
            return;
        };
        // Nothing is said of ways in a place that has not been seen.
        let Some(ways) = spot.ways.as_deref() else {
            return;
        };
        let offered = ways
            .iter()
            .find(|way| way.command.eq_ignore_ascii_case(&lower));
        let step = match offered {
            Some(way) => way.step.clone(),
            None => match direction(word).filter(|_| !word.contains(' ')) {
                Some(way) => Step::Way(way),
                None => return,
            },
        };
        let seen = ways.iter().find(|way| way.step.same(&step));
        match seen.map(|way| (way.state, way.said.as_deref())) {
            // W§16: the way leads back to the place it leaves.
            Some((PlaceState::Open | PlaceState::Door, said)) => {
                if said.is_some() && said == spot.name.as_deref() && !spot.returns.contains(&step) {
                    spot.returns.push(step);
                }
            }
            // W§8: not offered as open, tried, and the player is where they
            // were.
            _ => {
                self.refusals.insert((place, step));
            }
        }
    }

    /// Whether the player has walked `step` out of `place`, and where to.
    fn walked(&self, place: u16, step: &Step) -> Vec<u16> {
        let mut ends: Vec<u16> = Vec::new();
        for line in &self.lines {
            let end = if line.a == place && line.out.same(step) {
                line.b
            } else if line.b == place && line.back.as_ref().is_some_and(|b| b.same(step)) {
                line.a
            } else {
                continue;
            };
            if !ends.contains(&end) {
                ends.push(end);
            }
        }
        ends
    }

    /// A place's name as shown: numbered where another place has it (W§15).
    fn name(&self, id: u16) -> Option<String> {
        let name = self.places.get(&id)?.name.as_deref()?;
        let same: Vec<u16> = self
            .order
            .iter()
            .copied()
            .filter(|other| self.places[other].name.as_deref() == Some(name))
            .collect();
        if same.len() < 2 {
            return Some(name.to_string());
        }
        let number = same.iter().position(|other| *other == id)? + 1;
        Some(format!("{name} {number}"))
    }

    /// Whether the way `step` from `from` lies along the page to `to`
    /// (P§8): both on one grid of one sheet, `to` exactly along the way, and
    /// no place between.
    fn lies_along(&self, from: u16, step: &Step, to: u16) -> bool {
        let (a, b) = (&self.places[&from], &self.places[&to]);
        let Some(d) = step.delta() else { return false };
        if a.sheet != b.sheet || a.inset != b.inset {
            return false;
        }
        // Each grid is judged by itself.
        let sheet = &self.sheets[a.sheet];
        let layer = if a.inset {
            Sheet {
                cells: sheet.insets.clone(),
                ..Sheet::default()
            }
        } else {
            sheet.clone()
        };
        map_place::straight(&layer, a.cell, d, b.cell, None)
    }

    /// The joins to draw: one for each pair of places with a walked way
    /// between them, in the order first walked. Two places may be joined by
    /// several ways (north and northeast both lead from one to the other, and
    /// west and southwest back); the page shows that they are joined once,
    /// and each place's own ways say how.
    fn joins(&self) -> Vec<Join<'_>> {
        let mut joins: Vec<Join<'_>> = Vec::new();
        for line in &self.lines {
            let pair = (line.a.min(line.b), line.a.max(line.b));
            let at = match joins
                .iter()
                .position(|j| (j.a.min(j.b), j.a.max(j.b)) == pair)
            {
                Some(at) => at,
                None => {
                    joins.push(Join {
                        a: line.a,
                        b: line.b,
                        out: Vec::new(),
                        back: Vec::new(),
                    });
                    joins.len() - 1
                }
            };
            let join = &mut joins[at];
            let (forward, backward) = if join.a == line.a {
                (&mut join.out, &mut join.back)
            } else {
                (&mut join.back, &mut join.out)
            };
            forward.push(&line.out);
            backward.extend(line.back.as_ref());
        }
        joins
    }

    /// How a join is drawn, and the way from each end that it is drawn by:
    /// a way that lies along the page if there is one.
    fn drawn<'a>(&self, join: &Join<'a>) -> (LineKind, Option<&'a Step>, Option<&'a Step>) {
        let along_out = join
            .out
            .iter()
            .copied()
            .find(|s| self.lies_along(join.a, s, join.b));
        let along_back = join
            .back
            .iter()
            .copied()
            .find(|s| self.lies_along(join.b, s, join.a));
        let first =
            |steps: &[&'a Step], test: fn(&Step) -> bool| steps.iter().copied().find(|s| test(s));
        let compass: fn(&Step) -> bool = |s| s.delta().is_some();
        let ladder: fn(&Step) -> bool = |s| matches!(s, Step::Way(Place::Up | Place::Down));
        let any: fn(&Step) -> bool = |_| true;
        if along_out.is_some() || along_back.is_some() {
            return (
                LineKind::Straight,
                along_out
                    .or(first(&join.out, compass))
                    .or(first(&join.out, any)),
                along_back
                    .or(first(&join.back, compass))
                    .or(first(&join.back, any)),
            );
        }
        for (kind, test) in [(LineKind::Bent, compass), (LineKind::Ladder, ladder)] {
            if first(&join.out, test).is_some() || first(&join.back, test).is_some() {
                return (
                    kind,
                    first(&join.out, test).or(first(&join.out, any)),
                    first(&join.back, test).or(first(&join.back, any)),
                );
            }
        }
        (
            LineKind::Word,
            first(&join.out, any),
            first(&join.back, any),
        )
    }

    /// The sheet and position of a place: the sheet from 1.
    pub fn position(&self, id: u16) -> Option<(usize, i32, i32)> {
        let spot = self.places.get(&id)?;
        Some((spot.sheet + 1, spot.cell.0, spot.cell.1))
    }

    /// Whether a place is attached and drawn in the gap between positions;
    /// its position then counts insets (P§3).
    pub fn is_inset(&self, id: u16) -> bool {
        self.places.get(&id).is_some_and(|spot| spot.inset)
    }

    /// How many sheets hold a place.
    pub fn sheet_count(&self) -> usize {
        self.sheets
            .iter()
            .filter(|sheet| !sheet.cells.is_empty())
            .count()
    }

    /// The number shown for a sheet: sheets left empty by a group's moving
    /// are not counted.
    fn sheet_number(&self, index: usize) -> usize {
        self.sheets[..=index]
            .iter()
            .filter(|sheet| !sheet.cells.is_empty())
            .count()
    }

    /// The sheet the player is on, for a snapshot; none where there is no
    /// map for this game.
    pub fn view(&self) -> Option<MapView> {
        if !self.recognised || self.sheets.is_empty() {
            return None;
        }
        let on = |id: u16| self.places[&id].sheet == self.shown;
        let places = self
            .places
            .iter()
            .filter(|(_, spot)| spot.sheet == self.shown)
            .map(|(id, spot)| PlaceView {
                id: *id,
                x: spot.cell.0,
                y: spot.cell.1,
                inset: spot.inset,
                name: self.name(*id),
                attached_to: spot.attached_to.filter(|to| on(*to)),
            })
            .collect();
        let mut lines = Vec::new();
        for join in self.joins() {
            if !(on(join.a) && on(join.b)) {
                continue;
            }
            let (kind, out, back) = self.drawn(&join);
            // Drawn from the end that has a way to draw it by.
            let (a, b, out, back) = match out {
                Some(out) => (join.a, join.b, out, back),
                None => match back {
                    Some(back) => (join.b, join.a, back, None),
                    None => continue,
                },
            };
            lines.push(LineView {
                a,
                b,
                out: out.id().to_string(),
                back: back.map(|back| back.id().to_string()),
                kind,
                label: out.command().map(str::to_string),
            });
        }
        let mut leads = Vec::new();
        for line in &self.lines {
            let (from, to, step) = match (on(line.a), on(line.b)) {
                (true, false) => (line.a, line.b, Some(&line.out)),
                (false, true) => (line.b, line.a, line.back.as_ref()),
                _ => continue,
            };
            // A line only ever walked toward this sheet leads nowhere from
            // it that the player has seen.
            let Some(step) = step else { continue };
            leads.push(LeadView {
                from,
                way: step.id().to_string(),
                label: match step {
                    Step::Way(place) => place.label().to_string(),
                    Step::Word(command) => command.clone(),
                },
                to: self.name(to),
                sheet: self.sheet_number(self.places[&to].sheet),
            });
        }
        let mut stubs = Vec::new();
        for (id, spot) in &self.places {
            if spot.sheet != self.shown || Some(*id) == self.here {
                continue;
            }
            for way in spot.ways.iter().flatten() {
                if self.walked(*id, &way.step).is_empty() {
                    stubs.push(StubView {
                        of: *id,
                        way: way_name(way),
                        state: way.state.as_str().to_string(),
                        dark_ahead: way.dark_ahead,
                    });
                }
            }
        }
        let here_name = self.here.and_then(|id| self.name(id));
        let (note, spoken) = match (self.here, here_name) {
            (None, _) => (
                Some(NOTE_PLACE_UNKNOWN),
                format!("Map. {NOTE_PLACE_UNKNOWN}"),
            ),
            (Some(_), Some(name)) => (self.ways_note, format!("Map. You are in {name}.")),
            (Some(_), None) => (self.ways_note, "Map. You are somewhere dark.".to_string()),
        };
        Some(MapView {
            note,
            spoken,
            sheet: self.sheet_number(self.shown),
            sheets: self.sheet_count(),
            here: self.here,
            lamp: match (self.here, self.dark) {
                (None, _) | (_, None) => Lamp::Unknown,
                (_, Some(true)) => Lamp::Out,
                (_, Some(false)) => Lamp::Lit,
            },
            places,
            lines,
            leads,
            stubs,
            ways: self.here.map(|id| self.ways(id)).unwrap_or_default(),
        })
    }

    /// The ways of the place the player is in (S§7): those seen there, in the
    /// compass's order and then the extras, and those tried and refused.
    fn ways(&self, id: u16) -> Vec<WayView> {
        let spot = &self.places[&id];
        let seen = spot.ways.as_deref().unwrap_or(&[]);
        let refused = |step: &Step| {
            self.refusals
                .iter()
                .any(|(at, s)| *at == id && s.same(step))
        };
        let mut ways: Vec<(usize, WayView)> = Vec::new();
        let rank = |step: &Step, extra: usize| match step {
            Step::Way(place) => Place::ALL.iter().position(|p| p == place).unwrap_or(12),
            Step::Word(_) => 12 + extra,
        };
        for (n, way) in seen.iter().enumerate() {
            let ends = self.walked(id, &way.step);
            let varies = ends.len() > 1;
            // W§7: where it leads is named only once walked.
            let leads_to = match ends.as_slice() {
                [only] => self.name(*only),
                _ => None,
            };
            ways.push((
                rank(&way.step, n),
                WayView {
                    way: way_name(way),
                    label: way.label.clone(),
                    state: way.state.as_str().to_string(),
                    command: self.at_command.then(|| way.command.clone()),
                    spoken: compass::spoken(
                        &way.label,
                        way.state,
                        leads_to.as_deref(),
                        way.dark_ahead,
                    ),
                    leads_to,
                    dark_ahead: way.dark_ahead,
                    refused: refused(&way.step),
                    varies,
                    returns: spot.returns.iter().any(|step| step.same(&way.step)),
                },
            ));
        }
        // W§8: ways not offered that were tried.
        for (at, step) in &self.refusals {
            let Step::Way(place) = step else { continue };
            if *at != id || seen.iter().any(|way| way.step.same(step)) {
                continue;
            }
            ways.push((
                rank(step, 0),
                WayView {
                    way: place.id().to_string(),
                    label: place.label().to_string(),
                    state: "refused".to_string(),
                    command: None,
                    leads_to: None,
                    dark_ahead: false,
                    refused: true,
                    varies: false,
                    returns: false,
                    spoken: compass::spoken(place.label(), PlaceState::Closed, None, false),
                },
            ));
        }
        ways.sort_by_key(|(rank, _)| *rank);
        ways.into_iter().map(|(_, way)| way).collect()
    }

    /// The whole map as sorted lines of text, one for each place, line and
    /// refusal: for tests, and for the recorded pages of the stories the
    /// repository may hold. It holds rooms' names.
    pub fn page(&self) -> Vec<String> {
        if !self.recognised {
            return Vec::new();
        }
        let mut page = Vec::new();
        for (id, spot) in &self.places {
            page.push(format!(
                "place {id:5} sheet {:2} {} {:3},{:3} {} {}",
                self.sheet_number(spot.sheet),
                if spot.inset { "inset" } else { "at   " },
                spot.cell.0,
                spot.cell.1,
                if spot.ways.is_some() {
                    "seen  "
                } else {
                    "unseen"
                },
                self.name(*id).unwrap_or_else(|| "-".to_string()),
            ));
        }
        for line in &self.lines {
            let across = self.places[&line.a].sheet != self.places[&line.b].sheet;
            let step = |step: &Step| match step {
                Step::Way(place) => place.id().to_string(),
                Step::Word(command) => format!("\"{command}\""),
            };
            page.push(format!(
                "line  {:5} {} -> {:5}{}{}",
                line.a,
                step(&line.out),
                line.b,
                line.back
                    .as_ref()
                    .map_or(String::new(), |b| format!(", back {}", step(b))),
                if across { " across sheets" } else { "" },
            ));
        }
        // What is drawn: one join for each pair of places on a sheet.
        for join in self.joins() {
            if self.places[&join.a].sheet == self.places[&join.b].sheet {
                page.push(format!(
                    "drawn {:5} -- {:5} {}",
                    join.a.min(join.b),
                    join.a.max(join.b),
                    self.drawn(&join).0.as_str()
                ));
            }
        }
        for (id, step) in &self.refusals {
            if let Step::Way(place) = step {
                page.push(format!("refused {id:5} {}", place.id()));
            }
        }
        page.sort();
        page
    }

    /// How many pairs of straight lines of one sheet cross. For the tests of
    /// the measures.
    pub fn crossings(&self) -> usize {
        let joins = self.joins();
        let segments: Vec<(usize, Cell, Cell, u16, u16)> = joins
            .iter()
            .filter(|join| self.places[&join.a].sheet == self.places[&join.b].sheet)
            .filter(|join| self.drawn(join).0 == LineKind::Straight)
            .map(|join| {
                let (a, b) = (&self.places[&join.a], &self.places[&join.b]);
                (a.sheet, a.cell, b.cell, join.a, join.b)
            })
            .collect();
        let side = |o: Cell, a: Cell, b: Cell| {
            ((a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)).signum()
        };
        let mut count = 0;
        for (n, s) in segments.iter().enumerate() {
            for t in &segments[n + 1..] {
                let shares_an_end = [t.3, t.4].contains(&s.3) || [t.3, t.4].contains(&s.4);
                if s.0 != t.0 || shares_an_end {
                    continue;
                }
                if side(s.1, s.2, t.1) * side(s.1, s.2, t.2) < 0
                    && side(t.1, t.2, s.1) * side(t.1, t.2, s.2) < 0
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// For each sheet, the count of its places and of its compass lines that
    /// are straight and bent. For the tests of the measures.
    pub fn measures(&self) -> Vec<(usize, usize, usize)> {
        let mut sheets: Vec<(usize, usize, usize)> = self
            .sheets
            .iter()
            .filter(|sheet| !sheet.cells.is_empty())
            .map(|sheet| (sheet.cells.len() + sheet.insets.len(), 0, 0))
            .collect();
        for join in self.joins() {
            let (a, b) = (&self.places[&join.a], &self.places[&join.b]);
            if a.sheet != b.sheet {
                continue;
            }
            let at = self.sheet_number(a.sheet) - 1;
            match self.drawn(&join).0 {
                LineKind::Straight => sheets[at].1 += 1,
                LineKind::Bent => sheets[at].2 += 1,
                _ => {}
            }
        }
        sheets
    }
}

/// The name a snapshot gives a way: a compass place's id, or an extra's label.
fn way_name(way: &Seen) -> String {
    match &way.step {
        Step::Way(place) => place.id().to_string(),
        Step::Word(_) => way.label.clone(),
    }
}

/// The ways a compass offers: its places and extras that are open, uncertain
/// or a door (W§3).
fn ways_of(compass: &Compass) -> Vec<Seen> {
    let offered = |state: PlaceState| {
        matches!(
            state,
            PlaceState::Open | PlaceState::Maybe | PlaceState::Door
        )
    };
    let mut ways = Vec::new();
    for place in &compass.places {
        if let (true, Some(command)) = (offered(place.state), &place.command) {
            ways.push(Seen {
                step: Step::Way(place.place),
                label: place.label.to_string(),
                state: place.state,
                command: command.clone(),
                dark_ahead: place.dark_ahead,
                said: place.leads_to.clone(),
            });
        }
    }
    for extra in &compass.extras {
        if offered(extra.state) {
            ways.push(Seen {
                step: Step::Word(extra.command.clone()),
                label: extra.label.clone(),
                state: extra.state,
                command: extra.command.clone(),
                dark_ahead: extra.dark_ahead,
                said: extra.leads_to.clone(),
            });
        }
    }
    ways
}

// The words a screen reader adds for a way into the dark are the compass's.
const _: &str = SPOKEN_DARK;
