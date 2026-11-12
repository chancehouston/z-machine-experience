//! Read-only introspection of a running Z-machine game.
//!
//! The contract is `docs/introspection.md` in this repository; this crate is one
//! implementation of it. Nothing here writes to the machine, runs the game's code,
//! or knows any one game.
#![forbid(unsafe_code)]

mod answer;
mod family;
mod inform;
pub mod json;
mod table;
mod zil;

pub use answer::{Answer, Exactness, Part, Tri, Unknown};

use family::{Adapter, Place, Way};
use table::{Bytes, Layout};
use zmachine_core::View;

#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod family_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod inform_tests;
#[cfg(test)]
mod json_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod table_tests;
#[cfg(test)]
mod zil_tests;

/// The version of `docs/introspection.md` this crate implements.
pub const SPEC_VERSION: &str = "0.1.0";

/// The queries, as the specification names them and in its order.
pub const QUERIES: [&str; 6] = [
    "object_tree",
    "dictionary",
    "location",
    "inventory",
    "exits",
    "scope",
];

/// The compiler and library a story file was made with, which decide how it keeps
/// the player, rooms and exits (specification section 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Inform6,
    Zil,
    Unrecognised,
}

impl Family {
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Inform6 => "inform6",
            Family::Zil => "zil",
            Family::Unrecognised => "unrecognised",
        }
    }
}

/// One object of the game's object tree (specification section 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub number: u16,
    /// Its short name as stored; may be empty.
    pub name: String,
    /// What the game prints for it.
    pub printed_name: Part<String>,
    pub parent: Part<Option<u16>>,
    /// In the tree's order.
    pub children: Part<Vec<u16>>,
    /// The attributes that are set.
    pub attributes: Vec<u8>,
    /// The properties it provides, in table order.
    pub properties: Vec<u8>,
}

/// What a dictionary word can be (specification section 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Direction,
    Preposition,
    GrammarWord,
    Plural,
    Meta,
    Buzz,
}

impl PartOfSpeech {
    pub fn as_str(self) -> &'static str {
        match self {
            PartOfSpeech::Noun => "noun",
            PartOfSpeech::Verb => "verb",
            PartOfSpeech::Adjective => "adjective",
            PartOfSpeech::Direction => "direction",
            PartOfSpeech::Preposition => "preposition",
            PartOfSpeech::GrammarWord => "grammar-word",
            PartOfSpeech::Plural => "plural",
            PartOfSpeech::Meta => "meta",
            PartOfSpeech::Buzz => "buzz",
        }
    }
}

/// One word of the game's dictionary (specification section 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    /// As stored: cut to 6 characters in version 3, 9 in versions 5 and 8.
    pub word: String,
    pub parts: Part<Vec<PartOfSpeech>>,
    /// For a direction word, the direction's number.
    pub direction: Option<u8>,
}

/// Where the player is (specification section 6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub room: u16,
    /// The room's printed name.
    pub name: Part<String>,
    /// What the player is in or on within the room, if anything.
    pub holder: Part<Option<u16>>,
    pub lit: Tri,
}

/// Something the player carries (specification section 7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carried {
    pub object: u16,
    pub name: Part<String>,
    pub worn: Tri,
    pub contents: Vec<Carried>,
}

/// Whether an exit's destination is a stored fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExitKind {
    Fixed,
    Conditional,
}

impl ExitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ExitKind::Fixed => "fixed",
            ExitKind::Conditional => "conditional",
        }
    }
}

/// A way out of a room (specification section 8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exit {
    /// The direction's number, stable within a story file.
    pub direction: u8,
    /// The dictionary words that name it, shortest first. May be empty.
    pub words: Vec<String>,
    pub kind: ExitKind,
    /// The room it leads to, for a fixed exit.
    pub destination: Option<u16>,
    /// That room's printed name, for a fixed exit.
    pub destination_name: Option<Part<String>>,
}

/// A direction whose exit could not be determined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownDirection {
    pub direction: u8,
    pub words: Vec<String>,
    pub reason: Unknown,
}

