# Tasks: The Map, First Slice

**Input**: Design documents from `/specs/012-map-first-slice/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/map.md,
quickstart.md

**Tests**: Required, and first. In every phase the test tasks come before the code
that makes them pass, and each is run and seen to fail before that code is written.
Features 009 to 011 did not keep to this strictly. For this feature each test task's
record line is to say "seen failing" with the reason it failed, or say plainly that it
was not.

**Clause names**: W§, P§, S§ and A§ are clauses of `contracts/map.md`. R is an entry
of `research.md`.

**Stages**: three, each its own pull request (R11). A stage is not begun until the one
before is merged, or the owner says otherwise.

## Format: `[ID] [P?] [Story] Description`

---

# Stage 1: the layer

Nothing on the screen changes in this stage.

## Phase 1: Setup

- [ ] T001 Create branch state: confirm `specs/012-map-first-slice/` is committed and the spike's file `crates/zmachine-corpus/examples/map_spike.rs` is not in the working tree of this branch's commits (it is untracked; do not add it)
- [ ] T002 [P] Add built stories for the map's tests to `crates/zmachine-experience/tests/common/mod.rs` or a new `tests/common/maps.rs`, using `zmachine_testkit::toy::Toy`: a ring of four rooms; a room with ways up and down to single rooms; a chain of five rooms down a stair; two rooms with one name; a way whose table names one room and whose code leads to another; a dark room with a lamp. Note here which toy serves which clause

## Phase 2: Foundational (the core's count of jumps)

- [ ] T003 Write tests in `crates/zmachine-core/tests/` (a new `jumps.rs`): `Machine::jumps()` is 0 at the start; rises by one when the game restarts itself; by one when a restore is provided and accepted; by one when a version 5 game undoes a turn; and does not rise for a refused restore, a failed undo, a save, or any other instruction. See them fail (the method does not exist)
- [ ] T004 Implement `Machine::jumps()` in `crates/zmachine-core/src/lib.rs` with the counter raised in `ops_io.rs` and wherever restart is carried out. T003 passes. Run the fidelity tests of features 001 and 002 and confirm every transcript is unchanged (Principle I)

## Phase 3: User Story 1 - The map draws itself as I walk (P1) 🎯

**Goal**: the record and the placement by compass, in the layer.

**Independent Test**: the first ten moves of Zork I's walkthrough give exactly the
places stood in, each where first put, joined by exactly the lines walked.

### Tests

- [ ] T005 [US1] Write tests in `crates/zmachine-experience/src/map_tests.rs` for W§1, W§5, W§11, W§17 on a hand-fed sequence of observations: one place at the start; a direction to a new room gives a second place and one line; the same move again gives nothing new; a command that does not move gives nothing; an answer stop between two command stops is passed over; no place or line exists that was not walked. See them fail
- [ ] T006 [US1] Write tests in `crates/zmachine-experience/src/map_place_tests.rs` for P§1, P§2, P§7: the first place at (0, 0); each of the eight directions puts the new place one step that way; with that position taken, two steps, then three; with all three taken, the nearest free by rings on the side the direction points to, ties by lowest y then lowest x; no two places share a position after 200 seeded random moves on a built graph. See them fail
- [ ] T007 [US1] Write tests in `map_place_tests.rs` for P§8: straight when exactly along the direction; bent when off it; bent when the way back was walked in a direction that does not agree; bent when a place lies between; a diagonal counts as along only when the steps are equal. See them fail
- [ ] T008 [US1] Write tests in `map_tests.rs` for W§15 and W§16: two places of one name become "Name 1" and "Name 2" in the order reached, the first unnumbered until the second is reached; a way that leads back to its own place draws no line and is marked. See them fail
- [ ] T009 [US1] Write tests in `crates/zmachine-experience/tests/map.rs` on the ring of four rooms (T002): walking the ring closes it with four straight lines; and P§9: the same script twice gives the same map, compared whole. See them fail

### Implementation

- [ ] T010 [US1] Create `crates/zmachine-experience/src/map.rs` with the entities of data-model.md (`Place`, `Way`, `Line`, `Refusal`, `Sheet`, `Map`, `Observation`) and `Map::observe`, implementing W§1, W§5, W§11, W§15, W§16, W§17 with ordered maps only (R1). Declare the module in `lib.rs`. T005, T008 pass
- [ ] T011 [US1] Create `crates/zmachine-experience/src/map_place.rs` implementing P§1, P§2, P§7, P§8. T006, T007 pass
- [ ] T012 [US1] In `crates/zmachine-experience/src/lib.rs` add the map to `Session`, build the observation at the end of `advance` (R2) with `jumped` from `Machine::jumps()`, and reset the map in `new_game`. Add `Session::map()`. T009 passes

**Checkpoint**: a session keeps a map of compass moves.

## Phase 4: User Story 5 - Up, down, and other pages (P1)

**Goal**: ladders, attached groups, sheets, moves by a word.

### Tests

- [ ] T013 [US5] Write tests in `map_place_tests.rs` for P§3: up goes to (+1, −1) and down to (−1, +1) when free; passes over a position on a compass way the place has; takes the nearest free otherwise; the new place is attached to the place left. See them fail
- [ ] T014 [US5] Write tests in `map_place_tests.rs` for P§4: a group of one, two and three stays on the sheet; reaching a fourth moves all four to a new sheet with the first-attached place at (0, 0) and the others keeping their offsets; none is then attached; no place outside the group has moved; a group that has moved does not move again when it grows. See them fail
- [ ] T015 [US5] Write tests in `map_tests.rs` and `map_place_tests.rs` for W§6, P§5, P§6: a move by a word that is not a direction is a line marked with the command as sent and starts a new sheet; in and out stay on the sheet at the nearest free position; a second word-move from a place put by a word-move continues that sheet one step below. See them fail
- [ ] T016 [US5] Write tests in `crates/zmachine-experience/tests/map.rs` on the stair of five rooms and the up-and-down toy (T002): the single rooms stay attached; the stair becomes its own sheet at its fourth room; each end's lead names the other. See them fail
- [ ] T017 [US5] Implement P§3 to P§6 and W§6 in `map_place.rs` and `map.rs`, with `Group`. T013 to T016 pass

## Phase 5: User Story 6 - Honest about what it does not know (P2)

### Tests

- [ ] T018 [US6] Write tests in `map_tests.rs` for W§2, W§3, W§4, W§12: a place first entered dark has no name and no ways; entered lit, or with light unknown, it has both; seen dark then lit, they appear and its position is unchanged; a way's state follows the last lit visit and not a later dark one. See them fail
- [ ] T019 [US6] Write tests in `map_tests.rs` for W§8, W§9, W§10: a direction not offered that leaves the player in place is a refusal; one offered as open is not; a direction whose way named another room than the one reached is a carried move, marked with the command, on a new sheet and not placed in that direction; two different arrivals by one way keep both lines and mark the way. See them fail
- [ ] T020 [US6] Write tests in `map_tests.rs` for W§13 and W§14: with the room not known nothing is added and `here` is not known, and the next arrival has no line; with `jumped` set no line is drawn and a new place starts a new sheet. And in `tests/map.rs`, on a version 5 built story with undo and one that restarts itself: typing the undo or the restart draws no line. See them fail
- [ ] T021 [US6] Write a test in `tests/map.rs` on a story of no family (one of the layer's unrecognised fixtures): `Session::map()` reports that there is no map. See it fail
- [ ] T022 [US6] Implement W§2 to W§4, W§8 to W§10, W§12 to W§14 and the no-map case in `map.rs`. T018 to T021 pass

## Phase 6: The snapshot and the labels (User Stories 2 and 8, the layer's part)

### Tests

- [ ] T023 [US2] Write tests in `crates/zmachine-experience/src/json_tests.rs` for S§1 to S§10: the key `map` after `start_over`; `null` with no map; the example of the contract rendered from a hand-made map, key for key; only the current sheet's places; `leads` for lines that leave the sheet; `stubs` for every place but the one the player is in; `ways` in the compass's order then extras. See them fail
- [ ] T024 [US2] Write tests in `map_tests.rs` for W§7 and S§7: a way's `leads_to` and the destination in `spoken` are absent until its line is walked and present after; the label without a destination is the compass's label; with the game not waiting for a command no way has a command. See them fail
- [ ] T025 [US8] Write a test in `tests/map.rs` for SC-008 and FR-037: a session kept and brought back (feature 011) at ten points along Zork I's walkthrough has a `map` equal to that of the session never closed; and `new_game` gives a map of one place. See it fail
- [ ] T026 [US2] Implement the `map` key in `crates/zmachine-experience/src/json.rs`, the labels without a destination in `compass.rs`, and the layer's words of plan.md ("No map for this game.", "Where you are is not known.", the spoken summary) as constants. Update the checks in `tests/session.rs` and `tests/resume.rs` that name the snapshot's last key, listing each here. T023 to T025 pass

## Phase 7: Proof on the games, and limits

- [ ] T027 Write `crates/zmachine-experience/tests/map.rs` tests that play Zork I (with its pack) and Adventure along their walkthroughs and assert at every turn SC-002 (exactly the places stood in and the lines walked; no place moved but by P§4) and at the end SC-003 (at least four compass lines in five straight on a sheet; no two straight lines cross), SC-004 (at most twelve sheets and eight; no sheet over thirty places) and SC-005 (on Zork I, a place entered only dark has no name until lit). Record the figures here beside the spike's (49 of 60 on 11 sheets; 44 of 51 on 7). If a bar is missed, stop and report to the owner before changing a rule or a bar
- [ ] T028 Add the recorded pages (R12): a function in `tests/map.rs` that writes a map as sorted text lines, and a test that compares it with `stories/zork1/map/walkthrough.txt` and `stories/advent/map/walkthrough.txt`, rewriting them when `UPDATE_PAGES=1`. Create the two files and a `README.md` in each folder saying what they are and that they hold the story's room names under its license
- [ ] T029 [P] Write `crates/zmachine-corpus/tests/map.rs`: every recognised story of the corpus is crawled with a session recording the map, with no fault and P§7 holding (FR-044); and the seeded wander of R13 on Zork I and Adventure, 400 moves, printing the measures of SC-003 and asserting P§7 (FR-043). Record the wander's figures here. No room name is written to any report
- [ ] T030 Add to `crates/zmachine-experience/tests/speed.rs` ignored timing tests: over both walkthroughs the slowest command with its snapshot and map within the command limit; bringing back 2,000 turns with the map within the layer's second; the largest `map` in bytes (FR-039, SC-007). Run in release and record the figures here
- [ ] T031 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`; build the app with `scripts/build-ios.sh` and run `scripts/test-ios.sh 27` to confirm the app, which does not yet read `map`, is unchanged. Record the counts here
- [ ] T032 [P] Update `README.md` (what the layer now keeps; that the screen is unchanged until stage 2) and `docs/` only if a published interface changed (it does not: the introspection specification is untouched)

