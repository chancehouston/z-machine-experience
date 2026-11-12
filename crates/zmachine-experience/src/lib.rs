//! The experience layer: what a phone's screen shows of a game, and what a tap
//! sends it.
//!
//! Everything a shell draws is decided here, so that the iPhone and Android apps
//! share one tested implementation of behaviour and contain presentation only.
//! The contract is `specs/004-first-demo/contracts/experience-api.md`.
#![forbid(unsafe_code)]
