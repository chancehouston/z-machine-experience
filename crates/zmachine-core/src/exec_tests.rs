use crate::testing::{BIG, g, load, run, run_story, skip_if};
use crate::{Event, Fault, FaultKind};
use zmachine_testkit::op::{self, Arg, Arg::*};
use zmachine_testkit::{SCRATCH, Story};

fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

fn store(var: u8, value: u16) -> Vec<u8> {
    op::op2(op::STORE, S(var), L(value))
}

fn push(value: u16) -> Vec<u8> {
    op::var(op::PUSH, &[L(value)])
}

/// `add a b -> result`
fn add(a: Arg, b: Arg, result: u8) -> Vec<u8> {
    cat(&[op::op2(op::ADD, a, b), op::store(result)])
}

fn call(routine: u16, args: &[u16], result: u8) -> Vec<u8> {
    let mut operands = vec![L(routine)];
    operands.extend(args.iter().map(|&a| L(a)));
    cat(&[op::var(op::CALL, &operands), op::store(result)])
}

fn fault_of(event: Event) -> Fault {
    match event {
        Event::Fault(f) => f,
        other => panic!("expected a fault, got {other:?}"),
    }
}

// §4.2.2: variables 16 to 255 are the globals. §15 store.
#[test]
fn store_writes_a_global() {
    let (m, e) = run(&cat(&[store(g(0), 1234), store(g(239), 0xFFFF)]));
    assert_eq!(e, Event::Finished);
    assert_eq!(m.global(0), 1234);
    assert_eq!(m.global(239), 0xFFFF);
}

// §6.3: variable 0 as an operand pops the stack; as a store target it pushes.
#[test]
fn variable_zero_pops_as_an_operand_and_pushes_as_a_result() {
    let (m, _) = run(&cat(&[push(5), push(6), add(V(0), V(0), g(0))]));
    assert_eq!(m.global(0), 11);
    assert!(m.stack().is_empty());

    let (m, _) = run(&add(S(1), S(2), 0));
    assert_eq!(m.stack(), vec![3]);
}

// §4.2.3: operands are evaluated left to right, so the first pop is the top.
#[test]
fn stack_operands_are_popped_left_to_right() {
    let sub = cat(&[op::op2(op::SUB, V(0), V(0)), op::store(g(0))]);
    let (m, _) = run(&cat(&[push(3), push(10), sub]));
    assert_eq!(m.global(0), 7);
}

// §15 push, pull.
#[test]
fn push_and_pull() {
    let (m, _) = run(&cat(&[push(7), push(8), op::var(op::PULL, &[S(g(1))])]));
    assert_eq!(m.global(1), 8);
    assert_eq!(m.stack(), vec![7]);
}

// §6.3.4: an instruction that names the stack indirectly (store, load, inc, dec,
// inc_chk, dec_chk, pull) reads or writes its top in place, without pushing or
// popping.
#[test]
fn indirect_references_to_the_stack_work_in_place() {
    let (m, _) = run(&cat(&[push(5), op::op2(op::STORE, S(0), S(9))]));
    assert_eq!(m.stack(), vec![9], "store replaces the top");

    let load = cat(&[op::op1(op::LOAD, S(0)), op::store(g(0))]);
    let (m, _) = run(&cat(&[push(9), load]));
    assert_eq!(
        (m.global(0), m.stack()),
        (9, vec![9]),
        "load leaves the top in place"
    );

    let (m, _) = run(&cat(&[push(9), op::op1(op::INC, S(0))]));
    assert_eq!(m.stack(), vec![10]);
    let (m, _) = run(&cat(&[push(9), op::op1(op::DEC, S(0))]));
    assert_eq!(m.stack(), vec![8]);

    let (m, _) = run(&cat(&[push(1), push(2), op::var(op::PULL, &[S(0)])]));
    assert_eq!(m.stack(), vec![2], "pull pops, then replaces the new top");
}

// §15 load: reads a variable named by its operand.
#[test]
fn load_reads_a_named_variable() {
    let load = cat(&[op::op1(op::LOAD, S(g(3))), op::store(g(4))]);
    let (m, _) = run(&cat(&[store(g(3), 77), load]));
    assert_eq!(m.global(4), 77);
}

// §15 inc, dec: signed, wrapping.
#[test]
fn inc_and_dec_wrap() {
    let (m, _) = run(&cat(&[store(g(0), 0x7FFF), op::op1(op::INC, S(g(0)))]));
    assert_eq!(m.global(0), 0x8000);
    let (m, _) = run(&cat(&[store(g(0), 0), op::op1(op::DEC, S(g(0)))]));
    assert_eq!(m.global(0), 0xFFFF);
}

