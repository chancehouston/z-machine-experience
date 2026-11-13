# Tasks: Resume the Game

**Input**: Design documents from `/specs/011-resume-game/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/kept-game.md,
quickstart.md

**Tests**: Required. The constitution's Principle II is test-first: in every phase the
test tasks come before the code that makes them pass, and each is seen to fail first.

**Clause names**: K§, L§ and A§ are clauses of `contracts/kept-game.md`. R is an entry
of `research.md`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run alongside others (a different file, nothing unfinished it needs)
- **[Story]**: the user story of spec.md the task serves

---

## Phase 1: Setup

- [x] T001 Add `sha2 = "0.10"` to `[dependencies]` in `crates/zmachine-experience/Cargo.toml` and confirm `cargo build -p zmachine-experience` passes
- [x] T002 [P] Add a built story that decides something by chance at a command (a room whose reply prints one of several words by `random`) to `crates/zmachine-testkit/src/toy.rs`, or confirm `Toy::chance_room` already serves, and note which in this file

---

## Phase 2: Foundational (the kept form)

**Purpose**: the bytes of contract K, written and read. Every story needs them.

- [x] T003 Write tests in `crates/zmachine-experience/src/kept_tests.rs` for writing the form: the example of contract K is produced byte for byte from its parts; an empty input keeps the space after its word (K§6); `fixed` and `fresh` chances (K§3); the same parts give the same bytes (K§9). See them fail
- [x] T004 Write tests in `crates/zmachine-experience/src/kept_tests.rs` for reading the form: what was written is read back equal; and each departure of K§8 is refused without a panic: a missing line, lines out of order, a count that does not match (K§4), a digest that is not 64 hex digits, a `fixed` chance above 2147483647, bytes that are not UTF-8, a carriage return, bytes after the end, a wrong `check` (K§7), form `2` (K§1), and the example cut short at every length from 0 to its full length less one. See them fail
- [x] T005 Implement `crates/zmachine-experience/src/kept.rs`: a `Kept` value with the fields of data-model.md ("form `1`", "story: SHA-256 of the story file, 64 hex digits", "chance: `fixed N` or `fresh N`", "inputs: count, then the inputs in order", "transcript: SHA-256 over every turn's text in order, the opening first", "check: SHA-256 of every byte before the last line"), `to_bytes`, `from_bytes` giving a reason on failure, and the transcript digest of K§5 (each turn's text followed by one zero byte). Declare the module in `crates/zmachine-experience/src/lib.rs`. T003 and T004 pass

**Checkpoint**: the form is fixed and tested; no session uses it yet.

---

## Phase 3: User Story 3 - The game brought back is the game I left (Priority: P1)

**Goal**: a session made from kept bytes is the session that made them.

**Independent Test**: play a walkthrough twice with the same chance, once without
stopping and once kept and brought back at many points; every reply and snapshot is
the same.

This story comes first because stories 1 and 2 are this, seen from the app.

### Tests

- [x] T006 [US3] In a new `crates/zmachine-experience/tests/resume.rs`, on a built story: a session with no kept bytes opens as `new` (L§1, FR-006); `kept()` of a new game has no inputs and reads back; after three commands (one of them a tap) `kept()` holds them in order with their sources (L§6, FR-025). See them fail
- [x] T007 [US3] In `crates/zmachine-experience/tests/resume.rs`, on the built story of T002 with a fresh chance: after each of its commands, take `kept()`, open a second session from it, and check the outcome is `brought back`, the turns are equal in command, source and text, and `snapshot_json(0)` is equal but for `generation`; then give both the same further commands and check every reply and snapshot stays equal (L§2, L§5, FR-007, FR-008). Bring the second session back a second time and check again (US3 scenario 3). See it fail
- [x] T008 [US3] In `crates/zmachine-experience/tests/resume.rs`: a game waiting for an answer, and one waiting for a key, come back waiting for the same (FR-004); a game that ended comes back ended with the layer's message and refuses a command (edge case); after the game's own RESTART is typed and answered, what comes back is the restarted game (edge case). See them fail
- [x] T009 [US3] In `crates/zmachine-experience/tests/resume.rs`: a session whose last command faulted gives from `kept()` the bytes it gave before that command, and a session opened from them waits for a command with one turn fewer (L§6, R6). See it fail
- [x] T010 [US3] In `crates/zmachine-experience/tests/resume.rs`: bringing back gives the game exactly the kept inputs. Count the turns and compare each turn's command with the kept input; the transcript gains nothing (L§4, FR-009). See it fail
- [x] T011 [US3] In `crates/zmachine-experience/tests/walkthrough.rs`: Zork I with its pack and seed 11, kept and brought back after every one of the walkthrough's commands, gives turn for turn the replies and snapshots of the session never closed (SC-002, FR-028). Mark it `#[ignore]` only if it takes over ten seconds in a debug build, and say so here. See it fail

