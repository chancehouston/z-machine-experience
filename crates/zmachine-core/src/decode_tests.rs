use super::*;
use zmachine_testkit::op::{self, Arg};

const AT: u32 = 0x40;

/// A memory image with `code` at `AT` and nothing else.
fn mem(code: &[u8]) -> Memory {
    let mut bytes = vec![0u8; AT as usize];
    bytes.extend_from_slice(code);
    Memory::new(bytes, AT as usize)
}

fn dec(code: &[u8]) -> Instruction {
    decode(&mem(code), AT).expect("decodes")
}

// §4.3.2, §4.4.2: long form is always 2OP; bit 6 types the first operand and bit 5
// the second (0 small constant, 1 variable); the low five bits are the number.
#[test]
fn long_form_operand_types() {
    let i = dec(&[0x14, 2, 3, 0x00]);
    assert_eq!(i.opcode, Opcode::Add);
    assert_eq!(i.operands, vec![Operand::Small(2), Operand::Small(3)]);
    assert_eq!(i.store, Some(0));
    assert_eq!(i.branch, None);
    assert_eq!(i.next, AT + 4);
    assert_eq!(i.address, AT);

    assert_eq!(dec(&[0x54, 1, 3, 0]).operands, vec![Operand::Variable(1), Operand::Small(3)]);
    assert_eq!(dec(&[0x34, 2, 16, 0]).operands, vec![Operand::Small(2), Operand::Variable(16)]);
    assert_eq!(dec(&[0x74, 1, 2, 0]).operands, vec![Operand::Variable(1), Operand::Variable(2)]);
}

// §4.3.3, §4.4.3: top bits 11 is variable form; bit 5 clear means 2OP. A byte of
// four 2-bit types follows: 00 large, 01 small, 10 variable, 11 omitted.
#[test]
fn variable_form_2op_with_a_large_constant() {
    let i = dec(&[0xD4, 0b00_01_11_11, 0x12, 0x34, 3, 0x05]);
    assert_eq!(i.opcode, Opcode::Add);
    assert_eq!(i.operands, vec![Operand::Large(0x1234), Operand::Small(3)]);
    assert_eq!(i.store, Some(5));
    assert_eq!(i.next, AT + 6);
}

// §4.3.3 and §15 je: `je` in variable form may carry up to four operands.
#[test]
fn je_with_four_operands() {
    let i = dec(&[0xC1, 0b10_01_01_01, 1, 2, 3, 4, 0xC5]);
    assert_eq!(i.opcode, Opcode::Je);
    assert_eq!(
        i.operands,
        vec![Operand::Variable(1), Operand::Small(2), Operand::Small(3), Operand::Small(4)]
    );
    assert_eq!(i.store, None);
    assert_eq!(i.next, AT + 7);
}

// §4.4.3: operands stop at the first omitted type; later type bits are ignored.
#[test]
fn operand_types_stop_at_the_first_omitted() {
    let i = dec(&[0xE1, 0b01_11_01_01, 9]);
    assert_eq!(i.opcode, Opcode::Storew);
    assert_eq!(i.operands, vec![Operand::Small(9)]);
    assert_eq!(i.next, AT + 3);
}

// §4.3.1: short form, top bits 10. Bits 4-5 type the operand; 11 means no operand.
#[test]
fn short_form_1op_and_0op() {
    let i = dec(&[0x80, 0x01, 0x02, 0xC0]);
    assert_eq!(i.opcode, Opcode::Jz);
    assert_eq!(i.operands, vec![Operand::Large(0x0102)]);
    assert_eq!(dec(&[0x90, 7, 0xC0]).operands, vec![Operand::Small(7)]);
    assert_eq!(dec(&[0xA0, 0, 0xC0]).operands, vec![Operand::Variable(0)]);

    let i = dec(&[0xB0]);
    assert_eq!(i.opcode, Opcode::Rtrue);
    assert!(i.operands.is_empty());
    assert_eq!(i.next, AT + 1);
}

// §4.3.3: top bits 111 is VAR.
#[test]
fn variable_form_var() {
    let i = dec(&[0xE0, 0b00_11_11_11, 0x04, 0x00, 0x10]);
    assert_eq!(i.opcode, Opcode::Call);
    assert_eq!(i.operands, vec![Operand::Large(0x0400)]);
    assert_eq!(i.store, Some(0x10));
    assert_eq!(i.next, AT + 5);

    let i = dec(&[0xE6, 0xFF]);
    assert_eq!(i.opcode, Opcode::PrintNum);
    assert!(i.operands.is_empty());
}

// §4.6 then §4.7: an instruction that both stores and branches has the store byte
// first. `get_sibling` is one (§14).
#[test]
fn store_byte_comes_before_branch_data() {
    let i = dec(&[0x91, 5, 0x07, 0xC0]);
    assert_eq!(i.opcode, Opcode::GetSibling);
    assert_eq!(i.store, Some(7));
    assert_eq!(i.branch, Some(Branch { on_true: true, target: BranchTarget::ReturnFalse }));
    assert_eq!(i.branch_data, Some(AT + 3));
    assert_eq!(i.next, AT + 4);
}

