//! Tests of the answer types (docs/introspection.md section 2, written I§2).

use crate::answer::*;

// I§2: an unknown carries one of six reasons, each written as a hyphenated word.
#[test]
fn each_unknown_reason_has_its_word() {
    let words: Vec<&str> = Unknown::ALL.iter().map(|u| u.as_str()).collect();
    assert_eq!(
        words,
        ["unrecognised-family", "no-player", "ambiguous-player", "inconsistent", "not-determinable", "damaged"]
    );
}

// I§2, I§1.5: every answer says `exact` or `approximate`.
#[test]
fn exactness_has_two_values() {
    assert_eq!(Exactness::Exact.as_str(), "exact");
    assert_eq!(Exactness::Approximate.as_str(), "approximate");
    assert_eq!(Answer::exact(1).exactness, Exactness::Exact);
    assert_eq!(Answer::approximate(1).exactness, Exactness::Approximate);
    assert_eq!(Answer::exact(7).value, Ok(7));
}

// I§2, I§1.3: unknown is an answer, distinct from an empty one.
#[test]
fn unknown_is_distinct_from_empty() {
    let empty: Answer<Vec<u8>> = Answer::exact(vec![]);
    let unknown: Answer<Vec<u8>> = Answer::unknown(Unknown::NoPlayer, Exactness::Exact);
    assert_ne!(empty, unknown);
    assert!(!empty.is_unknown());
    assert!(unknown.is_unknown());
    assert_eq!(unknown.value, Err(Unknown::NoPlayer));
    assert_eq!(unknown.exactness, Exactness::Exact);
}

// I§2: three-valued fields are `yes`, `no` or `unknown`.
#[test]
fn three_valued_fields() {
    assert_eq!(Tri::Yes.as_str(), "yes");
    assert_eq!(Tri::No.as_str(), "no");
    assert_eq!(Tri::Unknown.as_str(), "unknown");
    assert_eq!(Tri::from_bool(true), Tri::Yes);
    assert_eq!(Tri::from_bool(false), Tri::No);
    let part: Part<u8> = Err(Unknown::NotDeterminable);
    assert!(part.is_err());
}
