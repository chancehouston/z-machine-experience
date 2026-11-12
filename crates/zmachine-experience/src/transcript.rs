//! Turns: what the game printed and the commands that were given
//! (specs/004-first-demo/data-model.md "Turn").

/// How a command was given.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    Typed,
    Tap,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Typed => "typed",
            Source::Tap => "tap",
        }
    }
}

/// One exchange: the command given, and everything the game printed in reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    /// From 0, the opening.
    pub number: usize,
    /// The line given to the game; none for the opening.
    pub command: Option<String>,
    pub source: Option<Source>,
    /// What the game printed, unaltered, including the prompt that ends it.
    pub text: String,
}
