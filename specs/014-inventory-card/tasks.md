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

- [x] T001 [US1] Add tests in `crates/zmachine-introspect/src/inform_tests.rs` and `zil_tests.rs`: a carried thing's `words` are those of its naming property, in order; a thing without the property has none; in a built Infocom story the property is found by its shape, and with two candidates or none the words are unknown (I§1 to I§3). See them fail: no field `words`
- [x] T002 [US1] Add `Carried.words`; in `inform.rs` read `name`; in `zil.rs` find the naming property once at recognition (R1). T001 passes
- [x] T003 [P] [US1] Add `noun-property` to the pack (`pack.rs`, `pack_tests.rs`, `packs/README.md`): a test that it is used in place of the search; see it fail; write it
- [x] T004 [US1] Add `words` to the inventory in `crates/zmachine-introspect/src/json.rs` with its test, and to `docs/introspection.md` section 7
- [x] T005 [US1] Add to the corpus (`crates/zmachine-corpus/tests/card.rs`): for every recognised story, the naming property is found; print how many carried things along each wander had words. Record the counts here. No name or word is printed

## Phase 2: The layer

- [x] T006 [US3] Rename `look.rs` to `verbs.rs`; add tests that `examine` then `x`, and `drop`, are found as `look` is (L§1); see them fail; write them. The session keeps all three
- [x] T007 [US1] Add `card_tests.rs` and tests in `strip_tests.rs`: each shown item has its `index`, counted with contents after their holder; `card` is true only at a command stop, with a word for examining, for an item with a word (L§2, S§1). See them fail; write them in `strip.rs`
- [x] T008 [US1] Write `crates/zmachine-experience/tests/card.rs`: on a built story, a card has the item's name, its state, and the copy's reply trimmed (L§4); two cards at one turn are equal (L§6); after a card the kept game, the snapshot, the turns and the map's page are what they were (L§5); stale and not-offered are refused (L§3). On Zork I: take the leaflet, the card's description is the reply a copy gives to the same line typed (SC-002). See them fail: no `Session::card`
- [x] T009 [US1] Write `card.rs` (`Card`, `Action`, the peek by R4) and `Session::card`. T008 passes
- [x] T010 [US3] Add tests with built stories: an item whose examining ends at a question, one that ends the game, and one whose copy passes the limit each give a card with the note and no description (L§4); a story with no word for examining offers no card (L§2). See them fail or pass and say which; make them pass
- [x] T011 [US2] Add tests: a card for an item not inside another has Drop when there is a word for it, and an item inside another has none (L§7); `tap_action` sends the command as a tap, the strip then lacks the item, and it is refused for a stale stamp, an item with no card and an action not on the card (L§8). See them fail; write `Session::tap_action`
- [x] T012 [US1] Add the strip's `index` and `card` to the snapshot and `card_json` for a card (`json.rs`, `json_tests.rs`; S§1 to S§3). See them fail; write them
- [x] T013 [P] [US1] Corpus and limits: in `crates/zmachine-corpus/tests/card.rs`, peek every tappable item at every stop of each story's wander; no fault, the game unchanged, and the share of peeks that gave a description printed for each story (SC-001, SC-003). In `tests/speed.rs` an ignored test: the slowest card on both walkthroughs (SC-004), and the snapshot's cost against FR-020. Record the figures here

## Phase 3: The binding

- [x] T014 [US1] Add `zmx_session_card` (JSON, or NULL when refused) and `zmx_session_tap_action` to the list in `tests/abi.rs`, with a test of each across the boundary. See it fail; add them to `src/lib.rs` and `include/zmx.h`

## Phase 4: The app

- [x] T015 [US1] Add UI tests: `testAnItemsCard` (open the mailbox, take the leaflet, touch `item.0`: `card` shows the name and the game's text, the moves and the newest turn are unchanged, the field still has the keyboard; `card.close` brings the map back; A§1, A§2, A§3, A§5); `testDroppingFromTheCard` (the action's button: a tapped command in the next turn, the card gone, the strip without the item; A§4); `testTheCardGoesAtANewTurn` (a command typed with the card open; A§3); `testTheCardIsSpoken` with `-speakAlways` (A§6); and in `testTheLargestTextSize`, the card's buttons at least 44 points. See them fail
- [x] T016 [US3] Add to `crates/zmachine-ffi/tests/app_rules.rs`: only `Engine.swift` calls the two bindings; `ItemCard.swift` shows `card.name`, the description or note, `card.ahead` and each action's `label` verbatim, and holds no other text; `examine`, `x`, `drop` join the words the app may not hold (A§7). See it fail
- [x] T017 [US1] `Snapshot.swift`: the items' `index` and `card`; `Card` and `CardAction`. `Engine.swift`, `Game.swift`: asking for a card, sending an action. `InventoryStrip.swift`: items as buttons. New `ItemCard.swift`: the card and the token. `GameView.swift` and `MapBand.swift`: the card over the band, held with its stamp, dropped at a new turn; spoken through `Announcer`. T015 and T016 pass