### Implementation

- [x] T012 [US3] In `crates/zmachine-experience/src/lib.rs` add to `Session` the fields of data-model.md: `chance` (the `Seed` the game was started with) and `answered` (the inputs answered so far). Record an input in `give` when the stop that follows is not a fault (R6). `new_game` empties `answered` and takes the new chance (L§11)
- [x] T013 [US3] In `crates/zmachine-experience/src/lib.rs` implement `Session::kept(&self) -> Vec<u8>` (L§6) from `chance`, `answered`, the story's SHA-256 (computed once when the session is made) and the digest of the turns those inputs gave
- [x] T014 [US3] In `crates/zmachine-experience/src/lib.rs` implement `Session::open(story, pack, chance, kept) -> Result<(Session, Outcome), StartError>` with `Outcome::{New, BroughtBack, CouldNot}` (L§1, L§2): read the bytes, check form and story, start with the kept chance, give each input with its source through the same path as `give`, with look-ahead off until the last input (L§3), stop on a fault or the instruction ceiling, then compare the transcript digest. Keep `Session::new` and `with_pack` as they are (L§16). T006 to T011 pass
- [x] T015 [US3] Run `cargo test -p zmachine-experience` and `cargo test --workspace`; every earlier test passes

**Checkpoint**: the layer can keep a game and bring it back, and it is proven the same.

---

## Phase 4: User Story 5 - A kept game that cannot be brought back (Priority: P2)

**Goal**: any unusable kept game gives a new game and one message.

**Independent Test**: open a session with damaged bytes and with another story's kept
game; each gives a new game and the notice.

Done before the app's stories because the app's launch path needs the outcome.

### Tests

- [x] T016 [US5] In `crates/zmachine-experience/tests/resume.rs`: for kept bytes that are empty, cut short at every length, altered at each single byte position in turn (at least 1,000 different damaged games over a kept game of twenty turns, SC-006), of form 2, and kept for a different built story, `Session::open` gives `CouldNot`, a new game with the given chance, and no panic (L§2, L§12, FR-017, FR-018, FR-029). See it fail
- [x] T017 [US5] In `crates/zmachine-experience/tests/resume.rs`: kept bytes that are well formed but whose transcript digest is wrong (made by writing a `Kept` with a false digest) give `CouldNot` (R3); kept bytes whose inputs make the game exceed a lowered instruction ceiling give `CouldNot` and return (L§12). See them fail
- [x] T018 [US5] In `crates/zmachine-experience/tests/resume.rs`: on `CouldNot` the snapshot's `notice` is "Your earlier game could not be brought back. This is a new game." and is at the front of `announcement`; after one command `notice` is null; `kept()` of that session reads back as the new game, so a second opening is `BroughtBack` with no notice (L§7, L§9, US5 scenario 4). On `New` and `BroughtBack` `notice` is null. See them fail
- [x] T019 [US5] In `crates/zmachine-experience/tests/resume.rs`: a session with a game pack, kept, is brought back by `Session::open` given no pack and given a different pack text (FR-019). See it fail
- [x] T020 [P] [US5] In `crates/zmachine-experience/src/json_tests.rs` add `notice` and `start_over` after `announcement` to the list of the snapshot's keys, with tests that `notice` renders as text or `null`. See them fail

### Implementation

