//! Writes the keypress probe story to the path given as the first argument.
//!
//! ```text
//! cargo run -p zmachine-testkit --example key_probe -- keys.z5
//! dfrotz -m -q -w 80 -S 0 -s 1 keys.z5 < crates/zmachine-core/tests/fixtures/keys/escapes.in
//! ```

fn main() {
    let path = std::env::args().nth(1).expect("usage: key_probe <file>");
    std::fs::write(path, zmachine_testkit::programs::key_probe()).expect("write story");
}
