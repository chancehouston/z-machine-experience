//! Writes each screen program and its input into the directory given as the first
//! argument, as `<name>.z5` and `<name>.in`.
//!
//! ```text
//! cargo run -p zmachine-testkit --example screen_programs -- screens
//! cd screens && dfrotz -m -r cn -r ln -r lt -h 24 -w 80 status_line.z5 < status_line.in > status_line.txt
//! ```

fn main() {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: screen_programs <dir>"),
    );
    std::fs::create_dir_all(&dir).expect("create directory");
    for program in zmachine_testkit::programs::screen_programs() {
        std::fs::write(dir.join(format!("{}.z5", program.name)), &program.image)
            .expect("write story");
        let input: String = program
            .input
            .iter()
            .map(|line| format!("{line}\n"))
            .collect();
        std::fs::write(dir.join(format!("{}.in", program.name)), input).expect("write input");
    }
}
