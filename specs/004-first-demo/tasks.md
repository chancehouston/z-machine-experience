# Tasks: First Demo on a Phone

**Input**: Design documents from `/specs/004-first-demo/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started, and the failing output goes in the test task's commit
message. Where code must exist for a test to compile, it is a stub that answers nothing.
Where a test checks only how something looks and cannot fail first, the task says so and
names the screenshot that stands in for it.

Tests of the experience layer cite the contract they verify: "E§n" is rule n of
`contracts/experience-api.md`, "C§n" rule n of `contracts/c-abi.md`, "S§" a section of
`contracts/screen.md` by its heading. Tests that touch the Z-machine cite the Standard
("§") and tests of introspection answers cite `docs/introspection.md` ("I§").

**The guard**: after every task that touches `crates/zmachine-core`, `cargo test
--workspace` must pass with no test from features 001 to 003 edited. A needed edit to an
earlier test stops the task for the project owner.

**Simulator destinations on this Mac** (research R9): `platform=iOS Simulator,name=iPhone
18 Pro,OS=27.2` and `platform=iOS Simulator,name=iPhone 17 Pro,OS=26.0`. UI tests run on
the first unless a task says both.

**Stop points**: stop for the project owner if the screen text and the transcript stream
differ in a way the transcript rules do not explain (T014); if fewer than 150 walkthrough
commands can be sent as taps; if the prompt rule mislabels any walkthrough stop other
than the Loud Room; or if a UI test can only be made to pass by putting game logic in
Swift.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US5)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create crate `crates/zmachine-experience` (edition 2024, `#![forbid(unsafe_code)]`, depends on `zmachine-core` and `zmachine-introspect`; dev-dependency `zmachine-testkit`) and crate `crates/zmachine-ffi` (`crate-type = ["staticlib", "lib"]`, depends on `zmachine-experience`), each with an empty `src/lib.rs` that builds; add both to the workspace `Cargo.toml`
- [x] T002 [P] Add to `.gitignore`: `apps/ios/*.xcodeproj`, `apps/ios/Lamplight/Resources/story.z3`, `apps/ios/Lamplight/Resources/story-license.txt`, `apps/ios/Frameworks/`, and Xcode's `xcuserdata`; create `apps/ios/README.md` saying what is generated and by what

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The core's one addition, and the experience layer's skeleton with its rules

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### The core's accessor

- [x] T003 Failing tests in `crates/zmachine-core/tests/waiting_at.rs` (research R6; §15 sread, read, read_char): a stub `Machine::waiting_at()` that returns `None` is added so the tests compile; then: it is `None` before the first `run`, after the game has finished, after a fault, and while a save, restore or transcript answer is owed; it is `Some(address)` while a line is wanted and while a key is wanted, where the byte at that address in the story file is the opcode of the read instruction; two reads at different places in a built story give different addresses and the same read reached twice gives the same; for Zork I the address at the first prompt equals the address at the prompt after `north`, and differs from the address after `quit` (the question); a clone reports the same address; it is unchanged by `view()` and by taking output
- [x] T004 Implement `Machine::waiting_at` in `crates/zmachine-core/src/lib.rs` until T003 passes; run the guard

### The experience layer's types and rules