// §15 inc_chk: increment, then branch if now greater than the value. dec_chk:
// decrement, then branch if now less. Both signed.
#[test]
fn inc_chk_and_dec_chk() {
    let mark = store(g(1), 1);
    let taken = |code: Vec<u8>| run(&code).0.global(1) == 0;

    let inc = |start: u16, value: u8| {
        cat(&[
            store(g(0), start),
            skip_if(op::op2(op::INC_CHK, S(g(0)), S(value)), true, &mark),
        ])
    };
    assert!(taken(inc(5, 5)), "6 > 5");
    assert!(!taken(inc(4, 5)), "5 > 5 is false");
    assert!(!taken(inc(0xFFFE, 0)), "-1 > 0 is false");

    let dec = |start: u16, value: u8| {
        cat(&[
            store(g(0), start),
            skip_if(op::op2(op::DEC_CHK, S(g(0)), S(value)), true, &mark),
        ])
    };
    assert!(taken(dec(5, 5)), "4 < 5");
    assert!(!taken(dec(6, 5)), "5 < 5 is false");
    assert!(taken(dec(0, 0)), "-1 < 0");

    let (m, _) = run(&inc(5, 5));
    assert_eq!(m.global(0), 6, "the variable is changed either way");
}

// §15 jz, je, jl, jg, test through the executor: the branch is taken or not.
#[test]
fn conditional_branches_follow_their_sense() {
    let mark = store(g(1), 1);
    let skipped = |instruction: Vec<u8>, on_true: bool| {
        run(&skip_if(instruction, on_true, &mark)).0.global(1) == 0
    };
    assert!(skipped(op::op1(op::JZ, S(0)), true));
    assert!(!skipped(op::op1(op::JZ, S(1)), true));
    assert!(skipped(op::op1(op::JZ, S(1)), false), "branch on false");
    assert!(!skipped(op::op1(op::JZ, S(0)), false));
    assert!(skipped(op::op2(op::JE, S(4), S(4)), true));
    assert!(skipped(
        op::op2_var(op::JE, &[S(4), S(1), S(2), S(4)]),
        true
    ));
    assert!(!skipped(
        op::op2_var(op::JE, &[S(4), S(1), S(2), S(3)]),
        true
    ));
    assert!(skipped(op::op2(op::JL, L(0xFFFF), S(0)), true));
    assert!(skipped(op::op2(op::JG, S(0), L(0xFFFF)), true));
    assert!(skipped(op::op2(op::TEST, S(0b111), S(0b101)), true));
    assert!(!skipped(op::op2(op::TEST, S(0b011), S(0b101)), true));
}

// §15 jump: an unconditional jump by a signed offset, to the address after the
// instruction plus offset minus 2.
#[test]
fn jump_skips_forward_and_backward() {
    let mark = store(g(1), 1);
    let forward = cat(&[op::op1(op::JUMP, L(mark.len() as u16 + 2)), mark.clone()]);
    assert_eq!(run(&forward).0.global(1), 0);

    // jump +N over a block; the block jumps back to after the first jump... laid out
    // as: A: jump to C; B: store g2 1; quit; C: jump back to B.
    let b = cat(&[store(g(2), 1), op::op0(op::QUIT)]);
    let to_c = op::op1(op::JUMP, L(b.len() as u16 + 2));
    let back = -(b.len() as i16) - 3 + 2;
    let code = cat(&[to_c, b, op::op1(op::JUMP, L(back as u16))]);
    assert_eq!(run(&code).0.global(2), 1);
}

// §6.4, §5.2: a call makes a frame whose locals start at the routine's defaults,
// with the arguments written over the first ones. §15 call, ret.
#[test]
fn call_passes_arguments_over_default_locals() {
    let body = cat(&[
        add(V(1), V(2), 0),
        add(V(0), V(3), 0),
        op::op1(op::RET, V(0)),
    ]);
    let result = |args: &[u16]| {
        let mut story = Story::new();
        let routine = story.routine(&[10, 20, 30], &body);
        story.main(&cat(&[call(routine, args, g(0)), op::op0(op::QUIT)]));
        run_story(&mut story).0.global(0)
    };
    assert_eq!(result(&[]), 60);
    assert_eq!(result(&[1]), 51);
    assert_eq!(result(&[1, 2]), 33);
    assert_eq!(result(&[1, 2, 3]), 6);
}

// §6.4.4.1: arguments beyond the routine's locals are thrown away.
#[test]
fn extra_arguments_are_discarded() {
    let mut story = Story::new();
    let routine = story.routine(&[5], &op::op1(op::RET, V(1)));
    story.main(&cat(&[call(routine, &[7, 8, 9], g(0)), op::op0(op::QUIT)]));
    assert_eq!(run_story(&mut story).0.global(0), 7);
}

