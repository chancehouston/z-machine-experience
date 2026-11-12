//! The corpus program (specs/005-corpus-testing/contracts/commands.md).
//! Run from the repository's root.

use zmachine_corpus::corpus::{Mode, command, explain};

const USAGE: &str = "usage: corpus check
       corpus run [ID...]
       corpus write [ID...]
       corpus explain ID ROOM DIRECTION
       corpus --help

  check   check the manifest, the ledger and the game packs
  run     run each named story, or every story present, and compare its report
          with the one kept
  write   run as `run` does, and write the reports in place of those kept
  explain show every trial of a story from a room in a direction, with what the
          game printed (a game's text: for the terminal only)";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = std::env::current_dir().expect("a current directory");
    let (lines, status) = match args.first().map(String::as_str) {
        Some("check") if args.len() == 1 => {
            let mut failures = zmachine_corpus::manifest::check(&root);
            let mut notes = Vec::new();
            if let Ok((stories, ledger)) = zmachine_corpus::corpus::load(&root) {
                let (bad, said) = zmachine_corpus::packs::check(&root, &stories, &ledger);
                failures.extend(bad);
                notes = said;
            }
            let status = !failures.is_empty() as i32;
            notes.extend(failures);
            (notes, status)
        }
        Some("run") => command(&root, Mode::Run, &args[1..]),
        Some("write") => command(&root, Mode::Write, &args[1..]),
        Some("explain") => match (
            args.get(1),
            args.get(2).and_then(|n| n.parse().ok()),
            args.get(3).and_then(|n| n.parse().ok()),
        ) {
            (Some(id), Some(room), Some(direction)) if args.len() == 4 => {
                explain(&root, id, room, direction)
            }
            _ => (vec![USAGE.to_string()], 2),
        },
        Some("--help") => (vec![USAGE.to_string()], 0),
        _ => (vec![USAGE.to_string()], 2),
    };
    for line in lines {
        println!("{line}");
    }
    std::process::exit(status);
}
