# Tasks: Compass Look-Ahead

**Input**: Design documents from `/specs/009-compass-lookahead/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started. Where code must exist for a test to compile, it is a stub
that answers nothing. Where a test passes before its code exists, or is written with
its code, the task's notes say so and say what was done to show it can fail.

Tasks cite what they serve: "FR-" and "SC-" are the spec's; "R" a section of
`research.md`; "L§" a numbered statement of `contracts/lookahead.md`; "C§" one of
`contracts/corpus.md`; "D" a table of `data-model.md`.

**The guard**: in `crates/zmachine-core` only `lib.rs`, `random.rs` and their tests are
edited, and only as T004 says. No source file under `apps/ios/Lamplight/` is edited
(L§24). After every phase `cargo test --workspace` passes. A test of features 001 to
008 is edited only where this feature changes the value or form it fixes for a way that
look-ahead now settles, and every such edit is listed at the end of this file (FR-024).
Any other failing earlier test is a stop.

**No text of a game is kept.** What a copy prints is dropped unread. No task writes a
line of any game into a tracked file, a commit message or these notes.

**Stop points**: stop for the project owner if a fidelity test gives other bytes after
T004; if the player's machine differs after a look-ahead and the cause cannot be found
(T010); if a finding on the corpus is not borne out and its cause is not chance (T026);
if, with the limits of T017 in force, fewer than nine in ten unsettled ways are settled
across the corpus (T027; SC-002); if a timing limit is exceeded on the owner's Mac
(T018, T035); and at T031, where two places whose ways have vanished wait for the
owner's eye.

**No downloads.** A story whose file is absent has its tests reported as ignored.

**Stories built in memory**: the ZIL fixture of
`crates/zmachine-introspect/src/fixtures.rs` and the corpus's toy in
`crates/zmachine-corpus/tests/common/mod.rs` each have ways decided by the game's code.
This feature needs, on one of them or on a new fixture in the testkit: a way allowed
when a flag is set and refused when it is not; a way allowed or refused by one draw of
`random 2`; a way that quits; a way that asks a yes-or-no question; a way that loops
without stopping; a way that walks to a room and is moved on from it before the prompt.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [ ] T001 Create `crates/zmachine-experience/src/lookahead.rs` and `lookahead_tests.rs`, each with a module comment that cites this feature, and declare them in `crates/zmachine-experience/src/lib.rs`; add `zmachine-experience` to the dependencies of `crates/zmachine-corpus/Cargo.toml`; `cargo test --workspace` passes, and `crates/zmachine-corpus/tests/crate_rules.rs` is edited and listed if it fixes the crate's dependencies

---

## Phase 2: Foundational

**⚠️ CRITICAL**: No user story starts until this phase is complete

- [ ] T002 [P] Failing tests in `crates/zmachine-core/src/exec_tests.rs` or a new `random_tests.rs` (L§1, 2, 3; R3): a machine given the new value for chance draws what a machine loaded with that value as its seed draws, from the start of the sequence; a machine whose game has seeded the generator into predictable mode (`random -5`) draws the same after the method as before it; a machine on which the method is not called draws as before
- [ ] T003 [P] Build the fixtures this feature needs, in `crates/zmachine-testkit/src/programs.rs` or beside the ZIL fixture: the six ways listed above under "Stories built in memory", each reachable from one room, with a check that the introspection interface recognises the story and reports each of those ways as conditional
- [ ] T004 Add the method to `Machine` in `crates/zmachine-core/src/lib.rs` and its part in `crates/zmachine-core/src/random.rs` (L§1, 2); T002 passes; run `cargo test -p zmachine-cli --test fidelity_zork1 --test fidelity_advent` and **stop** if either gives other bytes (L§3; FR-006)
- [ ] T005 In `docs/introspection.md`: version line to `0.5.0`; a new section after 10.5 holding L§21 and L§22; a line for 0.5.0 in the list of changes saying L§23; set `SPEC_VERSION` to `0.5.0`, and edit and list the tests that fix the version string or the count of cited sections (FR-022)

---

## Phase 3: User Story 1 - An unsettled way is settled before it is shown (Priority: P1) 🎯 MVP

**Goal**: a way the game would allow is shown open with its room; a way it would refuse
is shown as no way.

**Independent Test**: on the fixture of T003, with the flag set and not set, ask the
session for its compass.

### Tests

- [ ] T006 [P] [US1] Failing tests `one_copy_*` in `crates/zmachine-experience/src/lookahead_tests.rs` (L§4, 6; D "Outcome"): on the fixture, a copy given the allowed way's word is *walked to* that room; given the refused way's word, *stayed*; the copy was given exactly that word and nothing else (the fixture echoes what it reads into a global the test inspects)
- [ ] T007 [P] [US1] Failing tests `finding_*` in `crates/zmachine-experience/src/lookahead_tests.rs` (L§5, 7; D "Copy", "Finding"): the allowed way's finding is *walked* with the room's number; the refused way's is *refused*; the two values of chance given to copies 1 and 2 differ from each other, and are the same when asked again at the same count of commands and different at the next count
- [ ] T008 [P] [US1] Failing tests `untouched_*` in `crates/zmachine-experience/src/lookahead_tests.rs` (L§8, 9; FR-006, FR-009): after findings for every way of the fixture's room, the player's machine equals a clone taken before (compare by `Debug` text or by a save of each, and by the next twenty draws the fixture's `random` command prints); the session's turns, transcript and status are equal too
- [ ] T009 [P] [US1] Failing tests `settled_*` in `crates/zmachine-experience/src/compass_tests.rs` (L§10 to 17; D "What the compass then says"; FR-001 to FR-003, FR-010, FR-011, FR-013): a `maybe` place found walked is `open` with `leads_to` and its spoken words those of an open way; found refused is `closed` with no command; with no finding is unchanged; an extra found refused is not in the list; a place `unknown` because two exits claim it is not tried (a counter in the test's stub of the trial stays at zero); an `open`, `closed` or `door` place is not tried; the note of unknown ways goes only when none is left; the compass asked for twice is equal
- [ ] T010 [US1] Implement one copy, the three copies and the finding in `crates/zmachine-experience/src/lookahead.rs` (R2, R3; L§4 to 9), with the single-copy limit as a parameter; T006 to T008 pass. **Stop** if T008 cannot be made to pass
- [ ] T011 [US1] Settle a compass by findings in `crates/zmachine-experience/src/compass.rs` (L§10 to 16); T009 passes
- [ ] T012 [US1] Failing test, then the call, in `crates/zmachine-experience/tests/session.rs` and `crates/zmachine-experience/src/lib.rs` (L§10, 17; FR-002, FR-004): a session on the fixture reports the allowed way `open` and the refused way `closed` at a command prompt; at a stop that is not a command prompt, and on a story the interface does not recognise, the compass is what it was before this feature; without a pack the same findings are made
- [ ] T013 [US1] Failing test, then nothing or a fix, in `crates/zmachine-experience/tests/walkthrough.rs` (SC-004; FR-006): Zork I's walkthrough through a session, with look-ahead made at every prompt, gives its reference transcript byte for byte. Notes say whether the test could fail (make it fail once by letting copy 0 share the player's machine)
- [ ] T014 [P] [US1] Failing test in `crates/zmachine-experience/src/json_tests.rs` and `crates/zmachine-ffi/tests/abi.rs` (L§15; FR-014): the snapshot's keys and their order, and the C interface, are what they were

**Checkpoint**: the trap door's way down comes and goes with the trap door, in Rust.

---

## Phase 4: User Story 2 - A way that depends on chance is not promised (Priority: P1)

**Goal**: a way decided by a draw keeps its question mark.

**Independent Test**: on the fixture's way of `random 2`, ask for the compass at many
counts of commands.

- [ ] T015 [P] [US2] Failing tests `chance_*` in `crates/zmachine-experience/src/lookahead_tests.rs` (L§7; FR-008; R2): over 200 successive prompts the way of `random 2` has a finding at fewer than a third of them (two further copies agree with the first by luck one time in four), and at every prompt where it has one, copy 0's outcome is that finding; a way that is always allowed has a finding at all 200. If the first assertion is found too loose or too tight, the notes say what was measured
- [ ] T016 [US2] Make T015 pass in `crates/zmachine-experience/src/lookahead.rs`; if it passes already, the notes say so and say how it was shown able to fail (give all three copies the game's own chance: the way is then settled at all 200)

---

## Phase 5: User Story 3 - A copy that comes to harm tells the player nothing (Priority: P1)

**Goal**: a fatal way, a questioning way and a runaway way keep their question mark, and
the player's game goes on.

**Independent Test**: on the fixture's three such ways, ask for the compass, then send
an ordinary command.

- [ ] T017 [P] [US3] Failing tests `harm_*` and `limit_*` in `crates/zmachine-experience/src/lookahead_tests.rs` (L§6, 18, 19; D "Outcome", "Limits"; FR-007, FR-013, FR-016): the way that quits, the way that asks, the way that loops, and the way that asks to save each have no finding and the place stays `maybe`; the way that walks and is moved on names the room the copy is in at its prompt; with the total limit set below the cost of two ways, the first way in the compass's order has its finding and the later ones have none; after each, the session answers an ordinary command as it would have
- [ ] T018 [US3] Implement the limits in `crates/zmachine-experience/src/lookahead.rs` and set their counts (R7): add a kept, ignored measurement in `crates/zmachine-experience/tests/speed.rs` that prints, over Zork I's walkthrough, the most instructions any one copy ran and the most all copies ran at one prompt; set the single-copy limit at twenty times the first and the total as R7 says; T017 passes; write both counts and the figures they came from in this file's closing notes. **Stop** if the 10 ms test fails with look-ahead on (L§20; SC-005)

---

## Phase 6: User Story 4 - A way found open says whether it is dark ahead (Priority: P2)

**Goal**: with a pack that knows light, a way found walked carries the mark of darkness
when the room is dark for the player.

**Independent Test**: on the ZIL fixture with `lit-attribute`, a conditional way into
an unlit room, with the lamp off and on.

- [ ] T019 [P] [US4] Failing tests `dark_*` in `crates/zmachine-experience/src/compass_tests.rs` (L§11; FR-012; R5): with the pack and no light carried, a way found walked into an unlit room has `dark_ahead` true and is spoken ", into darkness"; with the lamp on, false; with no pack, false and nothing said of light; a way found refused is never marked
- [ ] T020 [US4] Carry the copy's `lit` into the finding and the compass in `crates/zmachine-experience/src/lookahead.rs` and `compass.rs`; T019 passes
- [ ] T021 [US4] Failing test, then nothing or a fix, in `crates/zmachine-experience/tests/session.rs` on Zork I with its pack (SC-001, SC-006): in the living room the way down is `closed` before `move rug` and `open trap door` and `open` after, leading to the cellar, `dark_ahead` true; after `take lamp` and `turn on lamp`, `dark_ahead` false. The test names rooms by the layer's `leads_to` and keeps no other text of the game

---

## Phase 7: User Story 5 - The corpus says how often look-ahead is right (Priority: P2)

**Goal**: every finding is set against a trial apart from it; an unexplained miss fails.

**Independent Test**: the toy with one way decided by a rare draw; the run fails until
the ledger names it.

### Tests

- [ ] T022 [P] [US5] Failing tests in `crates/zmachine-corpus/tests/formats.rs` and `ledger.rs` (C§5 to 9): a report with the `lookahead` line is read and written back byte for byte; `walked`, `refused` and `left` sum to the ways asked; a report without the line fails the comparison; the summary's lines end with the five numbers; a ledger entry with `lookahead: walked N` or `lookahead: refused` is read; one with both `reported` and `lookahead`, or with a pairing that is borne out, is refused with its line
- [ ] T023 [P] [US5] Failing tests in `crates/zmachine-corpus/tests/trial.rs` and a new `tests/lookahead.rs` (C§1 to 4, 9; FR-018, FR-019): on the toy, a way always allowed is counted walked and borne out; always refused, refused and borne out; the way of an even draw is counted left; a way refused one time in very many (the toy draws `random 1000`) is found walked and, at a count of commands chosen so that the corpus's own trial draws the refusal, is not borne out: the run fails and names story, room and direction, and passes once the ledger holds the entry; an entry no run needs is reported
- [ ] T024 [P] [US5] Failing tests in `crates/zmachine-corpus/tests/untouched.rs` and `no_text.rs` (C§4, 6, 11; FR-020, FR-021; SC-008, SC-009): every line of a toy's report but the new one is what it was; the kept files hold only numbers and the fixed words, with the new words added to the list the test allows

### Implementation

- [ ] T025 [US5] In `crates/zmachine-corpus/src/trial.rs`, `walk.rs`, `report.rs` and `ledger.rs`: let a trial be given a value for chance; at each place ask the layer for each unsettled way's finding with the session's limits and run the trial apart (R8; C§1 to 3); count, write and read the line (C§5, 6); read and apply the ledger's new entry (C§8, 9); `explain` prints a finding and its trial (C§10). T022 to T024 pass
- [ ] T026 [US5] Run `cargo run -q --release -p zmachine-corpus -- write` on every story present and read each `lookahead` line. For each finding not borne out, run `explain`, and write its ledger entry only if the cause is the game's own draw; **stop** if any has another cause. Rewrite `stories/corpus/reports/` (C§7)
- [ ] T027 [US5] Set the kept figures against research R1 and R2 and write both in this file's closing notes: ways walked, refused, left and not borne out, for each story and all; **stop** if fewer than nine in ten are settled with the limits in force (SC-002), or if any finding is not borne out without a ledger entry (SC-003)
- [ ] T028 [US5] Update `specs/005-corpus-testing/contracts/formats.md` with the new line and the new ledger key, by pointing to `contracts/corpus.md`

---

## Phase 8: The app (User Stories 1 and 4)

- [ ] T029 Run `scripts/build-ios.sh`, then `scripts/test-ios.sh 27`, and list every UI test that fails because a way it expected as `maybe` is now settled (L§25); change each to the settled state and no further. A test that fails for any other reason is a **stop**
- [ ] T030 Failing test, then nothing, `testTheTrapDoorComesAndGoes` in `apps/ios/LamplightUITests/LamplightUITests.swift` (L§24; SC-001, SC-006, SC-007): in the living room `compass.down` has value `closed` and is not enabled; after `move rug` and `open trap door` it has value `open`, is enabled, and its label ends ", into darkness"; after fetching and lighting the lamp its label does not contain "darkness". Since no app code changes, the notes say how the test was shown able to fail
- [ ] T031 Take simulator screenshots of two places in Zork I where a way the game refuses has left the compass (the living room before the rug is moved, and one place where the game's text speaks of a way that is guarded or shut), each beside the same place as drawn on `main`. Send them to the owner and **stop** for their eye. Record what was chosen
- [ ] T032 T029 and T030 pass on iOS 26 and 27; confirm `git diff main --stat -- apps/ios/Lamplight` is empty

---

## Phase 9: Polish

- [ ] T033 [P] `README.md`, `stories/corpus/README.md` and `apps/ios/README.md`: what look-ahead is, the three copies and why, what a refused way and a fatal way show, the proof and its figures
- [ ] T034 [P] Correct this feature's contracts, data model and quickstart to what was built
- [ ] T035 Run the timing tests of features 003 and 004 with `--release -- --ignored` on a quiet machine, and the session's start; **stop** if a limit is exceeded (FR-016, FR-017; L§20)
- [ ] T036 Write the list this file ends with (FR-024); confirm `git diff main --stat -- crates/zmachine-core` shows only the files the guard names
- [ ] T037 Run every step of `quickstart.md`; clippy and `cargo fmt --check` pass; write "What was done, and what was done differently from the plan"
- [ ] T038 Play by hand in the simulator: the living room before and after the trap door, with and without the lamp; a walk of twenty rooms watching for a way that is shown and then refused when touched. Record what was seen. Listening with VoiceOver needs a device and is the owner's, as it was for feature 008

---

## Dependencies & Execution Order

- Phase 2 blocks every story. T002 before T004; T003 before any test that uses its
  fixture.
- US1 first: US2, US3 and US4 each extend `lookahead.rs` and follow it, in that order,
  since they edit the same file. US5 after US3, because the corpus uses the session's
  limits. Phase 8 after T021 and T026.
- Within a phase, every test task before the task that makes it pass.
- In Phase 3: T010 before T011 before T012 before T013.
- In Phase 7: T025 before T026 before T027.
- In Phase 8: T029 before T030 before T031 before T032.

### Parallel Opportunities

- T002 and T003; T006 to T009 and T014; T022 to T024; T033 and T034.

## Implementation Strategy

### MVP

Phases 1 to 3: the compass settles its uncertain ways, in Rust, proven on a fixture and
on Zork I's walkthrough transcript. Not yet safe to ship: chance and harm are Phases 4
and 5.

### Incremental Delivery

1. Phases 1 to 5: look-ahead and everything it must not get wrong.
2. Phase 6: darkness on a way found open.
3. Phase 7: the proof on every story, and the reports rewritten.
4. Phase 8: the app's tests, and the owner's eye on what has vanished.
5. Phase 9: measurements, READMEs, the list, a hand-play.

One pull request.

### What was learned from the earlier features

- A probe test (`tests/zz_*.rs`) is the quick way to measure; it is removed before the
  task is marked done.
- Measure timings on a quiet machine, not while the UI suite runs.
- The formatter rewraps test text; edit by position, not by matching wrapped strings.
- The owner judges drawings on pictures from the simulator; build the plain version
  and stop.
- Text typed into the simulator arrives late; wait for the field before the next key.
- This feature's own research ran with temporary code since removed; T018 and T027
  repeat its figures as kept ones.

## Tests of earlier features edited because a value or form changed

To be written by T036.

## What was done, and what was done differently from the plan

To be written by T037.
