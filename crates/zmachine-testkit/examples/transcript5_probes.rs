//! Writes each version 5 transcript probe and its input into the directory given as
//! the first argument, as `<name>.z5` and `<name>.in`.
//!
//! The `.in` file starts with the transcript file name, as dfrotz expects:
//!
//! ```text
//! cargo run -p zmachine-testkit --example transcript5_probes -- probes
//! cd probes && dfrotz -m -q -w 80 -S 0 -s 1 key_not_written.z5 < key_not_written.in
//! ```

fn main() {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: transcript5_probes <dir>"),
    );
    std::fs::create_dir_all(&dir).expect("create directory");
    for probe in zmachine_testkit::programs::transcript5_probes() {
        std::fs::write(dir.join(format!("{}.z5", probe.name)), &probe.image).expect("write story");
        // The transcript file name, once for each time the program starts one.
        let mut input = format!("{}\n", probe.name).repeat(probe.starts);
        for line in probe.input {
            input.push_str(line);
            input.push('\n');
        }
        std::fs::write(dir.join(format!("{}.in", probe.name)), input).expect("write input");
    }
}