/// The exits of the player's location (specification section 8).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Exits {
    pub exits: Vec<Exit>,
    pub unknown: Vec<UnknownDirection>,
}

/// Whether the player's location is lit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Light {
    Lit,
    Dark,
    Unknown,
}

impl Light {
    pub fn as_str(self) -> &'static str {
        match self {
            Light::Lit => "lit",
            Light::Dark => "dark",
            Light::Unknown => "unknown",
        }
    }
}

/// How an object comes to be within reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reach {
    InRoom,
    Inside,
    On,
    Carried,
    Worn,
}

impl Reach {
    pub fn as_str(self) -> &'static str {
        match self {
            Reach::InRoom => "in-room",
            Reach::Inside => "inside",
            Reach::On => "on",
            Reach::Carried => "carried",
            Reach::Worn => "worn",
        }
    }
}

/// Whether an object inside another can be seen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    Visible,
    /// The family does not record whether its container is open.
    Unknown,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Visibility::Visible => "visible",
            Visibility::Unknown => "unknown",
        }
    }
}

/// One object within reach (specification section 9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeEntry {
    pub object: u16,
    pub name: Part<String>,
    pub reach: Reach,
    /// The object it is in or on.
    pub within: Option<u16>,
    pub visibility: Visibility,
    pub scenery: Tri,
    pub concealed: Tri,
}

/// The objects the player could plausibly act on (specification section 9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub light: Light,
    /// `Ok` when the player was found; otherwise nothing is marked carried.
    pub player: Part<()>,
    pub objects: Vec<ScopeEntry>,
}

/// The state of a walk through the objects in scope.
struct Walk {
    entries: Vec<ScopeEntry>,
    left_out: Vec<u16>,
    budget: usize,
}

/// Reads a game's state. Made once for a story file; holds what was worked out
/// from the file as shipped and nothing that changes during play.
#[derive(Clone, Debug)]
pub struct Introspector {
    layout: Layout,
    adapter: Adapter,
    /// The story file's length and checksum, to know it again.
    identity: (usize, Option<u16>),
}

impl Introspector {
    pub fn new(view: &View<'_>) -> Introspector {
        let shipped = Bytes::shipped(view);
        let layout = Layout::read(shipped);
        let adapter = family::recognise(&layout, shipped);
        Introspector {
            layout,
            adapter,
            identity: (view.len(), view.shipped_word(0x1C)),
        }
    }

