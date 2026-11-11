//! Writes the random-number probe story to the path given as the first argument.
//!
//! Used to produce the fixtures in `crates/zmachine-core/tests/fixtures/random/`:
//!
//! ```text
//! cargo run -p zmachine-testkit --example random_probe -- probe.z3
//! printf 'out\n' | dfrotz -m -q -S 0 -s <seed> probe.z3     # writes out.scr
//! ```

fn main() {
    let path = std::env::args().nth(1).expect("usage: random_probe <output.z3>");
    std::fs::write(&path, zmachine_testkit::programs::random_probe()).expect("write story");
}
