//! The C interface, called as C would call it
//! (specs/004-first-demo/contracts/c-abi.md, rules cited C§n).

use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::{CStr, CString, c_char};
use std::sync::atomic::{AtomicIsize, Ordering};
use zmachine_ffi::*;

/// Counts the bytes this test binary has allocated and not yet freed, so that a
/// leak across the boundary shows.
struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);

// SAFETY: every call is passed straight to the system allocator with the same
// arguments; the counter is the only addition.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        // SAFETY: the caller's contract for `alloc` is the system allocator's.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        // SAFETY: `ptr` came from `alloc` above with this layout.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn story() -> Vec<u8> {
    std::fs::read(format!("{ROOT}/stories/zork1/zork1.z3")).unwrap()
}

/// A session of the bundled game with a fixed seed.
fn open() -> *mut zmx_session {
    let story = story();
    let session = zmx_session_new(story.as_ptr(), story.len(), 11, 1, 0, std::ptr::null_mut());
    assert!(!session.is_null(), "the game starts");
    session
}

/// Takes a snapshot, copies it out, and frees it as the caller must.
fn snapshot(session: *mut zmx_session, since: usize) -> String {
    let pointer = zmx_session_snapshot(session, since);
    assert!(!pointer.is_null());
    // SAFETY: the interface returned a NUL-terminated string that it owns until
    // zmx_string_free.
    let text = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .expect("UTF-8")
        .to_string();
    zmx_string_free(pointer);
    text
}

