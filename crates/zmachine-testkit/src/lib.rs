//! Builds Z-machine version 3 story images in memory, for tests.
//!
//! Nothing here depends on the interpreter. Section numbers ("§") refer to the
//! Z-Machine Standards Document 1.1.
//!
//! Typical use:
//!
//! ```
//! use zmachine_testkit::{Story, op, op::Arg};
//!
//! let mut story = Story::new();
//! let mut code = op::op0(op::QUIT);
//! # code.len();
//! story.main(&code);
//! let image = story.build();
//! assert_eq!(image[0], 3);
//! ```

pub mod op;
pub mod programs;
mod story;
mod zenc;

pub use story::{Object, Story, GLOBALS, SCRATCH, SCRATCH_LEN};
pub use zenc::{zencode, zencode_dict};

#[cfg(test)]
mod tests;
