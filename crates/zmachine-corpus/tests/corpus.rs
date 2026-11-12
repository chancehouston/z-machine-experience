//! The corpus as a test target with its own `main`, so that a story that is
//! absent is reported as ignored and not as passed
//! (specs/005-corpus-testing/research.md R16; spec.md FR-021, SC-005).

fn main() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    println!();
    println!("running the corpus (stories/corpus/manifest.txt)");
    let (lines, passed) = zmachine_corpus::corpus::corpus_lines(root);
    for line in lines {
        println!("{line}");
    }
    println!();
    if !passed {
        std::process::exit(1);
    }
}
