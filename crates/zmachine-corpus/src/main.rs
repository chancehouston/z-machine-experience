//! The corpus program (specs/005-corpus-testing/contracts/commands.md).

const USAGE: &str = "usage: corpus check
       corpus run [ID...]
       corpus write [ID...]
       corpus --help";

fn main() {
    println!("{USAGE}");
    std::process::exit(2);
}
