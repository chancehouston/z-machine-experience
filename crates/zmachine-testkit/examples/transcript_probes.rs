//! Writes each transcript probe story and its input into the directory given as the
//! first argument, as `<name>.z3` and `<name>.in`.
//!
//! The `.in` file starts with the transcript file name, as dfrotz expects:
//!
//! ```text
//! cargo run -p zmachine-testkit --example transcript_probes -- probes
//! cd probes && dfrotz -m -q -S 0 -s 1 quit_mid_line.z3 < quit_mid_line.in   # writes quit_mid_line.scr
//! ```

fn main() {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: transcript_probes <dir>"),
    );
    std::fs::create_dir_all(&dir).expect("create directory");
    for probe in zmachine_testkit::programs::transcript_probes() {
        std::fs::write(dir.join(format!("{}.z3", probe.name)), &probe.image).expect("write story");
        let mut input = format!("{}\n", probe.name);
        for line in probe.input {
            input.push_str(line);
            input.push('\n');
        }
        std::fs::write(dir.join(format!("{}.in", probe.name)), input).expect("write input");
    }
}
