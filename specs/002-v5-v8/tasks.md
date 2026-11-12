# Tasks: Z-Machine Versions 5 and 8

**Input**: Design documents from `/specs/002-v5-v8/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. In every
phase the test tasks are written and seen to fail by assertion before the implementation
tasks beneath them. Every test of interpreter behavior carries a comment citing the
section of the Z-Machine Standard 1.1 it verifies (written "§"; "Q§" is Quetzal 1.4).
Requirement numbers are this feature's: cite them as "002 FR-n" where an existing file
already cites the first feature's.

**The version 3 guard**: after every task in Phases 2, 5, 6 and 7, `cargo test
--workspace` must pass, including the Zork I transcript match. Only three existing tests
may have their expectations edited, and only in the tasks that name them: T007 (two) and
T006 (one). Any other change to a version 3 expectation stops the task (research R14).

**Where tests live**: unit tests of internals in the module's own `_tests.rs` file;
tests through the public API in the crate's `tests/` directory.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US6)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The story files in place with their records

- [ ] T001 [P] Add `stories/advent/`: the unmodified `advent.inf` from the Inform 6 distribution; `advent.z5` and `advent.z8` compiled with Inform 6.44 and library 6.12.8 (`inform +include_path=<lib> -v5` and `-v8`), added with `git add -f`; `LICENSE.md` quoting the Inform package's `COPYING` in full; and `SOURCE.md` with a URL and revision for the source (the Inform 6 distribution's repository and commit, not "as installed"), SHA-256 digests of all three files, the exact compile commands, the statement that this directory is not under the repository's MIT license, that the game is a test fixture not cleared for bundling in a shipped app, and the caveats in research R1. Confirm that compiling twice gives identical files
- [ ] T002 [P] Add `stories/praxix/`: `praxix.inf` and `praxix.z5` from the IF Archive's `praxix.zip`, unmodified, the `.z5` with `git add -f`; `SOURCE.md` with URL, archive digest, file digests, the public-domain statement and Amir Karger's copyright notice from the source's header. Stop for the project owner if the header does not read as research R2 says
- [ ] T003 [P] Add to `stories/czech/`: `czech.z5` and `czech.z8` compiled from the committed `czech.inf` (`git add -f`), and `czech.out5` and `czech.out8` from the archive; extend `SOURCE.md` with their digests and compile commands
- [ ] T004 Run praxix (script `all`, `quit`), `czech.z5` and `czech.z8` under dfrotz and record in the two `SOURCE.md` files the summary each prints on a pass, and for czech which lines differ from the `.out` files; update the README's story-file section for Advent and praxix, including Advent's license terms

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Versions as data; the image builder for versions 5 and 8; the format modules
made version-aware. Nothing new runs yet.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. The version 3
guard applies to every task.

- [ ] T005 Write failing unit tests in `crates/zmachine-core/src/version_tests.rs` for every row of the Version table in data-model.md (§1.2.3, §11.1.6, §1.1.4, §12.2, §12.3.1, §12.3.2, §12.4.1, §12.4.2, §13.3, §13.4, §5.2.1), then implement `crates/zmachine-core/src/version.rs` until they pass
- [ ] T006 Restructure `crates/zmachine-core/tests/coverage.rs` into one table per version: the version 3 table and its 68 rows unchanged, compared with the decoder's version 3 instruction list; a version 5 table that starts with the 63 instructions shared with version 3, compared with the decoder's version 5 list. This is one of the three permitted edits to a version 3 test
- [ ] T007 Write failing tests, then implement, in `crates/zmachine-core/src/header.rs` and `lib.rs` (§11.1, §8.4.3, §6.1.2.2; contracts/core-api.md "Loading"): `Machine::load_with` and `Settings` with the defaults stated; versions 5 and 8 accepted and 1, 2, 4, 6, 7 and 9 refused with a message naming 3, 5 and 8; `TooLarge` for an oversize version 5 or 8 file and not for version 3; the optional tables at 0x2E, 0x34 and 0x36 read only for versions 5 and 8 and validated; every field in data-model.md "What the interpreter writes into the header", for both versions; version 3 header bytes unchanged. Switch the two existing refusal tests, `load_refuses_what_is_not_a_version_3_story` in `crates/zmachine-core/tests/events.rs` and `files_that_are_not_version_3_stories_are_refused` in `crates/zmachine-cli/tests/play.rs`, from version 5 to version 4; these are the other two permitted edits
- [ ] T008 Write failing tests in `crates/zmachine-testkit/src/tests.rs` for version 5 and 8 images, then implement them as new types beside the existing ones (research R10, R14): a test that every existing version 3 image is byte for byte what it was (digests recorded before any change); nine Z-character dictionary keys checked against hand-computed vectors and against the dictionary of `stories/advent/advent.z5`; 14-byte object entries, 48 attributes, 63 property defaults and both property size forms checked against Advent's object table; routine headers without initial values; packed addresses and alignment for 4 and 8; the version 5 header; helpers for the extended form, the two type-byte form with up to eight operands, and the version 5 text and parse buffers
- [ ] T009 [P] Write failing unit tests in `crates/zmachine-core/src/decode_tests.rs`, then implement in `decode.rs` (§4.3.4, §4.4.3.1, §14): the extended form; eight operands for `call_vs2` and `call_vn2`; for every instruction of version 5 the right opcode, store and branch, in a table transcribed from §14; every row of data-model.md "Opcode numbers whose meaning changes" decoded both ways; version 3 opcodes that were unknown still unknown in version 3; add each new instruction's row to the version 5 coverage table as its decode test is written
- [ ] T010 [P] Write failing unit tests in `crates/zmachine-core/src/text_tests.rs`, then implement in `text.rs` (§3.5.5, §3.8.5.2, §13.4, §3.3): nine Z-character encoding; a game's alphabet table used for decoding and encoding; a game's Unicode table; abbreviation addresses still times 2 in versions 5 and 8; Advent's object names decoded from `advent.z5` and `advent.z8`
- [ ] T011 [P] Write failing unit tests in `crates/zmachine-core/src/objects_tests.rs`, then implement in `objects.rs` (§12.3.2, §12.4.2): the version 4+ layout for every operation the version 3 tests cover; object numbers above 255; attributes 32 to 47; property numbers 32 to 63; property lengths 3 to 64 including the length-0-means-64 rule (§12.4.2.1.1); `get_prop_len` from both size forms; Advent's object tree read from `advent.z5`
- [ ] T012 [P] Write failing unit tests in `crates/zmachine-core/src/dictionary_tests.rs`, then implement in `dictionary.rs` (§13.2, §13.4, §15 `read`, `tokenise`): six-byte keys; the version 5 text buffer (length in byte 1, text from byte 2, no terminator) and parse buffer; tokenising against a given dictionary, sorted or unsorted (negative count); the flag that leaves unknown words' entries untouched; lookups in Advent's dictionary

**Checkpoint**: Version 5 and 8 images can be built, loaded and decoded; the version 3
suite is green and unchanged apart from the three named tests

---

## Phase 3: Evidence from the reference interpreter

**Purpose**: Record what dfrotz does before writing the code that must match it. Needs
T008 and dfrotz; runs alongside Phase 4.

- [ ] T013 [P] Add version 5 transcript probe programs to `crates/zmachine-testkit/src/programs.rs`, one or more for each of rules 13 to 21 in contracts/transcript-format.md, including buffering off with transcription toggled mid-word, pre-loaded input, a quotation box drawn the veneer's way, and a non-ASCII character; run each under dfrotz; commit the transcripts as `crates/zmachine-core/tests/fixtures/transcript5/`; correct the contract wherever dfrotz disagrees with it
- [ ] T014 [P] Add a keypress probe (several `read_char`s and an `aread`) and run it under dfrotz with scripts covering every rule in contracts/cli.md "Keypresses" and "Special keys", including each escape; commit the scripts and transcripts as `crates/zmachine-core/tests/fixtures/keys/`; correct the contract wherever dfrotz disagrees
- [ ] T015 [P] Add the ten screen test programs of contracts/screen-model.md to `crates/zmachine-testkit/src/programs.rs`; run programs 1 to 3 under `dfrotz -r cn -r ln -r lt -h 24 -w 80` and commit what it prints as `crates/zmachine-core/tests/fixtures/screen/`, with a note of which rows are comparable; if dfrotz's output cannot be parsed reliably, record why in research R11 and drop the cross-check (T051)
- [ ] T016 [P] Settle by probe under dfrotz, and record in data-model.md: what `check_arg_count` reports when more arguments are passed than the routine has locals; the value `catch` stores at each call depth; what `save_undo` and `restore_undo` store in each case of data-model.md "Undo position"

---

## Phase 4: Walkthrough (runs alongside Phases 5 to 8)

**Purpose**: The reference for the headline test. Needs only T001 and dfrotz.

- [ ] T017 Choose a seed and write `stories/advent/fidelity/walkthrough.txt` against `dfrotz -m -q -w 80 -S 0 -s <seed> stories/advent/advent.z5`, section by section, until the game is won with 350 points. It starts with `script` and a file name, starts no second transcript, includes the `version` command, uses no `save`, `restore` or `undo`, and keeps to the script rules of contracts/cli.md. Commands are entered by us while playing; maps and hints may be consulted; no published command list is copied. If a section cannot be completed after trying another seed, stop for the project owner
- [ ] T018 Confirm the same walkthrough and seed win the game on `advent.z8` and that the two transcripts are identical; if they differ, record how in research R3 and keep one reference per version. Run both under `dfrotz -Z 2` and record whether any warning appears
- [ ] T019 Extend `scripts/regenerate-references.sh` for Advent and generate `stories/advent/fidelity/reference_z5.scr` and `reference_z8.scr`; add `stories/advent/fidelity/README.md` in the form of Zork I's
- [ ] T020 [P] Write `stories/advent/fidelity/undo.txt`, thirty moves then thirty undos, and generate `undo_z5.scr` and `undo_z8.scr` with the regeneration script (SC-007)
- [ ] T021 [P] For three points in the walkthrough and each version, create under `stories/advent/fidelity/saves/` the `make`, save, `cont` and continuation transcript files in the form used for Zork I, generated by the regeneration script (SC-006)

**Checkpoint**: Committed reference transcripts of a won game exist for both versions

---

## Phase 5: User Story 1 - Play a version 5 game from the command line (Priority: P1) 🎯 MVP

**Goal**: A version 5 story file runs: opening text, commands in, responses out, quit

**Independent Test**: `cargo run -p zmachine-cli -- stories/advent/advent.z5`, then `in`,
`take lamp`, `undo`, `score`, `quit`, `y`; each response is correct and the exit status
is 0

### Tests for User Story 1 ⚠️

> Write these first and confirm they fail by assertion before any implementation below.
> Each adds its instructions' rows to the version 5 coverage table.

- [ ] T022 [P] [US1] Failing unit tests in `crates/zmachine-core/src/exec_tests.rs` (§5.2.1, §6.4, §15): `call_1s`, `call_2s`, `call_vs`, `call_vs2` storing, and `call_1n`, `call_2n`, `call_vn`, `call_vn2` discarding; up to seven arguments; locals starting at 0; packed addresses times 4; `check_arg_count` for each argument position, with the surplus case as T016 recorded; `catch` storing the value T016 recorded and `throw` returning from that frame with intermediate discarding calls storing nothing; `throw` to a missing frame as a fault; 1OP:15 is `call_1n` in version 5 and `not` in version 3; 0OP:9 is `catch` in version 5 and `pop` in version 3
- [ ] T023 [P] [US1] Failing unit tests in `crates/zmachine-core/src/ops_math_tests.rs` (§15): `log_shift` and `art_shift` left and right, by 0 and by 15, with the sign rule for each; VAR `not`
- [ ] T024 [P] [US1] Failing unit tests in `crates/zmachine-core/src/ops_table_tests.rs` (§15): `copy_table` for every row of data-model.md's outcome table; `scan_table` with word and byte forms and each field length, found and not found; `print_table` with width, height and skip, in the lower window; `encode_text`
- [ ] T025 [P] [US1] Failing tests in `crates/zmachine-core/tests/input5.rs` (§10, §15 `read`, `read_char`; contracts/core-api.md): `aread` storing 13, the buffer layout, an absent parse buffer; `max_len` and `preloaded_input()` with text already in the buffer, and typed text stored after it; `provide_line_ended_by` with a listed key storing that key, and `NotATerminator` for an unlisted one; `KeyWanted` and `provide_key` for a character, each special key and an unrepresentable one; a timed `aread` and a timed `read_char` behaving as untimed with the routine never called; no status line drawn by the interpreter before a version 5 read; the answers out of turn refused
- [ ] T026 [P] [US1] Failing tests in `crates/zmachine-core/tests/streams5.rs` (§7.1.2, §8.7.1, §15): the memory stream's table format; nesting to 16 and a fault at 17; exclusivity; closing with none open; a write outside dynamic memory as a fault; `output_stream 3` still ignored in version 3; `take_styled_output` giving runs with the style in force, adjacent equal styles merged, and `take_output` giving the same text plain; `set_text_style` accumulating and 0 clearing; fixed pitch forced by Flags 2 bit 1 (§8.1); `print_unicode`; `check_unicode` for each row of the outcome table; `set_colour` and `set_true_colour` with defined and undefined numbers, `sound_effect`, `input_stream`, `show_status` and an extended opcode above 28 all doing nothing; `piracy` branching; `set_font` for 0, 1, 3 and 4; `buffer_mode` accepted
- [ ] T027 [P] [US1] Failing tests in `crates/zmachine-core/tests/undo.rs` (§15 `save_undo`, `restore_undo`; §6.1.2; data-model.md "Undo position"): `save_undo` storing 1; `restore_undo` resuming at the `save_undo` with 2, with memory and call stack as they were, from inside nested calls, with the result going to a local, a global and the stack; 0 when nothing is saved; 26 positions saved and 25 restorable; Flags 2 kept and the interpreter's header fields re-applied; the random state, the screen and the streams untouched; `undo_slots` 0 storing -1 and clearing Flags 2 bit 4 (§6.1.4)
- [ ] T028 [P] [US1] Failing tests in `crates/zmachine-core/tests/save5.rs` (§15 `save`, `restore`; Q§5.8.2): `save` storing 1 or 0 by the host's answer; a restore resuming at the `save` with 2 stored; a refused restore storing 0 and changing nothing; the saved program counter pointing at the store byte; frames carrying the discard flag and argument count; the forms with operands storing 0 and raising no event; a save from the version 8 build refused by the version 5 build
- [ ] T029 [P] [US1] Failing tests in `crates/zmachine-core/tests/transcript5.rs`: every probe from T013 gives the fixture dfrotz wrote; the existing fifteen version 3 probes still give theirs
- [ ] T030 [P] [US1] Failing tests in `crates/zmachine-cli/tests/play5.rs`: `zm stories/advent/advent.z5` with the Independent Test's commands gives the game's replies and exits 0; versions 4, 6 and 7 are refused with status 2 naming 3, 5 and 8; every rule and escape of contracts/cli.md "Keypresses" and "Special keys" against the fixtures from T014; a line beginning with a backslash and a lower-case letter is refused with a message; `--interpreter-version F` changes what the game's `version` command prints

### Implementation for User Story 1

- [ ] T031 [US1] Implement in `crates/zmachine-core/src/exec.rs`: version-aware operand count, calls, routine headers, the discarding call forms, `check_arg_count`, `catch` and `throw`, until T022 passes
- [ ] T032 [P] [US1] Implement the shifts and VAR `not` in `crates/zmachine-core/src/ops_math.rs` until T023 passes
- [ ] T033 [P] [US1] Implement `crates/zmachine-core/src/ops_table.rs` until T024 passes
- [ ] T034 [US1] Implement in `crates/zmachine-core/src/ops_io.rs` and a new `screen.rs`: styled runs behind `take_output` and `take_styled_output`, styles, the memory stream, the instructions T026 lists, and the transcript rules 13 to 21 including buffering off, until T026 and T029 pass. The screen canvas itself is User Story 3; here `split_window`, `set_window`, `set_cursor` and the erases only keep upper-window text out of the lower window's output
- [ ] T035 [US1] Implement version 5 line input, key input and the new answers in `crates/zmachine-core/src/ops_io.rs` and `lib.rs` until T025 passes
- [ ] T036 [P] [US1] Implement `crates/zmachine-core/src/undo.rs` and its two instructions until T027 passes
- [ ] T037 [US1] Implement the version 5 forms of `save` and `restore` in `crates/zmachine-core/src/ops_io.rs` until T028 passes
- [ ] T038 [US1] Implement in `crates/zmachine-cli/src/lib.rs` and `main.rs`: versions 5 and 8, key input from lines with dfrotz's rules and escapes, `--interpreter-version`, and `--width` setting the screen width, until T030 passes
- [ ] T039 [US1] Play the opening of Advent by hand with `zm stories/advent/advent.z5`, and run the first section of the walkthrough through `zm` and dfrotz; for each defect, add a failing test in the matching file first, then fix it

**Checkpoint**: A version 5 game is playable from the terminal

---

## Phase 6: User Story 2 - Play a version 8 game (Priority: P2)

**Goal**: Version 8 story files run, including ones larger than version 5 allows

**Independent Test**: The same commands on `advent.z5` and `advent.z8` give the same text

- [ ] T040 [P] [US2] Failing tests in `crates/zmachine-core/tests/version8.rs` (§1.2.3, §11.1.6, §1.1.4): a built version 8 image above 256K that calls a routine and prints a string located beyond 256K; packed addresses times 8; `verify` using the length field times 8; a version 8 file above 512K refused
- [ ] T041 [P] [US2] Failing test in `crates/zmachine-cli/tests/play5.rs`: one script of twenty commands gives identical standard output on `advent.z5` and `advent.z8` (spec US2 scenario 1)
- [ ] T042 [US2] Make whatever T040 and T041 show is not yet version-aware so; no version 8 special cases outside `version.rs`

**Checkpoint**: Both builds of Advent play alike

---

## Phase 7: User Story 3 - See what the game draws in its upper window (Priority: P3)

**Goal**: A host can read the screen canvas with styles, and `zm` can show it

**Independent Test**: `cargo test -p zmachine-core --test screen`

- [ ] T043 [US3] Failing tests in `crates/zmachine-core/tests/screen.rs` (§8.7; contracts/screen-model.md): for each of the ten test programs, the canvas read through `screen()` equals an expected canvas written out cell for cell with styles, at each point the program waits; the overlay of program 2 is readable at the next input and empty after it; `take_screen_changed` true once per change; `split_window` clamped to the screen; nothing written to the canvas for a version 3 story; restart leaves it blank and unsplit; restore and undo leave it as it is
- [ ] T044 [P] [US3] Failing tests in `crates/zmachine-cli/tests/upper.rs`: with `--upper`, Advent's status line is printed above the prompt and changes after a move; a quotation box from program 2 is printed once and not after the next input; without `--upper` nothing of it appears; `--style ansi` wraps bold, italic and reverse text in escape codes and `--style plain` does not; neither the canvas nor any escape code appears in a transcript
- [ ] T045 [US3] Implement the canvas, overlay, cursor and erases in `crates/zmachine-core/src/screen.rs`, with `screen()` and `take_screen_changed()` in `lib.rs`, until T043 passes
- [ ] T046 [US3] Implement `--upper` and `--style` in `crates/zmachine-cli/src/lib.rs` and a new `term.rs` until T044 passes

**Checkpoint**: Status lines, menus and quotation boxes are visible to a host

---

## Phase 8: User Story 4 - Pass the conformance test programs (Priority: P4)

**Goal**: praxix passes; czech passes for versions 5 and 8

**Independent Test**: `cargo test -p zmachine-cli --test praxix --test czech`

- [ ] T047 [P] [US4] Failing test in `crates/zmachine-cli/tests/praxix.rs`: verifies the story file's digest; runs praxix with `all` and `quit`; fails naming each group that reports failed tests; requires the pass summary recorded in T004
- [ ] T048 [P] [US4] Extend `crates/zmachine-cli/tests/czech.rs` to versions 5 and 8 with the same checks as version 3 against `czech.out5` and `czech.out8`; the version 3 cases unchanged
- [ ] T049 [US4] For each failure T047 or T048 reports, read the Standard 1.1 section: if the interpreter is wrong, add a failing unit test in the matching source file and fix it; if the test program conflicts with Standard 1.1, add a rule to `docs/divergences.md` citing the section; if neither is clear, stop for the project owner. Repeat until both pass (SC-001, SC-002)

**Checkpoint**: The version 5 instruction set is conformant by two independent measures

---

## Phase 9: User Story 5 - Match the reference interpreter on a full game, in both versions (Priority: P5)

**Goal**: The Advent walkthrough transcript matches Frotz 2.55 byte for byte, for
version 5 and for version 8

**Independent Test**: `cargo test -p zmachine-cli --test fidelity_advent`

- [ ] T050 [US5] Failing tests in `crates/zmachine-cli/tests/fidelity_advent.rs`, for each of `advent.z5` and `advent.z8`: the story file's digest matches `SOURCE.md`; the walkthrough keeps to the script rules, starts exactly one transcript and contains `version`; `zm` with `--interpreter-version F` gives a transcript equal to the reference after ledger rules, reported as for Zork I; the transcript shows 350 points and the game's ending; two runs are identical. And marked `#[ignore = "needs dfrotz 2.55"]`: dfrotz still writes each reference; marked `#[ignore = "timing; run with --release"]`: each walkthrough takes under five seconds (SC-009)
- [ ] T051 [P] [US5] If T015 kept the cross-check: a test in `crates/zmachine-core/tests/screen.rs` comparing our canvas with the dfrotz screen fixtures for programs 1 to 3 on the comparable rows, and, marked `#[ignore = "needs dfrotz 2.55"]` in `crates/zmachine-cli/tests/upper.rs`, Advent's status row after the walkthrough's first five commands against dfrotz's (002 FR-033)
- [ ] T052 [US5] Work through each difference T050 reports, earliest first: if the interpreter is wrong, add a failing unit test in the matching source file and fix it; if Frotz departs from Standard 1.1, add a rule to `docs/divergences.md` citing the section; if neither is clear, stop for the project owner. Repeat until T050 passes (SC-003)

