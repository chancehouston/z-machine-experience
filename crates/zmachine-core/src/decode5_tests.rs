//! Instruction decoding for versions 5 and 8 (feature 002, T009).

use super::*;
use crate::version::Version;
use zmachine_testkit::op::{self, Arg};

const AT: u32 = 0x40;

fn mem(code: &[u8]) -> Memory {
    let mut bytes = vec![0u8; AT as usize];
    bytes.extend_from_slice(code);
    Memory::new(bytes, AT as usize)
}

fn dec(code: &[u8], v: Version) -> Instruction {
    decode_for(&mem(code), AT, v).unwrap_or_else(|e| panic!("{code:02x?} in {v:?}: {e:?}"))
}

fn dec5(code: &[u8]) -> Instruction {
    dec(code, Version::V5)
}

// §14: the complete version 5 instruction set, with which instructions store and
// which branch. Transcribed from the opcode tables. Version 8 is the same.
#[test]
fn every_version_5_instruction_decodes_with_the_right_store_and_branch() {
    use Opcode::*;
    let s = Arg::S(1);
    let two = |n: u8| op::op2(n, s, s);
    let one = |n: u8| op::op1(n, s);
    let zero = op::op0;
    let var = |n: u8| op::var(n, &[s]);
    let ext = |n: u8| op::ext(n, &[s]);
    // (encoded opcode and operands, opcode, stores, branches)
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
        (two(25), Call2s, true, false),
        (two(26), Call2n, false, false),
        (two(27), SetColour, false, false),
        (two(28), Throw, false, false),
        (one(0), Jz, false, true),
        (one(1), GetSibling, true, true),
        (one(2), GetChild, true, true),
        (one(3), GetParent, true, false),
        (one(4), GetPropLen, true, false),
        (one(5), Inc, false, false),
        (one(6), Dec, false, false),
        (one(7), PrintAddr, false, false),
        (one(8), Call1s, true, false),
        (one(9), RemoveObj, false, false),
        (one(10), PrintObj, false, false),
        (one(11), Ret, false, false),
        (one(12), Jump, false, false),
        (one(13), PrintPaddr, false, false),
        (one(14), Load, true, false),
        (one(15), Call1n, false, false),
        (zero(0), Rtrue, false, false),
        (zero(1), Rfalse, false, false),
        (zero(4), Nop, false, false),
        (zero(7), Restart, false, false),
        (zero(8), RetPopped, false, false),
        (zero(9), Catch, true, false),
        (zero(10), Quit, false, false),
        (zero(11), NewLine, false, false),
        (zero(13), Verify, false, true),
        (zero(15), Piracy, false, true),
        (var(0), Call, true, false),
        (var(1), Storew, false, false),
        (var(2), Storeb, false, false),
        (var(3), PutProp, false, false),
        (var(4), Aread, true, false),
        (var(5), PrintChar, false, false),
        (var(6), PrintNum, false, false),
        (var(7), Random, true, false),
        (var(8), Push, false, false),
        (var(9), Pull, false, false),
        (var(10), SplitWindow, false, false),
        (var(11), SetWindow, false, false),
        (op::var2(12, &[s]), CallVs2, true, false),
        (var(13), EraseWindow, false, false),
        (var(14), EraseLine, false, false),
        (var(15), SetCursor, false, false),
        (var(16), GetCursor, false, false),
        (var(17), SetTextStyle, false, false),
        (var(18), BufferMode, false, false),
        (var(19), OutputStream, false, false),
        (var(20), InputStream, false, false),
        (var(21), SoundEffect, false, false),
        (var(22), ReadChar, true, false),
        (var(23), ScanTable, true, true),
        (var(24), Not, true, false),
        (var(25), CallVn, false, false),
        (op::var2(26, &[s]), CallVn2, false, false),
        (var(27), Tokenise, false, false),
        (var(28), EncodeText, false, false),
        (var(29), CopyTable, false, false),
        (var(30), PrintTable, false, false),
        (var(31), CheckArgCount, false, true),
        (ext(0), Save, true, false),
        (ext(1), Restore, true, false),
        (ext(2), LogShift, true, false),
        (ext(3), ArtShift, true, false),
        (ext(4), SetFont, true, false),
        (ext(9), SaveUndo, true, false),
        (ext(10), RestoreUndo, true, false),
        (ext(11), PrintUnicode, false, false),
        (ext(12), CheckUnicode, true, false),
        (ext(13), SetTrueColour, false, false),
    ];
    // `print` and `print_ret` are covered by the inline-text test; with them the
    // table has all 98 instructions of version 5.
    assert_eq!(table.len() + 2, 98);

    for v in [Version::V5, Version::V8] {
        for (code, opcode, stores, branches) in &table {
            let mut code = code.clone();
            let operand_bytes = code.len();
            if *stores {
                code.push(0x21);
            }
            code.extend([0xC0, 0, 0]);
            let i = dec(&code, v);
            assert_eq!(i.opcode, *opcode, "{v:?}");
            assert_eq!(i.store.is_some(), *stores, "{opcode:?} store in {v:?}");
            assert_eq!(i.branch.is_some(), *branches, "{opcode:?} branch in {v:?}");
            let extra = *stores as usize + *branches as usize;
            assert_eq!(
                i.next as usize,
                AT as usize + operand_bytes + extra,
                "{opcode:?} length"
            );
            if *stores {
                assert_eq!(i.store, Some(0x21), "{opcode:?} store variable");
            }
        }
    }
}

