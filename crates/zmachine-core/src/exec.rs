//! The execution loop: frames, variables, calls and instruction dispatch (§4 to §6).

use crate::decode::{self, BranchTarget, DecodeError, Instruction, Opcode, Operand};
use crate::{Event, Fault, FaultKind, Machine, ops_math as math};

/// The deepest the call stack may go. The Standard sets no limit; this one keeps a
/// runaway recursion from exhausting the host's memory (data-model.md).
pub(crate) const MAX_CALL_DEPTH: usize = 1024;

pub(crate) type X<T> = Result<T, FaultKind>;

/// One routine call (§6.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    /// Where execution resumes in the caller.
    pub return_pc: u32,
    /// The caller's variable that receives the result.
    pub result: Option<u8>,
    pub locals: Vec<u16>,
    /// The routine's own evaluation stack (§6.3.2).
    pub stack: Vec<u16>,
    /// How many arguments were passed; recorded in a save.
    pub arg_count: u8,
}

impl Frame {
    /// The frame of the main code, which has no locals and no caller (§5.5).
    pub fn main() -> Self {
        Frame {
            return_pc: 0,
            result: None,
            locals: Vec::new(),
            stack: Vec::new(),
            arg_count: 0,
        }
    }
}

/// What the machine is doing between calls from the host (data-model.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Running,
    AwaitingLine {
        text: u16,
        parse: u16,
        max_len: usize,
    },
    AwaitingSaveResult,
    AwaitingRestoreData,
    AwaitingTranscriptAnswer,
    Finished,
    Faulted,
}

/// What executing one instruction leads to.
enum Flow {
    Continue,
    Stop(Event),
}

impl Machine {
    fn frame(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("the main frame is never removed")
    }

    pub(crate) fn push(&mut self, value: u16) {
        self.frame().stack.push(value);
    }

    /// §6.3.3: a routine cannot pop below the base of its own stack.
    pub(crate) fn pop(&mut self) -> X<u16> {
        self.frame().stack.pop().ok_or(FaultKind::StackUnderflow)
    }

    fn global_address(&self, variable: u8) -> u32 {
        self.header.globals as u32 + 2 * (variable as u32 - 16)
    }

    fn local(&mut self, variable: u8) -> X<&mut u16> {
        self.frame()
            .locals
            .get_mut(variable as usize - 1)
            .ok_or(FaultKind::InvalidVariable)
    }

    /// Reads a variable as an operand: variable 0 pops the stack (§4.2.2, §6.3).
    pub(crate) fn read_variable(&mut self, variable: u8) -> X<u16> {
        match variable {
            0 => self.pop(),
            1..=15 => self.local(variable).map(|l| *l),
            _ => Ok(self.memory.read_word(self.global_address(variable))?),
        }
    }

    /// Writes a result: variable 0 pushes onto the stack.
    pub(crate) fn write_variable(&mut self, variable: u8, value: u16) -> X<()> {
        match variable {
            0 => self.push(value),
            1..=15 => *self.local(variable)? = value,
            _ => self
                .memory
                .write_word(self.global_address(variable), value)?,
        }
        Ok(())
    }

    /// Reads a variable named indirectly by an instruction: the stack is read in
    /// place, without popping (§6.3.4).
    fn peek_variable(&mut self, variable: u8) -> X<u16> {
        match variable {
            0 => self
                .frame()
                .stack
                .last()
                .copied()
                .ok_or(FaultKind::StackUnderflow),
            _ => self.read_variable(variable),
        }
    }

    /// Writes a variable named indirectly: the top of the stack is replaced, not
    /// pushed onto (§6.3.4).
    fn poke_variable(&mut self, variable: u8, value: u16) -> X<()> {
        match variable {
            0 => {
                *self
                    .frame()
                    .stack
                    .last_mut()
                    .ok_or(FaultKind::StackUnderflow)? = value;
                Ok(())
            }
            _ => self.write_variable(variable, value),
        }
    }

    /// Stores an instruction's result in its store variable.
    fn store(&mut self, ins: &Instruction, value: u16) -> X<()> {
        match ins.store {
            Some(variable) => self.write_variable(variable, value),
            None => Ok(()),
        }
    }

    /// Takes or skips an instruction's branch according to `condition` (§4.7).
    pub(crate) fn branch(&mut self, ins: &Instruction, condition: bool) -> X<()> {
        let Some(branch) = ins.branch else {
            return Ok(());
        };
        if condition != branch.on_true {
            return Ok(());
        }
        match branch.target {
            BranchTarget::ReturnFalse => self.return_from_routine(0),
            BranchTarget::ReturnTrue => self.return_from_routine(1),
            BranchTarget::Address(address) => {
                self.pc = address;
                Ok(())
            }
        }
    }