- [x] T005 [P] Failing tests in `crates/zmachine-experience/tests/crate_rules.rs` (E§12; constitution Principles IV and V): the crate's dependencies are exactly `zmachine-core` and `zmachine-introspect`; no source file contains `unsafe`, `std::fs`, `std::io`, `std::net`, `std::env`, `std::time`, `rand`, `println!` or `eprintln!`; none names a game (`zork`, `advent`, `adventure`, `czech`, `praxix`) or holds a six-digit serial number
- [x] T006 Define the public types of `contracts/experience-api.md` and `data-model.md` in `crates/zmachine-experience/src/lib.rs`, `compass.rs` and `transcript.rs` (`Session`, `Seed`, `Stamp`, `Refused`, `StartError`, `Waiting`, `Status`, `Turn`, `Source`, `Place` with its twelve values in the data model's order, `PlaceState` with `open`, `maybe`, `closed`, `unknown`, `idle`, `Compass`, `CompassPlace`, `Extra`), with every function a stub: `Session::new` returns a `StartError`, the others refuse or return empty values. No behaviour
- [x] T007 [P] Extend `crates/zmachine-testkit` only if the fixtures of feature 003 cannot express what T008 to T011 need (a ZIL-shaped version 3 story with a second read at another address that asks a yes-or-no question; one that requests a save, a restore and a transcript; one that loops for ever; one that faults after a command). Prefer building these in a `fixtures` module inside `crates/zmachine-experience/tests/common/mod.rs` from the testkit's existing `Story` builder; record which was done

**Checkpoint**: The core can say where a game is waiting; the layer's shapes exist and do nothing

---

## Phase 3: User Story 5 - What a tap does is exactly what typing does (Priority: P5, built first)

**Goal**: The whole of the experience layer, proven in Rust

**Why first**: The spec ranks this story last because it is checked last by a player.
But everything a player sees is decided in this layer (FR-026), and it can be built and
tested without a phone (FR-027). User Stories 1 to 4 are then presentation.

**Independent Test**: `cargo test -p zmachine-experience`

### Sessions, turns and waiting

- [x] T008 [US5] Failing tests in `crates/zmachine-experience/tests/session.rs` (E§1, E§5, E§6, E§8, E§10; data-model "Turn", "Session"): `Session::new` on Zork I gives turn 0 with no command, whose text is the opening and ends with the prompt, and `waiting` is `command`; `send_line` adds a turn with that command, source `typed`, and the game's reply unaltered; the stamp's `turn_count` goes up by one per turn; the same story, seed and lines give the same turns twice; a story file the core refuses gives a `StartError` with the core's message; `Seed::Fixed` above 2,147,483,647 is a `StartError`; two sessions with different `Seed::Fresh` numbers play differently where the game is random (Zork I's combat) and two with the same number play the same
- [x] T009 [P] [US5] Failing tests in `tests/session.rs` (E§2, E§3, E§4; FR-012, FR-016): a `send_line`, `tap` or `tap_extra` with a stamp from an earlier turn is refused with `Stale` and the turn count does not change, including when the new room offers the same direction; a stamp from an earlier generation is `Stale`; a line containing a line feed or a carriage return is `BadLine`; an empty line is accepted and the game answers it; after the game has ended every command is `NotWaiting`
- [x] T010 [P] [US5] Failing tests in `tests/session.rs` on built stories (research R6, R7; E§6, E§7; FR-015, FR-023, FR-024, FR-025): the first request for a line is `command`; a request from another read is `answer`, and the next request from the first read is `command` again; a request for a key is `answer` and takes the first character of the line sent; a game that quits is `ended`; a game that faults is `fault` with a message naming the fault; a game that never stops is `fault` after the instruction ceiling with a message saying the game is not responding; a save request is answered "not stored", a restore "nothing to restore" and a transcript "declined", none of them is a stop, and the game's following text is in the same turn; in Zork I, `save`, `restore` and `script` each produce a turn and leave `waiting` at `command`
- [x] T011 [P] [US5] Failing tests in `tests/session.rs` (E§1; data-model "Session"; research R12a; FR-022, SC-008): `new_game` raises the generation, resets the turns to a fresh turn 0 equal to a new session's, and makes every earlier stamp `Stale`; it works from `command`, `answer`, `ended` and `fault`; ten new games in a row each show the opening; a `restart` typed by the player (answering the game's question with `y`) leaves the generation unchanged and the turns growing; the status after `ended` is the last one drawn
- [x] T012 [US5] Implement sessions, turns, waiting states, stamps, the declined requests, the ceiling and `new_game` in `crates/zmachine-experience/src/lib.rs` and `transcript.rs` until T008 to T011 pass, with the compass still a stub

### The headline comparison

- [x] T013 [US5] Failing test in `crates/zmachine-experience/tests/walkthrough.rs` (research R14; FR-031, SC-004 first half): replay lines 3 to 333 of `stories/zork1/fidelity/walkthrough.txt` with seed 11, every command typed; build `>` followed for each turn after the opening by the command, a line feed and the turn's text; require it to equal `stories/zork1/fidelity/reference.scr` from its first command line to its end, with the one stated rule for the final line feed. It fails against T006's stubs and may pass once T012 exists; if it then fails, go to T014
- [x] T014 [US5] If T013 fails with T012 in place: list every difference between the layer's text and the reference; explain each from `specs/001-v3-core/contracts/transcript-format.md` or from the core's code; encode each explained kind as a named rule in the test with its count; stop for the project owner at any difference that cannot be explained. Record the outcome in this file whichever way it goes

### The compass

