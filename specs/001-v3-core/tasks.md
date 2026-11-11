# Tasks: Faithful Z-Machine Version 3 Core

**Input**: Design documents from `/specs/001-v3-core/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Revised** after two independent reviews on 2026-11-11; see "Revision after review" in
plan.md.

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. In every
phase the test tasks are written and seen to fail before the implementation tasks beneath
them. "Fail" means the test compiles and its assertion fails, which is why public types
are stubbed in Phase 2. Every test of interpreter behavior carries a comment citing the
section of the Z-Machine Standard 1.1 it verifies (written "§" below; "Q§" is Quetzal 1.4).

**Where tests live** (research R8): tests of internals are unit tests in a
`#[cfg(test)] mod tests` inside the named source file. Tests through the public API are
files in the crate's `tests/` directory.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US5)

## Path Conventions

- Core: `crates/zmachine-core/` · Testkit: `crates/zmachine-testkit/` · CLI: `crates/zmachine-cli/`
- Paths are relative to the repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: A workspace that builds, and the story files in place with their records

- [ ] T001 Create the workspace: `Cargo.toml` (resolver 3, edition 2024, three members); `crates/zmachine-core/` with empty `[dependencies]`, dev-dependency on `zmachine-testkit`, and `src/lib.rs` starting `#![forbid(unsafe_code)]`; `crates/zmachine-testkit/` with no dependencies; `crates/zmachine-cli/` with `src/lib.rs`, a `src/main.rs` producing a binary named `zm`, a dependency on `zmachine-core` only, and dev-dependencies `zmachine-testkit` and `sha2`; confirm `cargo build --workspace` succeeds and commit `Cargo.lock`
- [ ] T002 [P] Add `stories/zork1/zork1.z3` from `historicalsource/zork1` (`COMPILED/zork1.z3`) with `git add -f`; confirm SHA-256 `37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79`; add `stories/zork1/LICENSE` (that repository's license text) and `stories/zork1/SOURCE.md` with URL, commit, digest, release 119, serial 880429, and the caveat from research R3
- [ ] T003 [P] Add `stories/czech/`: download `czech_0_8.zip` from the IF Archive; commit `czech.inf`, `README.txt` and `czech.out3`; read the license in `README.txt` and stop for the project owner if it does not permit redistribution; install Inform 6 (`brew install inform6`), compile `czech.inf` for version 3, and commit `czech.z3` with `git add -f`; write `stories/czech/SOURCE.md` with URL, digests, compiler version and the exact compile command
- [ ] T004 [P] Create `docs/divergences.md` with the two rule kinds from research R10 and no entries
- [ ] T005 [P] Record in `specs/001-v3-core/research.md` R1 the output of `dfrotz -v` on this machine, and write `scripts/regenerate-references.sh`, which refuses to run unless `dfrotz -v` reports 2.55

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The image builder, the public types, header, memory and decoder; and the two
pieces of early evidence that need only dfrotz

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Define the public API as compiling stubs in `crates/zmachine-core/src/lib.rs`, from contracts/core-api.md and data-model.md: `Machine` with `load`, `run`, the five answer methods and the four `take_` methods; `Event` (seven variants); `Fault` (nine kinds, each with an address); `Diagnostic`; `StatusLine`; `LoadError`; `HostError`; load options with `seed` and `entropy`. Every method body is `todo!()`-free and returns a placeholder error so tests can fail by assertion
- [ ] T007 Write failing unit tests in `crates/zmachine-testkit/src/lib.rs` for the image builder's encoders against known vectors: Z-encoding of words whose bytes are given in Standard §3 examples and of dictionary words read from `stories/zork1/zork1.z3`; each operand form of §4; a built image whose header fields read back at the offsets of §11
- [ ] T008 Implement the builder in `crates/zmachine-testkit/src/lib.rs` (header, globals, object table, dictionary, abbreviations, caller-supplied instruction bytes, helpers for operands and Z-strings) until T007 passes
- [ ] T009 [P] Write failing unit tests in `crates/zmachine-core/src/header.rs` (§11): "Must be 3, otherwise load is refused"; "Each must lie inside the file; static base must be at least 64"; a header length or checksum that does not match the file is **not** a load error; too short a file is `TooShort`; the interpreter-owned fields are set after load (§11.1)
- [ ] T010 [P] Write failing unit tests in `crates/zmachine-core/src/memory.rs` (§1): byte and word reads anywhere in the file; a read outside the file is an error; writes succeed below the static base and are errors at or above it
- [ ] T011 [P] Write failing unit tests in `crates/zmachine-core/src/decode.rs` (§4): long, short and variable forms; every operand type combination; the store byte; branch data of one and two bytes including offsets 0 and 1 meaning return false and return true; inline text; an opcode number not defined for version 3 is reported as unknown
- [ ] T012 Implement header parsing and `Machine::load` in `crates/zmachine-core/src/header.rs` and `lib.rs` until T009 passes
- [ ] T013 Implement memory access in `crates/zmachine-core/src/memory.rs` until T010 passes
- [ ] T014 Implement the decoder in `crates/zmachine-core/src/decode.rs` until T011 passes
- [ ] T015 [P] Random evidence: with the testkit, write a small example program `crates/zmachine-testkit/examples/random_probe.rs` that emits a version 3 image which turns transcription on and prints 200 results of `random 100`, then reseeds with -5, -1000 and 0 and prints 20 more after each; run it under dfrotz at seeds 0, 1, 999, 1000 and 123456; commit the image-independent number lists as `crates/zmachine-core/tests/fixtures/random/seed_<n>.txt`; correct research R2 if dfrotz disagrees with it
- [ ] T016 [P] Transcript evidence: using dfrotz and `stories/zork1/zork1.z3`, settle the three items under "To confirm" in `specs/001-v3-core/contracts/transcript-format.md` (case of echoed input, over-long input, turning transcription off), and record each with the command that shows it

**Checkpoint**: Images can be built, loaded and decoded; the generator and transcript
format are known from observation

---

## Phase 3: Walkthrough (runs alongside Phases 4 to 6)

**Purpose**: The reference for the headline test. Needs only dfrotz and T002, so it does
not wait for the interpreter

- [ ] T017 Choose a seed (0 to 2147483647), record it in `stories/zork1/fidelity/seed.txt`, and write the first section of `stories/zork1/fidelity/walkthrough.txt` against `dfrotz -m -q -w 80 -S 0 -s <seed>`: the `script` command and transcript file name, then from the start to the first treasure placed in the trophy case. Commands are entered by us while playing; maps and hints may be consulted; no published command list is copied; no line begins with a backslash; no `save`, `restore` or `unscript`
- [ ] T018 Extend `stories/zork1/fidelity/walkthrough.txt` section by section until the game is won with 350 points, ending with `quit` and `y`. After each section, confirm two dfrotz runs give identical transcripts. If a section cannot be completed at this seed after trying another seed, stop for the project owner
- [ ] T019 Run the finished walkthrough under `dfrotz -Z 2` and record in `specs/001-v3-core/research.md` R12 whether any warning appears in the transcript; if one does, add a `transcript` rule to `docs/divergences.md` for it
- [ ] T020 Generate `stories/zork1/fidelity/reference.scr` with `scripts/regenerate-references.sh` and commit it as generated

**Checkpoint**: A committed reference transcript of a won game exists

---

## Phase 4: User Story 1 - Play a version 3 game from the command line (Priority: P1) 🎯 MVP

**Goal**: A version 3 story file runs: opening text, commands in, responses out, quit

**Independent Test**: `cargo run -p zmachine-cli -- stories/zork1/zork1.z3`, then
`open mailbox`, `read leaflet`, `quit`, `y`; each response is correct and the exit status
is 0

### Tests for User Story 1 ⚠️

> Write these first and confirm they fail by assertion before any implementation below

- [ ] T021 [P] [US1] Failing unit tests in `crates/zmachine-core/src/text.rs` (§3): the three alphabets, shift characters, abbreviations from all three tables, the ten-bit ZSCII escape including at the end of a string, the end bit, ZSCII to Unicode on output (§3.8), encoding a word to dictionary form with padding and truncation to six Z-characters, and text running past the end of the file as an error
- [ ] T022 [P] [US1] Failing unit tests in `crates/zmachine-core/src/ops_math.rs` (§2, §15): `add`, `sub`, `mul`, `div`, `mod` with signed 16-bit wraparound and sign rules; division and modulo by zero as faults; `and`, `or`, `not`, `test`; `inc`, `dec`, `inc_chk`, `dec_chk`; `je` with two, three and four operands; `jl`, `jg`, `jz`
- [ ] T023 [P] [US1] Failing unit tests in `crates/zmachine-core/src/exec.rs` (§4.2, §5, §6, §15): `call` with zero to three arguments, locals initialised from the routine header, calling address 0 returns false; `ret`, `rtrue`, `rfalse`, `ret_popped`, `jump`, `nop`, `pop`, `push`, `pull`; reading and writing stack, local and global variables; `store`, `load` and the in-place rule for the stack (§6.3.4); `loadw`, `loadb`, `storew`, `storeb`; faults for unknown instruction, stack underflow below a frame's base, invalid variable number, a call stack deeper than 1024 frames, and a write outside dynamic memory, each carrying the instruction address
- [ ] T024 [P] [US1] Failing unit tests in `crates/zmachine-core/src/objects.rs` and `ops_object.rs` (§12, §15): `get_parent`, `get_child`, `get_sibling`, `jin`; `insert_obj` and `remove_obj` preserving sibling chains, including moving a first, middle and last child; `test_attr`, `set_attr`, `clear_attr` across all 32 attributes; `get_prop` with defaults; `put_prop` for one and two byte properties, and a missing property as a fault; `get_prop_addr`; `get_prop_len` including address 0 giving 0; `get_next_prop`; each instruction given object 0 does nothing or yields 0 and records a diagnostic (research R12)
- [ ] T025 [P] [US1] Failing unit tests in `crates/zmachine-core/src/dictionary.rs` (§13, §15 `sread`): lookup of the first, last and a missing entry; word separators as their own tokens; text and parse buffer layout for version 3; more words than the parse buffer holds; an empty line; input stored in lower case
- [ ] T026 [P] [US1] Failing tests in `crates/zmachine-core/tests/events.rs` (contracts/core-api.md): `run` returns `LineWanted` carrying text buffer byte 0 minus 1 (§15 `read`); `provide_line` truncates to that length and resumes; `quit` gives `Finished`; the budget stops an endless loop with `BudgetExhausted` and the event sequence and output are identical for budgets of 1, 100 and unlimited; `run` while an answer is owed, an answer out of turn, and `run` after `Finished` each return `HostError` and change nothing; draining with the `take_` methods does not affect execution; after a `Fault` a second `Machine` loads and runs normally (FR-018)
- [ ] T027 [P] [US1] Failing tests in `crates/zmachine-core/tests/streams.rs` (§7, §8, §15): `print`, `print_ret`, `new_line`, `print_char`, `print_num` with negatives, `print_addr`, `print_paddr`, `print_obj` appear in `take_output`; player input does not; the status line is produced before each read and by `show_status`, for a score game and a time game (§8.2); `output_stream` deselecting and reselecting stream 1 (§7.4); text printed with the upper window selected stays out of `take_output` (`split_window`, `set_window`); `input_stream` and `sound_effect` are accepted; `verify` on a good and a bad checksum; `restart` returns to the opening state with the interpreter-owned header fields re-applied and Flags 2 preserved (§6.1.3)
- [ ] T028 [P] [US1] Failing tests in `crates/zmachine-cli/tests/play.rs`: `zm` on a built image prints its text, waits for a line, responds, and exits 0 on quit; `zm stories/zork1/zork1.z3` with `open mailbox` and an unknown word on standard input gives the game's replies (spec US1 scenarios 1 to 3); a version 5 header, a truncated file and an unrelated file each exit 2 with a reason on standard error; a faulting image exits 1 with the fault kind and address on standard error; `--max-instructions` stops an endless loop with exit 1 (FR-032)

### Implementation for User Story 1

- [ ] T029 [US1] Implement text decoding and encoding in `crates/zmachine-core/src/text.rs` until T021 passes
- [ ] T030 [US1] Implement the execution loop in `crates/zmachine-core/src/exec.rs`: call frames, stack, variables, control-flow and memory instructions, `run(budget)`, faults, and the run states of data-model.md, until T023 passes
- [ ] T031 [P] [US1] Implement arithmetic, logic and comparison instructions in `crates/zmachine-core/src/ops_math.rs` until T022 passes
- [ ] T032 [P] [US1] Implement the object tree in `crates/zmachine-core/src/objects.rs` and its instructions in `ops_object.rs`, with the diagnostic for object 0, until T024 passes
- [ ] T033 [P] [US1] Implement dictionary lookup and tokenisation in `crates/zmachine-core/src/dictionary.rs` until T025 passes
- [ ] T034 [US1] Implement output, the screen buffer, stream 1 selection, windows, the status line, `sread`, `verify`, `restart` and `quit` in `crates/zmachine-core/src/ops_io.rs`, wiring `ops_math`, `ops_object` and `dictionary` into the dispatch in `exec.rs`, until T026 and T027 pass. `save` and `restore` take their failure branch until US5
- [ ] T035 [US1] Implement the driver and word wrapping in `crates/zmachine-cli/src/lib.rs` and interactive mode, `--width`, `--max-instructions` and exit statuses in `src/main.rs`, per contracts/cli.md, until T028 passes
- [ ] T036 [US1] Play the opening of Zork 1 by hand with `zm stories/zork1/zork1.z3`; for each defect found, add a failing test in the matching file first, then fix it

**Checkpoint**: A version 3 game is playable from the terminal

---

## Phase 5: User Story 2 - Run a scripted session and get a repeatable transcript (Priority: P2)

**Goal**: Unattended runs from a script with a seed, writing identical transcript files
every time

**Independent Test**: Run the same story file, script and seed into two directories;
`diff -r` reports no difference

### Tests for User Story 2 ⚠️

- [ ] T037 [P] [US2] Failing tests in `crates/zmachine-core/tests/random.rs` (§2.4, §15 `random`; research R2): for each fixture from T015 the sequence equals the dfrotz fixture, including host seeds below 1000; `random` with 0, with -1 to -999 (counter sequence, wrapping at the interval), with -1000 and beyond, and with -32768 without overflow; each returns 0; the generator is reseeded from the host seed on restart and untouched by a failed restore; with no seed, two different entropy values give different sequences
- [ ] T038 [P] [US2] Failing tests in `crates/zmachine-core/tests/streams.rs` for the transcript stream (§7.1.2, §7.4; contracts/transcript-format.md): setting Flags 2 bit 0 by `storew` or by `output_stream 2` raises `TranscriptRequested` before the next output or read; declining clears the bit; while on, `take_transcript` yields the game's text unwrapped plus each input line and a line feed, in the form settled by T016; turning it off stops it; Flags 2 bit 0 tracks the stream state
- [ ] T039 [P] [US2] Failing tests in `crates/zmachine-cli/tests/script.rs`: `--script` feeds lines in order; the line after the game starts a transcript is consumed as the file name and the file is written inside `--dir`; a name with a path separator is refused and the game is told transcription is off; ten runs of a built image that uses `random`, with the same script and seed, give ten identical transcript files (SC-003); changing `--width` does not change the transcript file; a script that ends while input is wanted exits 0 and the transcript ends with the prompt and no line feed; two unseeded runs of an image that prints random numbers differ (FR-014)

### Implementation for User Story 2

- [ ] T040 [US2] Implement the generator in `crates/zmachine-core/src/random.rs` and wire `random`, load and restart to it until T037 passes
- [ ] T041 [US2] Implement stream 2, the `TranscriptRequested` event and `take_transcript` in `crates/zmachine-core/src/ops_io.rs` until T038 passes
- [ ] T042 [US2] Implement `--script`, `--seed`, `--dir`, file-name handling and transcript writing in `crates/zmachine-cli/src/lib.rs` and `main.rs`, taking entropy from the operating system when no seed is given, until T039 passes

**Checkpoint**: Scripted runs are repeatable and write transcripts in the reference's format

---

## Phase 6: User Story 3 - Pass the standard conformance test program (Priority: P3)

**Goal**: czech for version 3 reports zero failed tests

**Independent Test**: `cargo test -p zmachine-cli --test czech`

### Tests for User Story 3 ⚠️

- [ ] T043 [US3] Compare `stories/czech/czech.out3` with the output of `dfrotz` on `stories/czech/czech.z3` and note in `stories/czech/SOURCE.md` which lines are interpreter-specific (interpreter number, header flags, formatting) and so are excluded from comparison
- [ ] T044 [US3] Failing test in `crates/zmachine-cli/tests/czech.rs`: verifies `czech.z3` against its recorded digest; runs it; fails naming each test czech reports as failed, except those with a `czech` rule in `docs/divergences.md`; compares the print-test output with `czech.out3` outside the lines excluded in T043; fails on a `czech` rule that matches no failing test

### Implementation for User Story 3

- [ ] T045 [US3] For each failure reported by T044, read the Standard 1.1 section: if the interpreter is wrong, add a failing unit test in the matching source file and fix it; if czech conflicts with Standard 1.1, add a `czech` rule to `docs/divergences.md` citing the section; if neither is clear, stop for the project owner. Repeat until T044 passes (SC-001)

**Checkpoint**: The instruction set is conformant by an independent measure

---

## Phase 7: User Story 4 - Match the reference interpreter on a full game (Priority: P4)

**Goal**: The Zork 1 walkthrough transcript matches Frotz 2.55 byte for byte

**Independent Test**: `cargo test -p zmachine-cli --test fidelity_zork1`

### Tests for User Story 4 ⚠️

- [ ] T046 [P] [US4] Failing tests in `crates/zmachine-cli/tests/ledger.rs` using synthetic inputs: parsing both rule kinds from a sample ledger; applying a `transcript` rule substitutes exactly the named text after the named command; a rule that matches nothing is an error (SC-008); the comparison of two differing texts reports the byte offset, the preceding command and context lines from both sides (spec US4 scenario 2); a malformed rule is an error
- [ ] T047 [US4] Failing test in `crates/zmachine-cli/tests/fidelity_zork1.rs`: verifies `stories/zork1/zork1.z3` against the digest in `SOURCE.md`; runs `zm` on `walkthrough.txt` with the seed from `seed.txt`; applies the ledger; compares the transcript file byte for byte with `reference.scr`, reporting as in T046; asserts the transcript shows the game won with 350 points
- [ ] T048 [P] [US4] Tests in `crates/zmachine-cli/tests/fidelity_zork1.rs` marked `#[ignore = "needs dfrotz 2.55"]`: check `dfrotz -v` reports 2.55, regenerate the reference transcript, and fail if it differs from `reference.scr` (FR-028); and, marked `#[ignore = "timing; run with --release"]`, the walkthrough completes in under five seconds (SC-005)

### Implementation for User Story 4

- [ ] T049 [US4] Implement ledger parsing, rule application and difference reporting in `crates/zmachine-cli/src/lib.rs` until T046 passes
- [ ] T050 [US4] Work through each difference T047 reports, earliest first: if the interpreter is wrong, add a failing unit test in the matching source file and fix it; if Frotz departs from Standard 1.1, add a `transcript` rule to `docs/divergences.md` citing the section; if neither is clear, stop for the project owner. Repeat until T047 passes (SC-002)
- [ ] T051 [P] [US4] Add `.github/workflows/ci.yml` for macOS and Linux: `cargo test --workspace`; then install dfrotz, and run `cargo test --workspace --release -- --ignored`

**Checkpoint**: The project's headline definition of faithful is met

---

## Phase 8: User Story 5 - Save, restore and restart (Priority: P5)

**Goal**: Positions can be saved and restored in the standard format, interchangeably
with Frotz

**Independent Test**: `cargo test -p zmachine-cli --test save_restore`

### Tests for User Story 5 ⚠️

- [ ] T052 [US5] Create reference artefacts under `stories/zork1/fidelity/saves/` with dfrotz, for five points spread through the walkthrough: for each point `n`, a script `make_n.txt` (the walkthrough up to that point, then `save` and a file name), the Frotz save `frotz_n.qzl`, a continuation script `cont_n.txt` (`script`, a transcript name, `restore`, the save name, then about twenty commands), and the Frotz transcript `cont_n.scr` from a fresh run with the pinned seed. Extend `scripts/regenerate-references.sh` to rebuild them, and commit all of them
- [ ] T053 [P] [US5] Failing unit tests in `crates/zmachine-core/src/quetzal.rs` (Q§3 to Q§5): a save has `IFhd` of 13 bytes plus pad, `CMem` and `Stks`; the program counter in `IFhd` points at the `save` instruction's branch data (Q§5.8.1); `Stks` begins with the dummy frame (Q§4.11); `CMem` "Decoded data shorter than dynamic memory is accepted, the rest unchanged; longer is refused"; `UMem` "Must be exactly the dynamic memory size"; `IFhd` "Identity must equal the loaded story's, otherwise refused"; `Stks` "Must parse completely"; each committed `frotz_n.qzl` parses; a truncated or corrupted save is refused
- [ ] T054 [P] [US5] Failing tests in `crates/zmachine-core/tests/save.rs` (§15 `save`, `restore`; §6.1.2): `SaveRequested` carries the bytes; `save_result(true)` and `(false)` take the success and failure branches; `provide_restore(None)` and a refused save take the failure branch and leave memory, stack and program counter byte-for-byte unchanged (FR-017); a successful restore from inside nested calls resumes in the saved `save` instruction with its branch taken, with the interpreter-owned header fields re-applied and the current Flags 2 kept; the random state is not changed by restore
- [ ] T055 [P] [US5] Failing tests in `crates/zmachine-cli/tests/save_restore.rs`: for each of the five points, `zm` running `cont_n.txt` with `frotz_n.qzl` produces a transcript identical to `cont_n.scr`; `zm` running `make_n.txt` writes a save, and `zm` running `cont_n.txt` with that save also produces `cont_n.scr` (SC-006); a save from a different story file is refused and play continues; an unwritable `--dir` makes the game report failure; and, marked `#[ignore = "needs dfrotz 2.55"]`, dfrotz running `cont_n.txt` with each save written by `zm` produces `cont_n.scr` (FR-031)

### Implementation for User Story 5

- [ ] T056 [US5] Implement Quetzal writing and reading in `crates/zmachine-core/src/quetzal.rs` until T053 passes
- [ ] T057 [US5] Replace the placeholder failure branches for `save` and `restore` in `crates/zmachine-core/src/ops_io.rs` with the events and answers of contracts/core-api.md until T054 passes
- [ ] T058 [US5] Implement save and restore file handling in `crates/zmachine-cli/src/lib.rs` until T055 passes, then confirm `crates/zmachine-cli/tests/fidelity_zork1.rs` still passes

**Checkpoint**: All five user stories work

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Evidence for the remaining success criteria, and documentation

- [ ] T059 [P] Add `crates/zmachine-cli/tests/malformed.rs`: at least twenty malformed or faulting images built with the testkit, covering every `Fault` kind and every `LoadError`, each giving a diagnostic on standard error and a non-zero exit status with no panic (SC-007)
- [ ] T060 [P] Add `crates/zmachine-core/tests/coverage.rs`: a list of all 68 version 3 instructions transcribed from Standard §14; for each, the test fails unless the core's source and tests contain a test function whose name includes the instruction's name and whose body cites a "§" section, found by scanning the files (SC-004, FR-027); and a check that `crates/zmachine-core/Cargo.toml` has no dependencies, `src/lib.rs` forbids `unsafe`, and no file under `crates/zmachine-core/src/` mentions `std::fs`, `std::io`, `std::net`, `std::time`, `std::env` or `std::process` (FR-006, Principle V)
- [ ] T061 [P] Update `README.md`: status, how to build, how to run `zm`, how to run the tests, and the prerequisites from quickstart.md
- [ ] T062 Run every step of `specs/001-v3-core/quickstart.md` on a fresh clone and correct the document or the code wherever they disagree
- [ ] T063 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` and fix what they report

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: needs Setup; blocks every user story. T015 and T016 need only T001, T002, T008 and dfrotz
- **Walkthrough (Phase 3)**: needs only T002 and T005; runs alongside Phases 4 to 6
- **US1 (Phase 4)**: needs Foundational
- **US2 (Phase 5)**: needs US1, T015 and T016
- **US3 (Phase 6)**: needs US2 and T003
- **US4 (Phase 7)**: needs US2 and the Walkthrough; best after US3, since czech pinpoints faults that the walkthrough shows only as a mismatch
- **US5 (Phase 8)**: needs US2 and the Walkthrough (T052 cuts its save points from it)
- **Polish (Phase 9)**: after the stories it covers

### User Story Dependencies

These stories build on one another more than is usual, because each is a stricter test of
the same interpreter. Only US1 stands entirely alone.

### Within Each User Story

- Tests are written and fail before implementation
- Core before CLI
- A story's checkpoint is reached before the next story begins

### Parallel Opportunities

- T002 to T005
- T009 to T011, and T015 and T016 with each other and with T009 to T014
- All of Phase 3 against Phases 4 to 6
- T021 to T028: separate files
- T031, T032 and T033 after T030: separate files
- T037 to T039; T053 to T055; T059 to T061

`crates/zmachine-core/src/exec.rs` is edited by T030 and T034 only, and
`ops_io.rs` by T034, T041 and T057; those run one at a time.

---

## Parallel Example: User Story 1

```text
# Write all failing tests for User Story 1 together:
T021 text.rs        T022 ops_math.rs     T023 exec.rs
T024 objects.rs     T025 dictionary.rs   T026 tests/events.rs
T027 tests/streams.rs                    T028 cli tests/play.rs

# After T030, in parallel:
T031 ops_math.rs    T032 objects.rs + ops_object.rs    T033 dictionary.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phases 1 and 2
2. Phase 4
3. Stop and play Zork 1 by hand; this is the first thing worth showing

### Incremental Delivery

1. US1: a game runs
2. US2: runs are scripted, repeatable, and write transcripts
3. US3: czech passes
4. US4: Zork 1 matches Frotz, the headline result
5. US5: saves interchange with Frotz
6. Polish: the remaining evidence

### Points where work stops for the project owner

- T003, if czech's license does not read as permitting redistribution
- T018, if the game cannot be won at a workable seed
- T045 and T050, when a difference is neither clearly our fault nor clearly a departure
  from the Standard by czech or Frotz

---

## Notes

- Commit after each task or small group of tasks
- A test is never weakened, skipped or deleted to make a task pass (Principle II)
- Frotz source may be read to learn how the reference behaves; none of its text is copied
  into this repository
- T045 and T050 are loops, not single edits; each defect found inside them gets its own
  failing test and its own commit
