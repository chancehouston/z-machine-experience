use super::*;
use crate::exec::Frame;
use crate::header::{self, Header};
use zmachine_testkit::{Story, op};

// "Q§" below is a section of the Quetzal specification, version 1.4.

struct Fixture {
    header: Header,
    original: Vec<u8>,
}

fn fixture() -> Fixture {
    let mut story = Story::new();
    story.release(7).serial("261111");
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let header = header::parse(&image).unwrap();
    let original = image[..header.static_base as usize].to_vec();
    Fixture { header, original }
}

fn frame(
    return_pc: u32,
    result: Option<u8>,
    locals: &[u16],
    stack: &[u16],
    arg_count: u8,
) -> Frame {
    Frame {
        return_pc,
        result,
        locals: locals.to_vec(),
        stack: stack.to_vec(),
        arg_count,
    }
}

fn main_frame(stack: &[u16]) -> Frame {
    frame(0, None, &[], stack, 0)
}

fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = id.to_vec();
    out.extend((body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    if body.len() % 2 == 1 {
        out.push(0);
    }
    out
}

fn form(chunks: &[Vec<u8>]) -> Vec<u8> {
    let body: Vec<u8> = [b"IFZS".to_vec(), chunks.concat()].concat();
    let mut out = b"FORM".to_vec();
    out.extend((body.len() as u32).to_be_bytes());
    out.extend(body);
    out
}

fn ifhd(f: &Fixture, pc: u32) -> Vec<u8> {
    let mut body = f.header.release.to_be_bytes().to_vec();
    body.extend(f.header.serial);
    body.extend(f.header.checksum.to_be_bytes());
    body.extend(&pc.to_be_bytes()[1..]);
    chunk(b"IFhd", &body)
}

/// A `Stks` chunk holding only the dummy frame, with an empty stack.
fn empty_stacks() -> Vec<u8> {
    chunk(b"Stks", &[0, 0, 0, 0, 0, 0, 0, 0])
}

/// Splits a Quetzal file into its chunks: (id, body).
fn chunks_of(file: &[u8]) -> Vec<(String, Vec<u8>)> {
    assert_eq!(&file[..4], b"FORM");
    assert_eq!(
        u32::from_be_bytes(file[4..8].try_into().unwrap()) as usize,
        file.len() - 8
    );
    assert_eq!(&file[8..12], b"IFZS");
    let mut out = Vec::new();
    let mut at = 12;
    while at < file.len() {
        let id = String::from_utf8(file[at..at + 4].to_vec()).unwrap();
        let len = u32::from_be_bytes(file[at + 4..at + 8].try_into().unwrap()) as usize;
        out.push((id, file[at + 8..at + 8 + len].to_vec()));
        at += 8 + len + len % 2;
    }
    assert_eq!(at, file.len(), "chunks fill the form exactly");
    out
}

// Q§5.4, Q§5.7: the file is an IFF FORM of type IFZS; `IFhd` comes before the
// memory and stack chunks, is 13 bytes, and is followed by a pad byte.
#[test]
fn a_save_has_the_three_chunks_in_order() {
    let f = fixture();
    let file = write(
        &f.header,
        &f.original,
        &f.original,
        &[main_frame(&[])],
        0x1234,
    );
    let chunks = chunks_of(&file);
    let ids: Vec<&str> = chunks.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["IFhd", "CMem", "Stks"]);
    assert_eq!(chunks[0].1.len(), 13);
    assert_eq!(file.len() % 2, 0, "every chunk is padded to an even length");
}

// Q§5.4.3 to Q§5.4.6: release, serial, checksum, then the program counter in three
// bytes. Q§5.8.1: for version 3 it is the address of the save instruction's branch
// data, which the caller supplies.
#[test]
fn ifhd_identifies_the_story_and_holds_the_program_counter() {
    let f = fixture();
    let file = write(
        &f.header,
        &f.original,
        &f.original,
        &[main_frame(&[])],
        0x01_2345,
    );
    let body = &chunks_of(&file)[0].1;
    assert_eq!(&body[0..2], &7u16.to_be_bytes());
    assert_eq!(&body[2..8], b"261111");
    assert_eq!(&body[8..10], &f.header.checksum.to_be_bytes());
    assert_eq!(&body[10..13], &[0x01, 0x23, 0x45]);
}