- [x] T015 [US5] Failing unit tests in `crates/zmachine-experience/src/compass_tests.rs` on built stories and on hand-made introspection answers (research R5; data-model "Compass"; I§8): a fixed exit whose words include a place's word cut to the dictionary's length is that place, `open`, with the full English word as its command and the destination's name; a conditional one is `maybe` with no `leads_to`; a place no exit matches is `closed`; an exit matching no place is an extra labelled with and sending its longest word; an exit with no words makes the compass not `known` and is not an extra; two exits matching one place make that place `unknown`; a direction in the answer's `unknown` list makes its place `unknown`; an unknown exits answer makes every place `unknown` and `known` false; version 3 cuts to 6 characters and versions 5 and 8 to 9 (`northeast` matches `northe` and `northeast`); no two places and no place and extra share a command
- [x] T016 [P] [US5] Failing unit tests in `src/compass_tests.rs` (data-model "Compass" overall fields; E§9 and "The words the layer owns"; FR-014, FR-015, FR-021, FR-030): when `waiting` is not `command`, `offered` is false, every place is `idle` with no command, and there are no extras; when offered, no place is `idle`; the `note` is "Exits unknown: type a direction" when not `known`, "No exits here" when known with nothing open or maybe and no extras, and none otherwise, and the two texts differ; each place's `label` and each state's `spoken` text are as the contract's table gives them, for a known and an unknown destination
- [x] T017 [US5] Failing tests in `tests/session.rs` (E§3, E§5; FR-010, FR-011, FR-012, FR-013): at Zork I's opening, `tap(north)` adds a turn whose command is `north`, source `tap`, with the same text as typing `north` in a second session; tapping a `closed` place is `NotOffered` and adds no turn; `tap_extra` sends the extra's command, and an index past the end is `NotOffered`; a tap during `answer` is `NotOffered`; after any turn the compass is that of the room the player is now in
- [x] T018 [US5] Implement `crates/zmachine-experience/src/compass.rs` and `tap`, `tap_extra` in `lib.rs` until T015 to T017 pass

### The whole game by taps

- [x] T019 [US5] Failing tests in `tests/walkthrough.rs` (research R14; SC-004, SC-005, SC-006; FR-008): replay the walkthrough sending a tap wherever the command equals the command of a lit place or extra, and typing otherwise; require the same text as T013; require at least 150 taps and print how many; require every tap's command to be the `command` of the turn it produced; beside the session keep a second `Machine` with the same seed given the same lines, and at each stop where the session is at `command` ask an `Introspector` for its exits and require the compass to light exactly those, by a words-to-places mapping written in the test; require the session to be at `command` at every stop where the second machine's text ends with a blank line and the prompt, except at the one stop research R6 names, and print any exception; at the walkthrough lines `stories/zork1/introspection/exits.txt` covers, require the compass to agree with that file
- [x] T020 [US5] Fix whatever T019 shows, each defect with a failing unit test in `compass_tests.rs` or `session.rs` first. If fewer than 150 commands go as taps, or the prompt rule mislabels a stop other than the Loud Room, stop for the project owner

### The snapshot

- [x] T021 [US5] Failing tests in `crates/zmachine-experience/src/json_tests.rs` (contracts/experience-api.md "Snapshot JSON"; data-model "Snapshot" and its validation rules), with a strict JSON reader in the test: the fields and their order; every field present, `null` where it has no value; the twelve places always listed in order; `turns` from `since` onward and `turn_count` the total; `since` past the end gives an empty list; text with quotes, backslashes, line feeds and non-ASCII characters is escaped and reads back equal; the snapshot at each of twenty stops of the Zork I walkthrough satisfies every validation rule of the data model; a `fault` snapshot carries its message
- [x] T022 [US5] Implement `crates/zmachine-experience/src/json.rs` and `snapshot_json` until T021 passes

**Checkpoint**: Everything a player will see is decided and tested, with no phone involved

---

## Phase 4: The bindings

**Goal**: The eight C functions

**Independent Test**: `cargo test -p zmachine-ffi`

- [x] T023 Write `crates/zmachine-ffi/include/zmx.h` and `include/module.modulemap` (`module ZMX { header "zmx.h" export * }`) exactly as `contracts/c-abi.md` declares them, and stub every function in `src/lib.rs` so that it links and returns 0 or null
- [x] T024 Failing tests in `crates/zmachine-ffi/tests/abi.rs`, calling the functions as C would (C§1 to C§4): a session made from Zork I's bytes gives a snapshot that parses and whose turn 0 text is the opening; `zmx_session_send_line` and `zmx_session_tap` with the current stamp return 1 and the next snapshot has the turn; with a stale stamp they return 0; `zmx_session_tap` with an unknown place name returns 0; `zmx_session_new_game` raises the generation; a null session makes every function return 0, null or nothing; a null line, a null place and bytes that are not UTF-8 return 0; a story that will not load returns null and sets the error string, and a null `error` pointer is accepted; a seed above 2147483647 with `has_seed` set fails with a message; `zmx_string_free(NULL)` and `zmx_session_free(NULL)` do nothing; a thousand sessions created, used and freed in a loop, and ten thousand snapshots taken and freed, leave the process's memory within a stated bound of where it began
- [x] T025 [P] Failing tests in `tests/abi.rs` (C§4, C§6): with a test-only hook that makes the next call panic inside the layer, the call returns 0 without unwinding into the caller, the snapshot then reports `fault` with a message, further commands return 0, and `zmx_session_new_game` restores a working game; the symbols `nm` lists in the built static library that begin `zmx_` are exactly the functions `zmx.h` declares (the test is skipped with a notice if `nm` is not found, and fails if the list differs)
- [x] T026 Implement `crates/zmachine-ffi/src/lib.rs` until T024 and T025 pass, each `unsafe` block carrying a comment that states what the caller promised (C§5); add a test in `tests/abi.rs` that every `unsafe` in the source is preceded by a `// SAFETY:` comment, and that `unsafe` appears in no other crate of the workspace

