//! The Quetzal save format, version 1.4, for version 3 stories.
//!
//! A save is an IFF `FORM` of type `IFZS` holding three chunks: `IFhd` identifies
//! the story and records the program counter; `CMem` (or `UMem`) holds dynamic
//! memory; `Stks` holds the call stack. "Q§" below refers to sections of the
//! Quetzal specification.

use crate::exec::Frame;
use crate::header::Header;

/// A saved position, decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Saved {
    /// For version 3, the address of the branch data of the `save` instruction
    /// that made the save (Q§5.8.1).
    pub pc: u32,
    pub dynamic: Vec<u8>,
    /// The main code's frame first, then the routines in progress, oldest first.
    pub frames: Vec<Frame>,
}

/// Why a file was not accepted as a saved position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RestoreError {
    /// Not an IFF form of type IFZS, or its chunks do not fit inside it.
    NotQuetzal,
    /// The save belongs to a different story file (Q§5.3).
    WrongStory,
    /// The memory chunk does not decode to dynamic memory (Q§3.5, Q§3.6).
    BadMemory,
    /// The stack chunk does not divide into whole frames (Q§4.3).
    BadStacks,
    /// `IFhd`, the memory chunk or `Stks` is absent, or `IFhd` comes too late.
    MissingChunk,
}

fn push_chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(id);
    out.extend((body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    // IFF chunks are padded to an even length; the pad is not counted (Q§5.7).
    if body.len() % 2 == 1 {
        out.push(0);
    }
}

/// Q§3.2: XOR with the original, then run-length encode the zeros. A final run of
/// zeros is left out, as Q§3.4 allows.
fn compress(original: &[u8], current: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut zeros = 0usize;
    for (&was, &is) in original.iter().zip(current) {
        let delta = was ^ is;
        if delta == 0 {
            zeros += 1;
            continue;
        }
        while zeros > 0 {
            let run = zeros.min(256);
            out.extend([0, (run - 1) as u8]);
            zeros -= run;
        }
        out.push(delta);
    }
    out
}

/// Reverses [`compress`]. Data shorter than dynamic memory leaves the rest as it
/// was in the original (Q§3.4); longer, or ending in half a run, is an error
/// (Q§3.5).
fn decompress(body: &[u8], original: &[u8]) -> Result<Vec<u8>, RestoreError> {
    let mut dynamic = original.to_vec();
    let mut at = 0usize;
    let mut bytes = body.iter();
    while let Some(&byte) = bytes.next() {
        if byte != 0 {
            *dynamic.get_mut(at).ok_or(RestoreError::BadMemory)? ^= byte;
            at += 1;
        } else {
            let count = *bytes.next().ok_or(RestoreError::BadMemory)? as usize + 1;
            at += count;
            if at > dynamic.len() {
                return Err(RestoreError::BadMemory);
            }
        }
    }
    Ok(dynamic)
}

/// Q§4.3: one frame. The first frame written is the dummy for the main code
/// (Q§4.11), which has every field zero except its stack.
fn push_frame(out: &mut Vec<u8>, frame: &Frame, is_main: bool) {
    if is_main {
        out.extend([0, 0, 0, 0, 0, 0]);
    } else {
        out.extend(&frame.return_pc.to_be_bytes()[1..]);
        // 000pvvvv: v counts the locals; p is set when the result is discarded.
        let discard = if frame.result.is_none() { 0x10 } else { 0 };
        out.push(discard | frame.locals.len() as u8);
        out.push(frame.result.unwrap_or(0));
        // 0gfedcba: one bit for each argument supplied.
        out.push(((1u16 << frame.arg_count.min(7)) - 1) as u8);
    }
    out.extend((frame.stack.len() as u16).to_be_bytes());
    if !is_main {
        for local in &frame.locals {
            out.extend(local.to_be_bytes());
        }
    }
    for word in &frame.stack {
        out.extend(word.to_be_bytes());
    }
}

/// Builds a save file.
///
/// `original` is dynamic memory as it is in the story file and `current` as it is
/// now. `frames` starts with the main code's frame. `pc` is the address of the
/// branch data of the `save` instruction being executed.
pub(crate) fn write(
    header: &Header,
    original: &[u8],
    current: &[u8],
    frames: &[Frame],
    pc: u32,
) -> Vec<u8> {
    let mut ifhd = Vec::with_capacity(13);
    ifhd.extend(header.release.to_be_bytes());
    ifhd.extend(header.serial);
    ifhd.extend(header.checksum.to_be_bytes());
    ifhd.extend(&pc.to_be_bytes()[1..]);

    let mut stacks = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        push_frame(&mut stacks, frame, i == 0);
    }

    let mut body = b"IFZS".to_vec();
    push_chunk(&mut body, b"IFhd", &ifhd);
    push_chunk(&mut body, b"CMem", &compress(original, current));
    push_chunk(&mut body, b"Stks", &stacks);

    let mut file = b"FORM".to_vec();
    file.extend((body.len() as u32).to_be_bytes());
    file.extend(body);
    file
}