// Q§3.2: dynamic memory is XORed with the original and run-length encoded: a
// non-zero byte stands for itself, a zero byte is followed by a count n meaning
// n + 1 zeros. Q§3.4: a final run of zeros may be left out.
#[test]
fn unchanged_memory_compresses_to_nothing_and_changes_are_literal() {
    let f = fixture();
    let file = write(&f.header, &f.original, &f.original, &[main_frame(&[])], 0);
    assert_eq!(
        chunks_of(&file)[1].1,
        Vec::<u8>::new(),
        "nothing changed, nothing stored"
    );

    let mut current = f.original.clone();
    current[0x40] ^= 0x5A;
    current[0x43] ^= 0x01;
    let file = write(&f.header, &f.original, &current, &[main_frame(&[])], 0);
    // 0x40 unchanged bytes, the change, two unchanged, the change; the rest left out.
    assert_eq!(chunks_of(&file)[1].1, vec![0, 0x3F, 0x5A, 0, 1, 0x01]);
}

// Q§3.2: a run longer than 256 zeros needs more than one pair.
#[test]
fn long_runs_are_split_at_256() {
    let f = fixture();
    let mut current = f.original.clone();
    current[600] ^= 0xFF;
    let file = write(&f.header, &f.original, &current, &[main_frame(&[])], 0);
    // 600 zeros = 256 + 256 + 88.
    assert_eq!(chunks_of(&file)[1].1, vec![0, 255, 0, 255, 0, 87, 0xFF]);
}

// Q§4.3: each frame is the return address in three bytes, a flags byte whose low
// four bits count the locals (bit 4 set when the result is discarded), the result
// variable, a bit per argument supplied, the evaluation stack size, the locals and
// the stack. Q§4.11: a dummy frame for the main code comes first.
#[test]
fn stack_frames_are_written_in_the_quetzal_layout() {
    let f = fixture();
    let frames = [
        main_frame(&[0xAAAA, 0xBBBB]),
        frame(0x01_0203, Some(0x10), &[1, 2, 3], &[9], 2),
        frame(0x00_0405, None, &[], &[], 0),
    ];
    let file = write(&f.header, &f.original, &f.original, &frames, 0);
    let body = &chunks_of(&file)[2].1;
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        // dummy frame: everything zero but the stack count, then the stack
        0, 0, 0,  0,  0,  0,  0, 2,  0xAA, 0xAA, 0xBB, 0xBB,
        // return 0x010203, 3 locals, result to variable 0x10, arguments a and b,
        // one stack word; locals 1 2 3; stack 9
        1, 2, 3,  3,  0x10,  0b11,  0, 1,  0, 1, 0, 2, 0, 3,  0, 9,
        // return 0x000405, no locals and the result discarded (bit 4)
        0, 4, 5,  0x10,  0,  0,  0, 0,
    ];
    assert_eq!(body, &expected);
}

// What is written can be read back exactly.
#[test]
fn a_save_round_trips() {
    let f = fixture();
    let mut current = f.original.clone();
    for (i, byte) in current.iter_mut().enumerate().skip(0x40) {
        if i % 37 == 0 {
            *byte = byte.wrapping_add(i as u8 | 1);
        }
    }
    let last = current.len() - 1;
    current[last] ^= 0x80;
    let frames = vec![
        main_frame(&[5, 6, 7]),
        frame(0x0666, Some(0), &[0xFFFF; 15], &[1, 2, 3, 4], 3),
        frame(0x1F_FFFE, None, &[], &[], 0),
        frame(0x0777, Some(255), &[1], &[], 1),
    ];
    let file = write(&f.header, &f.original, &current, &frames, 0x01_ABCD);
    let saved = read(&file, &f.header, &f.original).expect("our own save restores");
    assert_eq!(saved.pc, 0x01_ABCD);
    assert_eq!(saved.dynamic, current);
    assert_eq!(saved.frames, frames);
}

