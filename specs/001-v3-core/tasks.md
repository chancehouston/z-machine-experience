# Tasks: Faithful Z-Machine Version 3 Core

**Input**: Design documents from `/specs/001-v3-core/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable: in every
phase the test tasks are written and seen to fail before the implementation tasks beneath
them. Every test of interpreter behavior carries a comment citing the section of the
Z-Machine Standard 1.1 it verifies (written "§" below).

**Organization**: Tasks are grouped by user story so each story can be built and tested
on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US5)

## Path Conventions

- Core library: `crates/zmachine-core/`
- Command-line host: `crates/zmachine-cli/`
- Paths are relative to the repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: A workspace that builds and an environment that can run the reference tools

- [ ] T001 Create the Cargo workspace in `Cargo.toml` (resolver 3, edition 2024, members `crates/zmachine-core` and `crates/zmachine-cli`) and commit `Cargo.lock`
- [ ] T002 [P] Create `crates/zmachine-core/Cargo.toml` with an empty `[dependencies]` table and `crates/zmachine-core/src/lib.rs` starting with `#![forbid(unsafe_code)]` and a module list matching plan.md
- [ ] T003 [P] Create `crates/zmachine-cli/Cargo.toml` (binary named `zm`, depends only on `zmachine-core`; dev-dependency `sha2`) and a `crates/zmachine-cli/src/main.rs` that prints usage and exits with status 2
- [ ] T004 [P] Add `stories/zork1/zork1.z3` from `historicalsource/zork1` (`COMPILED/zork1.z3`) at a specific commit using `git add -f`, with `stories/zork1/LICENSE` (that repository's license text) and `stories/zork1/SOURCE.md` recording the URL, commit, SHA-256, release number and serial number read from the header
- [ ] T005 [P] Write `scripts/fetch-stories.sh` to download `czech_0_8.zip` from the IF Archive into `stories/local/`, unpack it, and verify a recorded SHA-256; record in `specs/001-v3-core/research.md` R4 whether the archive contains a version 3 build and an expected-output file
- [ ] T006 [P] Create `docs/divergences.md` with the entry format from research.md R10 and no entries
- [ ] T007 Install dfrotz 2.55 (`brew install frotz`), confirm `dfrotz -v` reports 2.55, and record the exact version string in `specs/001-v3-core/research.md` R1

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The pieces every story needs: a way to build test story files, and the
header, memory and instruction decoder

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement the in-memory story builder in `crates/zmachine-core/src/testkit.rs` (behind a `testkit` feature): builds a valid version 3 image with header, globals, an object table, a dictionary, abbreviations and caller-supplied instruction bytes, with helpers to encode each operand form (§4) and Z-encoded strings (§3)
- [ ] T009 [P] Write failing tests in `crates/zmachine-core/tests/header.rs` for load validation (§11): "Must be 3, otherwise load is refused"; "Each must lie inside the file; static base must be at least 64"; length "If non-zero must not exceed the file size"; a wrong checksum is not a load error; interpreter-owned flag bits are set after load (§11.1)
- [ ] T010 [P] Write failing tests in `crates/zmachine-core/tests/memory.rs` for the memory map (§1): byte and word reads anywhere in the file, writes allowed only below the static base, each violation reported as the matching Fault with the instruction address
- [ ] T011 [P] Write failing tests in `crates/zmachine-core/tests/decode.rs` for instruction decoding (§4): long, short and variable forms, all operand type combinations, store byte, branch offsets of one and two bytes including the return-false and return-true offsets 0 and 1, and inline text
- [ ] T012 Implement `LoadError` and header parsing in `crates/zmachine-core/src/header.rs` until T009 passes
- [ ] T013 Implement memory access and the write rule in `crates/zmachine-core/src/memory.rs` until T010 passes
- [ ] T014 Implement the instruction decoder in `crates/zmachine-core/src/decode.rs` until T011 passes
- [ ] T015 Define the public types from data-model.md and contracts/core-api.md in `crates/zmachine-core/src/lib.rs`: `Machine`, `Event`, `Fault` (all ten kinds, each with an address), `StatusLine`, `HostError`, load options with `seed` and `entropy`

**Checkpoint**: Images can be built, loaded, validated and decoded; nothing executes yet

---

## Phase 3: User Story 1 - Play a version 3 game from the command line (Priority: P1) 🎯 MVP

**Goal**: A version 3 story file runs: opening text, commands in, responses out, quit

**Independent Test**: `cargo run -p zmachine-cli -- stories/zork1/zork1.z3`, then
`open mailbox`, `read leaflet`, `quit`, `y`; each response is correct and the exit status
is 0

### Tests for User Story 1 ⚠️

> Write these first and confirm they fail before any implementation task below

- [ ] T016 [P] [US1] Failing tests in `crates/zmachine-core/tests/text.rs` (§3): the three alphabets, shift characters, abbreviations in all three tables, the ten-bit ZSCII escape including at the end of a string, the end bit, ZSCII to Unicode output, and encoding a word to dictionary form with padding and truncation to six Z-characters
- [ ] T017 [P] [US1] Failing tests in `crates/zmachine-core/tests/arithmetic.rs` (§15, §2): `add`, `sub`, `mul`, `div`, `mod` with signed 16-bit wraparound and sign rules, division and modulo by zero as faults, `and`, `or`, `not`, `inc`, `dec`, `inc_chk`, `dec_chk`
- [ ] T018 [P] [US1] Failing tests in `crates/zmachine-core/tests/control.rs` (§5, §6, §15): `call` with zero to three arguments, local defaults from the routine header, calling address 0 returns false, `ret`, `rtrue`, `rfalse`, `ret_popped`, `jump`, `jz`, `je` with two to four operands, `jl`, `jg`, `test`, `nop`, `pop`, `push`, `pull`, stack underflow below a frame's base as a fault, and `quit` returning `Event::Finished`
- [ ] T019 [P] [US1] Failing tests in `crates/zmachine-core/tests/variables.rs` (§4.2, §6.3, §15): reading and writing stack, local and global variables, `store`, `load`, the in-place rule for indirect references to the stack, invalid local number as a fault, `loadw`, `loadb`, `storew`, `storeb`
- [ ] T020 [P] [US1] Failing tests in `crates/zmachine-core/tests/objects.rs` (§12, §15): `get_parent`, `get_child`, `get_sibling`, `jin`, `insert_obj` and `remove_obj` preserving sibling chains, `test_attr`, `set_attr`, `clear_attr` across all 32 attributes, `get_prop` with defaults, `put_prop` for one and two byte properties and a missing property as a fault, `get_prop_addr`, `get_prop_len` including address 0, `get_next_prop`, `print_obj`, object 0 handling
- [ ] T021 [P] [US1] Failing tests in `crates/zmachine-core/tests/dictionary.rs` (§13, §15 `sread`): lookup of the first, last and a missing entry, word separators as their own tokens, text and parse buffer layout for version 3, input longer than the text buffer, more words than the parse buffer holds, an empty line, lowercasing, and the status line being produced before input
- [ ] T022 [P] [US1] Failing tests in `crates/zmachine-core/tests/output.rs` (§7, §8, §15): `print`, `print_ret`, `new_line`, `print_char`, `print_num` with negatives, `print_addr`, `print_paddr`, `show_status` for a score game and a time game, `split_window` and `set_window` keeping upper-window text out of the main output, `output_stream`, `input_stream` and `sound_effect` accepted without effect, `verify` on a good and a bad checksum
- [ ] T023 [P] [US1] Failing tests in `crates/zmachine-core/tests/events.rs` (contracts/core-api.md): `run` returns `LineWanted` with the maximum length, `provide_line` resumes, the budget stops an endless loop with `BudgetExhausted` and the result is identical for any budget size, calling `run` while an answer is owed returns `HostError` and changes nothing, `take_output` and `take_status` do not affect execution, `restart` returns to the opening state
- [ ] T024 [P] [US1] Failing end-to-end test in `crates/zmachine-cli/tests/play.rs`: `zm` on a tiny built image prints its text and exits 0; a version 5 header, a truncated file and an unrelated file each exit 2 with a reason on standard error (spec US1 scenario 5)

### Implementation for User Story 1

- [ ] T025 [US1] Implement text decoding and encoding in `crates/zmachine-core/src/text.rs` until T016 passes
- [ ] T026 [US1] Implement the execution loop, call frames, stack and variable access in `crates/zmachine-core/src/exec.rs`, with `run(budget)` and the Event and state transitions from data-model.md, until T018, T019 and T023 pass (save and restore instructions take their failure branch until US5)
- [ ] T027 [US1] Implement arithmetic and logic instructions in `crates/zmachine-core/src/exec.rs` until T017 passes
- [ ] T028 [P] [US1] Implement the object tree, attributes and properties in `crates/zmachine-core/src/objects.rs` and wire their instructions into `exec.rs` until T020 passes
- [ ] T029 [P] [US1] Implement dictionary lookup and tokenisation in `crates/zmachine-core/src/dictionary.rs` and wire `sread` into `exec.rs` until T021 passes
- [ ] T030 [US1] Implement output instructions, the output buffer, the status line and window handling in `crates/zmachine-core/src/exec.rs` until T022 passes
- [ ] T031 [US1] Implement the blocking driver in `crates/zmachine-cli/src/driver.rs` and interactive mode with exit statuses 0, 1 and 2 in `crates/zmachine-cli/src/main.rs` per contracts/cli.md until T024 passes
- [ ] T032 [US1] Play the opening of Zork 1 by hand with `zm stories/zork1/zork1.z3` and fix any fault, adding a failing test first for each defect found

**Checkpoint**: A version 3 game is playable from the terminal

---

## Phase 4: User Story 2 - Run a scripted session and get a repeatable transcript (Priority: P2)

**Goal**: Unattended runs from a command script with a seed, producing identical
transcripts every time

**Independent Test**: Run the same story file, script and seed twice; `cmp` reports no
difference

### Tests for User Story 2 ⚠️

- [ ] T033 [P] [US2] Build a tiny story image that prints a run of `random` results, run it under dfrotz with `-s` at seeds 1, 999, 1000 and 123456, and commit the observed outputs as fixtures in `crates/zmachine-core/tests/fixtures/random/`; correct research.md R2 if dfrotz disagrees with it
- [ ] T034 [US2] Failing tests in `crates/zmachine-core/tests/random.rs` (§2.4, §15 `random`): the sequence for each fixture seed equals the dfrotz fixture; a negative argument reseeds and returns 0; arguments 1 to 999 after reseeding give the counter sequence; zero reseeds from the host seed; the generator is reseeded on restart; without a seed two different entropy values give different sequences
- [ ] T035 [US2] Run dfrotz on a small built image and on the opening of Zork 1 and record the six answers in the "To be established by observation" section of `specs/001-v3-core/contracts/transcript-format.md`, each with the command that shows it; stop and raise with the project owner if any output cannot be derived from core events
- [ ] T036 [US2] Failing tests in `crates/zmachine-cli/tests/dumbterm.rs` for the conventions recorded in T035: wrapping at 80 and at a narrow width, a word ending exactly at the width, trailing spaces, width 0 disabling wrapping, ASCII transliteration, prompt and input handling, end-of-input and quit output
- [ ] T037 [P] [US2] Failing test in `crates/zmachine-cli/tests/determinism.rs`: ten runs of a built image that uses `random` with the same script and seed give ten identical transcripts (SC-003); a script that ends while input is wanted exits 0 with everything printed so far

### Implementation for User Story 2

- [ ] T038 [US2] Implement the generator in `crates/zmachine-core/src/random.rs` as described in research.md R2 and wire `random`, load and restart to it until T034 passes
- [ ] T039 [US2] Implement the dumb-terminal formatter in `crates/zmachine-cli/src/dumbterm.rs` until T036 passes
- [ ] T040 [US2] Implement `--script`, `--seed` and `--width` in `crates/zmachine-cli/src/main.rs` and `driver.rs`, with unseeded runs taking entropy from the operating system in the CLI only, until T037 passes

**Checkpoint**: Scripted runs are repeatable and formatted like the reference

---

## Phase 5: User Story 3 - Pass the standard conformance test program (Priority: P3)

**Goal**: czech for version 3 reports zero failed tests

**Independent Test**: `cargo test -p zmachine-cli --test czech`

### Tests for User Story 3 ⚠️

- [ ] T041 [US3] Failing test in `crates/zmachine-cli/tests/czech.rs`: runs `stories/local/czech.z3`, parses czech's summary, fails naming each failed test, and is reported as ignored with a reason when the file is absent (FR-025)

### Implementation for User Story 3

- [ ] T042 [US3] If T005 found no version 3 build in the archive, compile `czech.inf` for version 3 with Inform 6 into `stories/local/czech.z3` and document the command in `scripts/fetch-stories.sh`; if that is not possible, stop and ask the project owner
- [ ] T043 [US3] For each test czech reports as failed: check the Standard 1.1 section; if the interpreter is wrong, add a failing unit test in the matching `crates/zmachine-core/tests/` file and fix it; if czech conflicts with Standard 1.1, add an entry to `docs/divergences.md` and exclude only that test in `crates/zmachine-cli/tests/czech.rs`
- [ ] T044 [US3] Confirm T041 passes with zero unexplained failures (SC-001)

**Checkpoint**: The instruction set is conformant by an independent measure

---

## Phase 6: User Story 4 - Match the reference interpreter on a full game (Priority: P4)

**Goal**: A full Zork 1 walkthrough matches dfrotz 2.55 byte for byte

**Independent Test**: `cargo test -p zmachine-cli --test fidelity_zork1`

### Tests for User Story 4 ⚠️

- [ ] T045 [US4] Choose a seed of 1000 or more and write `tests/fidelity/zork1.walkthrough` against dfrotz (`dfrotz -m -p -q -w 80 -s <seed>`), building it up section by section until the game is won with 350 points; write the commands ourselves and do not copy a published walkthrough; record the seed in a comment-free sidecar `tests/fidelity/zork1.seed`
- [ ] T046 [US4] Generate `tests/fidelity/zork1.reference.txt` with dfrotz from the story file, walkthrough, seed and width 80, and commit it as generated
- [ ] T047 [US4] Failing test in `crates/zmachine-cli/tests/fidelity_zork1.rs`: verifies the story file's SHA-256 against `stories/zork1/SOURCE.md`, runs `zm` on the walkthrough, applies the rules parsed from `docs/divergences.md` to the reference, compares byte for byte, and on a difference reports the byte offset, the preceding command and a few lines of context from both sides (spec US4 scenario 2); fails on any ledger rule that matches nothing (SC-008); asserts the transcript ends with the game won at the maximum score
- [ ] T048 [P] [US4] Test in `crates/zmachine-cli/tests/fidelity_zork1.rs` that regenerates the reference with dfrotz and fails if it differs from the committed file; reported as ignored with a reason when dfrotz is not installed (FR-028)
- [ ] T049 [P] [US4] Failing test in `crates/zmachine-cli/tests/fidelity_zork1.rs` that the walkthrough completes in under five seconds in a release build (SC-005)

### Implementation for User Story 4

- [ ] T050 [US4] Work through each difference T047 reports, earliest first: if the interpreter or formatter is wrong, add a failing unit test in the matching test file and fix it; if Frotz departs from Standard 1.1, add a ledger entry to `docs/divergences.md` citing the section; if neither is clear, stop and ask the project owner
- [ ] T051 [US4] Confirm T047, T048 and T049 pass (SC-002, SC-005, SC-008)
- [ ] T052 [P] [US4] Add `.github/workflows/ci.yml` running `cargo test --workspace` on macOS and Linux, installing dfrotz and running `scripts/fetch-stories.sh` first, so determinism is shown on both platforms (SC-003)

**Checkpoint**: The project's headline definition of faithful is met

---

## Phase 7: User Story 5 - Save, restore and restart (Priority: P5)

**Goal**: Positions can be saved and restored in the standard format

**Independent Test**: `cargo test -p zmachine-cli --test save_restore`

### Tests for User Story 5 ⚠️

- [ ] T053 [P] [US5] Failing tests in `crates/zmachine-core/tests/quetzal.rs` (Quetzal 1.4; §15 `save`, `restore`): a save produces `IFhd`, `CMem` and `Stks`; `IFhd` "Identity must equal the loaded story's, otherwise refused"; memory "Length must equal the dynamic memory size"; `Stks` "Must parse completely"; `UMem` is accepted; round trip from inside nested calls restores every frame; a truncated or corrupted save is refused and the machine is byte-for-byte unchanged (FR-017)
- [ ] T054 [P] [US5] Failing tests in `crates/zmachine-core/tests/events.rs` for the save and restore events: `SaveRequested` carries the bytes, `save_result(false)` takes the failure branch, `provide_restore(None)` takes the failure branch, an answer given out of turn returns `HostError`
- [ ] T055 [P] [US5] Failing test in `crates/zmachine-cli/tests/save_restore.rs`: at five points in the Zork 1 walkthrough, save, play different commands, restore, and continue; the remaining transcript equals the uninterrupted run (SC-006); restoring a save from a different story file is refused and play continues; an unwritable save directory makes the game report failure

### Implementation for User Story 5

- [ ] T056 [US5] Implement Quetzal writing and reading in `crates/zmachine-core/src/quetzal.rs` until T053 passes
- [ ] T057 [US5] Replace the placeholder failure branches for `save` and `restore` in `crates/zmachine-core/src/exec.rs` with the events and answers from contracts/core-api.md until T054 passes
- [ ] T058 [US5] Implement `--save-dir` and save file handling (`<story-name>.qzl`) in `crates/zmachine-cli/src/driver.rs` and `main.rs` until T055 passes
- [ ] T059 [US5] Re-run `crates/zmachine-cli/tests/fidelity_zork1.rs` and confirm it still passes with real save and restore in place

**Checkpoint**: All five user stories work independently

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Evidence for the remaining success criteria, and documentation

- [ ] T060 [P] Add `crates/zmachine-cli/tests/malformed.rs`: at least twenty malformed or faulting images built with the story builder, covering every Fault kind in data-model.md and every `LoadError`, each producing a diagnostic and a non-zero exit status with no panic (SC-007)
- [ ] T061 [P] Add `crates/zmachine-core/tests/coverage.rs`: a table of every version 3 instruction in §14 mapped to the test that covers it, failing if any instruction has no entry (SC-004, FR-027)
- [ ] T062 [P] Add a test in `crates/zmachine-core/tests/coverage.rs` that `crates/zmachine-core/Cargo.toml` has no dependencies and `src/lib.rs` forbids `unsafe` (constitution Principle V)
- [ ] T063 [P] Update `README.md`: status, how to build, how to run `zm`, how to run the tests, and the prerequisites from quickstart.md
- [ ] T064 Run every step of `specs/001-v3-core/quickstart.md` on a clean checkout and correct the document or the code wherever they disagree
- [ ] T065 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` and fix what they report

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: needs Setup; blocks every user story
- **US1 (Phase 3)**: needs Foundational
- **US2 (Phase 4)**: needs US1 (a game must run before it can be scripted)
- **US3 (Phase 5)**: needs US2 (czech is run as a scripted session)
- **US4 (Phase 6)**: needs US2; best done after US3, since czech pinpoints faults that the walkthrough only shows as a mismatch
- **US5 (Phase 7)**: needs US1 and US2; T055 and T059 also need the walkthrough from US4
- **Polish (Phase 8)**: after the stories it covers

### User Story Dependencies

These stories build on one another more than is usual, because each is a stricter test of
the same interpreter. US1 is the only one that stands entirely alone.

### Within Each User Story

- Tests are written and fail before implementation
- Core before CLI
- A story's checkpoint is reached before the next story begins

### Parallel Opportunities

- T002 to T006 in Setup
- T009 to T011 in Foundational
- All of T016 to T024: nine test files, no shared code beyond the story builder
- T028 and T029 once T026 is in place
- T053 to T055
- T060 to T063

Tasks that edit `crates/zmachine-core/src/exec.rs` (T026, T027, T030, T057) must run one
at a time.

---

## Parallel Example: User Story 1

```text
# Write all failing tests for User Story 1 together:
T016 text.rs      T017 arithmetic.rs   T018 control.rs
T019 variables.rs T020 objects.rs      T021 dictionary.rs
T022 output.rs    T023 events.rs       T024 play.rs (CLI)

# Then, after T026, in parallel:
T028 objects.rs implementation
T029 dictionary.rs implementation
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 and Phase 2
2. Phase 3
3. Stop and play Zork 1 by hand; this is the first thing worth showing

### Incremental Delivery

1. US1: a game runs
2. US2: runs are scripted and repeatable
3. US3: czech passes
4. US4: Zork 1 matches dfrotz, the headline result
5. US5: save and restore
6. Polish: the remaining evidence

### Points where work stops for the project owner

- T035, if dfrotz emits something that cannot be derived from core events
- T042, if czech cannot be built for version 3
- T050, when a difference is neither clearly our fault nor clearly a Frotz departure

---

## Notes

- Commit after each task or small group of tasks
- A test is never weakened, skipped or deleted to make a task pass (constitution
  Principle II)
- No Frotz source text is copied into this repository; behavior is reproduced from
  observation and from research.md R2