// §4.7: bit 7 of the first branch byte is the sense; bit 6 set means the offset is
// the low six bits. §4.7.1: offsets 0 and 1 return false and true.
#[test]
fn one_byte_branches() {
    let b = |byte: u8| dec(&[0x90, 0, byte]).branch.unwrap();
    assert_eq!(b(0xC0), Branch { on_true: true, target: BranchTarget::ReturnFalse });
    assert_eq!(b(0xC1), Branch { on_true: true, target: BranchTarget::ReturnTrue });
    assert_eq!(b(0x40), Branch { on_true: false, target: BranchTarget::ReturnFalse });
    assert_eq!(b(0x41), Branch { on_true: false, target: BranchTarget::ReturnTrue });
    // §4.7.2: otherwise the target is the address after the branch data + offset - 2.
    // The instruction is 3 bytes, so "after" is AT + 3.
    assert_eq!(b(0xC2), Branch { on_true: true, target: BranchTarget::Address(AT + 3) });
    assert_eq!(b(0x7F), Branch { on_true: false, target: BranchTarget::Address(AT + 3 + 61) });
}

// §4.7: bit 6 clear means a signed 14-bit offset in the low six bits and next byte.
#[test]
fn two_byte_branches_are_signed_14_bit() {
    let b = |hi: u8, lo: u8| {
        let i = dec(&[0x90, 0, hi, lo]);
        assert_eq!(i.next, AT + 4);
        i.branch.unwrap()
    };
    assert_eq!(b(0x80, 0x40), Branch { on_true: true, target: BranchTarget::Address(AT + 4 + 62) });
    // -1: lands one byte after the start of this instruction.
    assert_eq!(b(0x3F, 0xFF), Branch { on_true: false, target: BranchTarget::Address(AT + 1) });
    // The most negative offset, -8192, from an instruction far enough up.
    let mut bytes = vec![0u8; 0x3000];
    bytes[0x2800..0x2804].copy_from_slice(&[0x90, 0, 0xA0, 0x00]);
    let i = decode(&Memory::new(bytes, 0x40), 0x2800).unwrap();
    assert_eq!(i.branch.unwrap().target, BranchTarget::Address(0x2804 - 8192 - 2));
    // §4.7.1 applies to the two-byte form as well.
    assert_eq!(b(0x80, 0x00).target, BranchTarget::ReturnFalse);
    assert_eq!(b(0x80, 0x01).target, BranchTarget::ReturnTrue);
}

// §4.8: `print` and `print_ret` are followed by a Z-string; the next instruction
// starts after the word with the end bit (§3.2).
#[test]
fn inline_text_is_skipped_to_its_end_bit() {
    let hello = [0x35, 0x51, 0xC6, 0x85];
    let mut code = vec![0xB2];
    code.extend(hello);
    code.push(0xB0);
    let i = dec(&code);
    assert_eq!(i.opcode, Opcode::Print);
    assert_eq!(i.text, Some(AT + 1));
    assert_eq!(i.next, AT + 5);

    code[0] = 0xB3;
    let i = dec(&code);
    assert_eq!(i.opcode, Opcode::PrintRet);
    assert_eq!(i.text, Some(AT + 1));
    assert_eq!(i.next, AT + 5);
}

// A Z-string with no end bit before the end of the file cannot be skipped.
#[test]
fn inline_text_running_off_the_file_is_an_error() {
    let m = mem(&[0xB2, 0x35, 0x51, 0x46, 0x85]);
    assert!(matches!(decode(&m, AT), Err(DecodeError::ReadOutsideFile(_))));
}

// An instruction cut off by the end of the file is an error, never a panic.
#[test]
fn a_truncated_instruction_is_an_error() {
    for code in [&[0x14u8, 2][..], &[0x14, 2, 3], &[0xE0], &[0xE0, 0x3F, 4], &[0x90, 0], &[0x90, 0, 0x80]] {
        assert!(
            matches!(decode(&mem(code), AT), Err(DecodeError::ReadOutsideFile(_))),
            "{code:02x?}"
        );
    }
    assert!(matches!(decode(&mem(&[]), AT), Err(DecodeError::ReadOutsideFile(_))));
}

// §14: opcode numbers with no version 3 instruction. 2OP:0 and 2OP:25-31; 1OP:8
// (call_1s is version 4); 0OP:14 (extended, version 5) and 0OP:15 (piracy,
// version 5); VAR:12-18 and VAR:22-31.
#[test]
fn numbers_not_defined_for_version_3_are_unknown() {
    let mut unknown: Vec<Vec<u8>> = vec![
        vec![0x00, 0, 0],
        vec![0xC0, 0x5F, 0, 0],
        vec![0x88, 0, 0],
        vec![0x98, 0],
        vec![0xBE],
        vec![0xBF],
    ];
    for n in 25..=31u8 {
        unknown.push(vec![n, 0, 0, 0]);
        unknown.push(vec![0xC0 | n, 0x5F, 0, 0, 0]);
    }
    for n in (12..=18u8).chain(22..=31) {
        unknown.push(vec![0xE0 | n, 0xFF, 0, 0]);
    }
    for code in unknown {
        assert_eq!(
            decode(&mem(&code), AT),
            Err(DecodeError::UnknownInstruction(AT)),
            "{code:02x?}"
        );
    }
}