    /// The version of the specification the answers follow.
    pub fn spec_version(&self) -> &'static str {
        SPEC_VERSION
    }

    /// The family the story file was recognised as (specification section 3).
    pub fn family(&self) -> Family {
        match self.adapter {
            Adapter::Inform(_) => Family::Inform6,
            Adapter::Zil(_) => Family::Zil,
            Adapter::None => Family::Unrecognised,
        }
    }

    /// Why a family-dependent query cannot be answered at all, if it cannot.
    fn refusal(&self, view: &View<'_>) -> Option<Unknown> {
        if self.identity != (view.len(), view.shipped_word(0x1C)) {
            // The view is of some other story file.
            return Some(Unknown::Inconsistent);
        }
        matches!(self.adapter, Adapter::None).then_some(Unknown::UnrecognisedFamily)
    }

    /// Where the player is, by the family's rules.
    fn place(&self, b: Bytes) -> Part<Place> {
        match &self.adapter {
            Adapter::Inform(inform) => inform.place(&self.layout, b),
            Adapter::Zil(zil) => zil.place(&self.layout, b),
            Adapter::None => Err(Unknown::UnrecognisedFamily),
        }
    }

    /// The room the player is in (specification section 6).
    pub fn location(&self, view: &View<'_>) -> Answer<Location> {
        if let Some(reason) = self.refusal(view) {
            return Answer::unknown(reason, Exactness::Exact);
        }
        let b = Bytes::now(view);
        match self.place(b) {
            Ok(place) => Answer::exact(Location {
                room: place.room,
                name: self.printed_name(b, place.room),
                holder: place.holder,
                lit: place.lit,
            }),
            Err(reason) => Answer::unknown(reason, Exactness::Exact),
        }
    }

    /// What the player carries (specification section 7).
    pub fn inventory(&self, view: &View<'_>) -> Answer<Vec<Carried>> {
        if let Some(reason) = self.refusal(view) {
            return Answer::unknown(reason, Exactness::Exact);
        }
        let b = Bytes::now(view);
        let carried = self
            .place(b)
            .and_then(|place| place.player)
            .and_then(|player| {
                // No tree holds more objects than exist; the budget stops a loop.
                let mut budget = self.layout.last_object as usize;
                self.carried(b, player, &mut budget)
            });
        match carried {
            Ok(carried) => Answer::exact(carried),
            Err(reason) => Answer::unknown(reason, Exactness::Exact),
        }
    }

    /// The children of an object, each with its own, as things carried.
    fn carried(&self, b: Bytes, holder: u16, budget: &mut usize) -> Part<Vec<Carried>> {
        let mut out = Vec::new();
        for object in self.children(b, holder)? {
            *budget = budget.checked_sub(1).ok_or(Unknown::Damaged)?;
            out.push(Carried {
                object,
                name: self.printed_name(b, object),
                worn: self.attribute(b, object, inform::attribute::WORN),
                contents: self.carried(b, object, budget)?,
            });
        }
        Ok(out)
    }

    /// An attribute whose number the Inform library fixes; unknown in a family
    /// that numbers its attributes game by game.
    fn attribute(&self, b: Bytes, object: u16, attribute: u8) -> Tri {
        match &self.adapter {
            Adapter::Inform(_) => self
                .layout
                .has_attribute(b, object, attribute)
                .map_or(Tri::Unknown, Tri::from_bool),
            _ => Tri::Unknown,
        }
    }

    /// The directions that lead out of the player's location (specification
    /// section 8).
    pub fn exits(&self, view: &View<'_>) -> Answer<Exits> {
        if let Some(reason) = self.refusal(view) {
            return Answer::unknown(reason, Exactness::Exact);
        }
        let b = Bytes::now(view);
        let room = match self.place(b) {
            Ok(place) => place.room,
            Err(reason) => return Answer::unknown(reason, Exactness::Exact),
        };
        // Each direction of the family, with what the room's table says of it.
        let ways: Vec<(u8, &[String], Way)> = match &self.adapter {
            Adapter::Inform(inform) => inform
                .directions
                .iter()
                .map(|d| {
                    (
                        d.property,
                        d.words.as_slice(),
                        inform.way(&self.layout, b, room, d),
                    )
                })
                .collect(),
            Adapter::Zil(zil) => zil
                .directions
                .iter()
                .map(|d| {
                    (
                        d.property,
                        d.words.as_slice(),
                        zil.way(&self.layout, b, room, d),
                    )
                })
                .collect(),
            Adapter::None => Vec::new(),
        };
        let mut exits = Exits::default();
        for (direction, words, way) in ways {
            let words = words.to_vec();
            match way {
                Way::None => {}
                Way::Fixed(destination) => exits.exits.push(Exit {
                    direction,
                    words,
                    kind: ExitKind::Fixed,
                    destination: Some(destination),
                    destination_name: Some(self.printed_name(b, destination)),
                }),
                Way::Conditional => exits.exits.push(Exit {
                    direction,
                    words,
                    kind: ExitKind::Conditional,
                    destination: None,
                    destination_name: None,
                }),
                Way::Unknown(reason) => exits.unknown.push(UnknownDirection {
                    direction,
                    words,
                    reason,
                }),
            }
        }
        Answer::exact(exits)
    }

    /// The objects the player could plausibly act on (specification section 9).
    /// Always approximate.
    pub fn scope(&self, view: &View<'_>) -> Answer<Scope> {
        if let Some(reason) = self.refusal(view) {
            return Answer::unknown(reason, Exactness::Approximate);
        }
        let b = Bytes::now(view);
        match self.place(b).and_then(|place| self.scope_of(b, place)) {
            Ok(scope) => Answer::approximate(scope),
            Err(reason) => Answer::unknown(reason, Exactness::Approximate),
        }
    }

    fn scope_of(&self, b: Bytes, place: Place) -> Part<Scope> {
        let light = match place.lit {
            Tri::Yes => Light::Lit,
            Tri::No => Light::Dark,
            Tri::Unknown => Light::Unknown,
        };
        // The player is never an entry. Where the player ties, every object that
        // tied is left out, with what it holds: one of them is the player.
        let left_out: Vec<u16> = match (&self.adapter, &place.player) {
            (_, Ok(player)) => vec![*player],
            (Adapter::Zil(zil), Err(Unknown::AmbiguousPlayer)) => {
                zil.contenders(&self.layout, b, place.room)?
            }
            _ => Vec::new(),
        };
        let mut walk = Walk {
            entries: Vec::new(),
            left_out,
            // No tree holds more objects than exist; the budget stops a loop.
            budget: self.layout.last_object as usize,
        };
        if light != Light::Dark {
            for object in self.children(b, place.room)? {
                self.enter(
                    b,
                    object,
                    Reach::InRoom,
                    None,
                    Visibility::Visible,
                    &mut walk,
                )?;
            }
            if let Adapter::Inform(inform) = &self.adapter {
                for object in 1..=self.layout.last_object {
                    let unplaced = self
                        .layout
                        .object(b, object)
                        .is_some_and(|node| node.parent == 0);
                    let listed = walk.entries.iter().any(|e| e.object == object);
                    if unplaced
                        && !listed
                        && object != place.room
                        && inform.found_in(&self.layout, b, object, place.room)
                    {
                        self.enter(
                            b,
                            object,
                            Reach::InRoom,
                            None,
                            Visibility::Visible,
                            &mut walk,
                        )?;
                    }
                }
            }
        }
        if let Ok(player) = place.player {
            for object in self.children(b, player)? {
                let worn = self.attribute(b, object, inform::attribute::WORN) == Tri::Yes;
                let reach = if worn { Reach::Worn } else { Reach::Carried };
                self.enter(b, object, reach, None, Visibility::Visible, &mut walk)?;
            }
        }
        Ok(Scope {
            light,
            player: place.player.map(|_| ()),
            objects: walk.entries,
        })
    }

    /// Lists an object, then whatever of its contents can be reached.
    fn enter(
        &self,
        b: Bytes,
        object: u16,
        reach: Reach,
        within: Option<u16>,
        visibility: Visibility,
        walk: &mut Walk,
    ) -> Part<()> {
        if walk.left_out.contains(&object) {
            return Ok(());
        }
        walk.budget = walk.budget.checked_sub(1).ok_or(Unknown::Damaged)?;
        walk.entries.push(ScopeEntry {
            object,
            name: self.printed_name(b, object),
            reach,
            within,
            visibility,
            scenery: self.attribute(b, object, inform::attribute::SCENERY),
            concealed: self.attribute(b, object, inform::attribute::CONCEALED),
        });
        // How this object's contents are reached, and whether that can be told.
        let (inner, seen) = match &self.adapter {
            Adapter::Inform(inform) => match inform.contents_reach(&self.layout, b, object) {
                Some(reach) => (reach, Visibility::Visible),
                // Closed and opaque, or not a holder of things: nothing shows.
                None => return Ok(()),
            },
            // The family does not record whether a container is open.
            _ => (Reach::Inside, Visibility::Unknown),
        };
        for child in self.children(b, object)? {
            self.enter(b, child, inner, Some(object), seen, walk)?;
        }
        Ok(())
    }

    /// Every object of the game, by number (specification section 4).
    pub fn object_tree(&self, view: &View<'_>) -> Answer<Vec<Object>> {
        let b = Bytes::now(view);
        let objects = (1..=self.layout.last_object)
            .map(|number| self.object(b, number))
            .collect();
        Answer::exact(objects)
    }

    fn object(&self, b: Bytes, number: u16) -> Object {
        let node = self.layout.object(b, number);
        let last = self.layout.last_object;
        let parent = match node {
            Some(node) if node.parent <= last => Ok((node.parent != 0).then_some(node.parent)),
            _ => Err(Unknown::Damaged),
        };
        let name = self.layout.name(b, number);
        Object {
            number,
            printed_name: self.printed_name(b, number),
            name: name.unwrap_or_default(),
            parent,
            children: self.children(b, number),
            attributes: self.layout.attributes(b, number).unwrap_or_default(),
            properties: self
                .layout
                .properties(b, number)
                .unwrap_or_default()
                .iter()
                .map(|p| p.number)
                .collect(),
        }
    }

    /// An object's children in the tree's order. The walk is bounded by the number
    /// of objects; a link out of range, a loop, or a child that names another
    /// parent makes the list `damaged`.
    pub(crate) fn children(&self, b: Bytes, number: u16) -> Part<Vec<u16>> {
        self.layout.children(b, number).ok_or(Unknown::Damaged)
    }

    /// What the game prints for an object.
    pub(crate) fn printed_name(&self, b: Bytes, number: u16) -> Part<String> {
        match &self.adapter {
            Adapter::Inform(inform) => inform.printed_name(&self.layout, b, number),
            _ => self.layout.name(b, number).ok_or(Unknown::Damaged),
        }
    }

    /// The parts of speech the family records in a dictionary entry's data, and
    /// the direction a direction word names.
    fn parts_of_speech(&self, word: &str, data: [u8; 3]) -> (Part<Vec<PartOfSpeech>>, Option<u8>) {
        use PartOfSpeech::*;
        let mut parts = Vec::new();
        let mut add = |set: bool, part| {
            if set {
                parts.push(part);
            }
        };
        match &self.adapter {
            Adapter::None => (Err(Unknown::UnrecognisedFamily), None),
            Adapter::Zil(_) => {
                let direction = zil::direction_of(data);
                add(data[0] & zil::flag::NOUN != 0, Noun);
                add(data[0] & zil::flag::VERB != 0, Verb);
                add(data[0] & zil::flag::ADJECTIVE != 0, Adjective);
                add(direction.is_some(), Direction);
                add(data[0] & zil::flag::PREPOSITION != 0, Preposition);
                add(data[0] & zil::flag::BUZZ != 0, Buzz);
                (Ok(parts), direction)
            }
            Adapter::Inform(inform) => {
                let direction = inform
                    .directions
                    .iter()
                    .find(|d| d.words.iter().any(|w| w == word))
                    .map(|d| d.property);
                add(data[0] & inform::flag::NOUN != 0, Noun);
                add(data[0] & inform::flag::VERB != 0, Verb);
                add(direction.is_some(), Direction);
                add(data[0] & inform::flag::GRAMMAR != 0, GrammarWord);
                add(data[0] & inform::flag::PLURAL != 0, Plural);
                add(data[0] & inform::flag::META != 0, Meta);
                (Ok(parts), direction)
            }
        }
    }

    /// Every word of the main dictionary, in its order (specification section 5).
    pub fn dictionary(&self, view: &View<'_>) -> Answer<Vec<Word>> {
        let b = Bytes::now(view);
        let Some(dictionary) = self.layout.dictionary(b) else {
            return Answer::unknown(Unknown::Damaged, Exactness::Exact);
        };
        let mut words = Vec::with_capacity(dictionary.count);
        for i in 0..dictionary.count {
            let entry = dictionary.first + i * dictionary.entry_len;
            let Some(word) = self.layout.word(b, entry) else {
                return Answer::unknown(Unknown::Damaged, Exactness::Exact);
            };
            let data = b.byte(entry + dictionary.text_len).unwrap_or(0);
            let more = [
                b.byte(entry + dictionary.text_len + 1).unwrap_or(0),
                b.byte(entry + dictionary.text_len + 2).unwrap_or(0),
            ];
            let (parts, direction) = self.parts_of_speech(&word, [data, more[0], more[1]]);
            words.push(Word {
                word,
                parts,
                direction,
            });
        }
        Answer::exact(words)
    }
}
