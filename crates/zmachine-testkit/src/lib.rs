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
mod story5;
mod zenc;

pub use story::{GLOBALS, Object, SCRATCH, SCRATCH_LEN, Story};
pub use story5::{Object5, Story5};
pub use zenc::{zencode, zencode_dict, zencode_dict9};

#[cfg(test)]
mod tests;

/// The limit a timing test holds to: what its specification says, times
/// `TIME_LIMIT_FACTOR` where that is set. The specification's limit is for a
/// developer's machine. A shared CI runner stalls now and then, by tens of
/// milliseconds, so CI sets a factor under which a test still catches code that
/// has become slow in earnest, and no longer fails on the runner's bad moment.
pub fn time_limit(specified: std::time::Duration) -> std::time::Duration {
    scaled(
        specified,
        std::env::var("TIME_LIMIT_FACTOR").ok().as_deref(),
    )
}

/// `specified` times the factor, which is a whole number from 1 to 1000;
/// anything else is no factor.
fn scaled(specified: std::time::Duration, factor: Option<&str>) -> std::time::Duration {
    let factor = factor
        .and_then(|text| text.parse::<u32>().ok())
        .filter(|factor| (1..=1000).contains(factor))
        .unwrap_or(1);
    specified * factor
}

#[cfg(test)]
mod time_limit_tests {
    use super::scaled;
    use std::time::Duration;

    #[test]
    fn a_limit_is_the_specifications_times_the_factor() {
        let ten = Duration::from_millis(10);
        assert_eq!(scaled(ten, None), ten);
        assert_eq!(scaled(ten, Some("1")), ten);
        assert_eq!(scaled(ten, Some("25")), Duration::from_millis(250));
        assert_eq!(
            scaled(Duration::from_secs(5), Some("4")),
            Duration::from_secs(20)
        );
        // Nothing that is not a whole number from 1 to 1000 loosens or tightens.
        for bad in ["", "0", "-3", "2.5", "many", "1001", " 4"] {
            assert_eq!(scaled(ten, Some(bad)), ten, "{bad:?}");
        }
    }
}
