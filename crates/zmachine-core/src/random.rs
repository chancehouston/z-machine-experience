//! The random number generator.
//!
//! The Standard leaves the generator to the interpreter (§2.4, §15 random). This
//! one produces the same sequence as Frotz 2.55 for the same seed, so that a game
//! with random events takes the same course in both and their transcripts can be
//! compared (research.md R2). The behavior is restated from observation of dfrotz
//! and a reading of its source; no Frotz code is used.

#[derive(Clone, Debug)]
pub(crate) struct Random {
    /// The value the host supplied: a fixed seed, or its own entropy.
    host: u32,
    state: u32,
    /// Non-zero in predictable mode: results count 0, 1, ... interval - 1.
    interval: u32,
    counter: u32,
}

impl Random {
    pub fn new(host: u32) -> Self {
        Random {
            host,
            state: host,
            interval: 0,
            counter: 0,
        }
    }

    /// Returns to the state the host seed gives. Done at start and on restart.
    pub fn reseed_from_host(&mut self) {
        self.seed(0);
    }

    fn seed(&mut self, value: u32) {
        match value {
            0 => {
                self.state = self.host;
                self.interval = 0;
            }
            1..=999 => {
                self.counter = 0;
                self.interval = value;
            }
            _ => {
                self.state = value;
                self.interval = 0;
            }
        }
    }

    /// The `random` instruction: a positive range gives a number from 1 to range;
    /// zero or a negative range seeds the generator and gives 0.
    pub fn random(&mut self, range: i16) -> u16 {
        if range <= 0 {
            // Widened before negating: -32768 has magnitude 32768.
            self.seed((-(range as i32)) as u32);
            return 0;
        }
        let raw = if self.interval != 0 {
            let value = self.counter;
            self.counter += 1;
            if self.counter == self.interval {
                self.counter = 0;
            }
            value
        } else {
            self.state = self.state.wrapping_mul(0x015a_4e35).wrapping_add(1);
            self.state >> 16 & 0x7fff
        };
        (raw % range as u32) as u16 + 1
    }
}
