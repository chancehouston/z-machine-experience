//! Corpus testing of introspection (specs/005-corpus-testing).
//!
//! At each place a game is brought to, every direction is tried on a throwaway
//! copy of the game, and what the copy did is set against what the introspection
//! interface reported. These are testing tools: no other crate depends on this
//! one, and nothing here reaches a player.

pub mod corpus;
pub mod directions;
pub mod inventory;
pub mod ledger;
pub mod manifest;
pub mod packs;
pub mod report;
pub mod trial;
pub mod verdict;
pub mod walk;
