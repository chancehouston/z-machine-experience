//! C bindings to the experience layer (`include/zmx.h`).
//!
//! This is the only crate in the workspace that may use `unsafe`
//! (constitution Principle V). The contract is
//! `specs/004-first-demo/contracts/c-abi.md`: the caller owns a session from
//! `zmx_session_new` to `zmx_session_free` and a string from the call that
//! returned it to `zmx_string_free`, and uses a session from one thread at a
//! time. Nothing is decided here: every function hands its arguments to the
//! experience layer and hands the answer back.

// The functions are called from C, which has no notion of an unsafe function;
// each checks its pointers for null and states what else it relies on.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString, c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use zmachine_experience::{Outcome, Place, Seed, Session, Stamp};

/// What the session reports after a call into it did not return.
const STOPPED: &str = "The game stopped unexpectedly.";

/// A session as C sees it: an opaque pointer.
#[allow(non_camel_case_types)]
pub struct zmx_session {
    session: Session,
    /// A call into the session panicked; it is not used again until a new game.
    failed: bool,
    /// Set by tests: the next command panics inside the layer.
    panic_next: bool,
}

/// The string as C owns it, or null if it cannot be a C string.
fn give_string(text: String) -> *mut c_char {
    CString::new(text).map_or(std::ptr::null_mut(), CString::into_raw)
}

/// Borrows the session behind a pointer, or `None` for null.
fn borrow<'a>(session: *mut zmx_session) -> Option<&'a mut zmx_session> {
    // SAFETY: a non-null pointer came from `zmx_session_new` and has not been
    // passed to `zmx_session_free`; the caller uses a session from one thread at
    // a time, so no other reference to it is live during this call.
    unsafe { session.as_mut() }
}

/// Reads a C string as UTF-8, or `None` for null or for bytes that are not.
fn text<'a>(string: *const c_char) -> Option<&'a str> {
    if string.is_null() {
        return None;
    }
    // SAFETY: the pointer is not null, and the caller passes a NUL-terminated
    // string that stays valid and unchanged for the length of the call.
    unsafe { CStr::from_ptr(string) }.to_str().ok()
}

/// Runs one command against the session. A refusal is 0; so is a panic, after
/// which the session is marked failed.
fn command(
    session: *mut zmx_session,
    generation: u64,
    turn_count: usize,
    act: impl FnOnce(&mut Session, Stamp) -> bool,
) -> c_int {
    let Some(wrapper) = borrow(session) else {
        return 0;
    };
    if wrapper.failed {
        return 0;
    }
    let at = Stamp {
        generation,
        turn_count,
    };
    let panic_next = std::mem::take(&mut wrapper.panic_next);
    let result = catch_unwind(AssertUnwindSafe(|| {
        if panic_next {
            panic!("a panic asked for by a test");
        }
        act(&mut wrapper.session, at)
    }));
    match result {
        Ok(reached) => c_int::from(reached),
        Err(_) => {
            wrapper.failed = true;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_new(
    story: *const u8,
    length: usize,
    seed: u32,
    has_seed: c_int,
    entropy: u32,
    error: *mut *mut c_char,
) -> *mut zmx_session {
    zmx_session_new_with_pack(
        story,
        length,
        seed,
        has_seed,
        entropy,
        std::ptr::null(),
        error,
    )
}

/// As `zmx_session_new`, with the text of a game pack for the story
/// (docs/introspection.md, section 12): a NUL-terminated string, or null for
/// none. A pack that is not UTF-8, cannot be read or is another story's changes
/// nothing.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_new_with_pack(
    story: *const u8,
    length: usize,
    seed: u32,
    has_seed: c_int,
    entropy: u32,
    pack: *const c_char,
    error: *mut *mut c_char,
) -> *mut zmx_session {
    zmx_session_open(
        story,
        length,
        seed,
        has_seed,
        entropy,
        pack,
        std::ptr::null(),
        0,
        std::ptr::null_mut(),
        error,
    )
}

/// As `zmx_session_new_with_pack`, with the bytes of a kept game
/// (specs/011-resume-game/contracts/kept-game.md): `kept_length` bytes at
/// `kept`, or null for none. With a kept game that can be brought back the
/// session is that game, and the seed and entropy are not used. `outcome`,
/// if not null, is written with 0 for a new game with nothing kept, 1 for a
/// game brought back, and 2 for a new game because the kept one could not be
/// brought back; the session then has a notice saying so.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_open(
    story: *const u8,
    length: usize,
    seed: u32,
    has_seed: c_int,
    entropy: u32,
    pack: *const c_char,
    kept: *const u8,
    kept_length: usize,
    outcome: *mut c_int,
    error: *mut *mut c_char,
) -> *mut zmx_session {
    let result = catch_unwind(|| {
        if story.is_null() {
            return Err("No story file was given.".to_string());
        }
        // SAFETY: `story` is not null, and the caller passes `length` readable
        // bytes that stay valid for the length of the call. They are copied
        // before the call returns.
        let bytes = unsafe { std::slice::from_raw_parts(story, length) }.to_vec();
        let seed = if has_seed != 0 {
            Seed::Fixed(seed)
        } else {
            Seed::Fresh(entropy)
        };
        // SAFETY: `kept` is checked for null, and the caller passes
        // `kept_length` readable bytes that stay valid for the length of the
        // call. They are read, and nothing of them is held, before it returns.
        let kept =
            (!kept.is_null()).then(|| unsafe { std::slice::from_raw_parts(kept, kept_length) });
        // Null, or bytes that are not UTF-8, are no pack. The text is copied
        // into the interface before the call returns.
        Session::open_kept(bytes, text(pack), seed, kept).map_err(|e| e.message)
    });
    let message = match result {
        Ok(Ok((session, came))) => {
            if !outcome.is_null() {
                let code = match came {
                    Outcome::New => 0,
                    Outcome::BroughtBack => 1,
                    Outcome::CouldNot => 2,
                };
                // SAFETY: `outcome` is not null, and the caller passes a
                // pointer to an `int` it may write to.
                unsafe { *outcome = code };
            }
            return Box::into_raw(Box::new(zmx_session {
                session,
                failed: false,
                panic_next: false,
            }));
        }
        Ok(Err(message)) => message,
        Err(_) => STOPPED.to_string(),
    };
    if !error.is_null() {
        // SAFETY: `error` is not null, and the caller passes a pointer to a
        // `char *` it may write to.
        unsafe { *error = give_string(message) };
    }
    std::ptr::null_mut()
}

/// The kept game for the session as it stands: text for the caller to store
/// as it is and hand back to `zmx_session_open`, freed with
/// `zmx_string_free`. Null for a null session or one that has stopped
/// unexpectedly; what was stored before is then left as it is.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_kept(session: *mut zmx_session) -> *mut c_char {
    let Some(wrapper) = borrow(session) else {
        return std::ptr::null_mut();
    };
    if wrapper.failed {
        return std::ptr::null_mut();
    }
    let kept = catch_unwind(AssertUnwindSafe(|| wrapper.session.kept()));
    match kept.map(String::from_utf8) {
        Ok(Ok(kept)) => give_string(kept),
        _ => std::ptr::null_mut(),
    }
}