**Stage 1 ends here: one pull request.**

---

# Stage 2: the band

## Phase 8: User Stories 2, 3 and 8 in the app (P1)

**Goal**: the band in place of the compass, with the keyboard up.

### Tests

- [ ] T033 [US2] In `apps/ios/LamplightUITests/LamplightUITests.swift` add `testTheMapShowsWhereIAm`: at the opening the element `map` exists with the layer's spoken summary; `map.place.<id>` exists for the opening place only; after tapping `compass.north` there are two (A§1, A§3). See it fail
- [ ] T034 [US2] Change the existing tests that walk by `compass.<way>` so that their expectations of a label's destination follow W§7 (a destination only once walked), and list each changed test here. Add `testWalkingToTheKitchenByTheMap` for SC-001 (A§2). See them fail
- [ ] T035 [US3] Add `testTheKeyboardStaysUp`: the keyboard is shown at launch; after five commands, two typed and three tapped, it has been shown at every check; `button.compass` does not exist; the inventory strip and, in the Attic unlit, the darkness note exist with the keyboard up (A§4, A§6, A§11). Remove or rewrite the tests of the compass being hidden by the keyboard, listing each. See it fail
- [ ] T036 [US3] Add `testALongReplyStartsAtItsFirstLine`: after a command with a long reply (Zork I's `look` in the Living Room, or the opening), the first line of the newest turn is within the transcript's frame (A§7). And `testScrollingBackLowersTheKeyboard`: dragging the transcript lowers it; touching the field raises it (A§4). See them fail
- [ ] T037 [US8] Change the tests of feature 010 that read a way's spoken label, as T034; add an assertion that each of the opening place's ways is a button whose label is the snapshot's `spoken` (A§2, FR-032), and that the announcement after a reply is what it was (FR-034). See them fail
- [ ] T038 [P] [US2] Rewrite the rules of `crates/zmachine-ffi/tests/app_rules.rs` that name `CompassView.swift` and `CompassLayout.swift` for `MapBand.swift`, `MapDrawing.swift` and `MapLayout.swift`: no direction word is written outside `MapLayout.swift`; the map's views send nothing but through the game's one place for a line; the app's source does not read the snapshot's `compass` key or any `leads_to` but the map's (plan.md, re-check). See them fail

### Implementation

- [ ] T039 [US2] Add the map to `apps/ios/Lamplight/Snapshot.swift` (S§1 to S§9), optional so that hand-made fixture snapshots without it still decode
- [ ] T040 [US2] Create `apps/ios/Lamplight/MapLayout.swift` (positions to points; where each way's button sits on a place's edge; the twelve ways' symbols moved from `CompassLayout.swift`) and `apps/ios/Lamplight/MapDrawing.swift` (a place as a box with its name; straight, bent, ladder and word lines; stubs with the compass's states; leads; the marker), as SwiftUI shapes (R9), plain strokes
- [ ] T041 [US2] Create `apps/ios/Lamplight/MapBand.swift`: the sheet clipped to a band 200 points tall, moved so `here` is centred; the ways' buttons over it with identifiers `compass.<way>` and `compass.extra.<n>` (A§2); the strip and the darkness note at its lower edge (A§6); the whole an accessibility container labelled `spoken` (A§9); the layer's note where the map has one (S§8)
- [ ] T042 [US3] In `apps/ios/Lamplight/GameView.swift` put the band between the status line and the transcript; keep the command field focused while the game waits for a line (A§4); lower the keyboard only on a drag of the transcript; show the newest reply from its first line (A§7); remove `CompassView`, `button.compass` and the `typing` switch that hid the compass. Delete `CompassView.swift` and `CompassLayout.swift`
- [ ] T043 Run `scripts/build-ios.sh`, `scripts/test-ios.sh 27` and `scripts/test-ios.sh 26`. T033 to T038 pass and every other test passes. Record the counts and the list of changed tests (FR-045)
- [ ] T044 [P] Update `README.md` and `apps/ios/README.md`: the map band; the keyboard; that the compass is gone and where its states are now; the files table
- [ ] T045 Build for the owner's phone and walk quickstart "Stage 2, by hand" with the owner: twenty rooms; is the band tall enough to be worth looking at; does the keyboard stay put. Record what they say. If they say the band does not work, stop: the stage is not to be merged until the band is changed (plan.md, strongest objection)

**Stage 2 ends here: one pull request.**

---

# Stage 3: the sheet and the hand

## Phase 9: User Story 4 - I can open the map out (P1)

- [ ] T046 [US4] Add `testOpeningTheMapOutAndBack`: walk twelve places; touch `button.openMap`; the keyboard is not shown and the sheet fills the space of band and transcript; drag to bring the farthest place into view; tap a way of the current place and stay on the sheet; touch `button.closeMap`, then separately the command field: the band and keyboard are back and the current place is centred (A§8, US4 scenarios 1 to 5). See it fail
- [ ] T047 [US4] Implement opening out in `apps/ios/Lamplight/MapBand.swift` and `GameView.swift`: the pull and the button, the drag with limits at the sheet's edges, the return. T046 passes

## Phase 10: User Story 7 - Drawn by hand, and alive (P2)

- [ ] T048 [US7] Add tests: `testAPlaceIsDrawnTheSameEachTime` (a debug-only element exposes each place's outline seed, equal across two launches of a kept game; A§10); `testTheLampIsOutInTheDark` (the marker's value is `out` in the Attic unlit and `lit` in the Kitchen; S§2); `testReducedMotion` with the launch argument that sets it (no animation is pending after a move); and extend `testTheLargestTextSize` so that every way's button is at least 44 points and names are not clipped to nothing. See them fail
- [ ] T049 [US7] In `apps/ios/Lamplight/MapDrawing.swift` give outlines and lines a pencil wobble worked out from the place's id; draw new lines and places in by `trim` over a quarter of a second and glide the view (FR-029), at once with Reduce Motion; draw the marker as a lamp, lit or out. T048 passes
- [ ] T050 Run `scripts/build-ios.sh`, `scripts/test-ios.sh 27` and `scripts/test-ios.sh 26`; record the counts. Update the READMEs for opening out and the look
- [ ] T051 Build for the owner's phone and walk quickstart "Stage 3, by hand" with the owner (SC-009): twenty rooms, opening the map out; does the page read as the places walked; what would they have drawn differently. Record what they say, in their words

**Stage 3 ends here: one pull request.**

---

## Dependencies & Execution Order

- Phases 1 to 7 in order; Phase 3 needs Phase 2 only for T012's `jumped`.
- Phases 4 and 5 both need Phase 3 and may be done in either order.
- Phase 6 needs 3, 4 and 5. Phase 7 needs 6.
- Stage 2 needs stage 1 merged. Stage 3 needs stage 2 merged.
- Phases 9 and 10 may be done in either order.

### Parallel opportunities

- T002 beside T003 and T004. T029 and T032 beside T027, T028 and T030.
- T038 and T044 beside the app's other work in stage 2.

## Implementation Strategy

**Stage 1 is the proof.** If the bars of T027 are met and the wander of T029 is not
alarming, the rules are sound and stage 2 is drawing. If they are not, the owner
decides before any screen changes.

**Stage 2 is the risk** the plan names: it is where the compass goes.

**Each stage one pull request**, opened by the owner's leave, merged by the owner.

## Notes

- 51 tasks: stage 1 has 32, stage 2 has 13, stage 3 has 6.
- T045 and T051 are the owner's, on their phone.
- T027 and T045 each have a stop written into them.
