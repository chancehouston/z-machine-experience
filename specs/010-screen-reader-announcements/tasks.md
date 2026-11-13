# Tasks: Screen Reader Announcements

**Input**: Design documents from `/specs/010-screen-reader-announcements/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started. Where code must exist for a test to compile, it is a stub
that answers nothing. Where a test passes before its code exists, or is written with
its code, the task's notes say so and say what was done to show it can fail.

Tasks cite what they serve: "FR-" and "SC-" are the spec's; "R" a section of
`research.md`; "N§" a numbered statement of `contracts/announcements.md`; "D" a table of
`data-model.md`.

**The guard**: no file under `crates/zmachine-core`, `crates/zmachine-introspect` or
`crates/zmachine-corpus` is edited. After every phase `cargo test --workspace` passes.
A test of features 001 to 009 is edited only where this feature changes the written
form it fixes (the snapshot's last key; `message` for an ended game), and every such
edit is listed at the end of this file (FR-019). No label, value or identifier an
earlier feature gave an element of the app changes (N§21).

**Stop points**: stop for the project owner if an earlier test that fixes a value other
than those two fails; if a timing limit is exceeded on the owner's Mac (T017); and at
T016, where the owner listens on a phone. If what is heard there is not what N§22
says, stop and report before changing anything.

**The owner's decision of 2026-11-13**, after trying the screen reader on a phone: the
owner will not use the feature themselves and wants it finished as an accessibility
feature, not left half done.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create `crates/zmachine-experience/src/announce.rs` and `announce_tests.rs`, each with a module comment that cites this feature, and declare them in `crates/zmachine-experience/src/lib.rs`; `cargo test --workspace` passes

---

## Phase 2: Foundational

**⚠️ CRITICAL**: No user story starts until this phase is complete

- [x] T002 [P] Failing tests `reply_*` in `crates/zmachine-experience/src/announce_tests.rs` (N§1; R1; D "Reply"): text ending in a line feed, a blank line and `>` gives the text before them when the game waits for a command or an answer; the same text when the game has ended or wants a key keeps its `>`; a `>` inside the text is kept; white space at both ends goes; text that is only a prompt gives nothing
- [x] T003 Implement the reply in `crates/zmachine-experience/src/announce.rs`; T002 passes

---

## Phase 3: User Story 1 - The game's reply is spoken when it arrives (Priority: P1) 🎯 MVP

**Goal**: the layer gives each turn's reply to be spoken, and the app posts it.

**Independent Test**: start the game with `-speakAlways`, give three commands; the
element the argument adds holds each reply in turn.

### Tests

- [x] T004 [P] [US1] Failing tests in `crates/zmachine-experience/tests/session.rs` (N§2, 8 to 11; FR-001 to FR-003, FR-007): on Zork I the opening's announcement is its text without the prompt; after `north` it is that turn's reply and does not hold the word typed; after `new_game` it is the opening's again; a command refused as stale leaves it as it was; on a built story whose reply is only a prompt it is none
- [x] T005 [P] [US1] Failing test in `crates/zmachine-experience/tests/walkthrough.rs` (SC-006; FR-014): over Zork I's walkthrough every turn's announcement, where the game is waiting for a line and it is not dark, is exactly that turn's text with its white space and final `>` removed
- [x] T006 [P] [US1] Failing tests in `crates/zmachine-experience/src/json_tests.rs` (N§12, 14): the snapshot's last key is `announcement`, with the text escaped as other text is, or `null`; every earlier key is where it was. Edit and list the tests in `crates/zmachine-experience/tests/session.rs` and `crates/zmachine-ffi/tests/` that fix a snapshot's whole text
- [x] T007 [US1] The session keeps the announcement and makes it when a turn is recorded, in `crates/zmachine-experience/src/lib.rs` and `announce.rs`, with a public reader; the snapshot writes it in `json.rs`; T004 to T006 pass
- [x] T008 [US1] Failing UI test `testEachReplyIsAskedToBeSpoken` in `apps/ios/LamplightUITests/LamplightUITests.swift` (N§16 to 20; FR-001, FR-009, FR-010; SC-004, SC-005): launched with `-speakAlways`, `announcement.last` has the opening as its label and `1` as its value; after a typed command and a tapped way its label is each reply and its value counts up; the command field still has the keyboard's focus after the typed one; launched without the argument there is no such element
- [x] T009 [US1] In `apps/ios/Lamplight/`: `Snapshot.swift` decodes `announcement`; a new `Announcer.swift` posts to the system, marked to interrupt, when a screen reader runs or `-speakAlways` is given, and counts; `Game.swift` asks it when a shown snapshot brought a new turn or a new game; `GameView.swift` shows the tests' element only with the argument. Run `scripts/build-ios.sh`; T008 passes

**Checkpoint**: a reply can be heard on a phone.

---

## Phase 4: User Story 2 - Darkness is said when it falls (Priority: P1)

- [x] T010 [P] [US2] Failing tests in `crates/zmachine-experience/tests/session.rs` on Zork I with its pack (N§3 to 5; FR-004; R3): up into the attic without the lamp, the announcement is the reply, a line feed, and "It is dark"; a `look` there leaves it the reply alone; down again, the reply alone; without the pack nothing is said of light. On a built story that opens in the dark, the opening's announcement ends with the note
- [x] T011 [US2] The session remembers whether it was dark at the stop before, in `crates/zmachine-experience/src/lib.rs`; T010 passes
- [x] T012 [US2] Extend the UI test of T008, or add `testDarknessIsAskedToBeSpoken` (SC-002): after walking up into the attic `announcement.last` ends with "It is dark"; after walking down it does not. Since no app code changes for it, the notes say how it was shown able to fail

---

## Phase 5: User Story 3 - The newest reply is the one heard (Priority: P1)

- [x] T013 [US3] Failing test, then nothing or a fix, in `apps/ios/LamplightUITests/LamplightUITests.swift` (N§16; FR-008; SC-003): three lines typed straight through Return leave `announcement.last` holding the third reply and a value three more than before. That each post is marked to interrupt is in the app's source and cannot be seen by a test; the notes say so

---

## Phase 6: User Story 4 - A question, an ending and a failure are heard (Priority: P2)

- [x] T014 [P] [US4] Failing tests in `crates/zmachine-experience/tests/session.rs` on the built stories of `tests/common` (N§6, 7, 13; FR-005): a turn that asks a question announces the reply with its `>` removed; a key request announces the reply as it is; a turn that ends the game announces the reply then "The game has ended." and the snapshot's `message` is those words; a fault announces its message alone. Edit and list the earlier tests that fix `message` as null for an ended game
- [x] T015 [US4] Make T014 pass in `crates/zmachine-experience/src/`; in `apps/ios/Lamplight/GameView.swift` show `message` for an ended game as for a fault and remove the app's own sentence (N§15); the UI tests that read the ending still pass

---

## Phase 7: The owner's ear, and polish

- [x] T016 Build for the owner's phone and install it. The owner, with the screen reader on, plays Zork I: north, east, open window, west, up, down; then sends a command while a reply is being spoken. **Stop** for what they heard (N§22; FR-017; SC-001 to SC-003, SC-007), and record it here. The same sitting closes T045 of feature 008 and the listening half of T038 of feature 009: record it in those files too
- [x] T017 Run `scripts/test-ios.sh 26` and `27` in full and the timing tests of features 003 and 004 with `--release -- --ignored` on a quiet machine; **stop** if a limit is exceeded (FR-018)
- [x] T018 [P] `README.md` and `apps/ios/README.md`: what is spoken and when, that the layer writes it, the launch argument
- [x] T019 [P] Correct this feature's contract, data model and quickstart to what was built
- [x] T020 Write the list this file ends with (FR-019); confirm `git diff main --stat` shows nothing under the three crates the guard names; clippy and `cargo fmt --check` pass; write "What was done, and what was done differently from the plan"

---

## Dependencies & Execution Order

- Phase 2 blocks every story. US1 before the others: they extend what it builds.
- Within a phase, every test task before the task that makes it pass.
- T016 after T015, so that one sitting hears everything.

### Parallel Opportunities

- T004 to T006; T018 and T019.

## Implementation Strategy

Phases 1 to 3 make a reply heard. Phases 4 to 6 add the note, the replacement and the
rarer endings. Phase 7 is the owner's ear and the record. One pull request.

### What was learned from the earlier features

- Text typed into the simulator through a tool arrives late; the UI tests' own typing
  does not have that fault.
- A scroll or a focus that behaves on one iOS version may not on the next: run both.
- The owner judges on the real thing; build the plain version and stop.

## Tests of earlier features edited because a form changed

Each fixes a written form that this feature changed. None fixed a value that changed.

| Test file | What changed in it |
| --- | --- |
| `crates/zmachine-experience/src/json_tests.rs` | the snapshot's list of keys ends with `announcement` |
| `crates/zmachine-experience/tests/session.rs` | five places that fixed how a snapshot's text ends now fix what comes before `announcement`; `a_game_that_quits_has_ended` expects `message` to be the layer's words for an ended game, where it was null |

No test under `crates/zmachine-ffi` or in the app's UI suite needed changing. Nothing
under `crates/zmachine-core`, `crates/zmachine-introspect` or `crates/zmachine-corpus`
was edited.

## What was done, and what was done differently from the plan

**Done**: the experience layer makes, for each turn, what a screen reader is to speak,
and the snapshot carries it as its last key; the app posts it to the system when a turn
arrives and a screen reader is running, marked to cut off speech in progress; "The game
has ended." is the layer's and the app shows it from the snapshot's `message`.

**The owner's ear** (T016; N§22; FR-017), on an iPhone 15 Pro, iOS 27.2.1, with
VoiceOver on, 2026-11-13. Asked whether, using double-taps, replies are spoken on their
own with "It is dark" once in the attic, after being told what to listen for (each
reply spoken by itself; the dark said once on going up and not on coming down; a new
reply cutting off the old; the screen reader's place staying put), the owner answered:
"Yes, all as expected." Two further things were asked and not answered one by one:
whether the screen reader talks over a reply by reading the button again, and whether
the prompt character is ever read aloud.

Before that, the owner reported that the build "breaks the app": replies stopped
appearing after a direction was used. A capture of the phone's screen, taken with the
owner's leave, showed every reply in place. The owner then found the cause themselves:
with the screen reader on, one tap speaks an item and a double-tap acts on it, and they
had been tapping once. Nothing was wrong with the app. It is recorded because the next
person to try the screen reader will meet the same thing; `apps/ios/README.md` now says
so.

**The owner's decision** (2026-11-13): they will not use the feature themselves, and
chose to finish it as an accessibility feature and not leave it half done.

**Features 008 and 009.** The same sitting was to close their listening checks. What
the owner confirmed is this feature's announcements. That the marked way up is spoken
"into darkness", and the living room's way down "no exit" and then "to Cellar, into
darkness", were on the list given to the owner earlier that day and were not confirmed
one by one. The UI tests hold those labels. Their task files are left as they were.

**The UI suite** (T017): 32 tests, none failed, one skipped (the pictures), on iOS
27.2 (822 s) and on iOS 26 (799 s). Four of the 32 are new.

**Timings on a quiet machine** (T017, the owner's Mac, release build): over Zork I's
walkthrough a command and its snapshot took 119 µs on average and 0.68 ms at most,
against 10 ms; the slowest of 20 starts 0.54 ms against 50 ms. The introspection
crate's timing test passed.

**Differently from the plan**:

- **Tests were written with their code, not before it**, for the layer and the app.
  Five earlier tests failed on the first run for the two reasons expected (the
  snapshot's last key; `message` for an ended game), and no other.
- **The three tests of T012 and T013 passed on their first run.** The layer's own
  tests of the same walk fail when the note or the reply is left out; the app's were
  not seen to fail.
- **That each post is marked to interrupt cannot be tested** and is in
  `Announcer.swift` alone. The owner heard it cut off an earlier reply.
- **The phone needed signing.** Xcode's automatic provisioning registered the app's
  identifier and made a development profile in the owner's Apple account, with the
  owner's leave. Nothing about signing is in the repository.

**Not done**: nothing of this feature's tasks. The two questions above were not
answered one by one.
