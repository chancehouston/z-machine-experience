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

// Opcodes that exist from version 4 or 5 (§14). The numbers are within their class.
pub const CALL_2S: u8 = 25;
pub const CALL_2N: u8 = 26;
pub const SET_COLOUR: u8 = 27;
pub const THROW: u8 = 28;
pub const CALL_1S: u8 = 8;
/// 1OP:15 is `call_1n` from version 5; in version 3 it is `not`.
pub const CALL_1N: u8 = 15;
/// 0OP:9 is `catch` from version 5; in version 3 it is `pop`.
pub const CATCH: u8 = 9;
pub const PIRACY: u8 = 15;
pub const CALL_VS: u8 = 0;
pub const AREAD: u8 = 4;
pub const CALL_VS2: u8 = 12;
pub const ERASE_WINDOW: u8 = 13;
pub const ERASE_LINE: u8 = 14;
pub const SET_CURSOR: u8 = 15;
pub const GET_CURSOR: u8 = 16;
pub const SET_TEXT_STYLE: u8 = 17;
pub const BUFFER_MODE: u8 = 18;
pub const READ_CHAR: u8 = 22;
pub const SCAN_TABLE: u8 = 23;
/// VAR:24 is `not` from version 5.
pub const NOT_VAR: u8 = 24;
pub const CALL_VN: u8 = 25;
pub const CALL_VN2: u8 = 26;
pub const TOKENISE: u8 = 27;
pub const ENCODE_TEXT: u8 = 28;
pub const COPY_TABLE: u8 = 29;
pub const PRINT_TABLE: u8 = 30;
pub const CHECK_ARG_COUNT: u8 = 31;

// Extended opcodes (§14.2), for use with [`ext`].
pub const EXT_SAVE: u8 = 0;
pub const EXT_RESTORE: u8 = 1;
pub const LOG_SHIFT: u8 = 2;
pub const ART_SHIFT: u8 = 3;
pub const SET_FONT: u8 = 4;
pub const SAVE_UNDO: u8 = 9;
pub const RESTORE_UNDO: u8 = 10;
pub const PRINT_UNICODE: u8 = 11;
pub const CHECK_UNICODE: u8 = 12;
pub const SET_TRUE_COLOUR: u8 = 13;

/// An instruction in extended form: 0xBE, the opcode number, a byte of operand
/// types, then the operands (§4.3.4). Version 5 and later.
pub fn ext(number: u8, args: &[Arg]) -> Vec<u8> {
    let mut out = vec![0xBE];
    out.extend(variable_form(number, args));
    out
}

/// `call_vs2` or `call_vn2`: a VAR instruction with two bytes of operand types and
/// up to eight operands (§4.4.3.1).
pub fn var2(number: u8, args: &[Arg]) -> Vec<u8> {
    assert!(
        number == CALL_VS2 || number == CALL_VN2,
        "only the two double-variable calls"
    );
    assert!(args.len() <= 8, "at most eight operands");
    let mut out = vec![0xE0 | number];
    for half in 0..2 {
        let mut types = 0u8;
        for i in 0..4 {
            let bits = args.get(half * 4 + i).map_or(0b11, |a| a.type_bits());
            types = types << 2 | bits;
        }
        out.push(types);
    }
    for a in args {
        a.push(&mut out);
    }
    out
}

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
