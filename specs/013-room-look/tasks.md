# Tasks: Touch the Room to Look

**Input**: `specs/013-room-look/` (spec, plan, research R1 to R7, data-model,
contracts/look.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass, and is run and seen to fail for the reason the code is missing. The record at
the end says, for each, whether that happened; where a test passed when written, or
was written after the code, it says so plainly.

One stage, one pull request, stacked on pull request 21.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: The layer (User Stories 1 and 2)

- [x] T001 [US2] Write `crates/zmachine-experience/src/look_tests.rs`: `look` is found in a dictionary that has it; `l` where only that is there; nothing where neither is; a candidate longer than the dictionary keeps is found cut (L§1). See it fail: no module `look`
- [x] T002 [US2] Write `crates/zmachine-experience/src/look.rs` (`word(words, kept) -> Option<String>`, the constant `SPOKEN_LOOK`) and name it in `lib.rs`. T001 passes
- [x] T003 [US1] Add to `map_tests.rs`: the view has a look with the word and the layer's words at a command stop in a known place; none with no word, at a question, or where the place is not known; it is offered in the dark and in a place with no name (L§2, L§6). See them fail
- [x] T004 [US1] In `map.rs` add `LookView`, `MapView.look`, the map's word (`Map::with_look`) and the rule of L§2. T003 passes
- [x] T005 [US1] Add to `json_tests.rs`: `map.look` after `ways`, `null` or the object (S§1, S§2). See it fail; write it in `json.rs`; it passes
- [x] T006 [US1] Add to `crates/zmachine-experience/tests/map.rs`: on Zork I, walk north and south, `tap_place`, and the reply holds the full description of West of House with the command recorded as a tap; the map's page is the same before and after (L§3, L§4, SC-001); the same on Adventure; `tap_place` with a stale stamp and at a question is refused (L§3); a kept game with such taps comes back the same, 100 times (L§5, SC-003). See them fail: no `tap_place`
- [x] T007 [US1] In `lib.rs` find the word when a session is opened (R2), give it to each new map, and add `Session::tap_place`. T006 passes
- [x] T008 [P] [US2] Add to `crates/zmachine-corpus/tests/map.rs`: for every recognised story, the look's command where offered is a word of the story's dictionary, and the count of stories with and without the word is printed (SC-002). Record the counts here

## Phase 2: The binding

- [x] T009 [US1] Add `zmx_session_tap_place` to the list in `crates/zmachine-ffi/tests/abi.rs` and a test that it sends the look and refuses a stale stamp. See it fail; add it to `src/lib.rs` and `include/zmx.h`; it passes

## Phase 3: The app (User Stories 1 to 4)

- [x] T010 [US1] Add UI tests: `testTouchingMyRoomLooks` (north, south, touch `map.here`: the next turn's command is the layer's and its text holds the full description; `map.here` is a button; again on the opened sheet, which stays open; A§1); `testMyRoomIsNotAButtonAtAQuestion` (A§2); `testOtherRoomsDoNothing` (touching another place adds no turn; A§3); and a check in `testTheLargestTextSize` that `map.here` is at least 44 points tall and overlaps no `compass.*` (A§4). See them fail
- [x] T011 [US3] Change `testOpeningTheMapOutAndBack` so its drag begins on `map.here`, and add `testPullingTheBandDownFromMyRoom`: the sheet opens and no turn is added (A§5). The first fails or passes as the band stands; record which
- [x] T012 [US2] Add to `crates/zmachine-ffi/tests/app_rules.rs`: only `Engine.swift` calls `zmx_session_tap_place`; `MapDrawing.swift` gives the hint as `Text(verbatim: look.spoken)`; no file of the app holds the literal `look` or `l` (A§6; add them to the list of command words). See it fail
- [x] T013 [US1] `Snapshot.swift`: `MapLook` and `MapSheet.look`. `Engine.swift` and `Game.swift`: the touch, sent. `MapBand.swift` and `GameView.swift`: the closure handed down. `MapDrawing.swift`: the player's place answers a tap when `look` is present and the app is ready, is a button to a screen reader with the layer's hint, and has the touch area of A§4. T010 to T012 pass

## Phase 4: Proof and record

- [x] T014 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, the timing tests in release, `scripts/build-ios.sh`, `scripts/test-ios.sh 27` and `26`. Record the counts and figures here
- [x] T015 [P] Update `README.md` and `apps/ios/README.md`
- [x] T016 Build for the owner's phone and walk the quickstart's hand trial with the owner (SC-005). Record what they say, in their words

## Dependencies

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T009 → T010 to T012 → T013 → T014.
T008 and T015 may be done beside the others once T007 is in.

## Record (2026-11-14)

**Test-first, as it was done.**

- **Written first and seen failing**: the layer's tests of the word, of the look in
  the map's view and the snapshot, and of `tap_place` on Zork I and Adventure (they
  did not build: no module `look`, no `with_look`, no field `look`; T001, T003,
  T005, T006 were written together and failed together, then the code for all four
  was written). The binding's test (no function `zmx_session_tap_place`). In the
  app: `testTouchingMyRoomLooks` (the place was not a button), the added lines of
  `testTheLargestTextSize` (the place was 31 points tall), and the rule test (no
  call of the binding).
- **Passed when written, as guards of what already held**:
  `testMyRoomIsNotAButtonAtAQuestion`, `testOtherRoomsDoNothing`,
  `testPullingTheBandDownFromMyRoom`, and `testOpeningTheMapOutAndBack` with its
  drag begun on the player's place. They passed again once the place answered a
  tap, which is what they are for (research R4 is confirmed: a tap gesture leaves
  the drag to the map).
- **Written after the layer's code**: the corpus test (T008).
- **One of my own tests was wrong**: `testTouchingMyRoomLooks` waited for the count
  of moves in the status element's label; it is the element's value. The first full
  run on iOS 27 failed there, with the look itself working. Corrected and passed.
- **One run was not what it seemed**: the first full run on iOS 26 reported 49
  tests, all passing, where there are 53. It was started while the app was being
  built for the owner's phone; the cause was not found. Run again by itself it ran
  53.

**Where the tests are thin.** No UI test can read a hint: that the hint is the
layer's words is held by the rule test of the source. The touch area's width is not
measured, only that no way's control overlaps it.

**T008, the corpus.** 9 stories have a word for looking and offer the touch; none
lacks the word; 5 have no map. In all 9 the word is `look`.

**Counts.** `cargo test --workspace`: no failure. `cargo clippy --workspace
--all-targets`: clean. App: 53 tests on iOS 27.2 (52 passed in the full run, and the
corrected one passed when run again by itself; the full suite was not run a second
time on iOS 27) and 53 on iOS 26.0, one skipped on each (`testScreenshots`).

**Timings, release.** Zork I: slowest command with its snapshot and map 0.59 ms.
Adventure: 9.7 ms, against the limit of 10 (it was run while the simulator's tests
were running). Bringing back 2,000 turns: 101 ms. Largest map 6,760 bytes.

**T016 (the owner's phone, SC-005)**: installed on 2026-11-14 at the owner's request
while the suites were still running. Asked whether tapping the room gives the full
description on the phone, they said: "Yes, it works". They were not asked again
whether it is the easy way they had in mind, and did not say.