/// The number that follows `"key":` in a snapshot.
fn number(snapshot: &str, key: &str) -> u64 {
    let at = snapshot
        .find(&format!("\"{key}\":"))
        .unwrap_or_else(|| panic!("no {key}"))
        + key.len()
        + 3;
    snapshot[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

fn stamp(session: *mut zmx_session) -> (u64, usize) {
    let s = snapshot(session, usize::MAX);
    (number(&s, "generation"), number(&s, "turn_count") as usize)
}

fn c(text: &str) -> CString {
    CString::new(text).unwrap()
}

// C§1: a session made from the story's bytes gives a snapshot of the opening.
// The bytes are copied: the caller's buffer may be freed at once.
#[test]
fn a_session_gives_a_snapshot_of_the_opening() {
    let session = {
        let bytes = story();
        zmx_session_new(bytes.as_ptr(), bytes.len(), 11, 1, 0, std::ptr::null_mut())
    };
    assert!(!session.is_null());
    let first = snapshot(session, 0);
    assert!(first.starts_with(r#"{"generation":1,"turn_count":1,"waiting":"command","message":null,"status":{"place":"West of House""#), "{}", &first[..120]);
    assert!(first.contains(
        r#""number":0,"command":null,"source":null,"text":"ZORK I: The Great Underground Empire\n"#
    ));
    assert!(first.contains(r#"{"place":"north","label":"North","state":"open","command":"north","leads_to":"North of House","spoken":"North, to North of House"}"#));
    zmx_session_free(session);
}

// C§1: a line and a tap, each with the stamp the caller was showing, reach the
// game; the next snapshot has the turn.
#[test]
fn a_line_and_a_tap_reach_the_game() {
    let session = open();
    let (generation, turns) = stamp(session);
    assert_eq!((generation, turns), (1, 1));
    assert_eq!(
        zmx_session_send_line(session, generation, turns, c("open mailbox").as_ptr()),
        1
    );
    let after = snapshot(session, 1);
    assert!(after.contains(r#""turns":[{"number":1,"command":"open mailbox","source":"typed","text":"Opening the small mailbox reveals a leaflet.\n\n>"}]"#), "{after}");
    assert_eq!(zmx_session_tap(session, 1, 2, c("north").as_ptr()), 1);
    let after = snapshot(session, 2);
    assert!(
        after.contains(r#""command":"north","source":"tap","text":"North of House\n"#),
        "{after}"
    );
    assert!(after.contains(r#""status":{"place":"North of House","score":0,"moves":2"#));
    assert_eq!(number(&after, "turn_count"), 3);
    zmx_session_free(session);
}

// C§1, FR-016: a command with a stale stamp is refused, and so is one the
// compass does not offer.
#[test]
fn stale_and_unoffered_commands_are_refused() {
    let session = open();
    assert_eq!(zmx_session_tap(session, 1, 1, c("north").as_ptr()), 1);
    assert_eq!(
        zmx_session_tap(session, 1, 1, c("north").as_ptr()),
        0,
        "the stamp is one turn old"
    );
    assert_eq!(zmx_session_send_line(session, 1, 1, c("look").as_ptr()), 0);
    assert_eq!(zmx_session_tap_extra(session, 1, 1, 0), 0);
    assert_eq!(
        zmx_session_send_line(session, 2, 2, c("look").as_ptr()),
        0,
        "no such game yet"
    );
    assert_eq!(stamp(session), (1, 2), "nothing reached the game");
    // Places the compass does not offer, and names that are not places.
    assert_eq!(zmx_session_tap(session, 1, 2, c("up").as_ptr()), 0);
    for name in ["North", "northe", "", "n", "land"] {
        assert_eq!(
            zmx_session_tap(session, 1, 2, c(name).as_ptr()),
            0,
            "{name:?}"
        );
    }
    assert_eq!(zmx_session_tap_extra(session, 1, 2, 0), 0);
    assert_eq!(zmx_session_tap_extra(session, 1, 2, usize::MAX), 0);
    assert_eq!(stamp(session), (1, 2));
    zmx_session_free(session);
}

// C§1: a new game has a new generation, and the old stamps are stale.
#[test]
fn a_new_game_has_a_new_generation() {
    let session = open();
    zmx_session_tap(session, 1, 1, c("north").as_ptr());
    zmx_session_new_game(session, 4242);
    assert_eq!(stamp(session), (2, 1));
    assert!(snapshot(session, 0).contains(r#""status":{"place":"West of House""#));
    assert_eq!(zmx_session_tap(session, 1, 1, c("north").as_ptr()), 0);
    assert_eq!(zmx_session_tap(session, 2, 1, c("north").as_ptr()), 1);
    zmx_session_free(session);
}

// C§2: null is never dereferenced.
#[test]
fn null_is_refused_everywhere() {
    let null = std::ptr::null_mut();
    assert_eq!(zmx_session_send_line(null, 1, 1, c("look").as_ptr()), 0);
    assert_eq!(zmx_session_tap(null, 1, 1, c("north").as_ptr()), 0);
    assert_eq!(zmx_session_tap_extra(null, 1, 1, 0), 0);
    zmx_session_new_game(null, 1);
    assert!(zmx_session_snapshot(null, 0).is_null());
    zmx_session_free(null);
    zmx_string_free(std::ptr::null_mut());

    let session = open();
    assert_eq!(zmx_session_send_line(session, 1, 1, std::ptr::null()), 0);
    assert_eq!(zmx_session_tap(session, 1, 1, std::ptr::null()), 0);
    // Bytes that are not UTF-8.
    let bad: [u8; 4] = [0xFF, 0xFE, b'x', 0];
    assert_eq!(
        zmx_session_send_line(session, 1, 1, bad.as_ptr() as *const c_char),
        0
    );
    assert_eq!(
        zmx_session_tap(session, 1, 1, bad.as_ptr() as *const c_char),
        0
    );
    assert_eq!(stamp(session), (1, 1));
    zmx_session_free(session);

    // A null story with a length, and no story at all.
    let mut error: *mut c_char = std::ptr::null_mut();
    assert!(zmx_session_new(std::ptr::null(), 100, 1, 1, 0, &mut error).is_null());
    assert!(!error.is_null());
    zmx_string_free(error);
    assert!(zmx_session_new(std::ptr::null(), 0, 1, 1, 0, std::ptr::null_mut()).is_null());
}

// C§2: a story that will not load gives no session, and a message if one is
// asked for; a fixed seed beyond the core's range is refused.
#[test]
fn a_story_that_will_not_load_says_why() {
    let message = |story: &[u8], seed: u32| -> String {
        let mut error: *mut c_char = std::ptr::null_mut();
        let session = zmx_session_new(story.as_ptr(), story.len(), seed, 1, 0, &mut error);
        assert!(session.is_null());
        assert!(!error.is_null());
        // SAFETY: the interface set a NUL-terminated string it owns until freed.
        let text = unsafe { CStr::from_ptr(error) }
            .to_str()
            .unwrap()
            .to_string();
        zmx_string_free(error);
        text
    };
    assert!(message(&[3u8; 10], 1).contains("too short"));
    let mut wrong = story();
    wrong[0] = 6;
    assert!(message(&wrong, 1).contains("version 6"));
    assert!(message(&story(), 2_147_483_648).contains("seed"));
    // The message is optional.
    assert!(zmx_session_new([3u8; 10].as_ptr(), 10, 1, 1, 0, std::ptr::null_mut()).is_null());
    // Without a fixed seed any number is accepted, and decides the game.
    let story = story();
    let session = zmx_session_new(
        story.as_ptr(),
        story.len(),
        u32::MAX,
        0,
        u32::MAX,
        std::ptr::null_mut(),
    );
    assert!(!session.is_null());
    zmx_session_free(session);
}

// C§1: nothing leaks. A thousand sessions made, used and freed, and ten thousand
// snapshots taken and freed, leave the live allocation where it began.
#[test]
fn nothing_leaks_across_the_boundary() {
    let story = story();
    let run = |n: usize| {
        for i in 0..n {
            let session =
                zmx_session_new(story.as_ptr(), story.len(), 11, 1, 0, std::ptr::null_mut());
            zmx_session_tap(session, 1, 1, c("north").as_ptr());
            zmx_session_send_line(session, 1, 2, c("look").as_ptr());
            if i % 7 == 0 {
                zmx_session_new_game(session, i as u32);
            }
            zmx_string_free(zmx_session_snapshot(session, 0));
            zmx_session_free(session);
            let mut error: *mut c_char = std::ptr::null_mut();
            zmx_session_new([3u8; 10].as_ptr(), 10, 1, 1, 0, &mut error);
            zmx_string_free(error);
        }
    };
    // Warm up, so that one-time allocations are not counted.
    run(3);
    let before = LIVE.load(Ordering::Relaxed);
    run(1000);
    let session = open();
    for _ in 0..10_000 {
        zmx_string_free(zmx_session_snapshot(session, 0));
    }
    zmx_session_free(session);
    let after = LIVE.load(Ordering::Relaxed);
    // Other tests run beside this one and allocate; a leak of even one session
    // (over a hundred kilobytes) a thousand times over would dwarf this bound.
    assert!(
        (after - before).abs() < 8_000_000,
        "live bytes went from {before} to {after}"
    );
}

// C§4: no panic crosses the boundary. A command that panics inside the layer
// returns 0; the session then reports a fault and refuses commands, until a new
// game replaces it.
#[test]
fn a_panic_does_not_cross_the_boundary() {
    let session = open();
    panic_on_next_command(session);
    assert_eq!(
        zmx_session_tap(session, 1, 1, c("north").as_ptr()),
        0,
        "the call returned"
    );
    let after = snapshot(session, 0);
    assert!(
        after.contains(r#""waiting":"fault""#),
        "{}",
        &after[..200.min(after.len())]
    );
    assert!(after.contains(r#""message":"The game stopped unexpectedly.""#));
    assert!(after.contains(r#""offered":false"#));
    let (generation, turns) = stamp(session);
    assert_eq!(
        zmx_session_send_line(session, generation, turns, c("look").as_ptr()),
        0
    );
    assert_eq!(
        zmx_session_tap(session, generation, turns, c("north").as_ptr()),
        0
    );
    zmx_session_new_game(session, 7);
    let fresh = snapshot(session, 0);
    assert!(fresh.contains(r#""waiting":"command""#));
    let (generation, turns) = stamp(session);
    assert_eq!(turns, 1);
    assert_eq!(
        zmx_session_tap(session, generation, turns, c("north").as_ptr()),
        1,
        "a working game again"
    );
    zmx_session_free(session);
}

/// The names of the functions `zmx.h` declares.
fn declared() -> Vec<String> {
    let header =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/include/zmx.h")).unwrap();
    let mut names: Vec<String> = header
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|word| word.starts_with("zmx_") && *word != "zmx_session")
        .map(str::to_string)
        .collect();
    names.sort();
    names.dedup();
    names
}

// C§6: the header and the library agree. The functions the header declares are
// exactly the symbols beginning `zmx_` that the static library exports.
#[test]
fn the_header_and_the_library_agree() {
    let declared = declared();
    assert_eq!(
        declared,
        [
            "zmx_session_free",
            "zmx_session_new",
            "zmx_session_new_game",
            "zmx_session_send_line",
            "zmx_session_snapshot",
            "zmx_session_tap",
            "zmx_session_tap_extra",
            "zmx_string_free"
        ]
    );
    // The static library sits beside the test binaries' directory. Running the
    // tests does not rebuild it, so build it here: a stale one would be checked
    // in place of this source.
    let exe = std::env::current_exe().unwrap();
    let library = exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("libzmachine_ffi.a");
    let built = std::process::Command::new(env!("CARGO"))
        .args(["build", "-p", "zmachine-ffi"])
        .current_dir(ROOT)
        .output()
        .expect("running cargo");
    assert!(
        built.status.success(),
        "building the static library: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let Ok(out) = std::process::Command::new("nm")
        .arg("-g")
        .arg(&library)
        .output()
    else {
        eprintln!("nm was not found: the exported symbols were not checked");
        return;
    };
    // The system's nm cannot read every object of Rust's standard library and
    // says so with a failing status; this crate's own objects are what matter,
    // and an empty listing fails the comparison below.
    let listing = String::from_utf8_lossy(&out.stdout);
    let mut exported: Vec<String> = listing
        .lines()
        .filter(|l| l.contains(" T "))
        .filter_map(|l| l.split_whitespace().last())
        .map(|symbol| symbol.trim_start_matches('_').to_string())
        .filter(|symbol| symbol.starts_with("zmx_"))
        .collect();
    exported.sort();
    exported.dedup();
    assert_eq!(exported, declared);
}

// C§5, constitution Principle V: every `unsafe` in this crate says why it is
// sound, and no other crate of the workspace has any.
#[test]
fn unsafe_is_explained_here_and_absent_elsewhere() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();
    let lines: Vec<&str> = source.lines().collect();
    let mut blocks = 0;
    for (n, line) in lines.iter().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        if code.contains("unsafe {") || code.contains("unsafe impl") {
            blocks += 1;
            let above: Vec<&str> = lines[n.saturating_sub(6)..n].to_vec();
            assert!(
                above
                    .iter()
                    .any(|l| l.trim_start().starts_with("// SAFETY:")),
                "src/lib.rs:{}: an unsafe block with no SAFETY comment",
                n + 1
            );
        }
    }
    assert!(
        blocks >= 5,
        "the bindings have unsafe blocks to check: {blocks}"
    );
    for other in [
        "zmachine-core",
        "zmachine-introspect",
        "zmachine-experience",
        "zmachine-cli",
        "zmachine-testkit",
    ] {
        let dir = format!("{ROOT}/crates/{other}/src");
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                for line in text.lines() {
                    let code = line.split("//").next().unwrap_or("");
                    assert!(
                        !code.contains("unsafe ")
                            || code.contains("forbid(unsafe_code)")
                            || code.contains('"'),
                        "{}: {line}",
                        path.display()
                    );
                }
            }
        }
    }
}
