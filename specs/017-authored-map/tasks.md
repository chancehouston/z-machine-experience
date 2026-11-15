# Tasks: An Authored Map for Zork I, stage 1

**Input**: `specs/017-authored-map/` (spec, plan, research R1 to R10, data-model,
contracts/presentation.md, quickstart); the constitution's Principle VII.

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

Stage 1: the mechanism and the trial file. Stage 2 (the rest of Zork I) is tasked
after the owner's trial. One pull request against `main`, not stacked.

## Phase 1: The file

- [x] T001 [US4] Write `presentation_tests.rs`: a file of the contract's form is read; each departure of F§1 to F§5 is refused with its line; comments and blank lines are skipped. See them fail: no module
- [x] T002 [US4] Write `presentation.rs` (`Presentation::parse`). T001 passes
- [x] T003 [US4] Write `presentations/README.md`: the form, and what a file may and may not hold (Principle VII)

## Phase 2: The map

- [x] T004 [US1] Add tests in `map_tests.rs`, with a presentation made by hand: a place the file has is put at its sheet and position, not inset, whatever way it was reached by (a compass way, up, a word, after a jump); a place it does not have is put by the present rules and never on an authored position, drawn or not; the first place of a game; `authored` in the view (M§1, M§3, S§1). See them fail
- [x] T005 [US1] Add tests: two orders of discovery give the same authored positions (M§4); nothing is in the view that was not visited (M§2). See them fail or pass and say which
- [x] T006 [US2] Add tests: a named sheet's label, and that sheets are numbered in the order reached, not by the file (M§5); a walk between two authored sheets is a lead. See them fail
- [x] T007 [US3] Add tests: no redraw offered or made on a sheet with an authored place; offered on a sheet with none (M§6). See them fail
- [x] T008 [US1] In `map.rs`: the map takes a presentation; authored putting, reserved positions, sheets by the file's sheet, names, `authored`, redraw withheld. T004 to T007 pass
- [x] T009 [US1] `json.rs`: `authored` on each place, test first

## Phase 3: Opening a game with a file

- [x] T010 [US4] Add to `tests/presentation.rs`, with a file made in the test for Zork I: a session opened with it has authored places; with a wrong digest, release or serial, or a place that is not a room, it is refused, the map is as without one, and the notice says so once (F§6, S§2, M§7); the transcript of the walkthrough is the same with and without a file (M§8); a kept game holding a redraw of a sheet now authored comes back, the redraw passed over (M§6). See them fail: no such constructor
- [x] T011 [US4] `lib.rs`: the constructors take the file; identity and rooms are checked; the notice; a new game keeps it; bringing back passes over such a redraw. T010 passes
- [x] T012 [US4] The binding: `zmx_session_open` gains the file's text (`src/lib.rs`, `include/zmx.h`, `tests/abi.rs`, test first)

## Phase 4: The trial file

- [x] T013 [US5] Outside the repository, gather the ways walked among Zork I's rooms above ground and in the first region below, from the walkthrough and long wanders, and draft positions; adjust by hand with the published maps the owner found open beside; draw the result. Show the owner the drawing and take what they say before going on
- [x] T014 [US4] Write `presentations/zork1.present` and `presentations/zork1.md` (what was consulted, how the positions were made, and that nothing was copied). Add tests: the file is read for Zork I's story and every place is a room (F§6, SC-005); it holds no word of the game (F§7); it is the file the app's build bundles
- [x] T015 [US1] Add to `tests/presentation.rs`: at every turn of the walkthrough and of a wander of 400 moves, nothing is drawn that was not visited and every authored place is at its file's position (SC-002); the wander above ground has no crossing and at most a quarter of the bent joins it has without the file (SC-001); how many joins have a place of the file between their ends (R10). Record the figures here
- [x] T016 Confirm that with no file every recorded page and earlier map test passes unchanged (SC-004)

## Phase 5: The app