// §6.4.3: calling packed address 0 does nothing and returns false.
#[test]
fn calling_address_zero_returns_false() {
    let (m, e) = run(&cat(&[store(g(0), 5), call(0, &[1, 2], g(0))]));
    assert_eq!(e, Event::Finished);
    assert_eq!(m.global(0), 0);
}

// §15 rtrue, rfalse, ret_popped, and branch offsets 0 and 1 (§4.7.1).
#[test]
fn the_ways_of_returning() {
    let result = |locals: &[u16], body: Vec<u8>, args: &[u16]| {
        let mut story = Story::new();
        let routine = story.routine(locals, &body);
        story.main(&cat(&[
            store(g(0), 99),
            call(routine, args, g(0)),
            op::op0(op::QUIT),
        ]));
        let (m, e) = run_story(&mut story);
        assert_eq!(e, Event::Finished);
        assert!(m.stack().is_empty());
        m.global(0)
    };
    assert_eq!(result(&[], op::op0(op::RTRUE), &[]), 1);
    assert_eq!(result(&[], op::op0(op::RFALSE), &[]), 0);
    assert_eq!(
        result(&[], cat(&[push(42), op::op0(op::RET_POPPED)]), &[]),
        42
    );

    // jz L1 ?rtrue / ?rfalse; then return 7.
    let branch_ret = |offset: i16| {
        cat(&[
            op::op1(op::JZ, V(1)),
            op::branch(true, offset),
            op::op1(op::RET, S(7)),
        ])
    };
    assert_eq!(result(&[0], branch_ret(1), &[0]), 1);
    assert_eq!(result(&[0], branch_ret(0), &[0]), 0);
    assert_eq!(result(&[0], branch_ret(1), &[5]), 7);
}

// §6.3.2: a routine's stack is its own; what it leaves behind is discarded and the
// caller's stack is as it was.
#[test]
fn a_routines_leftover_stack_is_discarded_on_return() {
    let mut story = Story::new();
    let routine = story.routine(&[], &cat(&[push(1), push(2), op::op0(op::RTRUE)]));
    story.main(&cat(&[
        push(50),
        call(routine, &[], g(0)),
        op::op0(op::QUIT),
    ]));
    let (m, _) = run_story(&mut story);
    assert_eq!(m.stack(), vec![50]);
}

// §15 nop, pop.
#[test]
fn nop_and_pop() {
    let (m, e) = run(&cat(&[
        push(1),
        push(2),
        op::op0(op::NOP),
        op::op0(op::POP),
    ]));
    assert_eq!(e, Event::Finished);
    assert_eq!(m.stack(), vec![1]);
}

// §15 loadw, loadb, storew, storeb: array is a byte address; words are indexed by
// two, bytes by one.
#[test]
fn array_reads_and_writes() {
    let code = cat(&[
        op::var(op::STOREW, &[L(SCRATCH), S(1), L(0xBEEF)]),
        op::var(op::STOREB, &[L(SCRATCH), S(0), S(0x7F)]),
        op::op2(op::LOADW, L(SCRATCH), S(1)),
        op::store(g(0)),
        op::op2(op::LOADB, L(SCRATCH), S(2)),
        op::store(g(1)),
        op::op2(op::LOADB, L(SCRATCH), S(0)),
        op::store(g(2)),
    ]);
    let (m, _) = run(&code);
    assert_eq!(
        (m.global(0), m.global(1), m.global(2)),
        (0xBEEF, 0xBE, 0x7F)
    );
}

// Globals live in dynamic memory at the address in the header (§6.2).
#[test]
fn globals_are_stored_in_the_globals_table() {
    let read = cat(&[
        op::op2(op::LOADW, L(zmachine_testkit::GLOBALS), S(3)),
        op::store(g(0)),
    ]);
    let (m, _) = run(&cat(&[store(g(3), 0x1234), read]));
    assert_eq!(m.global(0), 0x1234);
}

fn main_address() -> u32 {
    Story::new().next_code_address()
}

// §14: an opcode with no version 3 instruction. The fault carries the address of
// the instruction.
#[test]
fn an_unknown_instruction_is_a_fault() {
    let nop = op::op0(op::NOP);
    let (_, e) = run(&cat(&[nop.clone(), nop, vec![0xBE]]));
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::UnknownInstruction,
            address: main_address() + 2
        }
    );
}