**Checkpoint**: The layer can be called from C

---

## Phase 5: User Story 1 - Open the app and read the game (Priority: P1) 🎯 MVP

**Goal**: Lamplight launches in the simulator and shows the opening of the game

**Independent Test**: `xcodebuild … test` runs the launch test green

- [x] T027 [US1] Write `scripts/build-ios.sh` (research R9, R12): check for `xcodegen` and the two Rust targets and say how to install what is missing; build `zmachine-ffi` in release for `aarch64-apple-ios-sim` and `aarch64-apple-ios`; make `apps/ios/Frameworks/ZMX.xcframework` from the two static libraries and `crates/zmachine-ffi/include`; verify the SHA-256 of `stories/zork1/zork1.z3` against `stories/zork1/SOURCE.md` and copy it to `apps/ios/Lamplight/Resources/story.z3`, and `stories/zork1/LICENSE` to `story-license.txt`; run `xcodegen generate` in `apps/ios`. The script is idempotent and prints what it did
- [x] T028 [US1] Write `apps/ios/project.yml` from the spike's working file (research R9): app target `Lamplight` and UI-test target `LamplightUITests`; iOS 26.0; Swift 6 with complete strict concurrency and warnings as errors; iPhone only; portrait; display name `Lamplight`; bundle identifier `app.lamplight.demo`; simulator builds exclude x86_64; links `Frameworks/ZMX.xcframework` without embedding; the two resources declared with `buildPhase: resources`. Then the least Swift that builds: `LamplightApp.swift` showing an empty view, and an empty UI-test file. Run the script and `xcodebuild … build` for both destinations; record the exact commands in `apps/ios/README.md`
- [x] T029 [US1] Failing UI test `testOpeningIsShown` in `apps/ios/LamplightUITests/LamplightUITests.swift` (S§ "Starting", "For automated tests"; FR-001, FR-002; US1 scenario 1): launch with `-seed 11`; `turn.0.text` exists within 5 seconds and its label contains "West of House" and ends with the prompt. It fails: the app shows nothing
- [x] T030 [US1] Implement `Snapshot.swift` (Codable types matching the snapshot, every field), `Engine.swift` (an actor with `isolated deinit` that owns the session pointer, loads `story.z3` from the bundle, draws the random number with `SystemRandomNumberGenerator`, honours `-seed` and `-story`, and returns decoded snapshots), and the least `GameView.swift` that shows each turn's text with its identifier, until T029 passes on both destinations
- [x] T031 [P] [US1] Failing UI tests, then the views (S§ "Layout", "The transcript"; FR-003, FR-004; US1 scenarios 2 and 3): `status.place` reads "West of House" and `status.score`, `status.moves` read 0 at launch, written with `Text(verbatim:)`; after six typed commands (once T037 exists this test is extended; until then it is written and left failing at the typing step, and says so) the newest turn is on screen and `turn.0.text` can be reached by scrolling up. Implement the top bar and the scrolling transcript that brings the newest turn into view
- [x] T032 [P] [US1] Failing UI test `testEachLaunchIsANewGame`, then nothing to implement if it passes for the right reason (FR-005; US1 scenario 4): launch, terminate, launch again; the transcript has only turn 0. Confirm it can fail by temporarily persisting the transcript, and say so in the commit message

**Checkpoint**: The whole stack runs on a phone. This is the MVP

---

## Phase 6: User Story 2 - Walk by tapping the compass (Priority: P2)

**Goal**: The compass, lit from the snapshot, and taps

**Independent Test**: The compass UI tests