// §4.8: `print` and `print_ret` carry inline text in version 5 as in version 3.
#[test]
fn inline_text_in_version_5() {
    let code = [0xB2, 0x35, 0x51, 0xC6, 0x85, 0xB0];
    let i = dec5(&code);
    assert_eq!(
        (i.opcode, i.text, i.next),
        (Opcode::Print, Some(AT + 1), AT + 5)
    );
}

// §14: the opcode numbers whose meaning changes between version 3 and version 5
// (data-model.md). Each is decoded both ways.
#[test]
fn opcode_numbers_that_change_meaning() {
    use Opcode::*;
    let v3 = |code: &[u8]| decode_for(&mem(code), AT, Version::V3);
    let s = Arg::S(1);

    // 1OP:15 is `not` (storing) in version 3 and `call_1n` (not storing) in 5.
    let code = [op::op1(15, s), vec![0x10]].concat();
    assert_eq!(v3(&code).unwrap().opcode, Not);
    assert_eq!(v3(&code).unwrap().store, Some(0x10));
    assert_eq!((dec5(&code).opcode, dec5(&code).store), (Call1n, None));

    // 0OP:5 and 0OP:6 are `save` and `restore`, branching, in version 3 only.
    for (number, opcode) in [(5u8, Save), (6, Restore)] {
        let code = [op::op0(number), vec![0xC0]].concat();
        let i = v3(&code).unwrap();
        assert_eq!(
            (i.opcode, i.store, i.branch.is_some()),
            (opcode, None, true)
        );
        assert_eq!(
            decode_for(&mem(&code), AT, Version::V5),
            Err(DecodeError::UnknownInstruction(AT))
        );
    }

    // 0OP:9 is `pop` in version 3 and `catch` (storing) in 5.
    let code = [op::op0(9), vec![0x10]].concat();
    assert_eq!(
        (v3(&code).unwrap().opcode, v3(&code).unwrap().store),
        (Pop, None)
    );
    assert_eq!((dec5(&code).opcode, dec5(&code).store), (Catch, Some(0x10)));

    // 0OP:12 is `show_status` in version 3; in 5 it is illegal, and §15 says an
    // interpreter should treat it as doing nothing.
    assert_eq!(v3(&op::op0(12)).unwrap().opcode, ShowStatus);
    assert_eq!(dec5(&op::op0(12)).opcode, Nop);

    // 0OP:14 and 0OP:15 are not instructions in version 3.
    for number in [14u8, 15] {
        assert_eq!(
            v3(&op::op0(number)),
            Err(DecodeError::UnknownInstruction(AT))
        );
    }

    // VAR:4 is `sread` (no result) in version 3 and `aread` (storing) in 5.
    let code = [op::var(4, &[s, s]), vec![0x10]].concat();
    assert_eq!(
        (v3(&code).unwrap().opcode, v3(&code).unwrap().store),
        (Sread, None)
    );
    assert_eq!((dec5(&code).opcode, dec5(&code).store), (Aread, Some(0x10)));

    // 2OP:25-28, 1OP:8, VAR:12-18 and VAR:22-31 exist only from version 5.
    for code in [
        op::op2(25, s, s),
        op::op1(8, s),
        op::var(13, &[s]),
        op::var(31, &[s]),
    ] {
        assert_eq!(
            v3(&code),
            Err(DecodeError::UnknownInstruction(AT)),
            "{code:02x?}"
        );
    }
    // The extended prefix is an unknown instruction in version 3.
    assert_eq!(
        v3(&op::ext(9, &[])),
        Err(DecodeError::UnknownInstruction(AT))
    );
}

