//! Arithmetic, logic and comparison (§2, §15). Values are 16-bit words; where the
//! Standard says signed, they are read as two's complement.

use crate::FaultKind;

pub(crate) fn add(a: u16, b: u16) -> u16 {
    a.wrapping_add(b)
}

pub(crate) fn sub(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}

pub(crate) fn mul(a: u16, b: u16) -> u16 {
    a.wrapping_mul(b)
}

/// Signed division, rounding toward zero (§2.4.3).
pub(crate) fn div(a: u16, b: u16) -> Result<u16, FaultKind> {
    if b == 0 {
        return Err(FaultKind::DivisionByZero);
    }
    Ok((a as i16).wrapping_div(b as i16) as u16)
}

/// Signed remainder, taking the sign of the dividend (§2.4.3).
pub(crate) fn modulo(a: u16, b: u16) -> Result<u16, FaultKind> {
    if b == 0 {
        return Err(FaultKind::DivisionByZero);
    }
    Ok((a as i16).wrapping_rem(b as i16) as u16)
}

pub(crate) fn and(a: u16, b: u16) -> u16 {
    a & b
}

pub(crate) fn or(a: u16, b: u16) -> u16 {
    a | b
}

pub(crate) fn not(a: u16) -> u16 {
    !a
}

/// True if every bit of `flags` is set in `bitmap` (§15 test).
pub(crate) fn test(bitmap: u16, flags: u16) -> bool {
    bitmap & flags == flags
}

/// True if the first operand equals any of the others (§15 je).
pub(crate) fn je(operands: &[u16]) -> bool {
    match operands.split_first() {
        Some((first, rest)) => rest.contains(first),
        None => false,
    }
}

pub(crate) fn jl(a: u16, b: u16) -> bool {
    (a as i16) < (b as i16)
}

pub(crate) fn jg(a: u16, b: u16) -> bool {
    (a as i16) > (b as i16)
}

#[cfg(test)]
#[path = "ops_math_tests.rs"]
mod tests;
