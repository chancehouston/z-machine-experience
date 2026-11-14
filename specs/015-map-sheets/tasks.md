# Tasks: The Other Sheets of the Map

**Input**: `specs/015-map-sheets/` (spec, plan, research R1 to R7, data-model,
contracts/sheets.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass and is seen to fail for the reason the code is missing. The record says, for
each, whether that happened.

One stage, one pull request against `main`, not stacked.

## Phase 1: The layer

- [ ] T001 [US1] Add tests in `map_tests.rs`: `sheet(n)` for the player's sheet equals `view()`; for another it has `own` false, no `here`, ways or look, and stubs for every place; numbers out of range give none; the places of all sheets are every place once (L§1 to L§3). See them fail
- [ ] T002 [US3] Add tests: the label with and without a named place; the spoken words of another sheet; `before`, `after` going round, none with one sheet; `back` on another sheet and none on the player's (L§4, L§5). See them fail
- [ ] T003 [US1] Add a test: each lead has the id of its far place, which is on the sheet it names (L§6). See it fail
- [ ] T004 [US1] In `map.rs`: `view_of`, `Map::sheet`, `SheetRef`, the new parts of `MapView` and `LeadView`. T001 to T003 pass
- [ ] T005 [US1] `json.rs` and `json_tests.rs`: the new keys (S§1), test first
- [ ] T006 [US1] Add to `tests/map.rs`: at the end of Zork I's walkthrough every sheet is given, they hold every place once, every lead's place is on its sheet, and asking leaves the kept game, the snapshot and the page as they were (SC-001 to SC-003, L§7); a stale stamp and a number with no sheet are refused. See it fail: no `Session::sheet`. Write `Session::sheet`
- [ ] T007 [P] The same on Adventure's walkthrough; the time to make the largest sheet, in `tests/speed.rs`

## Phase 2: The binding

- [ ] T008 [US1] `zmx_session_sheet` in `tests/abi.rs` (the list, and a test across the boundary), then in `src/lib.rs` and `include/zmx.h`

## Phase 3: The app

- [ ] T009 [US1] Add UI tests: `testTurningToTheSheetAWayLeadsTo` (down to the Cellar and up again; open the map; touch the lead: the label names the other sheet, there is no lamp and no way control, the far place is in the middle, no turn is added; A§3, A§4, A§6); `testSteppingThroughTheSheets` (the arrows go round every sheet and back; with one sheet there are none; A§2); `testComingBackToMySheet` (the return control; closing the map; a new turn; and the band shows the player's sheet throughout; A§1, A§4, A§5). See them fail
- [ ] T010 [US3] Rule tests in `app_rules.rs`: only `Engine.swift` calls `zmx_session_sheet`; the map's files write no word for a sheet; the band is given `showing.map` (A§1, A§7). See them fail
- [ ] T011 [US1] `Snapshot.swift`: the new keys. `Engine.swift`, `Game.swift`: asking for a sheet; the sheet looked at and its centre, dropped at a new snapshot. `MapDrawing.swift`: leads as buttons when opened out. `MapBand.swift`: the label, the arrows, the return control, centring. `GameView.swift`: the band's own sheet; dropping the sheet looked at when the map closes. T009 and T010 pass

## Phase 4: Proof and record

- [ ] T012 `cargo fmt`, `cargo clippy`, `cargo test --workspace` totalled from the result lines; `scripts/build-ios.sh`; `scripts/test-ios.sh 27` then `26`, in turn, with nothing else building; check each ran every test. Record the counts
- [ ] T013 [P] `README.md`, `apps/ios/README.md`
- [ ] T014 Build for the owner's phone after the runs and walk the quickstart's trial with them (SC-005). Record what they say, in their words
