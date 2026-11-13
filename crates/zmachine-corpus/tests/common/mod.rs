//! Shared by the corpus crate's tests: paths, loading, and small stories built
//! in memory whose exit tables and behaviour can be made to disagree on purpose
//! (specs/005-corpus-testing/tasks.md T004).
#![allow(dead_code)]

use zmachine_core::{LoadOptions, Machine, Settings};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// A path from the repository's root.
pub fn repo(path: &str) -> String {
    format!("{ROOT}/{path}")
}

pub fn read(path: &str) -> Vec<u8> {
    std::fs::read(repo(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

pub fn read_text(path: &str) -> String {
    String::from_utf8(read(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A machine as the reference transcripts were made with: a fixed seed, and an
/// interpreter that calls itself `F`.
pub fn load_bytes(story: Vec<u8>, seed: u32) -> Machine {
    let options = LoadOptions {
        seed: Some(seed),
        entropy: 0,
    };
    let settings = Settings {
        interpreter_version: b'F',
        ..Settings::default()
    };
    Machine::load_with(story, options, settings).expect("the story loads")
}

pub fn load(path: &str, seed: u32) -> Machine {
    load_bytes(read(path), seed)
}

pub use zmachine_testkit::toy::*;

/// A toy as a machine, loaded as [`load_bytes`] loads.
pub trait ToyMachine {
    fn machine(&self) -> Machine;
}

impl ToyMachine for Toy {
    fn machine(&self) -> Machine {
        load_bytes(self.build(), 1)
    }
}

// ---------------------------------------------------------------------------
// The stories the repository holds a walkthrough for
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Game {
    pub name: &'static str,
    pub story: &'static str,
    pub walkthrough: &'static str,
    pub reference: &'static str,
    pub seed: u32,
}

/// As feature 003's tests have them (`crates/zmachine-introspect/tests/common`).
pub const GAMES: [Game; 3] = [
    Game {
        name: "zork1",
        story: "stories/zork1/zork1.z3",
        walkthrough: "stories/zork1/fidelity/walkthrough.txt",
        reference: "stories/zork1/fidelity/reference.scr",
        seed: 11,
    },
    Game {
        name: "advent.z5",
        story: "stories/advent/advent.z5",
        walkthrough: "stories/advent/fidelity/walkthrough.txt",
        reference: "stories/advent/fidelity/reference_z5.scr",
        seed: 1,
    },
    Game {
        name: "advent.z8",
        story: "stories/advent/advent.z8",
        walkthrough: "stories/advent/fidelity/walkthrough.txt",
        reference: "stories/advent/fidelity/reference_z8.scr",
        seed: 1,
    },
];

// ---------------------------------------------------------------------------
// A folder laid out like the repository, for tests of the commands
// ---------------------------------------------------------------------------

/// A new, empty folder under the system's temporary folder, removed when the
/// value is dropped.
pub struct Scratch(pub std::path::PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Scratch {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("zmachine-corpus-{}-{name}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("stories/corpus/reports")).unwrap();
        std::fs::write(path.join("stories/corpus/manifest.txt"), "").unwrap();
        std::fs::write(path.join("stories/corpus/ledger.txt"), "").unwrap();
        Scratch(path)
    }

    pub fn path(&self, relative: &str) -> std::path::PathBuf {
        self.0.join(relative)
    }

    pub fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    /// Writes `packs/<id>.pack` for an image, with its identity worked out from
    /// the image and these entries.
    pub fn pack(&self, id: &str, image: &[u8], entries: &str) {
        self.write(
            &format!("packs/{id}.pack"),
            pack_for(image, entries).as_bytes(),
        );
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
    }

    /// Adds a story built in memory as a committed story with a license record
    /// beside it, and its entry to the manifest. `extra` is further lines of the
    /// entry, each ending in a line feed.
    pub fn add(&self, id: &str, image: &[u8], extra: &str) {
        use sha2::{Digest, Sha256};
        let digest: String = Sha256::digest(image)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.write(&format!("stories/{id}/{id}.z3"), image);
        self.write(&format!("stories/{id}/LICENSE"), b"A test's own.\n");
        self.write(
            &format!("stories/{id}/SOURCE.md"),
            format!("| SHA-256 | `{digest}` |\n").as_bytes(),
        );
        let entry = format!(
            "id: {id}\nsha256: {digest}\nsize: {}\nversion: 3\nfamily: zil\nsource: built in memory\n\
             standing: a test's own\ntier: committed\npath: stories/{id}/{id}.z3\nseed: 1\n{extra}\n",
            image.len()
        );
        let manifest = self.read("stories/corpus/manifest.txt") + &entry;
        self.write("stories/corpus/manifest.txt", manifest.as_bytes());
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A game pack for an image: its identity, then these entries.
pub fn pack_for(image: &[u8], entries: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(image)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!(
        "pack: 1\nsha256: {digest}\nrelease: {}\nserial: {}\n{entries}",
        u16::from_be_bytes([image[2], image[3]]),
        String::from_utf8_lossy(&image[0x12..0x18])
    )
}
