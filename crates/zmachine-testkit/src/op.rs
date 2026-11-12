//! Instruction encoding helpers (§4).
//!
//! These produce the opcode and operand bytes only. A store variable, branch data or
//! inline text is appended by the caller, in that order, using [`store`], [`branch`]
//! and [`crate::zencode`].

/// One operand: a large constant, a small constant, or a variable number (§4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg {
    L(u16),
    S(u8),
    V(u8),
}

impl Arg {
    fn type_bits(self) -> u8 {
        match self {
            Arg::L(_) => 0b00,
            Arg::S(_) => 0b01,
            Arg::V(_) => 0b10,
        }
    }

    fn push(self, out: &mut Vec<u8>) {
        match self {
            Arg::L(v) => out.extend(v.to_be_bytes()),
            Arg::S(v) | Arg::V(v) => out.push(v),
        }
    }
}

// 2OP opcode numbers (§14).
pub const JE: u8 = 1;
pub const JL: u8 = 2;
pub const JG: u8 = 3;
pub const DEC_CHK: u8 = 4;
pub const INC_CHK: u8 = 5;
pub const JIN: u8 = 6;
pub const TEST: u8 = 7;
pub const OR: u8 = 8;
pub const AND: u8 = 9;
pub const TEST_ATTR: u8 = 10;
pub const SET_ATTR: u8 = 11;
pub const CLEAR_ATTR: u8 = 12;
pub const STORE: u8 = 13;
pub const INSERT_OBJ: u8 = 14;
pub const LOADW: u8 = 15;
pub const LOADB: u8 = 16;
pub const GET_PROP: u8 = 17;
pub const GET_PROP_ADDR: u8 = 18;
pub const GET_NEXT_PROP: u8 = 19;
pub const ADD: u8 = 20;
pub const SUB: u8 = 21;
pub const MUL: u8 = 22;
pub const DIV: u8 = 23;
pub const MOD: u8 = 24;

// 1OP opcode numbers.
pub const JZ: u8 = 0;
pub const GET_SIBLING: u8 = 1;
pub const GET_CHILD: u8 = 2;
pub const GET_PARENT: u8 = 3;
pub const GET_PROP_LEN: u8 = 4;
pub const INC: u8 = 5;
pub const DEC: u8 = 6;
pub const PRINT_ADDR: u8 = 7;
pub const REMOVE_OBJ: u8 = 9;
pub const PRINT_OBJ: u8 = 10;
pub const RET: u8 = 11;
pub const JUMP: u8 = 12;
pub const PRINT_PADDR: u8 = 13;
pub const LOAD: u8 = 14;
pub const NOT: u8 = 15;

// 0OP opcode numbers.
pub const RTRUE: u8 = 0;
pub const RFALSE: u8 = 1;
pub const PRINT: u8 = 2;
pub const PRINT_RET: u8 = 3;
pub const NOP: u8 = 4;
pub const SAVE: u8 = 5;
pub const RESTORE: u8 = 6;
pub const RESTART: u8 = 7;
pub const RET_POPPED: u8 = 8;
pub const POP: u8 = 9;
pub const QUIT: u8 = 10;
pub const NEW_LINE: u8 = 11;
pub const SHOW_STATUS: u8 = 12;
pub const VERIFY: u8 = 13;

// VAR opcode numbers.
pub const CALL: u8 = 0;
pub const STOREW: u8 = 1;
pub const STOREB: u8 = 2;
pub const PUT_PROP: u8 = 3;
pub const SREAD: u8 = 4;
pub const PRINT_CHAR: u8 = 5;
pub const PRINT_NUM: u8 = 6;
pub const RANDOM: u8 = 7;
pub const PUSH: u8 = 8;
pub const PULL: u8 = 9;
pub const SPLIT_WINDOW: u8 = 10;
pub const SET_WINDOW: u8 = 11;
pub const OUTPUT_STREAM: u8 = 19;
pub const INPUT_STREAM: u8 = 20;
pub const SOUND_EFFECT: u8 = 21;

fn variable_form(first: u8, args: &[Arg]) -> Vec<u8> {
    assert!(args.len() <= 4, "at most four operands in version 3");
    let mut types = 0u8;
    for i in 0..4 {
        let bits = args.get(i).map_or(0b11, |a| a.type_bits());
        types = types << 2 | bits;
    }
    let mut out = vec![first, types];
    for a in args {
        a.push(&mut out);
    }
    out
}

/// A two-operand instruction, in long form when both operands allow it (§4.3.2),
/// otherwise in variable form.
pub fn op2(number: u8, a: Arg, b: Arg) -> Vec<u8> {
    assert!(number < 32);
    if matches!(a, Arg::L(_)) || matches!(b, Arg::L(_)) {
        return op2_var(number, &[a, b]);
    }
    let mut first = number;
    if matches!(a, Arg::V(_)) {
        first |= 0x40;
    }
    if matches!(b, Arg::V(_)) {
        first |= 0x20;
    }
    let mut out = vec![first];
    a.push(&mut out);
    b.push(&mut out);
    out
}

/// A 2OP instruction in variable form, with any number of operands (§4.3.3).
pub fn op2_var(number: u8, args: &[Arg]) -> Vec<u8> {
    assert!(number < 32);
    variable_form(0xC0 | number, args)
}

/// A one-operand instruction in short form (§4.3.1).
pub fn op1(number: u8, a: Arg) -> Vec<u8> {
    assert!(number < 16);
    let mut out = vec![0x80 | a.type_bits() << 4 | number];
    a.push(&mut out);
    out
}

/// A zero-operand instruction in short form (§4.3.1).
pub fn op0(number: u8) -> Vec<u8> {
    assert!(number < 16);
    vec![0xB0 | number]
}

/// A VAR instruction (§4.3.3).
pub fn var(number: u8, args: &[Arg]) -> Vec<u8> {
    assert!(number < 32);
    variable_form(0xE0 | number, args)
}

/// The store-variable byte that follows a storing instruction (§4.6).
pub fn store(variable: u8) -> Vec<u8> {
    vec![variable]
}

/// Branch data with the given raw offset: one byte when the offset is 0 to 63, two
/// otherwise (§4.7). Offsets 0 and 1 mean "return false" and "return true"; any
/// other offset lands at the address after the branch data, plus offset, minus 2.
pub fn branch(on_true: bool, offset: i16) -> Vec<u8> {
    if (0..=63).contains(&offset) {
        let sense = if on_true { 0x80 } else { 0 };
        vec![sense | 0x40 | offset as u8]
    } else {
        branch_long(on_true, offset)
    }
}

/// Branch data in the two-byte form, whatever the offset (§4.7).
pub fn branch_long(on_true: bool, offset: i16) -> Vec<u8> {
    assert!(
        (-8192..=8191).contains(&offset),
        "offset must fit in 14 bits"
    );
    let raw = offset as u16 & 0x3FFF;
    let sense = if on_true { 0x80 } else { 0 };
    vec![sense | (raw >> 8) as u8, raw as u8]
}
