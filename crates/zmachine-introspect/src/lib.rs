//! Read-only introspection of a running Z-machine game.
//!
//! The contract is `docs/introspection.md` in this repository; this crate is one
//! implementation of it. Nothing here writes to the machine, runs the game's code,
//! or knows any one game.
#![forbid(unsafe_code)]

mod answer;
mod table;

pub use answer::{Answer, Exactness, Part, Tri, Unknown};

use table::{Bytes, Layout};
use zmachine_core::View;

#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod table_tests;

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

/// Reads a game's state. Made once for a story file; holds what was worked out
/// from the file as shipped and nothing that changes during play.
#[derive(Clone, Debug)]
pub struct Introspector {
    layout: Layout,
}

impl Introspector {
    pub fn new(view: &View<'_>) -> Introspector {
        Introspector {
            layout: Layout::read(Bytes::shipped(view)),
        }
    }

    /// The version of the specification the answers follow.
    pub fn spec_version(&self) -> &'static str {
        SPEC_VERSION
    }

    /// The family the story file was recognised as (specification section 3).
    pub fn family(&self) -> Family {
        Family::Unrecognised
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
        let node = self.layout.object(b, number).ok_or(Unknown::Damaged)?;
        let mut children = Vec::new();
        let mut next = node.child;
        while next != 0 {
            let child = self.layout.object(b, next).ok_or(Unknown::Damaged)?;
            if child.parent != number || children.len() >= self.layout.last_object as usize {
                return Err(Unknown::Damaged);
            }
            children.push(next);
            next = child.sibling;
        }
        Ok(children)
    }

    /// What the game prints for an object.
    pub(crate) fn printed_name(&self, b: Bytes, number: u16) -> Part<String> {
        self.layout.name(b, number).ok_or(Unknown::Damaged)
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
            words.push(Word {
                word,
                parts: Err(Unknown::UnrecognisedFamily),
                direction: None,
            });
        }
        Answer::exact(words)
    }
}