// §14: the complete version 3 instruction set, with which instructions store a
// result and which branch. Transcribed from the opcode tables.
#[test]
fn every_version_3_instruction_decodes_with_the_right_store_and_branch() {
    use Opcode::*;
    let s = Arg::S(1);
    // (encoded opcode and operands, opcode, stores, branches)
    let two = |n: u8| op::op2(n, s, s);
    let one = |n: u8| op::op1(n, s);
    let zero = op::op0;
    let var = |n: u8| op::var(n, &[s]);
    let table: Vec<(Vec<u8>, Opcode, bool, bool)> = vec![
        (two(1), Je, false, true),
        (two(2), Jl, false, true),
        (two(3), Jg, false, true),
        (two(4), DecChk, false, true),
        (two(5), IncChk, false, true),
        (two(6), Jin, false, true),
        (two(7), Test, false, true),
        (two(8), Or, true, false),
        (two(9), And, true, false),
        (two(10), TestAttr, false, true),
        (two(11), SetAttr, false, false),
        (two(12), ClearAttr, false, false),
        (two(13), Store, false, false),
        (two(14), InsertObj, false, false),
        (two(15), Loadw, true, false),
        (two(16), Loadb, true, false),
        (two(17), GetProp, true, false),
        (two(18), GetPropAddr, true, false),
        (two(19), GetNextProp, true, false),
        (two(20), Add, true, false),
        (two(21), Sub, true, false),
        (two(22), Mul, true, false),
        (two(23), Div, true, false),
        (two(24), Mod, true, false),
        (one(0), Jz, false, true),
        (one(1), GetSibling, true, true),
        (one(2), GetChild, true, true),
        (one(3), GetParent, true, false),
        (one(4), GetPropLen, true, false),
        (one(5), Inc, false, false),
        (one(6), Dec, false, false),
        (one(7), PrintAddr, false, false),
        (one(9), RemoveObj, false, false),
        (one(10), PrintObj, false, false),
        (one(11), Ret, false, false),
        (one(12), Jump, false, false),
        (one(13), PrintPaddr, false, false),
        (one(14), Load, true, false),
        (one(15), Not, true, false),
        (zero(0), Rtrue, false, false),
        (zero(1), Rfalse, false, false),
        (zero(4), Nop, false, false),
        (zero(5), Save, false, true),
        (zero(6), Restore, false, true),
        (zero(7), Restart, false, false),
        (zero(8), RetPopped, false, false),
        (zero(9), Pop, false, false),
        (zero(10), Quit, false, false),
        (zero(11), NewLine, false, false),
        (zero(12), ShowStatus, false, false),
        (zero(13), Verify, false, true),
        (var(0), Call, true, false),
        (var(1), Storew, false, false),
        (var(2), Storeb, false, false),
        (var(3), PutProp, false, false),
        (var(4), Sread, false, false),
        (var(5), PrintChar, false, false),
        (var(6), PrintNum, false, false),
        (var(7), Random, true, false),
        (var(8), Push, false, false),
        (var(9), Pull, false, false),
        (var(10), SplitWindow, false, false),
        (var(11), SetWindow, false, false),
        (var(19), OutputStream, false, false),
        (var(20), InputStream, false, false),
        (var(21), SoundEffect, false, false),
    ];
    // `print` and `print_ret` are covered by the inline-text test; with them the
    // table has all 68 instructions of version 3.
    assert_eq!(table.len() + 2, 68);

    for (mut code, opcode, stores, branches) in table {
        let operand_bytes = code.len();
        // The store variable comes first when there is one, then one-byte branch
        // data; the zeros after are padding the decoder must not consume.
        if stores {
            code.push(0x21);
        }
        code.extend([0xC0, 0, 0]);
        let i = dec(&code);
        assert_eq!(i.opcode, opcode);
        assert_eq!(i.store.is_some(), stores, "{opcode:?} store");
        assert_eq!(i.branch.is_some(), branches, "{opcode:?} branch");
        assert_eq!(i.text, None, "{opcode:?} text");
        let extra = stores as usize + branches as usize;
        assert_eq!(i.next as usize, AT as usize + operand_bytes + extra, "{opcode:?} length");
        if stores {
            assert_eq!(i.store, Some(0x21), "{opcode:?} store variable");
        }
    }
}

// §4.7: `jump` is not a branch instruction; its operand is a signed offset that the
// executor applies (§15 jump). The decoder must not read branch data for it.
#[test]
fn jump_has_an_operand_and_no_branch_data() {
    let i = dec(&[0x8C, 0xFF, 0xFE]);
    assert_eq!(i.opcode, Opcode::Jump);
    assert_eq!(i.operands, vec![Operand::Large(0xFFFE)]);
    assert_eq!(i.branch, None);
    assert_eq!(i.next, AT + 3);
}