- [x] T033 [US2] Failing UI tests in `LamplightUITests.swift` (S§ "The compass", "For automated tests"; FR-007, FR-008, FR-009, FR-012; US2 scenarios 1, 5, 6): at launch all twelve `compass.*` controls exist; `compass.north`, `compass.south` and `compass.west` have the value `open` and `compass.east` the value `closed`, as `stories/zork1/introspection/exits.txt` gives for the opening; each control's label is the snapshot's spoken text ("North, to North of House"); tapping `compass.east` adds no turn; at Behind House the window's direction has the value `maybe`
- [x] T034 [US2] Implement `CompassLayout.swift` (the positions of the twelve places and nothing else) and `CompassView.swift` (each control drawn from its state: filled arrow, outlined arrow with a question mark, faint dot, dashed outline, dimmed; fixed 44-point targets; identifier, value and label from the snapshot) until T033 passes. Take a screenshot of the compass at the opening and at Behind House in light and dark appearance and keep them under `specs/004-first-demo/screenshots/`: the look of the states cannot be asserted by a test
- [x] T035 [US2] Failing UI tests (S§ "What the app sends", "Waiting states"; FR-010, FR-011, FR-016; US2 scenarios 2, 3, 4): tapping `compass.north` adds turn 1 whose `turn.1.command` is `north` and whose text begins "North of House"; `status.place` becomes "North of House"; the compass's values change to that room's; tapping north then north again reaches "Forest Path", the destination the first room's label promised; two taps as fast as the test can make them add exactly one turn each or one in all, never a turn the compass did not offer; ten taps following whatever is `open` reach at least five rooms
- [x] T036 [US2] Implement tapping in `GameView.swift` and `Engine.swift`: a tap sends the place and the stamp on screen; controls that send are disabled while a command is in flight; the new snapshot's turns are appended. Implement extras as a wrapping row of buttons with `compass.extra.N` identifiers (FR-013; US2 scenario 7) with a UI test first, at a place in Zork I that has one (the walkthrough reaches `land` on the river: use a typed prefix to get there, or, if that is too long for a UI test, cover extras by a Swift unit test of the view given a hand-made snapshot, and say which was done)

**Checkpoint**: The constitution's acceptance test for this phase passes: a tappable compass of exits works end to end

---

## Phase 7: User Story 3 - Type whenever tapping is not enough (Priority: P3)

- [x] T037 [US3] Failing UI tests (S§ "Layout" keyboard rule, "Waiting states"; FR-017, FR-018, FR-019; US3 scenarios 1, 2, 3): typing `open mailbox` in `field.command` and tapping `button.send` adds a turn with that command and "Opening the small mailbox reveals a leaflet."; `button.send` is disabled while the field is empty; typing `north` moves the player and changes the compass as a tap does; with the keyboard showing, the compass controls are not hittable, the newest turn's text and the field are, and the keyboard bar's button brings the compass back; sending a line leaves the keyboard up
- [x] T038 [US3] Implement the field, the send button, the keyboard rule and the keyboard bar's button in `GameView.swift` until T037 passes; complete T031's scrolling test
- [x] T039 [US3] Failing UI test, then nothing new to implement if the layer's `idle` state is drawn (S§ "Waiting states"; FR-015; US3 scenario 4): type `quit`; every compass control has the value `idle` and none is enabled; the field's placeholder is "Type your answer"; type `n`; the compass is offered again. If the test passes at once, confirm it can fail by drawing `idle` as `closed`, and say so

---

## Phase 8: User Story 4 - The app never guesses and never gets stuck (Priority: P4)

- [x] T040 [US4] Failing UI tests (S§ "Waiting states", "Starting"; FR-022, FR-024; SC-008, SC-009; US4 scenarios 2, 5): type `quit` then `y`: "The game has ended." is shown, the field is replaced by `button.newGame`, and no compass control is enabled; tapping it shows turn 0 again with only one turn; do that ten times; launched with `-story nosuchfile`, the screen shows "The game could not be started" and no compass, field or crash
- [x] T041 [US4] Implement the ended and fault states and the could-not-start screen in `GameView.swift` until T040 passes
- [x] T042 [P] [US4] Failing UI tests, then fixes if any (FR-006, FR-023; US4 scenarios 3, 4): `save`, `restore` and `script` each add a turn and leave the field live; press the home button and reactivate the app: the transcript, status and compass are as they were and no turn was added
- [x] T043 [P] [US4] Failing UI test of the unknown-exits note, by a Swift unit test of `CompassView` given a hand-made snapshot whose compass is not `known`, since no room on Zork I's walkthrough has unknown exits (FR-014; US4 scenario 1): `compass.note` reads "Exits unknown: type a direction", no control is enabled, and the note differs from the one shown for a known compass with nothing open. Add a unit-test target to `project.yml` if there is none

