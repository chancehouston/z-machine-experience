//! Instruction decoding (§4), for version 3.

use crate::memory::{Memory, MemoryError};
use crate::version::Version;

/// One operand as encoded, before any variable is read (§4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operand {
    Large(u16),
    Small(u8),
    Variable(u8),
}

/// Where a branch goes (§4.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BranchTarget {
    /// Offset 0: return false from the current routine.
    ReturnFalse,
    /// Offset 1: return true from the current routine.
    ReturnTrue,
    Address(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Branch {
    /// Branch when the condition is true (bit 7 set) or when it is false.
    pub on_true: bool,
    pub target: BranchTarget,
}

/// The instructions of versions 3, 5 and 8 (§14).
///
/// The 68 of version 3 come first, under their version 3 names. The 33 names that
/// exist only from version 5 follow the marker line. Version 5 has 98 instructions:
/// these 101 names less `sread`, `pop` and `show_status`. `Call` is `call` in
/// version 3 and `call_vs` in version 5; `Not`, `Save` and `Restore` keep their
/// names but move to other opcode numbers.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Opcode {
    // 2OP
    Je, Jl, Jg, DecChk, IncChk, Jin, Test, Or, And, TestAttr, SetAttr, ClearAttr, Store,
    InsertObj, Loadw, Loadb, GetProp, GetPropAddr, GetNextProp, Add, Sub, Mul, Div, Mod,
    // 1OP
    Jz, GetSibling, GetChild, GetParent, GetPropLen, Inc, Dec, PrintAddr, RemoveObj, PrintObj,
    Ret, Jump, PrintPaddr, Load, Not,
    // 0OP
    Rtrue, Rfalse, Print, PrintRet, Nop, Save, Restore, Restart, RetPopped, Pop, Quit, NewLine,
    ShowStatus, Verify,
    // VAR
    Call, Storew, Storeb, PutProp, Sread, PrintChar, PrintNum, Random, Push, Pull, SplitWindow,
    SetWindow, OutputStream, InputStream, SoundEffect,
    // -- versions 5 and 8 only --
    Call2s, Call2n, SetColour, Throw,
    Call1s, Call1n,
    Catch, Piracy,
    Aread, CallVs2, EraseWindow, EraseLine, SetCursor, GetCursor, SetTextStyle, BufferMode,
    ReadChar, ScanTable, CallVn, CallVn2, Tokenise, EncodeText, CopyTable, PrintTable,
    CheckArgCount,
    LogShift, ArtShift, SetFont, SaveUndo, RestoreUndo, PrintUnicode, CheckUnicode,
    SetTrueColour,
}

/// What an extended opcode number means in versions 5 and 8 (§14.2).
enum Extended {
    Instruction(Opcode),
    /// Numbers above 28 "should be simply ignored" (§14.2.1).
    Ignored,
    /// Numbers that are instructions only in version 6.
    Unknown,
}

impl Opcode {
    fn two_op(number: u8) -> Option<Opcode> {
        use Opcode::*;
        Some(match number {
            1 => Je,
            2 => Jl,
            3 => Jg,
            4 => DecChk,
            5 => IncChk,
            6 => Jin,
            7 => Test,
            8 => Or,
            9 => And,
            10 => TestAttr,
            11 => SetAttr,
            12 => ClearAttr,
            13 => Store,
            14 => InsertObj,
            15 => Loadw,
            16 => Loadb,
            17 => GetProp,
            18 => GetPropAddr,
            19 => GetNextProp,
            20 => Add,
            21 => Sub,
            22 => Mul,
            23 => Div,
            24 => Mod,
            _ => return None,
        })
    }

    fn one_op(number: u8) -> Option<Opcode> {
        use Opcode::*;
        Some(match number {
            0 => Jz,
            1 => GetSibling,
            2 => GetChild,
            3 => GetParent,
            4 => GetPropLen,
            5 => Inc,
            6 => Dec,
            7 => PrintAddr,
            9 => RemoveObj,
            10 => PrintObj,
            11 => Ret,
            12 => Jump,
            13 => PrintPaddr,
            14 => Load,
            15 => Not,
            _ => return None,
        })
    }