**Checkpoint**: The project's definition of faithful is met for versions 5 and 8

---

## Phase 10: User Story 6 - Undo, save and restore (Priority: P6)

**Goal**: Undo matches Frotz; saves interchange with Frotz in both versions

**Independent Test**: `cargo test -p zmachine-cli --test undo --test save_restore`

- [ ] T053 [P] [US6] Failing test in `crates/zmachine-cli/tests/undo.rs`: `zm` running `undo.txt` on each build gives the committed `undo_z5.scr` and `undo_z8.scr`, in which twenty-five undos succeed and five are refused (SC-007); marked `#[ignore = "needs dfrotz 2.55"]`, dfrotz still writes them
- [ ] T054 [P] [US6] Extend `crates/zmachine-cli/tests/save_restore.rs` to Advent's three save points in each version, with the same three cases as Zork I: `zm` restoring Frotz's save, `zm` restoring its own, and, ignored without dfrotz, dfrotz restoring `zm`'s (SC-006); a damaged save and one from the other build refused with play continuing; the Zork I cases unchanged
- [ ] T055 [US6] Fix whatever T053 and T054 show, each defect with a failing unit test first

**Checkpoint**: All six user stories work

---

## Phase 11: Polish & Cross-Cutting Concerns

- [ ] T056 [P] Extend `crates/zmachine-cli/tests/malformed.rs` with at least thirty named version 5 and 8 cases, each with the diagnostic it must give: every new `LoadError`, every new fault kind, the old fault kinds reached through version 5 instructions, bad extended opcodes below 29, a bad terminating-characters table, a routine at an address the multiplier takes outside the file (SC-008)
- [ ] T057 [P] Extend `crates/zmachine-core/tests/robustness.rs` to run damaged copies of `advent.z5`, `advent.z8`, `praxix.z5` and the Advent saves, answering key requests as well as line requests; nothing may panic in a debug build (002 FR-024); confirm guarantee 7 by a story that asks for a huge window, a deep memory-stream nest and endless undo saves
- [ ] T058 [P] Confirm the version 5 coverage table has all 98 instructions and that the check requires a cited test for each (SC-005, 002 FR-031)
- [ ] T059 [P] Update `README.md`: versions 5 and 8 in the status, Advent and praxix in the story files, the new options, and where this interpreter knowingly differs from Frotz
- [ ] T060 Update `.github/workflows/ci.yml` if the new ignored tests need anything the workflow lacks; run every step of `specs/002-v5-v8/quickstart.md` on a copy with only the files a clone would have, and correct the document or the code wherever they disagree
- [ ] T061 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` and fix what they report; confirm with `git diff main` that no version 3 test expectation changed beyond the three named in research R14, and list in this file what was done differently from the plan

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: needs T001 for the checks against Advent's tables; blocks every user story. T005 first; T006 and T007 next; T008 before T009 to T012
- **Evidence (Phase 3)**: needs T008 and dfrotz
- **Walkthrough (Phase 4)**: needs only T001 and dfrotz; can begin alongside Phase 2
- **US1 (Phase 5)**: needs Foundational, and T013, T014, T016 for the tests that use their fixtures
- **US2 (Phase 6)**: needs US1
- **US3 (Phase 7)**: needs US1 and T015
- **US4 (Phase 8)**: needs US1, which includes undo; czech's print tests do not need US3
- **US5 (Phase 9)**: needs US1, US2 and the Walkthrough; best after US4
- **US6 (Phase 10)**: needs US1, US2, T020 and T021
- **Polish (Phase 11)**: after the stories it covers

### Within Each User Story

- Tests are written and fail before implementation
- Core before CLI
- The version 3 suite is green after every task

### Parallel Opportunities

- T001 to T003; T009 to T012 after T008; all of Phase 3; all of Phase 4 against Phases 2
  and 5 to 8
- T022 to T030: separate files
- T032, T033 and T036 against T031: separate files
- T047 and T048; T053 and T054; T056 to T059

`crates/zmachine-core/src/ops_io.rs` is edited by T034, T035 and T037, and `exec.rs` by
T031 and the dispatch additions of T033, T036 and T045; those run one at a time.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phases 1 and 2, with the version 3 suite green throughout
2. Phase 3's probes, then Phase 5
3. Stop and play Advent by hand

### Incremental Delivery

1. US1: a version 5 game runs, with undo
2. US2: version 8
3. US3: the screen is visible
4. US4: praxix and czech pass
5. US5: Advent matches Frotz in both versions, the headline result
6. US6: undo and saves match Frotz

### Points where work stops for the project owner

- T002, if praxix's header does not read as expected
- T015, if dfrotz's screen output cannot be used (the cross-check is then dropped, not
  the feature)
- T017, if Advent cannot be won at a workable seed
- T049 and T052, when a difference is neither clearly our fault nor clearly a departure
  from the Standard
- Any task that would change a version 3 expectation other than the three named

---

## Notes

- Commit after each task or small group of tasks
- A test is never weakened, skipped or deleted to make a task pass (Principle II); the
  three named edits are the whole of the exception
- Frotz source may be read to learn how the reference behaves; none of its text is copied
  into this repository
- T049 and T052 are loops; each defect found inside them gets its own failing test