// §4.3.4: the extended form's operands are laid out as in variable form.
#[test]
fn extended_form_operands() {
    let i = dec5(
        &[
            op::ext(op::LOG_SHIFT, &[Arg::L(0x1234), Arg::V(3)]),
            vec![0x05],
        ]
        .concat(),
    );
    assert_eq!(i.opcode, Opcode::LogShift);
    assert_eq!(
        i.operands,
        vec![Operand::Large(0x1234), Operand::Variable(3)]
    );
    assert_eq!(i.store, Some(5));
    assert_eq!(i.next, AT + 7);
}

// §14.2.1: "EXT:29 to EXT:255 are unspecified ... should be simply ignored." The
// operands are still stepped over. Numbers that are instructions only in version 6
// are unknown.
#[test]
fn unspecified_extended_opcodes_are_ignored_and_version_6_ones_are_unknown() {
    for number in [29u8, 100, 255] {
        let i = dec5(&op::ext(number, &[Arg::L(1), Arg::S(2)]));
        assert_eq!(i.opcode, Opcode::Nop, "EXT:{number}");
        assert_eq!((i.store, i.branch), (None, None));
        assert_eq!(i.next, AT + 6, "operands are skipped");
    }
    for number in [5u8, 6, 7, 8, 14, 16, 28] {
        let code = op::ext(number, &[]);
        assert_eq!(
            decode_for(&mem(&code), AT, Version::V5),
            Err(DecodeError::UnknownInstruction(AT)),
            "EXT:{number}"
        );
    }
}

// §4.4.3.1: `call_vs2` and `call_vn2` have two bytes of operand types and so up to
// eight operands: a routine and seven arguments.
#[test]
fn double_variable_calls_take_eight_operands() {
    let args: Vec<Arg> = (1..=8).map(Arg::S).collect();
    let i = dec5(&[op::var2(op::CALL_VS2, &args), vec![0x00]].concat());
    assert_eq!(i.opcode, Opcode::CallVs2);
    assert_eq!(i.operands.len(), 8);
    assert_eq!(i.operands[7], Operand::Small(8));
    assert_eq!(i.store, Some(0));

    // Fewer than eight: the list ends at the first omitted type, in either byte.
    let i = dec5(&op::var2(
        op::CALL_VN2,
        &[Arg::L(0x0400), Arg::S(1), Arg::S(2), Arg::S(3), Arg::V(4)],
    ));
    assert_eq!(i.operands.len(), 5);
    assert_eq!(i.operands[4], Operand::Variable(4));
    let i = dec5(&op::var2(op::CALL_VN2, &[Arg::S(9)]));
    assert_eq!(i.operands, vec![Operand::Small(9)]);
    assert_eq!(
        i.next,
        AT + 4,
        "both type bytes are read even when the second is unused"
    );
}

// A version 5 instruction cut off by the end of the file is an error, not a panic.
#[test]
fn truncated_version_5_instructions_are_errors() {
    for code in [
        &[0xBEu8][..],
        &[0xBE, 9],
        &[0xEC, 0x55],
        &[0xEC, 0x55, 0x55, 1, 2],
    ] {
        assert!(
            matches!(
                decode_for(&mem(code), AT, Version::V5),
                Err(DecodeError::ReadOutsideFile(_))
            ),
            "{code:02x?}"
        );
    }
}