fn read_frames(mut body: &[u8]) -> Result<Vec<Frame>, RestoreError> {
    let word = |bytes: &[u8]| u16::from_be_bytes([bytes[0], bytes[1]]);
    let mut frames = Vec::new();
    while !body.is_empty() {
        if body.len() < 8 {
            return Err(RestoreError::BadStacks);
        }
        let return_pc = u32::from_be_bytes([0, body[0], body[1], body[2]]);
        let flags = body[3];
        let locals_count = (flags & 0x0F) as usize;
        let stack_count = word(&body[6..8]) as usize;
        let words = locals_count + stack_count;
        let rest = &body[8..];
        if rest.len() < 2 * words {
            return Err(RestoreError::BadStacks);
        }
        let values: Vec<u16> = rest[..2 * words].chunks(2).map(word).collect();
        frames.push(Frame {
            return_pc,
            result: (flags & 0x10 == 0 && !frames.is_empty()).then_some(body[4]),
            locals: values[..locals_count].to_vec(),
            stack: values[locals_count..].to_vec(),
            arg_count: (body[5] & 0x7F).count_ones() as u8,
        });
        body = &rest[2 * words..];
    }
    // Q§4.11.2: the dummy frame is always there.
    if frames.is_empty() {
        return Err(RestoreError::BadStacks);
    }
    Ok(frames)
}

/// Reads a save file for the story whose header and original dynamic memory are
/// given. Nothing is changed by a failed read.
pub(crate) fn read(bytes: &[u8], header: &Header, original: &[u8]) -> Result<Saved, RestoreError> {
    if bytes.len() < 12 || &bytes[..4] != b"FORM" || &bytes[8..12] != b"IFZS" {
        return Err(RestoreError::NotQuetzal);
    }
    let form_len = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let end = form_len
        .checked_add(8)
        .filter(|&end| end <= bytes.len())
        .ok_or(RestoreError::NotQuetzal)?;

    let mut pc = None;
    let mut dynamic = None;
    let mut frames = None;
    let mut at = 12;
    while at < end {
        if end - at < 8 {
            return Err(RestoreError::NotQuetzal);
        }
        let id = &bytes[at..at + 4];
        let len = u32::from_be_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
            as usize;
        let start = at + 8;
        let stop = start
            .checked_add(len)
            .filter(|&stop| stop <= end)
            .ok_or(RestoreError::NotQuetzal)?;
        let body = &bytes[start..stop];
        match id {
            b"IFhd" => {
                if body.len() < 13 {
                    return Err(RestoreError::NotQuetzal);
                }
                let same_story = body[0..2] == header.release.to_be_bytes()
                    && body[2..8] == header.serial
                    && body[8..10] == header.checksum.to_be_bytes();
                if !same_story {
                    return Err(RestoreError::WrongStory);
                }
                pc = Some(u32::from_be_bytes([0, body[10], body[11], body[12]]));
            }
            // Q§5.4: IFhd comes before the memory and stack chunks, so that the
            // wrong story is found out before they are decoded.
            b"CMem" | b"UMem" | b"Stks" if pc.is_none() => return Err(RestoreError::MissingChunk),
            b"CMem" => dynamic = Some(decompress(body, original)?),
            b"UMem" => {
                if body.len() != original.len() {
                    return Err(RestoreError::BadMemory);
                }
                dynamic = Some(body.to_vec());
            }
            b"Stks" => frames = Some(read_frames(body)?),
            // Annotations and other interpreters' own chunks are not ours to read.
            _ => {}
        }
        at = stop + len % 2;
    }

    match (pc, dynamic, frames) {
        (Some(pc), Some(dynamic), Some(frames)) => Ok(Saved {
            pc,
            dynamic,
            frames,
        }),
        _ => Err(RestoreError::MissingChunk),
    }
}

#[cfg(test)]
#[path = "quetzal_tests.rs"]
mod tests;
