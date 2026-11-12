//! C bindings to the experience layer (`include/zmx.h`).
//!
//! This is the only crate in the workspace that may use `unsafe`
//! (constitution Principle V). The contract is
//! `specs/004-first-demo/contracts/c-abi.md`.

use std::ffi::{c_char, c_int};

/// A session as C sees it: an opaque pointer.
#[allow(non_camel_case_types)]
pub struct zmx_session {
    _private: (),
}

/// Not yet built: every function refuses.
#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_new(
    _story: *const u8,
    _length: usize,
    _seed: u32,
    _has_seed: c_int,
    _entropy: u32,
    _error: *mut *mut c_char,
) -> *mut zmx_session {
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_free(_session: *mut zmx_session) {}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_send_line(
    _session: *mut zmx_session,
    _generation: u64,
    _turn_count: usize,
    _utf8_line: *const c_char,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_tap(
    _session: *mut zmx_session,
    _generation: u64,
    _turn_count: usize,
    _place: *const c_char,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_tap_extra(
    _session: *mut zmx_session,
    _generation: u64,
    _turn_count: usize,
    _index: usize,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_new_game(_session: *mut zmx_session, _entropy: u32) {}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_session_snapshot(_session: *mut zmx_session, _since: usize) -> *mut c_char {
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn zmx_string_free(_string: *mut c_char) {}

/// For tests only: makes the next command on this session panic inside the
/// layer, to show that no panic crosses the boundary. Not part of the C
/// interface, and not exported to it.
#[doc(hidden)]
pub fn panic_on_next_command(_session: *mut zmx_session) {}
