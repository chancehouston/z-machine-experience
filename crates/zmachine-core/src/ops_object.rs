//! The object instructions (§12, §15).

use crate::decode::{Instruction, Opcode};
use crate::exec::X;
use crate::objects::Objects;
use crate::{Diagnostic, DiagnosticKind, Machine};

impl Machine {
    pub(crate) fn objects(&self) -> Objects {
        Objects::for_version(self.header.objects, self.header.v)
    }

    /// Records that an instruction was applied to object 0. There is no object 0
    /// (§12.3.1), so the Standard makes this illegal, but published games do it.
    /// The instruction then does nothing or yields 0, and the game continues
    /// (research.md R12).
    pub(crate) fn object_zero(&mut self) {
        self.diagnostics.push(Diagnostic {
            kind: DiagnosticKind::ObjectZero,
            address: self.current,
        });
    }

    pub(crate) fn object_instruction(
        &mut self,
        ins: &Instruction,
        a: u16,
        b: u16,
        c: u16,
    ) -> X<()> {
        use Opcode::*;
        let objects = self.objects();

        // get_prop_len takes an address, not an object.
        if ins.opcode == GetPropLen {
            let len = objects.prop_len(&self.memory, a)?;
            return self.write_result(ins, len);
        }

        let needs_second = ins.opcode == InsertObj;
        if a == 0 || (needs_second && b == 0) {
            self.object_zero();
            return match ins.opcode {
                GetParent | GetProp | GetPropAddr | GetNextProp => self.write_result(ins, 0),
                GetChild | GetSibling => {
                    self.write_result(ins, 0)?;
                    self.branch(ins, false)
                }
                // Object 0 has no parent, which reads as parent 0.
                Jin => self.branch(ins, b == 0),
                TestAttr => self.branch(ins, false),
                _ => Ok(()),
            };
        }

        match ins.opcode {
            GetParent => {
                let parent = objects.parent(&self.memory, a)?;
                self.write_result(ins, parent)
            }
            // §15 get_child, get_sibling: store the result, then branch if there
            // is one.
            GetChild => {
                let child = objects.child(&self.memory, a)?;
                self.write_result(ins, child)?;
                self.branch(ins, child != 0)
            }
            GetSibling => {
                let sibling = objects.sibling(&self.memory, a)?;
                self.write_result(ins, sibling)?;
                self.branch(ins, sibling != 0)
            }
            Jin => {
                let parent = objects.parent(&self.memory, a)?;
                self.branch(ins, parent == b)
            }
            TestAttr => {
                let set = objects.test_attr(&self.memory, a, b)?;
                self.branch(ins, set)
            }
            SetAttr => objects.set_attr(&mut self.memory, a, b),
            ClearAttr => objects.clear_attr(&mut self.memory, a, b),
            InsertObj => objects.insert(&mut self.memory, a, b),
            RemoveObj => objects.remove(&mut self.memory, a),
            GetProp => {
                let value = objects.get_prop(&self.memory, a, b)?;
                self.write_result(ins, value)
            }
            GetPropAddr => {
                let address = objects.get_prop_addr(&self.memory, a, b)?;
                self.write_result(ins, address)
            }
            GetNextProp => {
                let next = objects.get_next_prop(&self.memory, a, b)?;
                self.write_result(ins, next)
            }
            PutProp => objects.put_prop(&mut self.memory, a, b, c),
            PrintObj => {
                let name = self.object_name(a)?;
                self.emit(&name);
                Ok(())
            }
            _ => unreachable!("only object instructions are dispatched here"),
        }
    }

    fn write_result(&mut self, ins: &Instruction, value: u16) -> X<()> {
        match ins.store {
            Some(variable) => self.write_variable(variable, value),
            None => Ok(()),
        }
    }
}
