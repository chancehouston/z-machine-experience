# Tasks: An Authored Map for Zork I, stage 1

**Input**: `specs/017-authored-map/` (spec, plan, research R1 to R10, data-model,
contracts/presentation.md, quickstart); the constitution's Principle VII.

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

Stage 1: the mechanism and the trial file. Stage 2 (the rest of Zork I) is tasked
after the owner's trial. One pull request against `main`, not stacked.

## Phase 1: The file

- [ ] T001 [US4] Write `presentation_tests.rs`: a file of the contract's form is read; each departure of F§1 to F§5 is refused with its line; comments and blank lines are skipped. See them fail: no module
- [ ] T002 [US4] Write `presentation.rs` (`Presentation::parse`). T001 passes
- [ ] T003 [US4] Write `presentations/README.md`: the form, and what a file may and may not hold (Principle VII)

## Phase 2: The map

- [ ] T004 [US1] Add tests in `map_tests.rs`, with a presentation made by hand: a place the file has is put at its sheet and position, not inset, whatever way it was reached by (a compass way, up, a word, after a jump); a place it does not have is put by the present rules and never on an authored position, drawn or not; the first place of a game; `authored` in the view (M§1, M§3, S§1). See them fail
- [ ] T005 [US1] Add tests: two orders of discovery give the same authored positions (M§4); nothing is in the view that was not visited (M§2). See them fail or pass and say which
- [ ] T006 [US2] Add tests: a named sheet's label, and that sheets are numbered in the order reached, not by the file (M§5); a walk between two authored sheets is a lead. See them fail
- [ ] T007 [US3] Add tests: no redraw offered or made on a sheet with an authored place; offered on a sheet with none (M§6). See them fail
- [ ] T008 [US1] In `map.rs`: the map takes a presentation; authored putting, reserved positions, sheets by the file's sheet, names, `authored`, redraw withheld. T004 to T007 pass
- [ ] T009 [US1] `json.rs`: `authored` on each place, test first

## Phase 3: Opening a game with a file

- [ ] T010 [US4] Add to `tests/presentation.rs`, with a file made in the test for Zork I: a session opened with it has authored places; with a wrong digest, release or serial, or a place that is not a room, it is refused, the map is as without one, and the notice says so once (F§6, S§2, M§7); the transcript of the walkthrough is the same with and without a file (M§8); a kept game holding a redraw of a sheet now authored comes back, the redraw passed over (M§6). See them fail: no such constructor
- [ ] T011 [US4] `lib.rs`: the constructors take the file; identity and rooms are checked; the notice; a new game keeps it; bringing back passes over such a redraw. T010 passes
- [ ] T012 [US4] The binding: `zmx_session_open` gains the file's text (`src/lib.rs`, `include/zmx.h`, `tests/abi.rs`, test first)

## Phase 4: The trial file

- [ ] T013 [US5] Outside the repository, gather the ways walked among Zork I's rooms above ground and in the first region below, from the walkthrough and long wanders, and draft positions; adjust by hand with the published maps the owner found open beside; draw the result. Show the owner the drawing and take what they say before going on
- [ ] T014 [US4] Write `presentations/zork1.present` and `presentations/zork1.md` (what was consulted, how the positions were made, and that nothing was copied). Add tests: the file is read for Zork I's story and every place is a room (F§6, SC-005); it holds no word of the game (F§7); it is the file the app's build bundles
- [ ] T015 [US1] Add to `tests/presentation.rs`: at every turn of the walkthrough and of a wander of 400 moves, nothing is drawn that was not visited and every authored place is at its file's position (SC-002); the wander above ground has no crossing and at most a quarter of the bent joins it has without the file (SC-001); how many joins have a place of the file between their ends (R10). Record the figures here
- [ ] T016 Confirm that with no file every recorded page and earlier map test passes unchanged (SC-004)

## Phase 5: The app

- [ ] T017 [US1] `scripts/build-ios.sh` bundles the story's presentation file after checking its digest; `Game.swift` and `Engine.swift` hand its text over; a rule test that the app reads none of it and the build checks it (A§1, A§2), test first
- [ ] T018 [US1] UI tests: `testTheHouseIsARing` (a walk round the house: each side is exactly along its way from the last; no redraw control); `testASheetHasItsName` (the opened map's label); the existing redraw tests run on a story or a sheet with no file, or are changed to say what is now true. See them fail first where they are new

## Phase 6: Proof and record

- [ ] T019 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26` in turn with nothing else building; check each ran every test. Record the counts
- [ ] T020 [P] `README.md`, `apps/ios/README.md`
- [ ] T021 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-006). Record what they say, in their words