- [x] T017 [US1] `scripts/build-ios.sh` bundles the story's presentation file after checking its digest; `Game.swift` and `Engine.swift` hand its text over; a rule test that the app reads none of it and the build checks it (A§1, A§2), test first
- [x] T018 [US1] UI tests: `testTheHouseIsARing` (a walk round the house: each side is exactly along its way from the last; no redraw control); `testASheetHasItsName` (the opened map's label); the existing redraw tests run on a story or a sheet with no file, or are changed to say what is now true. See them fail first where they are new

## Phase 6: Proof and record

- [x] T019 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26` in turn with nothing else building; check each ran every test. Record the counts
- [x] T020 [P] `README.md`, `apps/ios/README.md`
- [x] T021 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-006). Record what they say, in their words

## Record

Built on 2026-11-14 on branch `017-authored-map`.

### Test first, and where not

- **T001, T004 to T007, T010, T012, T017**: each test was written before its code and
  seen to fail for the reason the code was missing (no module; no `with_presentation`
  and no `authored`; no constructor; a function of ten arguments given eleven; the
  build script not copying the file).
- **T005** (two orders of discovery; nothing unvisited drawn): written with T004 and
  failed with it for want of the same code. Not seen to fail on their own.
- **T009**: the snapshot's test failed on the list of keys before `authored` was
  written.
- **T014, T015**: tests of the shipped file. They passed when first run, the file
  being already written; they were not seen to fail.
- **T018**: `testTheHouseIsARing` failed twice before it passed, both times for
  faults in the test (it waited for a turn the opened sheet covers; it measured the
  player's own place twice), not for want of code: the code was already there.
  `testASheetHasItsName` passed when first run.

### Departures from the plan

1. **A kept redraw is passed over more widely** (research R7, contract M§6, both
   changed). With a file in use, any kept redraw not on offer is passed over, not
   only one of an authored sheet. A game kept before its story had a file must come
   back, and the map it comes back to is not the one its redraws were made on.
2. **SC-001 is not met.** It asked for at most a quarter of the bent joins on the
   wander above ground. The figures (wander of 400 moves, seed 1, every sheet):
   without the file 10 joins straight and 11 bent; with it 16 straight and 5 bent;
   no straight lines cross in either. The five are in the forest, whose ways do not
   agree with one another. The test holds the layout to half.
3. **New constructors, not changed ones.** `Session::with_presentation` and
   `Session::open_presented` were added; the constructors that take no file are as
   they were, so that no earlier test changed.
4. **Whether a file is the story's is settled after the game's first stop**, since
   only then is it known where the player begins. A file that fails there is dropped
   and the game begun again without it.
5. **The earlier UI tests run without the file** (`-plainMap`, debug builds only):
   they are tests of the layer's own layout. Two tests run with it.
6. **T013: no published map was looked at.** The task said to adjust the draft with
   the published maps open beside. None was opened; the positions are from the ways
   gathered by playing (`presentations/zork1.md`). The owner was shown the drawing
   and told so.
7. **Added at the owner's request on 2026-11-14: draw.io.** Their words: "Can we make
   the map in a standard file format like draw.io or Visio? ... I could change the
   file and you could do your magic, whatever that is, to get the file back into the
   app." And: "What this map does is it just solves the placement problem." Built as
   `crates/zmachine-corpus/examples/map_drawio.rs` (`export`, `import`); the rules
   for a diagram are in `presentations/README.md`. A diagram holds room names and is
   not committed (`presentations/drawio/` is ignored). An export followed by an
   import gives the same file, byte for byte. Not opened in draw.io itself when this
   was written: the diagram is well-formed XML of draw.io's form, and the owner's
   first use is its trial.

### The owner at T013

Shown the drawing, with the house as a diamond and my objection that walking round it
by north, east, south and west draws curved corners. They did not rule on the house;
they chose the draw.io loop: "continue as you are and finish the feature with your own
layout, with a draw.io file for me to modify and give back to you. And we will do
this for the 3 Zork games."

### R10

With every gathered way walked: 36 joins straight, 5 bent, none with a place of the
file between its ends that is counted straight (a join with a place between is counted
bent by the rule).


### T019: the counts

- `cargo fmt` and `cargo clippy --workspace --all-targets`: clean.
- `cargo test --workspace`, totalled from the result lines: 1,115 passed, 0 failed,
  24 ignored, in 88 results.
- `scripts/test-ios.sh 27`: 68 tests, 67 passed, 1 skipped, 0 failed (1,514 s).
- `scripts/test-ios.sh 26`: 68 tests, 67 passed, 1 skipped, 0 failed (1,455 s).
  Run in turn, with nothing else building. 68 is the 66 of feature 016 and the two
  new ones.

### T021: the owner at the phone trial (2026-11-15)

Built for the phone and installed with their leave ("Yes, install it"). They were
asked to walk round the house, look at the sheets' names, the absence of the redraw
on those sheets, the forest's bent joins, and where the file's rooms meet the rest.
Asked how the authored map looks on the phone, they said: "Looks right". They said
nothing more of any one of those points, and nothing of the house as a diamond.
