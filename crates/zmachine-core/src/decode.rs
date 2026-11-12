//! Instruction decoding (§4), for version 3.

use crate::memory::{Memory, MemoryError};

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

/// The 68 instructions of version 3 (§14).
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

    /// Whether the instruction is followed by a store variable (§4.6, §14).
    pub fn stores(self) -> bool {
        use Opcode::*;
        matches!(
            self,
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
    pub fn branches(self) -> bool {
        use Opcode::*;
        matches!(
            self,
            Je | Jl
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
pub(crate) fn decode(memory: &Memory, address: u32) -> Result<Instruction, DecodeError> {
    let mut c = Cursor {
        memory,
        start: address,
        at: address,
    };
    let unknown = DecodeError::UnknownInstruction(address);
    let first = c.byte()?;
    let mut operands = Vec::with_capacity(4);

    let opcode = match first >> 6 {
        // Variable form (§4.3.3): bit 5 picks 2OP or VAR.
        0b11 => {
            let number = first & 0x1F;
            let opcode = if first & 0x20 == 0 {
                Opcode::two_op(number)
            } else {
                Opcode::var(number)
            }
            .ok_or(unknown)?;
            let types = c.byte()?;
            for shift in [6, 4, 2, 0] {
                let bits = types >> shift & 0b11;
                if bits == 0b11 {
                    break;
                }
                operands.push(c.operand(bits)?);
            }
            opcode
        }
        // Short form (§4.3.1): operand type in bits 4-5, 11 meaning none.
        0b10 => {
            let number = first & 0x0F;
            let bits = first >> 4 & 0b11;
            if bits == 0b11 {
                Opcode::zero_op(number).ok_or(unknown)?
            } else {
                let opcode = Opcode::one_op(number).ok_or(unknown)?;
                operands.push(c.operand(bits)?);
                opcode
            }
        }
        // Long form (§4.3.2): always 2OP, each operand a small constant or variable.
        _ => {
            let opcode = Opcode::two_op(first & 0x1F).ok_or(unknown)?;
            for bit in [0x40, 0x20] {
                let bits = if first & bit == 0 { 0b01 } else { 0b10 };
                operands.push(c.operand(bits)?);
            }
            opcode
        }
    };

    let store = if opcode.stores() {
        Some(c.byte()?)
    } else {
        None
    };

    let mut branch = None;
    let mut branch_data = None;
    if opcode.branches() {
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