    fn zero_op(number: u8) -> Option<Opcode> {
        use Opcode::*;
        Some(match number {
            0 => Rtrue,
            1 => Rfalse,
            2 => Print,
            3 => PrintRet,
            4 => Nop,
            5 => Save,
            6 => Restore,
            7 => Restart,
            8 => RetPopped,
            9 => Pop,
            10 => Quit,
            11 => NewLine,
            12 => ShowStatus,
            13 => Verify,
            _ => return None,
        })
    }

    fn var(number: u8) -> Option<Opcode> {
        use Opcode::*;
        Some(match number {
            0 => Call,
            1 => Storew,
            2 => Storeb,
            3 => PutProp,
            4 => Sread,
            5 => PrintChar,
            6 => PrintNum,
            7 => Random,
            8 => Push,
            9 => Pull,
            10 => SplitWindow,
            11 => SetWindow,
            19 => OutputStream,
            20 => InputStream,
            21 => SoundEffect,
            _ => return None,
        })
    }

    /// The 2OP instruction with this number in the given version.
    fn two_op_in(number: u8, v: Version) -> Option<Opcode> {
        use Opcode::*;
        match (v.is_v3(), number) {
            (false, 25) => Some(Call2s),
            (false, 26) => Some(Call2n),
            (false, 27) => Some(SetColour),
            (false, 28) => Some(Throw),
            _ => Self::two_op(number),
        }
    }

    /// The 1OP instruction with this number in the given version. From version 5,
    /// 1OP:15 is `call_1n`; `not` moves to VAR:24.
    fn one_op_in(number: u8, v: Version) -> Option<Opcode> {
        use Opcode::*;
        match (v.is_v3(), number) {
            (false, 8) => Some(Call1s),
            (false, 15) => Some(Call1n),
            _ => Self::one_op(number),
        }
    }

    /// The 0OP instruction with this number in the given version. From version 5,
    /// 0OP:5 and 0OP:6 are not instructions (`save` and `restore` are extended);
    /// 0OP:9 is `catch`; 0OP:12, `show_status`, is illegal but §15 says to treat it
    /// as doing nothing; 0OP:15 is `piracy`. 0OP:14 is the extended prefix and is
    /// handled before this is reached.
    fn zero_op_in(number: u8, v: Version) -> Option<Opcode> {
        use Opcode::*;
        match (v.is_v3(), number) {
            (false, 5 | 6) => None,
            (false, 9) => Some(Catch),
            (false, 12) => Some(Nop),
            (false, 15) => Some(Piracy),
            _ => Self::zero_op(number),
        }
    }

    /// The VAR instruction with this number in the given version.
    fn var_in(number: u8, v: Version) -> Option<Opcode> {
        use Opcode::*;
        if v.is_v3() {
            return Self::var(number);
        }
        Some(match number {
            4 => Aread,
            12 => CallVs2,
            13 => EraseWindow,
            14 => EraseLine,
            15 => SetCursor,
            16 => GetCursor,
            17 => SetTextStyle,
            18 => BufferMode,
            22 => ReadChar,
            23 => ScanTable,
            24 => Not,
            25 => CallVn,
            26 => CallVn2,
            27 => Tokenise,
            28 => EncodeText,
            29 => CopyTable,
            30 => PrintTable,
            31 => CheckArgCount,
            _ => return Self::var(number),
        })
    }

    fn extended(number: u8) -> Extended {
        use Opcode::*;
        Extended::Instruction(match number {
            0 => Save,
            1 => Restore,
            2 => LogShift,
            3 => ArtShift,
            4 => SetFont,
            9 => SaveUndo,
            10 => RestoreUndo,
            11 => PrintUnicode,
            12 => CheckUnicode,
            13 => SetTrueColour,
            29.. => return Extended::Ignored,
            _ => return Extended::Unknown,
        })
    }