- [x] T021 [US5] In `crates/zmachine-experience/src/lib.rs` add `notice: Option<String>` to `Session`, set it in `Session::open` on `CouldNot`, clear it when the next turn is recorded, and put it before the turn's text in `announce()` when present (L§7, L§9). Export the words as a constant `NOTE_NOT_BROUGHT_BACK`
- [x] T022 [US5] In `crates/zmachine-experience/src/json.rs` render `notice` and `start_over` (the latter `null` until T031) after `announcement`. Update the checks in `crates/zmachine-experience/tests/session.rs` that name the snapshot's last key (R11) and list each changed test in this file. T016 to T020 pass

**Checkpoint**: the layer never brings back a wrong game and never fails on bad bytes.

---

## Phase 5: The C interface

**Purpose**: what the app calls. Needed by stories 1, 2 and 4.

- [x] T023 Write tests in `crates/zmachine-ffi/src/lib.rs` (beside the existing ones): `zmx_session_open` with null kept gives outcome 0; with the bytes of `zmx_session_kept` gives outcome 1 and the same snapshot but for `generation`; with damaged bytes gives outcome 2; a null story or zero length gives the existing error; `zmx_bytes_free` accepts what `zmx_session_kept` gave and accepts null (L§13, L§14). See them fail
- [x] T024 Implement in `crates/zmachine-ffi/src/lib.rs`: `zmx_session_open(story, length, seed, has_seed, entropy, pack, kept, kept_length, outcome, error)`, `zmx_session_kept(session, length)`, `zmx_bytes_free(buffer, length)` and `zmx_session_keep_failed(session)` (a stub until T036), each inside `catch_unwind` as the existing functions are. Regenerate or update the C header the app uses. T023 passes

---

## Phase 6: User Story 1 - The game is where I left it (Priority: P1) 🎯 MVP

**Goal**: opening the app shows the game left, ready for a command.

**Independent Test**: play five commands, close the app completely, open it; the same
five turns and ways out are shown and a sixth command is answered.

### Tests