    /// §6.4.5: returns `value` to the caller, discarding the routine's stack.
    fn return_from_routine(&mut self, value: u16) -> X<()> {
        if self.frames.len() == 1 {
            // §5.5: the main code is not a routine and has nothing to return to.
            return Err(FaultKind::StackUnderflow);
        }
        let frame = self.frames.pop().expect("checked above");
        self.pc = frame.return_pc;
        match frame.result {
            Some(variable) => self.write_variable(variable, value),
            None => Ok(()),
        }
    }

    /// §6.4: calls the routine at a packed address with the remaining operands as
    /// arguments.
    fn call(&mut self, ins: &Instruction, operands: &[u16]) -> X<()> {
        let packed = operands.first().copied().unwrap_or(0);
        if packed == 0 {
            // §6.4.3: calling address 0 does nothing and returns false.
            return self.store(ins, 0);
        }
        if self.frames.len() > MAX_CALL_DEPTH {
            return Err(FaultKind::CallStackOverflow);
        }
        // §1.2.3: a version 3 packed address is half the byte address.
        let address = packed as u32 * 2;
        let count = self.memory.read_byte(address)? as usize;
        if count > 15 {
            return Err(FaultKind::InvalidVariable);
        }
        // §5.2.1: in version 3 the initial values of the locals follow the count.
        let mut locals = Vec::with_capacity(count);
        for i in 0..count {
            locals.push(self.memory.read_word(address + 1 + 2 * i as u32)?);
        }
        // §6.4.4: arguments are written into the first locals; extras are dropped.
        let arguments = &operands[1..];
        for (local, &argument) in locals.iter_mut().zip(arguments) {
            *local = argument;
        }
        self.frames.push(Frame {
            return_pc: self.pc,
            result: ins.store,
            locals,
            stack: Vec::new(),
            arg_count: arguments.len().min(count) as u8,
        });
        self.pc = address + 1 + 2 * count as u32;
        Ok(())
    }

    /// Runs up to `budget` instructions. The state is `Running` on entry.
    pub(crate) fn execute(&mut self, budget: u64) -> Event {
        if let Some(fault) = self.pending_fault.take() {
            self.state = State::Faulted;
            return Event::Fault(fault);
        }
        for _ in 0..budget {
            // §7.1.1.2: a game starts a transcript by setting a bit in the header,
            // which any instruction that writes memory can do. The host is asked
            // before anything further runs.
            if self.transcript_wanted() {
                self.state = State::AwaitingTranscriptAnswer;
                return Event::TranscriptRequested;
            }
            self.current = self.pc;
            match self.step() {
                Ok(Flow::Continue) => {}
                Ok(Flow::Stop(event)) => return event,
                Err(kind) => {
                    self.state = State::Faulted;
                    return Event::Fault(Fault {
                        kind,
                        address: self.current,
                    });
                }
            }
        }
        Event::BudgetExhausted
    }