## Phase 5: Proof and record

- [x] T018 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, the timing tests in release, `scripts/build-ios.sh`, then `scripts/test-ios.sh 27` and `26` one after the other with nothing else building. Check each ran every test. Record the counts here
- [x] T019 [P] Update `README.md`, `apps/ios/README.md`, `packs/README.md`
- [x] T020 Build for the owner's phone and walk the quickstart's hand trial with the owner (SC-006). Record what they say, in their words

## Dependencies

T001 → T002 → T004 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T014 → T015,
T016 → T017 → T018 → T020. T003, T005, T013 and T019 may be done beside the others
once what they test exists.

## Record of stage 1 (2026-11-14)

**Test-first, as it was done.**

- **Written first and seen failing**: the introspection tests of `words` in both
  families (did not build: no field `words`); the layer's tests of the verbs, the
  tappable items, the peek, the card and Drop (did not build: no module `card`, no
  `Session::card`); the binding's test (no such functions); the rule tests for the
  strip and the card (the strip's expectations differed; there was no
  `ItemCard.swift`); and the five UI tests of the card (no element `item.0`).
- **Written together, not one by one**: T007 to T012's tests were written in one
  go and failed together, then the layer's code was written for all of them. T010's
  cases (a question, the end, a fault, the limit, an empty reply) are unit tests of
  the peek on small built stories, not of a whole session.
- **Written after the code they test**: the pack's `noun-property` test passed
  only after its story fixture was changed to lay itself out twice; the corpus test
  (T005, T013) and the timing test.
- **Passed the first time the code existed**: all seven of the layer's card tests,
  and all five UI tests.
- **Caught by tests I had not thought of**: the rule that nothing under `apps/ios`
  names a game failed on my UI tests, which looked for the game's name in the
  leaflet's text; they look for other words of it now. And `spec_agreement` failed
  because the introspection document gained section 12.12 with no test citing it:
  the tests now cite I§7.3 and I§12.12, and the specification's version is 0.6.0
  (a field was added). I had run the app's suites before finding that failure: it
  was hidden by a count I misread. The fix changed a version string and comments,
  not the app.

**Changed in building, for the owner to see.**

1. **Which word is sent for an item** (research R2). The plan sent the first word
   of the item's naming property. On Zork I that made Drop send `drop advert` for
   the leaflet. The layer now sends the word of the item's printed name that the
   game knows it by, in full (`drop leaflet`), and falls back to the first word.
2. **The card has no "touch outside"** (FR-013, A§3). It fills the band, so it
   closes on its close control and at any new turn.
3. **The verbs are sent in full** (`examine`, not the `examin` a version 3
   dictionary keeps). The game cuts what it reads the same way.
4. **An item's spoken label is its name**, not the phrase the strip speaks for it
   with its state and contents (User Story 4 scenario 1 said the latter). The
   strip as a whole still speaks the layer's summary.
5. **The introspection specification is version 0.6.0**, and the pack format has a
   version 4.

**T005, T013: the corpus.** On a wander of 120 moves through each story, taking
what could be taken, every item that could be tapped was peeked at every stop, and
the kept game and the snapshot were identical after each. Items shown, tappable,
described: Zork I 38, 38, 38; Zork II 139, 139, 139; Zork III 15, 14, 14; Mini-Zork
98, 98, 98; Adventure (both files) 296, 296, 296; Adventureland 224, 224, 224;
Anchorhead 468, 117, 117. Six stories showed no carried item on the wander
(sampler1, advent-zil, curses, balances, nine-oh-five, advent-crowther), so they
say nothing either way. Every peek that was made gave a description. In
Anchorhead three items in four have no naming words the interface can read and
cannot be tapped; why was not looked into.

