//! Version 8 (feature 002, T040; §1.1.4, §1.2.3, §11.1.6). Version 8 is version 5
//! with packed addresses multiplied by 8 and a larger limit on the file's size.

mod common;

use common::*;
use zmachine_core::{Event, LoadError, Machine};
use zmachine_testkit::Story5;
use zmachine_testkit::op::{self, Arg::*};

const K: usize = 1024;

/// A story of `version` whose routine and string lie after `filler` bytes of high
/// memory. The main code calls the routine with 20 and 22, prints the result, and
/// prints the string by its packed address.
fn far_story(version: u8, filler: usize) -> (Story5, u32) {
    let mut story = Story5::new(version);
    // Main code has to sit in the first 64K, so it is added first and refers to
    // packed addresses worked out from where the filler will end.
    let multiple = story.packed_multiplier();
    let start = story.next_code_address() as usize;
    let main_len = 64;
    let far = (start + main_len + filler).next_multiple_of(multiple);
    let routine = (far / multiple) as u16;
    let routine_code = cat(&[to(op::op2(op::ADD, V(1), V(2)), 0), op::op1(op::RET, V(0))]);
    let string = (far + 1 + routine_code.len()).div_ceil(multiple) as u16;

    let mut main = cat(&[
        to(op::var(op::CALL_VS, &[L(routine), S(20), S(22)]), g(0)),
        print_num(V(g(0))),
        print(" "),
        op::op1(op::PRINT_PADDR, L(string)),
        quit(),
    ]);
    assert!(main.len() <= main_len);
    main.resize(main_len, 0);
    story.main(&main);
    story.filler(filler);
    assert_eq!(story.routine(2, &routine_code), routine);
    assert_eq!(story.string("far away"), string);
    (story, far as u32)
}

// §1.2.3: a version 8 packed address is the byte address divided by 8, which
// reaches the whole 512K a version 8 file may fill (§1.1.4). The routine and the
// string here lie beyond 256K, where no version 5 packed address can point.
#[test]
fn a_version_8_story_runs_code_and_prints_text_beyond_256k() {
    let (mut story, far) = far_story(8, 300 * K);
    assert!(far > 256 * K as u32);
    assert_eq!(story.packed_multiplier(), 8);
    let image = story.build();
    assert!(image.len() > 256 * K && image.len() <= 512 * K);
    let mut m = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert_eq!(m.take_output(), "42 far away\n");
}

// The same program near the top of each version's range: just under 256K for
// version 5 (packed times 4) and just under 512K for version 8 (packed times 8).
#[test]
fn packed_addresses_reach_the_top_of_each_versions_range() {
    for (version, filler) in [(5u8, 250 * K), (8, 500 * K)] {
        let (mut story, _) = far_story(version, filler);
        let mut m = Machine::load(story.build(), OPTIONS).unwrap();
        assert_eq!(
            run_until_event(&mut m, BIG),
            Event::Finished,
            "version {version}"
        );
        assert_eq!(m.take_output(), "42 far away\n", "version {version}");
    }
}

/// A story of `version` padded with `filler` bytes that prints whether `verify`
/// succeeds.
fn verify_story(version: u8, filler: usize) -> Vec<u8> {
    let mut story = Story5::new(version);
    story.main(&cat(&[
        skip_if(op::op0(op::VERIFY), true, &print("bad ")),
        print("checked"),
        quit(),
    ]));
    story.filler(filler);
    story.build()
}

// §11.1.6: the header's length field is the file length divided by 8 in version 8
// and by 4 in version 5; `verify` sums the bytes up to that length (§15 verify).
#[test]
fn verify_uses_the_versions_length_multiplier() {
    for (version, filler) in [(5u8, 100 * K), (8, 300 * K)] {
        let image = verify_story(version, filler);
        let multiple = if version == 8 { 8 } else { 4 };
        let stated = u16::from_be_bytes([image[0x1A], image[0x1B]]) as usize;
        assert_eq!(stated * multiple, image.len(), "version {version}");

        let mut m = Machine::load(image.clone(), OPTIONS).unwrap();
        run_until_event(&mut m, BIG);
        assert_eq!(
            m.take_output(),
            "checked\n",
            "version {version}: an intact file"
        );

        // A changed byte in the last part of the file, which only the right
        // multiplier reaches, makes verify fail.
        let mut damaged = image;
        let last = damaged.len() - 9;
        damaged[last] ^= 0x55;
        let mut m = Machine::load(damaged, OPTIONS).unwrap();
        run_until_event(&mut m, BIG);
        assert_eq!(
            m.take_output(),
            "bad checked\n",
            "version {version}: a damaged file"
        );
    }
}

// §1.1.4: a version 8 file may be 512K and no more; a version 5 file 256K.
#[test]
fn a_file_larger_than_its_version_allows_is_refused() {
    for (version, limit) in [(5u8, 256 * K), (8, 512 * K)] {
        let mut image = verify_story(version, 1000);
        image.resize(limit, 0);
        assert!(
            Machine::load(image.clone(), OPTIONS).is_ok(),
            "version {version} at its limit"
        );
        image.push(0);
        assert_eq!(
            Machine::load(image, OPTIONS).err(),
            Some(LoadError::TooLarge),
            "version {version} one byte over"
        );
    }
}

// Guarantee: nothing but the multipliers and the size limit differs. The same
// version 5 program built as version 8 prints the same text.
#[test]
fn the_same_program_behaves_alike_in_both_versions() {
    let run = |version: u8| {
        let mut story = Story5::new(version);
        story.word("look");
        let text = story.text_buffer(20, "");
        let parse = story.parse_buffer(4);
        let table = story.reserve(16);
        let double = story.routine(
            1,
            &cat(&[to(op::op2(op::ADD, V(1), V(1)), 0), op::op1(op::RET, V(0))]),
        );
        let message = story.string("packed string");
        story.main(&cat(&[
            to(op::op2(op::CALL_2S, L(double), S(21)), g(0)),
            show(0),
            op::op1(op::PRINT_PADDR, L(message)),
            print(" "),
            op::var(op::OUTPUT_STREAM, &[S(3), L(table)]),
            print("xyz"),
            op::var(op::OUTPUT_STREAM, &[L(0xFFFD)]),
            to(op::op2(op::LOADW, L(table), S(0)), g(1)),
            show(1),
            aread(text, parse, g(2)),
            show(2),
            to(op::op2(op::LOADB, L(parse), S(1)), g(3)),
            show(3),
            quit(),
        ]));
        let mut m = Machine::load(story.build(), OPTIONS).unwrap();
        assert!(matches!(
            run_until_event(&mut m, BIG),
            Event::LineWanted { .. }
        ));
        m.provide_line("look around").unwrap();
        assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
        m.take_output()
    };
    assert_eq!(run(5), "42 packed string 3 13 2 \n");
    assert_eq!(run(5), run(8));
}
