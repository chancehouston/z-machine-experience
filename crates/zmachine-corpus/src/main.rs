//! The corpus program (specs/005-corpus-testing/contracts/commands.md).
//! Run from the repository's root.

use zmachine_corpus::corpus::{Mode, command};

const USAGE: &str = "usage: corpus check
       corpus run [ID...]
       corpus write [ID...]
       corpus --help

  check   check the manifest and the ledger
  run     run each named story, or every story present, and compare its report
          with the one kept
  write   run as `run` does, and write the reports in place of those kept";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = std::env::current_dir().expect("a current directory");
    let (lines, status) = match args.first().map(String::as_str) {
        Some("run") => command(&root, Mode::Run, &args[1..]),
        Some("write") => command(&root, Mode::Write, &args[1..]),
        Some("--help") => (vec![USAGE.to_string()], 0),
        _ => (vec![USAGE.to_string()], 2),
    };
    for line in lines {
        println!("{line}");
    }
    std::process::exit(status);
}