**Timings, release, each test run alone.** Slowest card: Zork I 0.17 ms, Adventure
1.6 ms, over 1,393 and 660 cards along the walkthroughs, every one with a
description and the game unchanged. Adventure's slowest command with its snapshot:
6.1 ms with this feature and 6.1 ms without it (FR-020). Run six at once the same
test read 11 ms; the timing tests are to be run one at a time.

**Counts.** `cargo test --workspace`: 1,062 passed, none failed. `cargo clippy
--workspace --all-targets`: clean. App: 58 tests on iOS 27.2 and 58 on iOS 26.0,
one skipped on each (`testScreenshots`), no failure; run one after the other with
nothing else building.

**Where the tests are thin.** The token's drawing is not tested, only looked at in
the simulator by me. The card with no map (an unrecognised story) is built and not
tested. No game in the corpus gave a peek that failed, so the note for "no
description" is proven only on built stories.

**T020 (the owner's phone, SC-006)**: installed on 2026-11-14. The owner said
"Something is wrong", and then: "The problem is the card says that this is a look
ahead and the game has not been told. And then after I examined the egg and it told
me the egg was closed, the card still says that this is a look ahead and the game
has not been told. And I'm not sure what that means." The note was mine; it was on
every card whatever had been typed, and it did not say what it meant. Asked what
the card should say in its place, the owner chose "Only when it matters": no note
on a card that has a description. The note, its field in the card and its element
in the app are removed, tests first (the card's JSON test and the rule test were
changed and seen to fail). The trial is to be done again with the new build.

**The trial, again (2026-11-14).** With the note gone the owner said: "Yes, it
works, but two changes. The card is the same color ... is hard to see the edge. If
we could maybe darken the card or put a border. And then secondly, the drop button
is large, and I accidentally ... dropped the inventory item rather than closing the
window. And I'm wondering if we could look at the placement ... make it easier to
close the window, which is going to be the default action most of the time."

What was done: the card has an edge drawn in the map's hand and a shadow, and is set
in from the band's sides; a touch anywhere on it that is not an action closes it, as
does a touch on the band beside it and a touch on its item again; Drop is a small
outlined button, still 44 points tall to touch. `testAnItemsCard` was extended first
and seen to fail (a touch on the card's text did not close it). The checks on Drop's
size and distance from the close control passed when written: they do not measure
how large the button looks, which is what the owner meant. Two checks of 44 points
then failed by rounding (43.99999999999997) with the card at a fraction of a point,
and now allow for it. The full app suites have not been run since these changes.

**The trial, a third time (2026-11-14).** The owner: "yes. works on my phone. can we
[move] the close button to the bottom left with the action buttons to the right of
the close button." They had also asked whether the text scrolls; it did when tried
in the simulator, with nothing to say there was more, and they said yes to a mark
and a test.

What was done: the close control is the first thing in a row at the card's foot,
bottom left, and the actions follow it to its right; a description taller than its
room has the transcript's mark that there is more below (`card.more`), gone when its
end is in view. Tests first: the check of where the close control is, and
`testALongDescriptionScrolls`, were written and seen to fail (the control was at the
top right; there was no mark). The earlier check that Drop is far from the close
control is gone: they are now side by side by the owner's choice, and a touch that
misses either closes the card. The full app suites have not been run since.

**The trial, a fourth time (2026-11-14).** The owner: "Can you add some airspace
around the close button or some padding in between the close button and the first
action button? Just because it's small and my thumb is reaching all the way across
the phone, it still makes hitting the drop button easy." There are now 48 points of
clear space between the close control's touch area and the first action's. The
check was written first and seen to fail (4 points).

**The trial's end (2026-11-14).** With the gap in, the owner said: "Yes, it works".
Stage 1's last task is done. In the owner's words over the trial, what the card
needed that the specification had not seen: no note they had to puzzle over; an
edge they could see; closing as the easy thing; the close control under the thumb;
and room between it and anything that acts.

**Final counts, after every change of the trial (2026-11-14).** `cargo fmt --check`
and `cargo clippy --workspace --all-targets`: clean. `cargo test --workspace`: 1,062
passed, none failed. App: 59 tests on iOS 27.2 and 59 on iOS 26.0, one skipped on
each (`testScreenshots`), no failure; run one after the other with nothing else
building. These replace the counts above, which were taken before the trial.

