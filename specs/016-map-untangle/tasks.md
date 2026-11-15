# Tasks: Untangling the Map

**Input**: `specs/016-map-untangle/` (spec, plan, research R1 to R8, data-model,
contracts/untangle.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

One stage, one pull request against `main`, not stacked.

## Phase 1: The redraw, in the layer

- [x] T001 [US1] Write `map_redraw_tests.rs`, on places and ways made by hand: the anchor keeps its position; each place joined by a way walked out of it is along that way at the first free of one to three steps; two ways to one place, the first in the compass's order decides; a place joined only by a way walked in is put opposite (R§1 to R§3). See them fail: no module
- [x] T002 [US1] Write `map_redraw.rs`: `around` (research R1). T001 passes
- [x] T003 [US2] Add tests: with the anchor and the places it joins held, the others are moved to fewer bent joins, then fewer crossings; held places do not move; the search stops at its budget and what it has is sound; no two places share a position (R§4, R§6, R§9). See them fail
- [x] T004 [US2] Write the tidying (research R2). T003 passes
- [x] T005 [US1] Add tests in `map_tests.rs`: `Map::redraw` moves the sheet's places as `around` and the tidying say and leaves other sheets alone; attached places keep their corner; where the player's place is attached neither it nor its parent moves; a second redraw changes nothing; only walked ways are used; the view offers `redraw` by R7 and not otherwise (R§5 to R§8, S§2). See them fail
- [x] T006 [US1] `Map::redraw`, and `redraw` in the view. T005 passes
- [x] T007 [US1] `json.rs`: `map.redraw` (S§1), test first

## Phase 2: Keeping

- [x] T008 [US4] Add tests in `kept_tests.rs`: form 2 with `redraw` lines is written and read; form 1 is still read; with no redraw the bytes are form 1's but for the number; a damaged or misplaced `redraw` is refused (K§1, K§3). See them fail
- [x] T009 [US4] `kept.rs`: form 2. T008 passes. Update `specs/011-resume-game/contracts/kept-game.md`
- [x] T010 [US4] Add to `tests/resume.rs` and `tests/map.rs`: `Session::redraw_map` leaves the turns and the transcript as they were and is refused for a stale stamp and where not offered (R§10); a game with redraws at several turns, brought back, has the same map, 100 times; a redraw one input early or late does not (K§2); a `redraw` where none can be made cannot be brought back (K§4); Zork I's walkthrough brought back at every tenth turn is as today (SC-003). See them fail: no `redraw_map`
- [x] T011 [US4] `Session::redraw_map`, the session's record of redraws, `kept()` and `bring_back`. T010 passes

## Phase 3: The measures

- [x] T012 [US1] Add to `tests/map.rs`: on the wander of feature 012's tests, a redraw in each positioned room of the house's sheet in turn (each from the same wandered game, brought back): every join of the room straight in at least four rooms of five, and never fewer than before; the sheet never has more bent joins than today (SC-001, SC-002). Record the figures here
- [x] T013 [P] Corpus (`crates/zmachine-corpus/tests/map.rs`): after each story's wander, a redraw where offered leaves no two places in one position and faults nowhere (SC-005). `tests/speed.rs`: the slowest redraw with its snapshot on both walkthroughs and the wander, within 50 ms, and that a command's cost is unchanged (FR-013, FR-014). Record the figures
- [x] T014 Confirm with no press that every recorded page and earlier map test passes unchanged (SC-004); update `specs/012-map-first-slice/contracts/map.md` for the clauses a redraw changes

## Phase 4: The binding and the app

- [x] T015 [US1] `zmx_session_redraw_map`: the list and a test across the boundary in `tests/abi.rs`, then `src/lib.rs` and `include/zmx.h`
- [x] T016 [US1] Add UI tests: `testRedrawingTheMapAroundMe` (a walk that leaves the player's room with a bent join; press `button.map.redraw`; each joined room's frame is exactly along its way from `map.here`, which has not moved; no turn is added; A§16, A§17); `testARedrawnMapComesBack` (relaunch: the frames are as they were left); the control absent on another sheet and at a question; in `testTheLargestTextSize`, 44 points and no overlap with a way. Rule tests: only `Engine.swift` calls the binding, and the map's files work out no position (A§19). See them fail
- [x] T017 [US3] `Snapshot.swift`, `Engine.swift`, `Game.swift`: the offer, the press, the snapshot again, keeping. `MapBand.swift`: the control. `MapDrawing.swift`: places glide; every join a shape with animatable ends (A§18). T016 passes; `testReducedMotion` still passes

## Phase 5: Proof and record

- [x] T018 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; the timing tests one at a time; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26` in turn with nothing else building; check each ran every test. Record the counts
- [x] T019 [P] `README.md`, `apps/ios/README.md`
- [x] T020 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-006). Record what they say, in their words

## Record (2026-11-15)

**Test-first, as it was done.**

- **Written first and seen failing**: the tests of laying out around a place and of
  tidying (no module); the map's tests of a redraw (no `Map::redraw`, no `redraw`
  in the view); the snapshot's key (the key list differed); the kept game's form 2
  (no field `redraws`); the session's tests (no `redraw_map`); the binding's test
  (no function); the three UI tests (no `button.map.redraw`) and the rule test.
- **Passed the first time the code existed**: every one of the above. Nothing in
  the layer or the app needed a second attempt.
- **An older test changed**: `another_form_is_not_read` used form 2 as the form not
  known; it uses 3.
- **Written after the code**: the measure on the wander (T012), the corpus test and
  the timing (T013).
- **Not done as tasked**: T012 was to redraw in each room of one finished sheet. A
  test cannot put the player in a room without walking there, so it redraws on a
  copy of the game at every tenth move of the wander, around wherever the player
  is. T016's check in `testTheLargestTextSize` was not added: at that test's one
  place there is no control to measure. T014: the earlier contracts are noted; the
  proof that nothing changes without a press is that every earlier test passes
  unchanged.

**The measure (T012; SC-001, SC-002).** A wander of 400 moves through Zork I, a
redraw on a copy at every tenth move, 38 cases. The room's own joins were all
straight in 4 cases before and 34 after (the bar was four in five; this is nine in
ten). In no case were fewer of them straight than before. The sheet's bent joins,
summed over the cases, were 391 before and 231 after, and in no case did the sheet
have more than it had. The player's place moved in none.

**Keeping (T010; SC-003).** A game with four redraws is kept in form 2 and, brought
back 100 times, has the same map, view and kept bytes each time. A redraw moved one
input later in the kept game gives another map. A redraw where none can be made is
not brought back.

**The corpus (T013; SC-005).** A redraw every seventh move of a wander of 200:
offered and sound in Zork I (29 times, moved places 18), Zork II (24, 12), Zork III
(2, 0), Mini-Zork (29, 22), Adventure (28, 21), Adventureland (28, 3), Anchorhead
(28, 15). Never two places in one position, never a turn added. Six stories were
never offered one on their wander.

**Time (T013; FR-013).** Slowest redraw with its snapshot: 3.7 ms on the wander (80
redraws), 0.5 ms on Zork I's walkthrough, 4.4 ms on Adventure's. The limit was 50.

**Where it is thin.** The tidying's budget of 3,000 layouts was never seen to be
reached, so stopping at it is tested only on a built case. Whether the gliding
reads well is judged by eye. The control at the largest text size is not measured.

**Counts (T018).** `cargo fmt --check` and `cargo clippy --workspace --all-targets`:
clean. `cargo test --workspace`: 1,097 passed, none failed. App: 66 tests on iOS 27.2
and 66 on iOS 26.0, one skipped on each, no failure, run in turn with nothing else
building.

**T020 (the owner's phone, SC-006)**: installed on 2026-11-15 after the runs. Asked
whether the redraw button makes the rooms round them clear on the phone, the owner
said: "Yes, it works". They were asked whether the gliding was easy to follow and
whether the far rooms ended up anywhere sensible, and said nothing of either.