// Q§3.4: "If the decoded data is shorter than the length of dynamic memory, then
// the missing section is assumed to be a run of zeroes". data-model.md: "Decoded
// data shorter than dynamic memory is accepted, the rest unchanged".
#[test]
fn compressed_memory_shorter_than_dynamic_memory_is_accepted() {
    let f = fixture();
    let file = form(&[
        ifhd(&f, 0x99),
        chunk(b"CMem", &[0, 0x3F, 0x5A]),
        empty_stacks(),
    ]);
    let saved = read(&file, &f.header, &f.original).unwrap();
    let mut expected = f.original.clone();
    expected[0x40] ^= 0x5A;
    assert_eq!(saved.dynamic, expected);
    assert_eq!(saved.pc, 0x99);
    assert_eq!(saved.frames, vec![main_frame(&[])]);
}

// Q§3.5: "the decoded data may be larger than dynamic memory, and the encoded data
// may finish with an incomplete run". data-model.md: "longer is refused".
#[test]
fn bad_compressed_memory_is_refused() {
    let f = fixture();
    let too_long: Vec<u8> = vec![1; f.original.len() + 1];
    let cases: [(&str, Vec<u8>); 3] = [
        ("one byte too many", too_long),
        (
            "a run past the end",
            [vec![1; f.original.len() - 1], vec![0, 1]].concat(),
        ),
        ("an incomplete run", vec![1, 2, 0]),
    ];
    for (what, body) in cases {
        let file = form(&[ifhd(&f, 0), chunk(b"CMem", &body), empty_stacks()]);
        assert_eq!(
            read(&file, &f.header, &f.original),
            Err(RestoreError::BadMemory),
            "{what}"
        );
    }
}

// Q§3.6: an uncompressed dump is read too. "It is an error for this dump to be
// shorter or longer than the expected length of dynamic memory."
#[test]
fn uncompressed_memory_must_be_exactly_the_right_size() {
    let f = fixture();
    let mut dump = f.original.clone();
    dump[0x50] = 0xEE;
    let file = form(&[ifhd(&f, 0), chunk(b"UMem", &dump), empty_stacks()]);
    assert_eq!(read(&file, &f.header, &f.original).unwrap().dynamic, dump);

    for wrong in [
        &dump[..dump.len() - 1],
        &[dump.clone(), vec![0]].concat()[..],
    ] {
        let file = form(&[ifhd(&f, 0), chunk(b"UMem", wrong), empty_stacks()]);
        assert_eq!(
            read(&file, &f.header, &f.original),
            Err(RestoreError::BadMemory)
        );
    }
}

// Q§5.3: release, serial and checksum must all match the loaded story.
// data-model.md: "Identity must equal the loaded story's, otherwise refused".
#[test]
fn a_save_from_another_story_is_refused() {
    let f = fixture();
    let good = form(&[ifhd(&f, 0), chunk(b"CMem", &[]), empty_stacks()]);
    assert!(read(&good, &f.header, &f.original).is_ok());
    // Byte offsets inside the file: release at 20, serial at 22, checksum at 28.
    for at in [20, 21, 22, 27, 28, 29] {
        let mut file = good.clone();
        file[at] ^= 1;
        assert_eq!(
            read(&file, &f.header, &f.original),
            Err(RestoreError::WrongStory),
            "byte {at}"
        );
    }
}

// data-model.md: Stks "Must parse completely".
#[test]
fn malformed_stacks_are_refused() {
    let f = fixture();
    let cases: [(&str, Vec<u8>); 5] = [
        ("empty: no dummy frame", vec![]),
        ("a frame header cut short", vec![0, 0, 0, 0, 0, 0, 0]),
        ("stack words missing", vec![0, 0, 0, 0, 0, 0, 0, 2, 0, 1]),
        (
            "locals missing",
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0, 0, 0, 5],
        ),
        ("a trailing odd byte", vec![0, 0, 0, 0, 0, 0, 0, 0, 7]),
    ];
    for (what, body) in cases {
        let file = form(&[ifhd(&f, 0), chunk(b"CMem", &[]), chunk(b"Stks", &body)]);
        assert_eq!(
            read(&file, &f.header, &f.original),
            Err(RestoreError::BadStacks),
            "{what}"
        );
    }
}