---

## Phase 9: Polish & Cross-Cutting Concerns

- [x] T044 [P] Failing UI test, then `AboutView.swift` (S§ "About"; FR-029): `button.about` opens a sheet showing "Lamplight", a line saying it is a demonstration, and the text of `story-license.txt`
- [x] T045 [P] Tests in `crates/zmachine-ffi/tests/app_rules.rs`, reading `apps/ios` (FR-028, SC-011; research R12): no file under `apps/ios` other than the generated resources contains "zork" in any case; `zmx_session_send_line` is called from exactly one place in the Swift sources, with the text field's value; the Swift sources contain no string literal equal to a direction's command word except the place identifiers in `CompassLayout.swift`; no Swift file other than `Engine.swift` imports `ZMX` or mentions a pointer type; no Swift source contains the words "exit", "leads" or "blocked" in a string literal (those texts come from the layer). Each rule is confirmed able to fail by a planted violation, removed again
- [x] T046 [P] Failing UI test for large text (SC-010; FR-030; S§ "At large text sizes"): launched with the content size category at the largest accessibility size, on the smallest iPhone simulator available for iOS 26 or 27 (find and record which), every compass control, the field and the send button are hittable and no two controls' frames intersect; with VoiceOver's labels read through the accessibility tree, each compass control's label is its spoken text. Fix the layout until it passes. Screenshots at default and largest sizes, light and dark, go under `specs/004-first-demo/screenshots/`
- [x] T047 [P] Test in `crates/zmachine-experience/tests/speed.rs`, marked `#[ignore = "timing; run with --release"]` (SC-003, SC-007): over the walkthrough, the slowest `send_line` or `tap` together with `snapshot_json(since: latest)` is under 10 ms; after the whole game the same holds for one more command; creating a session is under 50 ms
- [x] T048 [P] UI test for launch time (SC-001; S§ launch arguments): with `-showLaunchTime`, `debug.launchMillis` exists and reads under 2000 on each destination; implement the measurement in `LamplightApp.swift` and `Engine.swift` first as an element that reads 0, see the test fail on a sanity bound (above 0), then implement it
- [x] T049 [P] UI test for a long session (SC-007 second half): thirty typed commands from a fixed list, then scroll to `turn.0.text` and back to the newest; every turn's text is reachable
- [x] T050 Run every UI test on both destinations and record the results and times in this file; fix what differs between iOS 26.0 and 27.2
- [x] T051 [P] Update `.github/workflows/ci.yml` (research R11): find which Xcode the macOS runner has; if it can build for iOS 26, add a job that runs `scripts/build-ios.sh` and the UI tests on a simulator the runner has, and otherwise add the job with a step that prints a notice and skips. Record what was found. The existing jobs are unchanged apart from now covering the two new crates
- [x] T052 [P] Update `README.md`: the demo in the status and the phases; how to build and run Lamplight; the new crates and `apps/ios` in the layout; what the demo does not do (remember a game, show inventory or objects, run on Android); that the compass is dark at questions and at one prompt of Zork I
- [x] T053 Run every step of `specs/004-first-demo/quickstart.md` and correct the document or the code wherever they disagree
- [x] T054 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` and fix what they report; confirm with `git diff main -- crates/zmachine-core/tests crates/zmachine-cli/tests crates/zmachine-introspect/tests` that no earlier test changed; launch the app in the simulator and play twenty moves by hand through the simulator tool, by taps and by typing, noting anything a test did not catch; list at the end of this file what was done differently from the plan, including any test that was not seen to fail before its code was written

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: T003 → T004 (the core) before anything in Phase 3 that needs waiting states (T010, T012). T006 before every Phase 3 test, which compile against its stubs
- **Phase 3 (the layer)**: T008 to T011 → T012; T013 can be written with T008 and passes or leads to T014 after T012; T015 to T017 → T018; T019 needs T018; T021 → T022 needs T018
- **Phase 4 (bindings)**: needs T022. T023 → T024, T025 → T026
- **Phase 5 (US1)**: needs T026. T027 → T028 → T029 → T030; T031 and T032 after T030
- **Phase 6 (US2)**: needs T030. T033 → T034 → T035 → T036
- **Phase 7 (US3)**: needs T030; T039 needs T034. T037 → T038
- **Phase 8 (US4)**: needs T038. T040 → T041
- **Polish**: T045 can start once `apps/ios` exists and is completed last among the source checks; T050 after every UI test exists; T054 last

### Within Each Phase

- A test task is committed, run and seen to fail before the implementation task after it is begun
- UI tests are written against `contracts/screen.md`'s identifiers, which do not change without the contract changing first

### Parallel Opportunities

- T002 with T001; T005 and T007 with T003 and T004
- T009, T010 and T011 with each other (one file, separate test functions: parallel in thought, sequential in commits)
- T016 with T015
- In Polish, T044 to T049, T051 and T052 touch different files

---

## Implementation Strategy

### MVP First

Phases 1 to 5. At the end of Phase 5, Lamplight launches in the simulator and shows the
opening of the game, with the status line. Nothing is tappable yet, but every layer is
in place and linked, which is the risky part.

### Incremental Delivery

1. Phases 1 to 3: the experience layer, proven by `cargo test`, with the whole game
   replayed by taps. No phone
2. Phase 4: callable from C
3. Phase 5: on the phone, reading (MVP)
4. Phase 6: the compass. The constitution's phase-three acceptance test
5. Phases 7 and 8: typing, questions, endings
6. Phase 9: accessibility, timing, the source checks, CI, the README

### What was learned from the earlier features

- Tests written after code that already passed by hand (002) and tests that could only
  pass (003): here each task says "failing", and where a test may pass at once the task
  says how to show it can fail.
- Research that was wrong until re-tested (003, and this feature's prompt rule): T019
  re-measures the prompt rule and the tap count on the whole game, with stop points.
- Toolchain assumptions: the spike's findings are in research R8 and R9, and T028 starts
  from its working project file, not from memory.

## What was done, and what was done differently from the plan

Every task above is marked as it stands. This section records each place the work
departed from the task as written, including every test that was not seen to fail before
its code existed.

### Results

- **The walkthrough typed** gives the reference text from the first command to the end,
  byte for byte. T014 had nothing to explain: the screen text and the transcript stream
  are the same from the first command on.
- **The walkthrough by taps** gives the same text with 186 commands sent as taps
  (SC-004 asks for 150) and two direction commands typed because the compass did not
  offer them (lines 250 `east` and 332 `west`).
- **The compass agrees** with the introspection interface asked of an identical game at
  all 329 command prompts, and with the 58 hand-made exit entries.
- **The prompt rule** mislabels one ordinary prompt, the Loud Room at line 87, as
  research R6 predicted, and no other. No stop point was reached.
- **Timing** (release build, this Mac): the slowest command with its snapshot took
  0.35 ms over 331 commands; a session starts in under 0.6 ms.

- **UI tests on both systems** (T050), 19 run and `testScreenshots` skipped on each:

  | Destination | Result | Time |
  | --- | --- | --- |
  | iPhone 17 Pro, iOS 26.0 | 19 passed, 0 failed | 566 s |
  | iPhone 18 Pro, iOS 27.2 | 19 passed, 0 failed | 591 s |
  | iPhone 16e, iOS 26.0 (the smallest available), largest text only | passed | 10 s |

  Nothing differed between iOS 26.0 and 27.2. The longest tests are the ten taps
  (about 150 s, most of it reading twelve compass values before each tap), the long
  session (about 135 s) and ten endings (about 110 s).

### Tests not seen to fail before their code

- `crates/zmachine-experience/tests/crate_rules.rs` (T004) passed at once against the
  empty crate. It was shown able to fail by planting a game's name in a comment.
- T013 (the typed walkthrough) passed as soon as T012 existed; it had failed before only
  for want of the `Session` type.
- `testALongSession` (T049) and `testTheLargestTextSize` (T046) passed at once against
  views already written. The large-text test was then found too weak: a screenshot
  showed the status bar wrapping "Score" a letter to a line while the test passed. The
  test was strengthened, seen to fail, and the bar fixed. `testALongSession` was not
  shown able to fail.
- `testEachLaunchIsANewGame` (T032) and `testAQuestionDarkensTheCompass` (T039) passed
  with the views. Each was then shown to fail on a planted defect (a turn carried across
  launches; `idle` drawn as `closed`). The first plant for T032 was itself wrong twice
  before it was right; the test was tightened along the way to wait before concluding
  that nothing of the last game is shown.
- The UI tests of Phases 5 to 8 were written as one batch (thirteen failing) and the
  views as one piece of work, not story by story: a simulator run costs a minute or
  more per test. Each was seen to fail before the views existed.
- T048: the launch-time test was seen to fail because the element was absent, not on the
  "reads 0" bound the task describes.

### Departures

- **T007**: the testkit needed no change.
- **T012**: `Session::set_instruction_ceiling` was added so the tests can reach the
  "not responding" fault without running fifty million instructions.
- **T019**: the tap count was first guessed at 185 in the test and measured as 186.
- **T026**: `Session::mark_failed` was added to the experience layer (test first), so
  that a session whose call panicked reports its fault through the ordinary snapshot.
  The symbol test found the static library stale (running tests does not rebuild it);
  it now builds it first. The system's `nm` cannot read some of the Rust standard
  library's objects and exits non-zero; the test compares the symbols it does list.
- **A `Game.swift` file** holds what is on screen (the turns and the newest snapshot).
  The plan put this in `GameView.swift`.
- **T034, the look of the states**: open is a filled disc with a white arrow, maybe an
  outlined disc with the arrow and a question-mark badge, closed and idle a faint dot,
  unknown a dashed ring. Up, down, in and out have their own symbols and carry the
  snapshot's label beneath, in a fixed small size, since an arrow alone could not tell
  up from north. The contract's "filled arrow" and "outlined arrow" became discs.
- **T036 and T043, extras and unknown exits**: tested through the real screen, not by a
  Swift unit test of the view. Debug builds draw a hand-made snapshot given in the
  environment variable `LAMPLIGHT_FIXTURE_SNAPSHOT` in place of a game
  (contracts/screen.md records it). The bundled game's first extra is 113 commands in.
  That a tap on an extra reaches the game is tested in the layer and the bindings, not
  on the screen.
- **T037, the keyboard's button**: first put on the keyboard's bar, where iOS 26 floats
  it over the send button (seen in a screenshot). It is now beside the field, shown
  while the keyboard is up. The contract is updated.
- **T044, About**: the license text is preceded by a line giving the story's source,
  which the build script takes from the story's source record.
- **T034 and T046, screenshots**: taken by a UI routine (`testScreenshots`, skipped
  unless given a folder), in light and dark, of the opening, Behind House, the keyboard,
  a question, the end and unknown exits; and the opening at the largest text size.
- **T045**: two rules failed on real findings before any plant: the app's README named
  the story's path, and the snapshot's field name `leads_to` tripped the rule against
  the app's own wording about exits. The README was reworded; the field name is exempted
  by name.
- **T051, CI**: which Xcode the hosted macOS runner has was not found out, because
  nothing has been pushed. The job detects it at run time: with no iPhone simulator on
  iOS 26 or later it prints a notice and does nothing more. It has never run.

### Played by hand (T054, 12 November 2026, after the merge)

Twenty moves on iPhone 17 Pro, iOS 26.0, through the simulator tool: nine taps and
eleven typed lines, from West of House through the kitchen window, the living room, the
trap door, the cellar, the gallery, the studio, up the chimney to the kitchen and the
attic, and back to the living room. Then `quit`, `y`, "New game", and About. The place,
score (35) and moves (20) were right at every step; every lit control moved the player;
the two "may be blocked" taps did what the game said (`down` in the living room before
the rug was moved: "You can't go that way."; `up` in the studio: let through); the
compass was dimmed at the question and at the end; "New game" gave move 0.

What no test caught:

- **Characters typed straight after Return were lost.** SwiftUI's field gives the
  focus up on Return and `send()` gave it back; what was typed in between went nowhere.
  With the tool typing at once after Return, "read leaflet" did not arrive at all. At a
  human pace nothing was lost. **Fixed afterwards** (branch `fix-keystrokes-after-return`):
  the field is a UIKit one that keeps the focus on Return
  (`apps/ios/Lamplight/CommandField.swift`), and lines typed while a command is being
  answered wait their turn. The same three lines typed straight through then arrived
  whole. `testTypingStraightThroughReturn` was added, but it passed before the fix: the
  test framework waits between keys, so it cannot show this defect. It was shown by hand,
  before and after. A first attempt, a field that grows downward, was worse (Return put
  the keyboard away and sent nothing) and was thrown away.
  The change first broke `testTheEndAndANewGameTenTimes` (the keyboard stayed up after the
  game ended, hiding the compass in the new game); the field now puts it away when it
  goes. The new test failed once in a full run, reading no moves count, and then passed
  six times alone and in the next full run (20 passed, 1 skipped, iOS 26.0); why it failed
  that once is not known. Not run on iOS 27.
- **A way that has been opened still says "may be blocked".** After `open window`, in
  and west at Behind House keep the question mark. That is what the introspection
  interface reports, and it is not wrong, but it reads oddly.
- **The compass shows a way the text has not mentioned.** In the living room, down is
  "may be blocked" before the rug is moved.
- **The opening sits at the bottom of the transcript at launch and at the top after
  "New game".**
- **The license in About keeps its own line breaks**, so each line wraps twice on a
  phone.

Not done by hand: dark appearance, the largest text size, and iOS 27 (the automated
tests cover all three).

### Not done

No task is left unmarked. Of the findings above, only the first is fixed.