// §6.3.3: "The stack is considered as empty at the start of each routine: it is
// illegal to pull values from it unless values have first been pushed on."
#[test]
fn popping_an_empty_stack_is_a_fault() {
    let (_, e) = run(&op::op0(op::POP));
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::StackUnderflow,
            address: main_address()
        }
    );

    let mut story = Story::new();
    let routine = story.routine(&[], &op::op0(op::RET_POPPED));
    story.main(&cat(&[
        push(5),
        call(routine, &[], g(0)),
        op::op0(op::QUIT),
    ]));
    let (_, e) = run_story(&mut story);
    assert_eq!(
        fault_of(e).kind,
        FaultKind::StackUnderflow,
        "the caller's 5 is out of reach"
    );

    let (_, e) = run(&add(V(0), S(1), g(0)));
    assert_eq!(fault_of(e).kind, FaultKind::StackUnderflow);
    let (_, e) = run(&op::op1(op::INC, S(0)));
    assert_eq!(
        fault_of(e).kind,
        FaultKind::StackUnderflow,
        "in-place needs a top"
    );
}

// §5.4: the main code is not a routine; returning from it is illegal.
#[test]
fn returning_from_the_main_code_is_a_fault() {
    let (_, e) = run(&op::op0(op::RTRUE));
    assert_eq!(fault_of(e).kind, FaultKind::StackUnderflow);
}

// §4.2.2: locals exist only up to the number the routine declares.
#[test]
fn a_local_that_does_not_exist_is_a_fault() {
    let (_, e) = run(&add(V(1), S(1), g(0)));
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::InvalidVariable,
            address: main_address()
        }
    );

    let mut story = Story::new();
    let routine = story.routine(&[0, 0], &store(3, 1));
    story.main(&cat(&[call(routine, &[], g(0)), op::op0(op::QUIT)]));
    assert_eq!(
        fault_of(run_story(&mut story).1).kind,
        FaultKind::InvalidVariable
    );
}

// data-model.md: the call stack is "Limited to 1024 frames; exceeding it is a fault."
#[test]
fn runaway_recursion_is_a_fault_not_a_crash() {
    let mut story = Story::new();
    let at = (story.next_code_address() / 2) as u16;
    let routine = story.routine(&[], &cat(&[call(at, &[], 0), op::op0(op::RTRUE)]));
    assert_eq!(routine, at);
    story.main(&cat(&[call(routine, &[], g(0)), op::op0(op::QUIT)]));
    let (m, e) = run_story(&mut story);
    assert_eq!(fault_of(e).kind, FaultKind::CallStackOverflow);
    assert_eq!(m.frame_depth(), 1024);
}

// §1.1: only dynamic memory may be written.
#[test]
fn writing_outside_dynamic_memory_is_a_fault() {
    let mut story = Story::new();
    let static_base = story.static_base();
    story.main(&op::var(op::STOREB, &[L(static_base), S(0), S(1)]));
    let (_, e) = run_story(&mut story);
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::WriteOutsideDynamicMemory,
            address: main_address()
        }
    );

    let mut story = Story::new();
    let static_base = story.static_base();
    story.main(&op::var(op::STOREW, &[L(static_base - 1), S(0), S(1)]));
    assert_eq!(
        fault_of(run_story(&mut story).1).kind,
        FaultKind::WriteOutsideDynamicMemory
    );
}

// Reading past the end of the file is a fault.
#[test]
fn reading_outside_the_file_is_a_fault() {
    let read = cat(&[op::op2(op::LOADB, L(0xFFFF), S(0)), op::store(g(0))]);
    let (_, e) = run(&read);
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::ReadOutsideFile,
            address: main_address()
        }
    );
}

// §15 div: division by zero halts, reporting where.
#[test]
fn division_by_zero_is_a_fault_at_the_instruction() {
    let nop = op::op0(op::NOP);
    let div = cat(&[op::op2(op::DIV, S(5), S(0)), op::store(g(0))]);
    let (_, e) = run(&cat(&[nop, div]));
    assert_eq!(
        fault_of(e),
        Fault {
            kind: FaultKind::DivisionByZero,
            address: main_address() + 1
        }
    );
}

// Execution running off the end of the file is a fault, not a panic.
#[test]
fn running_off_the_end_of_the_file_is_a_fault() {
    let mut story = Story::new();
    story.main(&op::op0(op::NOP));
    let mut m = load(&mut story);
    let kind = fault_of(m.run(BIG).unwrap()).kind;
    assert!(matches!(
        kind,
        FaultKind::ReadOutsideFile | FaultKind::UnknownInstruction
    ));
}

// §5.2: a routine may not declare more than 15 locals.
#[test]
fn a_routine_with_too_many_locals_is_a_fault() {
    let mut story = Story::new();
    let bogus = story.raw_string(&[16, 0, 0]);
    story.main(&cat(&[call(bogus, &[], g(0)), op::op0(op::QUIT)]));
    assert_eq!(
        fault_of(run_story(&mut story).1).kind,
        FaultKind::InvalidVariable
    );
}
