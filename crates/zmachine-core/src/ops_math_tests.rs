use super::*;
use crate::FaultKind;

fn s(v: i16) -> u16 {
    v as u16
}

// §2.2, §15 add/sub/mul: arithmetic is on signed 16-bit values and wraps.
#[test]
fn add_sub_mul_wrap_at_16_bits() {
    assert_eq!(add(0x7FFF, 1), 0x8000);
    assert_eq!(add(0xFFFF, 1), 0);
    assert_eq!(sub(0, 1), 0xFFFF);
    assert_eq!(sub(0x8000, 1), 0x7FFF);
    assert_eq!(mul(s(-2), 3), s(-6));
    assert_eq!(mul(s(-2), s(-3)), 6);
    assert_eq!(mul(0x4000, 4), 0);
    assert_eq!(mul(300, 300), (90_000u32 & 0xFFFF) as u16);
}

// §2.4.3, §15 div: signed division rounds toward zero.
#[test]
fn div_rounds_toward_zero() {
    assert_eq!(div(s(-11), 2), Ok(s(-5)));
    assert_eq!(div(s(-11), s(-2)), Ok(5));
    assert_eq!(div(11, s(-2)), Ok(s(-5)));
    assert_eq!(div(11, 2), Ok(5));
    assert_eq!(div(0, 7), Ok(0));
}

// §2.4.3, §15 mod: the remainder takes the sign of the dividend.
#[test]
fn mod_takes_the_sign_of_the_dividend() {
    assert_eq!(modulo(s(-13), 5), Ok(s(-3)));
    assert_eq!(modulo(13, s(-5)), Ok(3));
    assert_eq!(modulo(s(-13), s(-5)), Ok(s(-3)));
    assert_eq!(modulo(13, 5), Ok(3));
}

// §15 div, mod: "Division by zero should halt the interpreter with a suitable
// error message."
#[test]
fn division_and_modulo_by_zero_are_faults() {
    assert_eq!(div(5, 0), Err(FaultKind::DivisionByZero));
    assert_eq!(div(0, 0), Err(FaultKind::DivisionByZero));
    assert_eq!(modulo(5, 0), Err(FaultKind::DivisionByZero));
}

// -32768 / -1 does not fit in 16 bits; it must wrap, not panic.
#[test]
fn the_one_overflowing_division_wraps() {
    assert_eq!(div(s(-32768), s(-1)), Ok(s(-32768)));
    assert_eq!(modulo(s(-32768), s(-1)), Ok(0));
}

// §15 and, or, not: bitwise.
#[test]
fn bitwise_operations() {
    assert_eq!(and(0xF0F0, 0x3C3C), 0x3030);
    assert_eq!(or(0xF0F0, 0x3C3C), 0xFCFC);
    assert_eq!(not(0x00FF), 0xFF00);
    assert_eq!(not(0), 0xFFFF);
}

// §15 test: "Jump if all of the flags in bitmap are set (i.e. if bitmap & flags
// == flags)."
#[test]
fn test_needs_every_flag_set() {
    assert!(test(0b1110, 0b0110));
    assert!(!test(0b1010, 0b0110));
    assert!(test(0x1234, 0));
    assert!(test(0xFFFF, 0xFFFF));
}

// §15 je: "Jump if a is equal to any of the subsequent operands."
#[test]
fn je_matches_any_later_operand() {
    assert!(je(&[5, 5]));
    assert!(!je(&[5, 6]));
    assert!(je(&[5, 1, 5]));
    assert!(je(&[5, 1, 2, 5]));
    assert!(!je(&[5, 1, 2, 3]));
    // With a single operand there is nothing to equal.
    assert!(!je(&[5]));
    assert!(!je(&[]));
}

// §15 jl, jg: signed comparison.
#[test]
fn jl_and_jg_compare_signed() {
    assert!(jl(s(-1), 0));
    assert!(!jl(0, s(-1)));
    assert!(!jl(3, 3));
    assert!(jg(0, s(-1)));
    assert!(!jg(0x8000, 0x7FFF));
    assert!(!jg(3, 3));
}