    fn step(&mut self) -> X<Flow> {
        let ins = decode::decode_for(&self.memory, self.pc, self.header.v).map_err(|e| match e {
            DecodeError::UnknownInstruction(_) => FaultKind::UnknownInstruction,
            DecodeError::ReadOutsideFile(_) => FaultKind::ReadOutsideFile,
        })?;
        self.pc = ins.next;

        // §4.2.3: operands are evaluated in order, so stack operands pop left to
        // right.
        let mut operands = [0u16; 4];
        let count = ins.operands.len();
        for (slot, operand) in operands.iter_mut().zip(&ins.operands) {
            *slot = match *operand {
                Operand::Large(value) => value,
                Operand::Small(value) => value as u16,
                Operand::Variable(variable) => self.read_variable(variable)?,
            };
        }
        // An operand the instruction needs but the story omitted reads as 0.
        let [a, b, c, _] = operands;
        let given = &operands[..count];
        // For the instructions that name a variable by number (§6.3.4).
        let named = a as u8;

        use Opcode::*;
        match ins.opcode {
            // ---- Arithmetic, logic, comparison ----
            Add => self.store(&ins, math::add(a, b))?,
            Sub => self.store(&ins, math::sub(a, b))?,
            Mul => self.store(&ins, math::mul(a, b))?,
            Div => self.store(&ins, math::div(a, b)?)?,
            Mod => self.store(&ins, math::modulo(a, b)?)?,
            And => self.store(&ins, math::and(a, b))?,
            Or => self.store(&ins, math::or(a, b))?,
            Not => self.store(&ins, math::not(a))?,
            Je => self.branch(&ins, math::je(given))?,
            Jl => self.branch(&ins, math::jl(a, b))?,
            Jg => self.branch(&ins, math::jg(a, b))?,
            Jz => self.branch(&ins, a == 0)?,
            Test => self.branch(&ins, math::test(a, b))?,

            // ---- Variables and memory ----
            Store => self.poke_variable(named, b)?,
            Load => {
                let value = self.peek_variable(named)?;
                self.store(&ins, value)?;
            }
            Inc => {
                let value = self.peek_variable(named)?.wrapping_add(1);
                self.poke_variable(named, value)?;
            }
            Dec => {
                let value = self.peek_variable(named)?.wrapping_sub(1);
                self.poke_variable(named, value)?;
            }
            IncChk => {
                let value = self.peek_variable(named)?.wrapping_add(1);
                self.poke_variable(named, value)?;
                self.branch(&ins, math::jg(value, b))?;
            }
            DecChk => {
                let value = self.peek_variable(named)?.wrapping_sub(1);
                self.poke_variable(named, value)?;
                self.branch(&ins, math::jl(value, b))?;
            }
            Push => self.push(a),
            Pull => {
                let value = self.pop()?;
                self.poke_variable(named, value)?;
            }
            Pop => {
                self.pop()?;
            }
            // §15 loadw, storew: the address is array + 2 * index, as a 16-bit sum.
            Loadw => {
                let address = a.wrapping_add(b.wrapping_mul(2)) as u32;
                let value = self.memory.read_word(address)?;
                self.store(&ins, value)?;
            }
            Loadb => {
                let value = self.memory.read_byte(a.wrapping_add(b) as u32)?;
                self.store(&ins, value as u16)?;
            }
            Storew => self
                .memory
                .write_word(a.wrapping_add(b.wrapping_mul(2)) as u32, c)?,
            Storeb => self.memory.write_byte(a.wrapping_add(b) as u32, c as u8)?,

            // ---- Control flow ----
            Call => self.call(&ins, given)?,
            Ret => self.return_from_routine(a)?,
            Rtrue => self.return_from_routine(1)?,
            Rfalse => self.return_from_routine(0)?,
            RetPopped => {
                let value = self.pop()?;
                self.return_from_routine(value)?;
            }
            // §15 jump: a signed offset from the address after the instruction,
            // less 2, as for a branch.
            Jump => self.pc = (ins.next as i64 + (a as i16) as i64 - 2) as u32,
            Nop => {}
            Quit => {
                self.end_last_line();
                self.state = State::Finished;
                return Ok(Flow::Stop(Event::Finished));
            }
            Restart => {
                self.flush_chunk();
                self.restart();
            }

            // ---- Objects ----
            Jin | TestAttr | SetAttr | ClearAttr | InsertObj | RemoveObj | GetParent | GetChild
            | GetSibling | GetProp | GetPropAddr | GetNextProp | GetPropLen | PutProp
            | PrintObj => self.object_instruction(&ins, a, b, c)?,

            // ---- Output ----
            Print | PrintRet => {
                let address = ins.text.expect("the decoder sets text for print");
                self.print_at(address)?;
                if ins.opcode == PrintRet {
                    self.emit("\n");
                    self.return_from_routine(1)?;
                }
            }
            PrintAddr => self.print_at(a as u32)?,
            PrintPaddr => self.print_at(a as u32 * 2)?,
            PrintChar => self.print_zscii(a),
            PrintNum => self.emit(&(a as i16).to_string()),
            NewLine => self.emit("\n"),
            ShowStatus => {
                self.flush_chunk();
                self.update_status()?;
            }
            SplitWindow => self.flush_chunk(),
            SetWindow => {
                // The word in progress belongs to the window it was printed in.
                self.flush_chunk();
                self.window = a;
            }
            OutputStream => {
                self.flush_chunk();
                self.output_stream(a as i16)?;
            }
            // §10.2, §15 sound_effect: command files and bleeps are not offered.
            InputStream | SoundEffect => {}

            // ---- Input, the generator, and game state ----
            Sread => {
                let event = self.begin_read(a, b)?;
                return Ok(Flow::Stop(event));
            }
            Random => {
                let value = self.random.random(a as i16);
                self.store(&ins, value)?;
            }
            Verify => {
                let intact = self.verify();
                self.branch(&ins, intact)?;
            }
            Save => {
                let event = self.begin_save(ins);
                return Ok(Flow::Stop(event));
            }
            Restore => {
                self.flush_chunk();
                self.state = State::AwaitingRestoreData;
                self.pending = Some(ins);
                return Ok(Flow::Stop(Event::RestoreRequested));
            }
            // The instructions of versions 5 and 8 are decoded but not yet executed.
            _ => return Err(FaultKind::UnknownInstruction),
        }
        Ok(Flow::Continue)
    }
}

#[cfg(test)]
impl Machine {
    /// Global variable `n`, counting from 0.
    pub(crate) fn global(&self, n: usize) -> u16 {
        self.memory
            .read_word(self.header.globals as u32 + 2 * n as u32)
            .unwrap()
    }

    /// The current routine's evaluation stack, bottom first.
    pub(crate) fn stack(&self) -> Vec<u16> {
        self.frames.last().unwrap().stack.clone()
    }

    /// How many routine calls are in progress.
    pub(crate) fn frame_depth(&self) -> usize {
        self.frames.len() - 1
    }
}

#[cfg(test)]
#[path = "exec_tests.rs"]
mod tests;