- [x] T025 [US1] In `apps/ios/LamplightUITests/LamplightUITests.swift` change the shared `launch` helper to pass `-fresh` by default, and add `relaunch()` which terminates the app and launches it with the same arguments less `-fresh` (R10). No existing test's expected values change; list here any that needed more than the helper
- [x] T026 [US1] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testTheGameComesBackWhereItWasLeft`: with seed 11, send `open mailbox` and tap a way; relaunch; the three turns' texts are in the transcript, the status and the compass's states are what they were, the command field takes text at once with nothing to dismiss, and `take leaflet` is answered as in a session never closed (US1 scenarios 1 to 4). See it fail
- [x] T027 [P] [US1] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testAnEndedGameComesBackEnded`: on the fixture story that ends, end the game, relaunch; the layer's ending words and `button.newGame` are shown; New game starts without asking (edge case, FR-016). See it fail
- [x] T028 [P] [US1] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testTheNewestReplyIsSpokenOnceOnReopening`: with `-speakAlways`, play two commands, relaunch; `announcement.last` holds the second reply and the count of announcements since launch is one (FR-027). See it fail

### Implementation

- [x] T029 [US1] Create `apps/ios/Lamplight/KeptFile.swift`: the file's URL (Application Support, folder `Kept`, named for the story's resource name), `read() -> Data?`, `write(_:) throws` (atomic, creating the folder, setting `isExcludedFromBackup`), and `remove()` (A§3)
- [x] T030 [US1] In `apps/ios/Lamplight/Engine.swift` make the engine with `zmx_session_open`, passing kept bytes and reading the outcome; add `kept() -> Data`. In `apps/ios/Lamplight/Game.swift`: with `-fresh` remove the kept file first (A§6); read the kept file and hand it over (A§1); after the first snapshot and after every command's snapshot, before `busy` is cleared, write `engine.kept()` (A§2, FR-010). T026 to T028 pass
- [x] T031 [US1] Run `scripts/build-ios.sh` and `scripts/test-ios.sh 27`; every earlier test passes with `-fresh`

**Checkpoint**: the feature's purpose works in the simulator.

---

## Phase 7: User Story 2 - Nothing I saw answered is lost (Priority: P1)

**Goal**: an abrupt end loses nothing answered, and never leaves half a game.

**Independent Test**: play five commands and end the process abruptly; open; all five
turns are there.

- [x] T032 [US2] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testNothingAnsweredIsLostWhenTheAppIsEndedAbruptly`: send five commands, and after each one terminate and relaunch at once; every answered turn so far is shown each time (US2 scenario 1, a small case of SC-003). See it pass or fail and record which: T030 may already meet it
- [x] T033 [US2] In `crates/zmachine-experience/tests/resume.rs`: take `kept()` after every turn of a twenty-turn session and check each is a whole kept game that opens as `BroughtBack` at exactly that turn (FR-011 from the layer's side: any file the app could have finished writing is whole)
- [x] T034 [US2] Review `apps/ios/Lamplight/KeptFile.swift` and `Game.swift` against FR-010 to FR-012 and write the finding here: the write is atomic, it completes before `busy` clears, and a failed or interrupted write leaves the earlier file. Correct what does not hold
- [x] T035 [US2] Write tests in `crates/zmachine-experience/tests/resume.rs`: `keep_failed()` sets `notice` to "Your place is not being kept." once; a second call in the same game sets nothing; after `new_game` it sets it again (L§8, FR-013). See them fail
- [x] T036 [US2] Implement `Session::keep_failed` in `crates/zmachine-experience/src/lib.rs` with the field `keep_failed_told` and the constant `NOTE_NOT_KEPT`; complete `zmx_session_keep_failed` in `crates/zmachine-ffi/src/lib.rs`; in `apps/ios/Lamplight/Game.swift` call it when the write throws and show the snapshot that follows (A§2). Show `notice` above the command field in `apps/ios/Lamplight/GameView.swift` with identifier `notice`, and add `notice` to `apps/ios/Lamplight/Snapshot.swift` (A§5). T035 passes
- [x] T037 [US2] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testADamagedKeptGameGivesANewGameAndSaysSoOnce`, using a debug-only launch argument `-damageKept` that overwrites the kept file with a few bytes before it is read: the opening text is shown with the notice; after relaunch without the argument the notice is gone and the game is the new one (US5 scenarios 1 and 4, in the app). Implement the argument in `apps/ios/Lamplight/Game.swift` under `#if DEBUG`

**Checkpoint**: keeping is safe against the ways a phone ends an app.

---

## Phase 8: User Story 4 - I can start over when I choose (Priority: P1)

**Goal**: a way to start over during play, which asks first.

**Independent Test**: in the middle of a game choose Start over and confirm; the
opening text is alone in the transcript; relaunch; it is the new game.

### Tests

- [x] T038 [US4] In `crates/zmachine-experience/tests/resume.rs`: the snapshot's `start_over` is "Start over? The game in progress will be given up." while the game waits for a command or an answer, and `null` when it has ended or faulted (L§10). After `new_game`, `kept()` has no inputs and the new chance (L§11). See them fail
- [x] T039 [US4] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testStartingOverAsksAndIsNotUndone`: play two commands; open About; choose `button.startOver`; the confirmation shows the layer's question; cancel, and the transcript is unchanged; choose again and confirm; the sheet closes and the transcript holds the opening alone; relaunch; it still does (US4 scenarios 1 to 3, SC-007). See it fail

### Implementation

- [x] T040 [US4] In `crates/zmachine-experience/src/lib.rs` and `json.rs` give `start_over` its value (L§10) with the constant `ASK_START_OVER`. T038 passes
- [x] T041 [US4] In `apps/ios/Lamplight/Snapshot.swift` add `startOver`; in `apps/ios/Lamplight/AboutView.swift` add the "Start over" button (`button.startOver`) with a confirmation whose message is the snapshot's text, starting a new game without asking when it is null; pass the game into the sheet from `apps/ios/Lamplight/GameView.swift`. `Game.newGame` writes the new kept game before the snapshot is shown (A§4, FR-015). T039 passes

**Checkpoint**: all five stories work.

---

## Phase 9: Limits, documents and the hand check

- [x] T042 In `crates/zmachine-experience/tests/speed.rs` add ignored timing tests that print and assert: bringing back Zork I's walkthrough (331 turns) and a 2,000-turn session whose commands all do work (walk a loop of rooms in a live game, not answers to a question), each under the launch limit by `time_limit` (FR-020); `kept()` at the first turn and the 2,000th (FR-021); the size in bytes at 2,000 turns and that it is in proportion to the turns (FR-022). Run them in release and record the figures here
- [x] T043 Run `cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture` whole and confirm every command of the walkthrough is within the earlier limit with keeping added (SC-005); record the figures here
- [x] T044 [P] In `apps/ios/LamplightUITests/LamplightUITests.swift` extend the launch-time test, or add one, that reads `launchMillis` after a relaunch into a kept game and asserts the launch limit; record the figure on iOS 26 and 27 here (FR-020 in the simulator)
- [x] T045 [P] Update `README.md` and `apps/ios/README.md`: the game is kept and brought back; where the file is; that it holds the player's inputs and no text of a game; `-fresh`; what the notice means. State in `apps/ios/README.md` that feature 004's FR-005 is replaced
- [x] T046 Run the whole of quickstart.md sections 1 to 3: `cargo test --workspace`, the timing tests, `scripts/test-ios.sh 27` and `scripts/test-ios.sh 26`. Record the counts here
- [x] T047 List here every test of an earlier feature that was changed and why (FR-031), from T022 and T025
- [x] T048 Build for the owner's phone and walk quickstart.md section 4 with the owner, including the restart of the phone and the launch time by eye for a short game (SC-008). Record what was seen. This task is the owner's and cannot be done without them

---

## Dependencies & Execution Order

- **Phase 1** first. **Phase 2** needs T001.
- **Phase 3 (US3)** needs Phase 2. It is the layer's whole ability.
- **Phase 4 (US5)** needs Phase 3.
- **Phase 5** needs Phases 3 and 4.
- **Phase 6 (US1)** needs Phase 5. **Phase 7 (US2)** needs Phase 6.
- **Phase 8 (US4)** needs Phase 6; it does not need Phase 7.
- **Phase 9** needs all of them.

Within a phase the tests come first and are seen to fail.

### Parallel opportunities

- T002 beside T001.
- T020 beside T016 to T019 (a different file).
- T027 and T028 beside each other once T025 is in.
- T044 and T045 beside T042 and T043.
- Phases 7 and 8 could be done in either order.

## Implementation Strategy

**The smallest thing worth having** is Phases 1 to 6: the layer keeps and brings back,
refuses bad bytes, and the app resumes. Without Phase 8 the player cannot start over
during play, so the feature is not to be merged before Phase 8.

**Order to build**: as numbered. Stop at each checkpoint and run the workspace's tests.

**One pull request** for the feature, as for the earlier ones.

## Notes

- 48 tasks: setup 2, the form 3, US3 10, US5 7, the C interface 2, US1 7, US2 6, US4 4,
  limits and documents 7.
- T032 may pass as soon as it is written; that is recorded, not hidden.
- T048 needs the owner and their phone.
- Nothing here has been measured on a phone (plan.md, second objection). T048 is where
  it first is, and only by eye; a measured figure on the device is not among these
  tasks and would need the launch time shown on screen in a debug build.

---

## Record of the building (2026-11-13)

**Test-first, as it was done.** Not strictly. The tests of the kept form (T003, T004)
and its code (T005) were written together and first run together. The tests of
Phase 3 onward were written after `Session::open_kept` and `kept` existed, and were
not each seen to fail against code that lacked them. Seven of them did fail on their
first run, for a defect in a test story (below), which is not the same thing. The
constitution asks for more than this.

**What differs from the tasks as written**

- **T001**: undone. A test that exists holds the experience crate to depending only on
  the layers beneath it. `zmachine-introspect` now offers `pack::sha256_hex` and the
  layer uses it (research R4).
- **T002**: `Toy` with `COIN` serves for ways decided by chance. For replies decided by
  chance the tests of `tests/resume.rs` build their own small story. The shared
  `roller` of `tests/common/mod.rs` was found to be good for one command only: it
  stores its draws in the global a version 3 status line reads the place from, so its
  second read faults. It is left as it is; its one user sends one command.
- **T011**: in `tests/resume.rs`, not `walkthrough.rs`. The ordinary run brings Zork I
  back before every sixteenth command; `zork_brought_back_after_every_command_is_the_
  same_game` does every command and is `#[ignore]`, being slow in a debug build. Run in
  release on 2026-11-13: passed, 3.3 s.
- **T014**: the function is `Session::open_kept`, since `open` was a private name, with
  `open_kept_within` for a test's lower ceiling.
- **T016**: 1,030 damaged games (each length cut short, and each byte altered three
  ways), over a kept game of twenty turns.
- **T023, T024**: `zmx_session_kept` gives a string freed with `zmx_string_free`; there
  is no `zmx_bytes_free` (contract L§14).
- **T025, T026**: `testEachLaunchIsANewGame` is removed and
  `testTheGameComesBackWhereItWasLeft` stands in its place.
- **T027**: the bundle holds one story, so the game is ended as the earlier test ends
  it: Zork I's `quit`, then `y`.
- **T030, T034**: the kept game is written before the snapshot is shown, not after:
  an answered turn the player has seen is always stored. The write is atomic, is made
  inside the engine's actor, and completes before `busy` clears. A failed write leaves
  the earlier file and the layer says so once.
- **T032**: passed when first run correctly. Its first run failed for the test's own
  mistake (a way that is not there).
- **T039**: iOS 27 gives the confirmation no Cancel button; the test declines by
  touching away from it where there is none.

**Timings (T042, T043)**, development Mac, release build, Zork I with its pack:

| Measure | Figure | Limit |
| --- | --- | --- |
| Bringing back the walkthrough (332 turns) | 28 ms | 1 s (the layer's share of 2 s) |
| Bringing back 2,000 turns, each a step of a walk round the house | 91 ms | 1 s |
| Slowest command with its snapshot and kept game, over the walkthrough | 0.75 ms | 10 ms |
| Making the kept game: first turn / 2,000th | 0.005 ms / 0.32 ms | 10 ms |
| Kept game: walkthrough / 1,000 turns / 2,000 turns | 5,209 / 11,763 / 23,263 bytes | in proportion |

Nothing has been measured on a phone.

**The whole of quickstart sections 1 to 3 (T046)**, 2026-11-13:

- `cargo test --workspace`: 964 passed, 0 failed, 20 ignored. `cargo clippy` clean.
- The timing tests in release: all four passed, with the figures above.
- The app's tests: 37 passed and none failed on iOS 27.2, and the same on iOS 26.0.
  An earlier run on iOS 26.0 had one failure, in `testStartingOverAsksAndIsNotUndone`:
  the test touched the sheet's own "Start over" where it meant the confirmation's.
  The test was corrected; the app was not at fault.
- **T044**: `testLaunchTimeIntoAKeptGame` passes on both, under the limit of 2,000 ms,
  for a kept game of seven turns. The figure itself was not captured from the log.
- **Changed after the first run**: "Start over" was moved above the license text in the
  About sheet, where it was below it and out of sight without scrolling.

**T048, on the owner's phone** (iPhone 15 Pro, iOS 27.2.1), 2026-11-13: a debug build
was installed and the owner was given the walk of quickstart section 4, steps 1 to 6:
play, close from the app switcher and reopen, restart the phone and reopen, Start over
declined, Start over confirmed, close and reopen. Their answer to "did the game come
back where you left it" was "Yes, all worked". They did not report on the steps one by
one, so no step is recorded as seen apart from that answer. Step 7 (VoiceOver speaking
the newest reply on reopening) was not among the steps given to them and is not
confirmed on a device. No launch time was measured on the phone; none was remarked on.

**Tests of earlier features that changed (T047, FR-031)**

- `crates/zmachine-experience/src/json_tests.rs`: the snapshot's key list gains
  `notice` and `start_over`.
- `crates/zmachine-experience/tests/session.rs`: `the_snapshot_ends_with_the_
  announcement` is now `the_snapshot_holds_the_announcement`; the announcement is no
  longer the last key.
- `crates/zmachine-ffi/tests/abi.rs`: the list of declared functions gains three.
- `crates/zmachine-ffi/tests/app_rules.rs`: the engine's one way of starting a session
  is `zmx_session_open`.
- `apps/ios/LamplightUITests`: the `launch` helper passes `-fresh`;
  `testEachLaunchIsANewGame` is replaced (above). No other test's expected values
  changed.
