//! The kept form, written and read
//! (specs/011-resume-game/contracts/kept-game.md, part K, cited K§).

use crate::kept::{Kept, Unread, digest, transcript};
use crate::{Seed, Source};

const STORY: &str = "3f2a000000000000000000000000000000000000000000000000000000000000";
const TEXT: &str = "9c41000000000000000000000000000000000000000000000000000000000000";

fn example() -> Kept {
    Kept {
        story: STORY.to_string(),
        chance: Seed::Fresh(2_891_337_402),
        inputs: vec![
            (Source::Typed, "open mailbox".to_string()),
            (Source::Tap, "north".to_string()),
            (Source::Typed, String::new()),
        ],
        transcript: TEXT.to_string(),
    }
}

/// The example's lines before the check, as the contract writes them.
fn body() -> String {
    format!(
        "lamplight-kept 1\nstory {STORY}\nchance fresh 2891337402\ninputs 3\n\
         transcript {TEXT}\ntyped open mailbox\ntap north\ntyped \n"
    )
}

/// A body with its check line made for it, so that only the body is at fault.
fn sealed(body: &str) -> Vec<u8> {
    format!("{body}check {}\n", digest(body.as_bytes())).into_bytes()
}

// K§1 to K§7: the example of the contract, byte for byte.
#[test]
fn the_example_is_written_as_the_contract_shows_it() {
    assert_eq!(example().to_bytes(), sealed(&body()));
}

// K§6: an empty input keeps the space after its word.
#[test]
fn an_empty_input_keeps_its_space() {
    let bytes = example().to_bytes();
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("\ntyped \ncheck ")
    );
}

// K§3: both kinds of chance.
#[test]
fn a_fixed_chance_is_written_and_read() {
    let mut kept = example();
    kept.chance = Seed::Fixed(11);
    let bytes = kept.to_bytes();
    assert!(
        String::from_utf8(bytes.clone())
            .unwrap()
            .contains("\nchance fixed 11\n")
    );
    assert_eq!(Kept::from_bytes(&bytes), Ok(kept));
}

// K§9: the same parts give the same bytes.
#[test]
fn the_same_game_gives_the_same_bytes() {
    assert_eq!(example().to_bytes(), example().to_bytes());
}

#[test]
fn what_was_written_is_read_back() {
    assert_eq!(Kept::from_bytes(&example().to_bytes()), Ok(example()));
    let mut none = example();
    none.inputs.clear();
    assert_eq!(Kept::from_bytes(&none.to_bytes()), Ok(none));
}

// An input may hold anything a line can, a word of the form included.
#[test]
fn an_input_that_looks_like_the_form_is_only_an_input() {
    let mut kept = example();
    kept.inputs = vec![
        (Source::Typed, "check 1".to_string()),
        (Source::Typed, "  two  spaces ".to_string()),
        (Source::Tap, "naïve ☃".to_string()),
    ];
    assert_eq!(Kept::from_bytes(&kept.to_bytes()), Ok(kept));
}

// K§5: each turn's text and a zero byte, so that where one turn ends matters.
#[test]
fn the_transcript_digest_marks_where_turns_end() {
    assert_eq!(transcript(["ab", "c"].into_iter()), digest(b"ab\0c\0"));
    assert_ne!(
        transcript(["ab", "c"].into_iter()),
        transcript(["a", "bc"].into_iter())
    );
    assert_ne!(
        transcript(["a"].into_iter()),
        transcript(["a", ""].into_iter())
    );
}

// K§8: each departure is refused, and none panics.
#[test]
fn every_departure_from_the_form_is_refused() {
    let good = body();
    let wrong = [
        // A missing line.
        good.replace(&format!("story {STORY}\n"), ""),
        good.replace("inputs 3\n", ""),
        // Lines out of order.
        good.replace(
            &format!("story {STORY}\nchance fresh 2891337402\n"),
            &format!("chance fresh 2891337402\nstory {STORY}\n"),
        ),
        // A count that does not match (K§4).
        good.replace("inputs 3", "inputs 2"),
        good.replace("inputs 3", "inputs 4"),
        good.replace("inputs 3", "inputs 03"),
        good.replace("inputs 3", "inputs +3"),
        good.replace("inputs 3", "inputs"),
        // A digest that is not 64 lower-case hex digits.
        good.replace(STORY, &STORY[1..]),
        good.replace(STORY, &STORY.to_uppercase().replace('3', "A")),
        good.replace(TEXT, "none"),
        // A chance that is not one (K§3).
        good.replace("fresh 2891337402", "fresh 4294967296"),
        good.replace("fresh 2891337402", "fixed 2147483648"),
        good.replace("fresh 2891337402", "stale 5"),
        good.replace("fresh 2891337402", "fresh"),
        // An input that is neither typed nor tapped.
        good.replace("tap north", "spoken north"),
        good.replace("tap north", "tap"),
        // A carriage return.
        good.replace("tap north\n", "tap north\r\n"),
        // No first line.
        good.replace("lamplight-kept 1\n", ""),
        good.replace("lamplight-kept 1", "lamplight-keep 1"),
    ];
    for body in wrong {
        assert_ne!(body, good);
        assert_eq!(
            Kept::from_bytes(&sealed(&body)),
            Err(Unread::Damaged),
            "{body}"
        );
    }
}

// K§1: another form is known for what it is.
#[test]
fn another_form_is_not_read() {
    let body = body().replace("lamplight-kept 1", "lamplight-kept 2");
    assert_eq!(Kept::from_bytes(&sealed(&body)), Err(Unread::OtherForm));
}

// K§7: a wrong check, a missing one, and bytes after the end.
#[test]
fn the_check_must_be_there_and_must_match() {
    let good = example().to_bytes();
    let text = String::from_utf8(good.clone()).unwrap();
    assert_eq!(Kept::from_bytes(body().as_bytes()), Err(Unread::Damaged));
    let mut after = good.clone();
    after.push(b'\n');
    assert_eq!(Kept::from_bytes(&after), Err(Unread::Damaged));
    let mut more = good.clone();
    more.extend_from_slice(b"typed x\n");
    assert_eq!(Kept::from_bytes(&more), Err(Unread::Damaged));
    let altered = text.replace("open mailbox", "open mailbag");
    assert_eq!(Kept::from_bytes(altered.as_bytes()), Err(Unread::Damaged));
    assert_eq!(
        Kept::from_bytes(text.trim_end().as_bytes()),
        Err(Unread::Damaged)
    );
}

// K§8: not UTF-8, empty, and cut short at every length.
#[test]
fn bytes_cut_short_or_not_text_are_refused() {
    let good = example().to_bytes();
    for length in 0..good.len() {
        assert_eq!(
            Kept::from_bytes(&good[..length]),
            Err(Unread::Damaged),
            "{length}"
        );
    }
    let mut bad = good.clone();
    bad[20] = 0xff;
    assert_eq!(Kept::from_bytes(&bad), Err(Unread::Damaged));
    assert_eq!(Kept::from_bytes(&[0xff, 0xfe, 0x00]), Err(Unread::Damaged));
}

// Every single byte altered: refused, whatever it becomes.
#[test]
fn any_one_byte_altered_is_refused() {
    let good = example().to_bytes();
    for at in 0..good.len() {
        for change in [1u8, 0x20, 0x80] {
            let mut bad = good.clone();
            bad[at] ^= change;
            assert!(Kept::from_bytes(&bad).is_err(), "byte {at} xor {change}");
        }
    }
}
