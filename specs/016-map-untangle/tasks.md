# Tasks: Untangling the Map

**Input**: `specs/016-map-untangle/` (spec, plan, research R1 to R8, data-model,
contracts/untangle.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

One stage, one pull request against `main`, not stacked.

## Phase 1: The redraw, in the layer

- [ ] T001 [US1] Write `map_redraw_tests.rs`, on places and ways made by hand: the anchor keeps its position; each place joined by a way walked out of it is along that way at the first free of one to three steps; two ways to one place, the first in the compass's order decides; a place joined only by a way walked in is put opposite (R§1 to R§3). See them fail: no module
- [ ] T002 [US1] Write `map_redraw.rs`: `around` (research R1). T001 passes
- [ ] T003 [US2] Add tests: with the anchor and the places it joins held, the others are moved to fewer bent joins, then fewer crossings; held places do not move; the search stops at its budget and what it has is sound; no two places share a position (R§4, R§6, R§9). See them fail
- [ ] T004 [US2] Write the tidying (research R2). T003 passes
- [ ] T005 [US1] Add tests in `map_tests.rs`: `Map::redraw` moves the sheet's places as `around` and the tidying say and leaves other sheets alone; attached places keep their corner; where the player's place is attached neither it nor its parent moves; a second redraw changes nothing; only walked ways are used; the view offers `redraw` by R7 and not otherwise (R§5 to R§8, S§2). See them fail
- [ ] T006 [US1] `Map::redraw`, and `redraw` in the view. T005 passes
- [ ] T007 [US1] `json.rs`: `map.redraw` (S§1), test first

## Phase 2: Keeping

- [ ] T008 [US4] Add tests in `kept_tests.rs`: form 2 with `redraw` lines is written and read; form 1 is still read; with no redraw the bytes are form 1's but for the number; a damaged or misplaced `redraw` is refused (K§1, K§3). See them fail
- [ ] T009 [US4] `kept.rs`: form 2. T008 passes. Update `specs/011-resume-game/contracts/kept-game.md`
- [ ] T010 [US4] Add to `tests/resume.rs` and `tests/map.rs`: `Session::redraw_map` leaves the turns and the transcript as they were and is refused for a stale stamp and where not offered (R§10); a game with redraws at several turns, brought back, has the same map, 100 times; a redraw one input early or late does not (K§2); a `redraw` where none can be made cannot be brought back (K§4); Zork I's walkthrough brought back at every tenth turn is as today (SC-003). See them fail: no `redraw_map`
- [ ] T011 [US4] `Session::redraw_map`, the session's record of redraws, `kept()` and `bring_back`. T010 passes

## Phase 3: The measures

- [ ] T012 [US1] Add to `tests/map.rs`: on the wander of feature 012's tests, a redraw in each positioned room of the house's sheet in turn (each from the same wandered game, brought back): every join of the room straight in at least four rooms of five, and never fewer than before; the sheet never has more bent joins than today (SC-001, SC-002). Record the figures here
- [ ] T013 [P] Corpus (`crates/zmachine-corpus/tests/map.rs`): after each story's wander, a redraw where offered leaves no two places in one position and faults nowhere (SC-005). `tests/speed.rs`: the slowest redraw with its snapshot on both walkthroughs and the wander, within 50 ms, and that a command's cost is unchanged (FR-013, FR-014). Record the figures
- [ ] T014 Confirm with no press that every recorded page and earlier map test passes unchanged (SC-004); update `specs/012-map-first-slice/contracts/map.md` for the clauses a redraw changes

## Phase 4: The binding and the app

- [ ] T015 [US1] `zmx_session_redraw_map`: the list and a test across the boundary in `tests/abi.rs`, then `src/lib.rs` and `include/zmx.h`
- [ ] T016 [US1] Add UI tests: `testRedrawingTheMapAroundMe` (a walk that leaves the player's room with a bent join; press `button.map.redraw`; each joined room's frame is exactly along its way from `map.here`, which has not moved; no turn is added; A§16, A§17); `testARedrawnMapComesBack` (relaunch: the frames are as they were left); the control absent on another sheet and at a question; in `testTheLargestTextSize`, 44 points and no overlap with a way. Rule tests: only `Engine.swift` calls the binding, and the map's files work out no position (A§19). See them fail
- [ ] T017 [US3] `Snapshot.swift`, `Engine.swift`, `Game.swift`: the offer, the press, the snapshot again, keeping. `MapBand.swift`: the control. `MapDrawing.swift`: places glide; every join a shape with animatable ends (A§18). T016 passes; `testReducedMotion` still passes

## Phase 5: Proof and record

- [ ] T018 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; the timing tests one at a time; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26` in turn with nothing else building; check each ran every test. Record the counts
- [ ] T019 [P] `README.md`, `apps/ios/README.md`
- [ ] T020 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-006). Record what they say, in their words
