//! Read-only introspection of a running Z-machine game.
//!
//! The contract is `docs/introspection.md` in this repository; this crate is one
//! implementation of it. Nothing here writes to the machine, runs the game's code,
//! or knows any one game.
#![forbid(unsafe_code)]

mod answer;
mod table;

pub use answer::{Answer, Exactness, Part, Tri, Unknown};

#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod table_tests;
