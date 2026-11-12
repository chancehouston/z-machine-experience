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
pub const QUERIES: [&str; 6] = ["object_tree", "dictionary", "location", "inventory", "exits", "scope"];

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
        Introspector { layout: Layout::read(Bytes::shipped(view)) }
    }

    /// The version of the specification the answers follow.
    pub fn spec_version(&self) -> &'static str {
        SPEC_VERSION
    }

    /// Every object of the game, by number (specification section 4).
    pub fn object_tree(&self, _view: &View<'_>) -> Answer<Vec<Object>> {
        Answer::unknown(Unknown::Damaged, Exactness::Exact)
    }

    /// Every word of the main dictionary, in its order (specification section 5).
    pub fn dictionary(&self, _view: &View<'_>) -> Answer<Vec<Word>> {
        Answer::unknown(Unknown::Damaged, Exactness::Exact)
    }
}
