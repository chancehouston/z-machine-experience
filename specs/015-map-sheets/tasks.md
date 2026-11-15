# Tasks: The Other Sheets of the Map

**Input**: `specs/015-map-sheets/` (spec, plan, research R1 to R7, data-model,
contracts/sheets.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

One stage, one pull request against `main`, not stacked.

## Phase 1: The layer

- [x] T001 [US1] Add tests in `map_tests.rs`: `sheet(n)` for the player's sheet equals `view()`; for another it has `own` false, no `here`, ways or look, and stubs for every place; numbers out of range give none; the places of all sheets are every place once (L§1 to L§3). See them fail
- [x] T002 [US3] Add tests: the label with and without a named place; the spoken words of another sheet; `before`, `after` going round, none with one sheet; `back` on another sheet and none on the player's (L§4, L§5). See them fail
- [x] T003 [US1] Add a test: each lead has the id of its far place, which is on the sheet it names (L§6). See it fail
- [x] T004 [US1] In `map.rs`: `view_of`, `Map::sheet`, `SheetRef`, the new parts of `MapView` and `LeadView`. T001 to T003 pass
- [x] T005 [US1] `json.rs` and `json_tests.rs`: the new keys (S§1), test first
- [x] T006 [US1] Add to `tests/map.rs`: at the end of Zork I's walkthrough every sheet is given, they hold every place once, every lead's place is on its sheet, and asking leaves the kept game, the snapshot and the page as they were (SC-001 to SC-003, L§7); a stale stamp and a number with no sheet are refused. See it fail: no `Session::sheet`. Write `Session::sheet`
- [x] T007 [P] The same on Adventure's walkthrough; the time to make the largest sheet, in `tests/speed.rs`

## Phase 2: The binding

- [x] T008 [US1] `zmx_session_sheet` in `tests/abi.rs` (the list, and a test across the boundary), then in `src/lib.rs` and `include/zmx.h`

## Phase 3: The app

- [x] T009 [US1] Add UI tests: `testTurningToTheSheetAWayLeadsTo` (down to the Cellar and up again; open the map; touch the lead: the label names the other sheet, there is no lamp and no way control, the far place is in the middle, no turn is added; A§3, A§4, A§6); `testSteppingThroughTheSheets` (the arrows go round every sheet and back; with one sheet there are none; A§2); `testComingBackToMySheet` (the return control; closing the map; a new turn; and the band shows the player's sheet throughout; A§1, A§4, A§5). See them fail
- [x] T010 [US3] Rule tests in `app_rules.rs`: only `Engine.swift` calls `zmx_session_sheet`; the map's files write no word for a sheet; the band is given `showing.map` (A§1, A§7). See them fail
- [x] T011 [US1] `Snapshot.swift`: the new keys. `Engine.swift`, `Game.swift`: asking for a sheet; the sheet looked at and its centre, dropped at a new snapshot. `MapDrawing.swift`: leads as buttons when opened out. `MapBand.swift`: the label, the arrows, the return control, centring. `GameView.swift`: the band's own sheet; dropping the sheet looked at when the map closes. T009 and T010 pass

## Phase 4: Proof and record

- [x] T012 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26`, in turn, with nothing else building; check each ran every test. Record the counts
- [x] T013 [P] `README.md`, `apps/ios/README.md`
- [x] T014 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-005). Record what they say, in their words

## Record (2026-11-14)

**Test-first, as it was done.**

- **Written first and seen failing**: the layer's tests of any sheet, its label, its
  neighbours and its leads (did not build: no `Map::sheet`, no such fields); the
  walkthrough tests (no `Session::sheet`); the binding's test (no function); the
  two UI tests (no `map.sheet.label`).
- **Written with the code**: the snapshot's keys (the test was changed first and
  the code followed in the same step, without a run between). The rule tests: my
  first version did not build, for a name of mine that hid a function, and was
  mended before the app's code was written.
- **Not done as tasked**: T007's separate timing of a sheet. A sheet is made by
  the code that makes the player's own, which the timing tests already measure
  (under 0.1 ms on both walkthroughs); no new figure was taken. Adventure's sheets
  are tested (seven).
- **Two things the UI test found.** The way it meant to touch was off the screen:
  a sheet turned to by an arrow is shown from its first place, and the test now
  drags the sheet as a player would. And a way that leads back to the player's
  own sheet did not put its place in the middle: the place to centre on was kept
  only for other sheets. It is kept for the player's own now.

**What the proof shows.** At the end of Zork I's walkthrough all 11 sheets can be
had, and Adventure's 7; together they hold every place of the map once; every way
that leaves a sheet leads to a place on the sheet it names; and asking for every
sheet leaves the kept game, the snapshot and the map's page identical.

**Where the tests are thin.** "Any new turn returns to the player's sheet" is
tested by closing the map, which a typed command must do first; a turn made with
the map still open (an action on an item's card) is not tested. The screen
reader's words for the arrows are tested as labels, not listened to.

**Counts (T012).** `cargo fmt --check` and `cargo clippy --workspace --all-targets`:
clean. `cargo test --workspace`: 1,079 passed, none failed. App: 63 tests on iOS 27.2
and 63 on iOS 26.0, one skipped on each, no failure, run in turn with nothing else
building.

**A fix to the tests themselves, made with this feature (2026-11-14).** Three runs on
the hosted CI runner failed today, each at the first touch of a different test
(`testADoorClosedThenOpen`, `testLaunchTime`, `testLaunchTimeIntoAKeptGame`), and each
passed when run again; none failed here. The owner asked for it to be mended and not
run again. The tests now wait for the keyboard to have come up after a launch before
their first touch, and 79 of their touches wait until the control is on screen and
enabled. If a control never becomes touchable the touch is made all the same, so
that a test of a dead control still makes it. The cause was not reproduced here, so
this is not proven to be it: it closes the two ways seen by which a first touch can
be lost on a slow machine. With it the whole suite passed on iOS 27.2, 63 tests, in
1,394 seconds against 1,302 before. It was not run again on iOS 26.

**T014 (the owner's phone, SC-005)**: installed on 2026-11-14 after the runs. Asked
whether the map's other sheets work on the phone, the owner said: "Yes, it works".
They were told that a sheet turned to by an arrow is shown from its first place,
which can leave most of it to one side until dragged; they said nothing of it.