// A file that is not a Quetzal save, or lacks a required chunk, is refused.
#[test]
fn files_that_are_not_complete_saves_are_refused() {
    let f = fixture();
    let good = form(&[ifhd(&f, 0), chunk(b"CMem", &[]), empty_stacks()]);
    for len in [0, 3, 8, 11, 12, 20, good.len() - 1] {
        assert!(
            read(&good[..len], &f.header, &f.original).is_err(),
            "cut to {len} bytes"
        );
    }
    let mut not_form = good.clone();
    not_form[0] = b'X';
    assert_eq!(
        read(&not_form, &f.header, &f.original),
        Err(RestoreError::NotQuetzal)
    );
    let mut not_ifzs = good.clone();
    not_ifzs[8..12].copy_from_slice(b"AIFF");
    assert_eq!(
        read(&not_ifzs, &f.header, &f.original),
        Err(RestoreError::NotQuetzal)
    );
    assert_eq!(
        read(
            b"just some text, not a save file at all",
            &f.header,
            &f.original
        ),
        Err(RestoreError::NotQuetzal)
    );

    let missing = |chunks: &[Vec<u8>]| read(&form(chunks), &f.header, &f.original);
    assert_eq!(
        missing(&[chunk(b"CMem", &[]), empty_stacks()]),
        Err(RestoreError::MissingChunk)
    );
    assert_eq!(
        missing(&[ifhd(&f, 0), empty_stacks()]),
        Err(RestoreError::MissingChunk)
    );
    assert_eq!(
        missing(&[ifhd(&f, 0), chunk(b"CMem", &[])]),
        Err(RestoreError::MissingChunk)
    );
    // A chunk length that runs past the end of the file.
    let mut overlong = good.clone();
    let at = overlong.len() - 12;
    overlong[at..at + 4].copy_from_slice(&1000u32.to_be_bytes());
    assert!(read(&overlong, &f.header, &f.original).is_err());
}

// Q§7 allows other chunks (annotations, an interpreter's own); they are skipped,
// including their pad bytes.
#[test]
fn unknown_chunks_are_skipped() {
    let f = fixture();
    let file = form(&[
        ifhd(&f, 0x42),
        chunk(b"ANNO", b"odd"),
        chunk(b"CMem", &[0, 0x3F, 0x11]),
        chunk(b"IntD", &[1, 2, 3, 4]),
        empty_stacks(),
        chunk(b"AUTH", b"someone"),
    ]);
    let saved = read(&file, &f.header, &f.original).unwrap();
    assert_eq!(saved.pc, 0x42);
    assert_eq!(saved.dynamic[0x40], f.original[0x40] ^ 0x11);
}

// The real thing: saves written by Frotz 2.55 from Zork I
// (stories/zork1/fidelity/README.md) are read without complaint.
#[test]
fn saves_written_by_frotz_are_read() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1");
    let story = std::fs::read(format!("{root}/zork1.z3")).unwrap();
    let header = header::parse(&story).unwrap();
    let original = &story[..header.static_base as usize];
    for n in 1..=5 {
        let file = std::fs::read(format!("{root}/fidelity/saves/frotz_{n}.qzl")).unwrap();
        let saved =
            read(&file, &header, original).unwrap_or_else(|e| panic!("frotz_{n}.qzl: {e:?}"));
        assert_eq!(saved.dynamic.len(), original.len());
        assert_ne!(
            saved.dynamic, original,
            "frotz_{n}.qzl: the game had changed its memory"
        );
        // Q§5.8.1: the saved program counter points at the branch data of a save
        // instruction, which is the 0OP opcode 0xB5 (§14).
        assert_eq!(story[saved.pc as usize - 1], 0xB5, "frotz_{n}.qzl");
        // Q§4.11: the dummy frame, then the routines in progress.
        assert!(saved.frames.len() >= 2, "frotz_{n}.qzl");
        assert_eq!(
            (saved.frames[0].return_pc, saved.frames[0].locals.len()),
            (0, 0)
        );
        // A frame that stores its result returns to just after the store byte.
        for frame in &saved.frames[1..] {
            if let Some(variable) = frame.result {
                assert_eq!(
                    story[frame.return_pc as usize - 1],
                    variable,
                    "frotz_{n}.qzl"
                );
            }
        }
    }
}