    /// Whether the instruction is followed by a store variable (§4.6, §14). From
    /// version 5 `save` and `restore` store a result.
    pub fn stores(self, v: Version) -> bool {
        use Opcode::*;
        if matches!(self, Save | Restore) {
            return !v.is_v3();
        }
        matches!(
            self,
            Call2s
                | Call1s
                | Catch
                | Aread
                | CallVs2
                | ReadChar
                | ScanTable
                | LogShift
                | ArtShift
                | SetFont
                | SaveUndo
                | RestoreUndo
                | CheckUnicode
                |
            Or | And
                | Loadw
                | Loadb
                | GetProp
                | GetPropAddr
                | GetNextProp
                | Add
                | Sub
                | Mul
                | Div
                | Mod
                | GetSibling
                | GetChild
                | GetParent
                | GetPropLen
                | Load
                | Not
                | Call
                | Random
        )
    }

    /// Whether the instruction is followed by branch data (§4.7, §14). In version 3
    /// `save` and `restore` branch.
    pub fn branches(self, v: Version) -> bool {
        use Opcode::*;
        if matches!(self, Save | Restore) {
            return v.is_v3();
        }
        matches!(
            self,
            ScanTable
                | CheckArgCount
                | Piracy
                | Je
                | Jl
                | Jg
                | DecChk
                | IncChk
                | Jin
                | Test
                | TestAttr
                | Jz
                | GetSibling
                | GetChild
                | Save
                | Restore
                | Verify
        )
    }

    /// Whether the instruction is followed by a Z-string (§4.8).
    pub fn has_text(self) -> bool {
        matches!(self, Opcode::Print | Opcode::PrintRet)
    }
}

/// A decoded instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Instruction {
    pub address: u32,
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
    pub store: Option<u8>,
    pub branch: Option<Branch>,
    /// Address of the branch data, when the instruction branches. A version 3 save
    /// records this address as its program counter.
    pub branch_data: Option<u32>,
    /// Address of inline text, for `print` and `print_ret`.
    pub text: Option<u32>,
    /// Address of the instruction that follows.
    pub next: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecodeError {
    /// The opcode at this address is not a version 3 instruction.
    UnknownInstruction(u32),
    /// The instruction at this address runs past the end of the file.
    ReadOutsideFile(u32),
}

struct Cursor<'a> {
    memory: &'a Memory,
    start: u32,
    at: u32,
}

impl Cursor<'_> {
    fn byte(&mut self) -> Result<u8, DecodeError> {
        let b = self.memory.read_byte(self.at).map_err(|_| self.outside())?;
        self.at += 1;
        Ok(b)
    }

    fn word(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes([self.byte()?, self.byte()?]))
    }

    fn outside(&self) -> DecodeError {
        DecodeError::ReadOutsideFile(self.start)
    }

    /// Reads `type_bytes` bytes of 2-bit operand types and then the operands they
    /// describe, stopping at the first type that says "omitted" (§4.4.3).
    fn variable_operands(
        &mut self,
        operands: &mut Vec<Operand>,
        type_bytes: usize,
    ) -> Result<(), DecodeError> {
        let mut types = Vec::with_capacity(8);
        for _ in 0..type_bytes {
            let byte = self.byte()?;
            types.extend([6, 4, 2, 0].map(|shift| byte >> shift & 0b11));
        }
        for bits in types {
            if bits == 0b11 {
                break;
            }
            operands.push(self.operand(bits)?);
        }
        Ok(())
    }

    fn operand(&mut self, type_bits: u8) -> Result<Operand, DecodeError> {
        Ok(match type_bits {
            0b00 => Operand::Large(self.word()?),
            0b01 => Operand::Small(self.byte()?),
            _ => Operand::Variable(self.byte()?),
        })
    }
}

impl Cursor<'_> {
    /// Reads branch data (§4.7).
    fn branch(&mut self) -> Result<Branch, DecodeError> {
        let b = self.byte()?;
        let on_true = b & 0x80 != 0;
        let offset: i32 = if b & 0x40 != 0 {
            (b & 0x3F) as i32
        } else {
            // Signed 14 bits: sign-extend from bit 13.
            let raw = ((b & 0x3F) as u16) << 8 | self.byte()? as u16;
            ((raw << 2) as i16 >> 2) as i32
        };
        let target = match offset {
            0 => BranchTarget::ReturnFalse,
            1 => BranchTarget::ReturnTrue,
            _ => BranchTarget::Address((self.at as i64 + offset as i64 - 2) as u32),
        };
        Ok(Branch { on_true, target })
    }
}

