//! Small ready-made programs used to compare interpreters.

use crate::op::{self, Arg};
use crate::Story;

/// How many results [`random_probe`] prints after each seeding.
pub const RANDOM_PROBE_COUNTS: [usize; 4] = [200, 20, 20, 20];

/// A program that turns transcription on and prints the results of `random 100`,
/// one per line: 200 with the generator as the interpreter seeded it, then 20 after
/// each of `random -5`, `random -1000` and `random 0` (§15 random).
///
/// Run under two interpreters with the same seed, the transcripts show whether
/// their generators agree.
pub fn random_probe() -> Vec<u8> {
    let mut story = Story::new();
    let mut code = Vec::new();

    // Set Flags 2 bit 0 (§11.1, word at 0x10): transcription on.
    code.extend(op::var(op::STOREW, &[Arg::S(0), Arg::S(8), Arg::S(1)]));

    let reseeds: [Option<u16>; 4] = [None, Some(-5i16 as u16), Some(-1000i16 as u16), Some(0)];
    for (i, (reseed, count)) in reseeds.iter().zip(RANDOM_PROBE_COUNTS).enumerate() {
        if let Some(value) = reseed {
            // random <value> -> (stack); the result, always 0, is thrown away.
            code.extend(op::var(op::RANDOM, &[Arg::L(*value)]));
            code.extend(op::store(0));
            code.extend(op::op0(op::POP));
        }
        // Each loop counts in its own global, which starts at 0.
        let counter = 16 + i as u8;
        let body_start = code.len();
        code.extend(op::var(op::RANDOM, &[Arg::S(100)]));
        code.extend(op::store(0));
        code.extend(op::var(op::PRINT_NUM, &[Arg::V(0)]));
        code.extend(op::op0(op::NEW_LINE));
        // inc_chk counter (count - 1): loop again unless the counter now exceeds it.
        code.extend(op::op2(op::INC_CHK, Arg::S(counter), Arg::S(count as u8 - 1)));
        let after = code.len() + 2;
        let offset = body_start as i16 - after as i16 + 2;
        code.extend(op::branch_long(false, offset));
    }
    code.extend(op::op0(op::QUIT));

    story.main(&code);
    story.build()
}