/// The caller could not store the kept game. The session's next snapshot
/// says so, the first time in a game.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_keep_failed(session: *mut zmx_session) {
    let Some(wrapper) = borrow(session) else {
        return;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| wrapper.session.keep_failed()));
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_free(session: *mut zmx_session) {
    if session.is_null() {
        return;
    }
    // SAFETY: a non-null pointer came from `Box::into_raw` in
    // `zmx_session_new`, and the caller frees a session exactly once and does
    // not use it afterwards.
    let wrapper = unsafe { Box::from_raw(session) };
    // Dropping runs no code of the game's, but nothing may unwind into C.
    let _ = catch_unwind(AssertUnwindSafe(|| drop(wrapper)));
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_send_line(
    session: *mut zmx_session,
    generation: u64,
    turn_count: usize,
    utf8_line: *const c_char,
) -> c_int {
    let Some(line) = text(utf8_line) else {
        return 0;
    };
    command(session, generation, turn_count, |session, at| {
        session.send_line(at, line).is_ok()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_tap(
    session: *mut zmx_session,
    generation: u64,
    turn_count: usize,
    place: *const c_char,
) -> c_int {
    let Some(place) = text(place).and_then(Place::from_id) else {
        return 0;
    };
    command(session, generation, turn_count, |session, at| {
        session.tap(at, place).is_ok()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_tap_extra(
    session: *mut zmx_session,
    generation: u64,
    turn_count: usize,
    index: usize,
) -> c_int {
    command(session, generation, turn_count, |session, at| {
        session.tap_extra(at, index).is_ok()
    })
}

/// The player touched a way of the map: one of the snapshot's `map.ways`, by
/// its index. Returns 1 if the game received its command and 0 if it was
/// refused.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_tap_way(
    session: *mut zmx_session,
    generation: u64,
    turn_count: usize,
    index: usize,
) -> c_int {
    command(session, generation, turn_count, |session, at| {
        session.tap_way(at, index).is_ok()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_new_game(session: *mut zmx_session, entropy: u32) {
    let Some(wrapper) = borrow(session) else {
        return;
    };
    let started = catch_unwind(AssertUnwindSafe(|| {
        wrapper.session.new_game(Seed::Fresh(entropy));
    }));
    wrapper.failed = started.is_err();
    wrapper.panic_next = false;
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_snapshot(session: *mut zmx_session, since: usize) -> *mut c_char {
    let Some(wrapper) = borrow(session) else {
        return std::ptr::null_mut();
    };
    let failed = wrapper.failed;
    let json = catch_unwind(AssertUnwindSafe(|| {
        if failed {
            wrapper.session.mark_failed(STOPPED);
        }
        wrapper.session.snapshot_json(since)
    }));
    match json {
        Ok(json) => give_string(json),
        Err(_) => {
            wrapper.failed = true;
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_string_free(string: *mut c_char) {
    if string.is_null() {
        return;
    }
    // SAFETY: a non-null pointer came from `CString::into_raw` in this crate,
    // and the caller frees a string exactly once and does not use it afterwards.
    drop(unsafe { CString::from_raw(string) });
}

/// For tests only: makes the next command on this session panic inside the
/// layer, to show that no panic crosses the boundary. Not part of the C
/// interface, and not exported to it.
#[doc(hidden)]
pub fn panic_on_next_command(session: *mut zmx_session) {
    if let Some(wrapper) = borrow(session) {
        wrapper.panic_next = true;
    }
}