/// Decodes the branch data at `address` on its own, returning it with the address
/// that follows. A version 3 save records the address of its `save` instruction's
/// branch data, and a restore resumes from there (Quetzal §5.8.1).
pub(crate) fn decode_branch(memory: &Memory, address: u32) -> Result<(Branch, u32), DecodeError> {
    let mut c = Cursor {
        memory,
        start: address,
        at: address,
    };
    let branch = c.branch()?;
    Ok((branch, c.at))
}

/// Decodes the instruction at `address`.
#[cfg(test)]
pub(crate) fn decode(memory: &Memory, address: u32) -> Result<Instruction, DecodeError> {
    decode_for(memory, address, Version::V3)
}

/// Decodes the instruction at `address` as an instruction of the given version.
pub(crate) fn decode_for(
    memory: &Memory,
    address: u32,
    v: Version,
) -> Result<Instruction, DecodeError> {
    let mut c = Cursor {
        memory,
        start: address,
        at: address,
    };
    let unknown = DecodeError::UnknownInstruction(address);
    let first = c.byte()?;
    let mut operands = Vec::with_capacity(8);

    let opcode = match first >> 6 {
        // Extended form (§4.3.4), from version 5: 0xBE, then the opcode number, then
        // operands as in variable form.
        _ if first == 0xBE && !v.is_v3() => {
            let number = c.byte()?;
            let opcode = match Opcode::extended(number) {
                Extended::Instruction(opcode) => opcode,
                Extended::Ignored => Opcode::Nop,
                Extended::Unknown => return Err(unknown),
            };
            c.variable_operands(&mut operands, 1)?;
            opcode
        }
        // Variable form (§4.3.3): bit 5 picks 2OP or VAR.
        0b11 => {
            let number = first & 0x1F;
            let opcode = if first & 0x20 == 0 {
                Opcode::two_op_in(number, v)
            } else {
                Opcode::var_in(number, v)
            }
            .ok_or(unknown)?;
            // §4.4.3.1: the two double-variable calls have two bytes of types.
            let type_bytes = if matches!(opcode, Opcode::CallVs2 | Opcode::CallVn2) { 2 } else { 1 };
            c.variable_operands(&mut operands, type_bytes)?;
            opcode
        }
        // Short form (§4.3.1): operand type in bits 4-5, 11 meaning none.
        0b10 => {
            let number = first & 0x0F;
            let bits = first >> 4 & 0b11;
            if bits == 0b11 {
                Opcode::zero_op_in(number, v).ok_or(unknown)?
            } else {
                let opcode = Opcode::one_op_in(number, v).ok_or(unknown)?;
                operands.push(c.operand(bits)?);
                opcode
            }
        }
        // Long form (§4.3.2): always 2OP, each operand a small constant or variable.
        _ => {
            let opcode = Opcode::two_op_in(first & 0x1F, v).ok_or(unknown)?;
            for bit in [0x40, 0x20] {
                let bits = if first & bit == 0 { 0b01 } else { 0b10 };
                operands.push(c.operand(bits)?);
            }
            opcode
        }
    };

    let store = if opcode.stores(v) {
        Some(c.byte()?)
    } else {
        None
    };

    let mut branch = None;
    let mut branch_data = None;
    if opcode.branches(v) {
        branch_data = Some(c.at);
        branch = Some(c.branch()?);
    }

    let mut text = None;
    if opcode.has_text() {
        text = Some(c.at);
        // §3.2: the string ends with the first word whose top bit is set.
        while c.word()? & 0x8000 == 0 {}
    }

    Ok(Instruction {
        address,
        opcode,
        operands,
        store,
        branch,
        branch_data,
        text,
        next: c.at,
    })
}

impl From<MemoryError> for DecodeError {
    fn from(e: MemoryError) -> Self {
        match e {
            MemoryError::ReadOutsideFile(a) | MemoryError::WriteOutsideDynamicMemory(a) => {
                DecodeError::ReadOutsideFile(a)
            }
        }
    }
}

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "decode5_tests.rs"]
mod tests5;
