# Tasks: The Inventory Card, stage 1

**Input**: `specs/014-inventory-card/` (spec, plan, research R1 to R9, data-model,
contracts/card.md, quickstart)

**Test-first (Principle II)**: each test task is done before the task that makes it
pass, and is run and seen to fail for the reason the code is missing. The record at
the end says, for each, whether that happened; where a test passed when written, or
was written after the code, it says so plainly.

Stage 1 only: the card, the description by peek, and Drop. Stage 2's tasks are
written when stage 1 is merged. One pull request, stacked on feature 013.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Introspection, the words that name a thing

- [ ] T001 [US1] Add tests in `crates/zmachine-introspect/src/inform_tests.rs` and `zil_tests.rs`: a carried thing's `words` are those of its naming property, in order; a thing without the property has none; in a built Infocom story the property is found by its shape, and with two candidates or none the words are unknown (I§1 to I§3). See them fail: no field `words`
- [ ] T002 [US1] Add `Carried.words`; in `inform.rs` read `name`; in `zil.rs` find the naming property once at recognition (R1). T001 passes
- [ ] T003 [P] [US1] Add `noun-property` to the pack (`pack.rs`, `pack_tests.rs`, `packs/README.md`): a test that it is used in place of the search; see it fail; write it
- [ ] T004 [US1] Add `words` to the inventory in `crates/zmachine-introspect/src/json.rs` with its test, and to `docs/introspection.md` section 7
- [ ] T005 [US1] Add to the corpus (`crates/zmachine-corpus/tests/card.rs`): for every recognised story, the naming property is found; print how many carried things along each wander had words. Record the counts here. No name or word is printed

## Phase 2: The layer

- [ ] T006 [US3] Rename `look.rs` to `verbs.rs`; add tests that `examine` then `x`, and `drop`, are found as `look` is (L§1); see them fail; write them. The session keeps all three
- [ ] T007 [US1] Add `card_tests.rs` and tests in `strip_tests.rs`: each shown item has its `index`, counted with contents after their holder; `card` is true only at a command stop, with a word for examining, for an item with a word (L§2, S§1). See them fail; write them in `strip.rs`
- [ ] T008 [US1] Write `crates/zmachine-experience/tests/card.rs`: on a built story, a card has the item's name, its state, and the copy's reply trimmed (L§4); two cards at one turn are equal (L§6); after a card the kept game, the snapshot, the turns and the map's page are what they were (L§5); stale and not-offered are refused (L§3). On Zork I: take the leaflet, the card's description is the reply a copy gives to the same line typed (SC-002). See them fail: no `Session::card`
- [ ] T009 [US1] Write `card.rs` (`Card`, `Action`, the peek by R4) and `Session::card`. T008 passes
- [ ] T010 [US3] Add tests with built stories: an item whose examining ends at a question, one that ends the game, and one whose copy passes the limit each give a card with the note and no description (L§4); a story with no word for examining offers no card (L§2). See them fail or pass and say which; make them pass
- [ ] T011 [US2] Add tests: a card for an item not inside another has Drop when there is a word for it, and an item inside another has none (L§7); `tap_action` sends the command as a tap, the strip then lacks the item, and it is refused for a stale stamp, an item with no card and an action not on the card (L§8). See them fail; write `Session::tap_action`
- [ ] T012 [US1] Add the strip's `index` and `card` to the snapshot and `card_json` for a card (`json.rs`, `json_tests.rs`; S§1 to S§3). See them fail; write them
- [ ] T013 [P] [US1] Corpus and limits: in `crates/zmachine-corpus/tests/card.rs`, peek every tappable item at every stop of each story's wander; no fault, the game unchanged, and the share of peeks that gave a description printed for each story (SC-001, SC-003). In `tests/speed.rs` an ignored test: the slowest card on both walkthroughs (SC-004), and the snapshot's cost against FR-020. Record the figures here

## Phase 3: The binding

- [ ] T014 [US1] Add `zmx_session_card` (JSON, or NULL when refused) and `zmx_session_tap_action` to the list in `tests/abi.rs`, with a test of each across the boundary. See it fail; add them to `src/lib.rs` and `include/zmx.h`

## Phase 4: The app

- [ ] T015 [US1] Add UI tests: `testAnItemsCard` (open the mailbox, take the leaflet, touch `item.0`: `card` shows the name and the game's text, the moves and the newest turn are unchanged, the field still has the keyboard; `card.close` brings the map back; A§1, A§2, A§3, A§5); `testDroppingFromTheCard` (the action's button: a tapped command in the next turn, the card gone, the strip without the item; A§4); `testTheCardGoesAtANewTurn` (a command typed with the card open; A§3); `testTheCardIsSpoken` with `-speakAlways` (A§6); and in `testTheLargestTextSize`, the card's buttons at least 44 points. See them fail
- [ ] T016 [US3] Add to `crates/zmachine-ffi/tests/app_rules.rs`: only `Engine.swift` calls the two bindings; `ItemCard.swift` shows `card.name`, the description or note, `card.ahead` and each action's `label` verbatim, and holds no other text; `examine`, `x`, `drop` join the words the app may not hold (A§7). See it fail
- [ ] T017 [US1] `Snapshot.swift`: the items' `index` and `card`; `Card` and `CardAction`. `Engine.swift`, `Game.swift`: asking for a card, sending an action. `InventoryStrip.swift`: items as buttons. New `ItemCard.swift`: the card and the token. `GameView.swift` and `MapBand.swift`: the card over the band, held with its stamp, dropped at a new turn; spoken through `Announcer`. T015 and T016 pass

## Phase 5: Proof and record

- [ ] T018 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, the timing tests in release, `scripts/build-ios.sh`, then `scripts/test-ios.sh 27` and `26` one after the other with nothing else building. Check each ran every test. Record the counts here
- [ ] T019 [P] Update `README.md`, `apps/ios/README.md`, `packs/README.md`
- [ ] T020 Build for the owner's phone and walk the quickstart's hand trial with the owner (SC-006). Record what they say, in their words

## Dependencies

T001 → T002 → T004 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T014 → T015,
T016 → T017 → T018 → T020. T003, T005, T013 and T019 may be done beside the others
once what they test exists.
