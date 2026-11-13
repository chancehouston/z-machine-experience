//! Comparing what the interface says about light with whether a game describes
//! the room when asked (specs/008-darkness/research.md R6, R7, R9).

use zmachine_core::Machine;
use zmachine_introspect::Introspector;

/// Asks a copy of `machine`, which must be waiting at a command prompt, to
/// describe where the player is, with `command`, and says whether it did: whether
/// the room's own printed name is in the reply. The reply is read and dropped.
/// `None` where the interface cannot name the room. `machine` is not touched.
pub fn described(machine: &Machine, introspector: &Introspector, command: &str) -> Option<bool> {
    let name = introspector
        .location(&machine.view())
        .value
        .ok()?
        .name
        .ok()?
        .to_lowercase();
    if name.is_empty() {
        return None;
    }
    let mut copy = machine.clone();
    copy.provide_line(command).ok()?;
    copy.run(crate::trial::LIMIT).ok()?;
    Some(copy.take_output().to_lowercase().contains(&name))
}
