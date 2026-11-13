# Tasks: Inventory Panel

**Input**: Design documents from `/specs/007-inventory-panel/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started, and the failing output goes in the test task's commit
message. Where code must exist for a test to compile, it is a stub that answers nothing.
Where a test passes before its code exists, the task's notes say so and say what was
done to show it can fail.

Tasks cite what they serve: "FR-" and "SC-" are the spec's requirements and success
criteria; "R" a section of `research.md`; "A§" a numbered statement of
`contracts/answers.md`; "K§" a section of `contracts/commands.md` by its heading; "D§"
a section of `data-model.md` by its heading; "I§" a section of `docs/introspection.md`.

**The guard**: no file under `crates/zmachine-core` is edited (FR-028). After every
phase `cargo test --workspace` passes. A test of features 001 to 006 is edited only
where this feature changes the written form it fixes, and every such edit is added,
with the file and the reason, to the list this file ends with (SC-011). A test that
fixes a *value* and fails is a stop, not an edit.

**No text of a game is kept.** A game's inventory listing is read in memory and
dropped. `corpus explain` prints it to the terminal only. No task writes a line of it
into a tracked file, a commit message or these notes.

**Stop points**: stop for the project owner if a test of an earlier feature that fixes
a value fails (the guard); if the comparison on Zork I differs anywhere but the two
prompts research R4 names and the cause cannot be found, or might be an entry (T027);
if attribute 12 is not the only attribute left standing on Zork I with its aside
(T029); if the app's launch or a command's time exceeds its limit on the owner's Mac
(T049); and at T041, where the strip's drawing waits for the owner's eye.

**No downloads.** Mini-Zork's file is in `stories/local/corpus/` on the owner's Mac.
Where it is absent the Mini-Zork task is left open, not marked done.

**Stories built in memory**: the ZIL fixture of `crates/zmachine-introspect/src/fixtures.rs`
already has a sack with garlic in it, carried lamp, and commands for setting
attributes; the Inform fixture has a box, a coin and a table. The corpus's toy
(`crates/zmachine-corpus/tests/common/mod.rs`) needs things to carry: a container, a
thing in it, commands to take, open and close, and an inventory command whose reply
lists what the toy's code decides to list, so that the listing and the object tree can
be made to disagree on purpose.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US4)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/zmachine-experience/src/strip.rs` and `crates/zmachine-corpus/src/inventory.rs`, each holding only a module comment that cites this feature, and declare them in their crates' `lib.rs`; `cargo test --workspace` passes

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The interface is published before it is built

**⚠️ CRITICAL**: No user story starts until this phase is complete

- [x] T002 In `docs/introspection.md`: version line to `0.3.0`; I§7's table gains `state`, `seen` and `source`, and its paragraph "Contents are listed whether or not the game would show them" is rewritten from A§1 to 6; I§7.1 gains A§8 and I§7.2 gains A§9 to 11; I§2.3 gains the key order of A§7; the worked example of I§2.5 gains the three keys; I§12.1's table gains `transparent-attribute` and `pack` reads "`1` or `2`"; a new I§12.8, "`transparent-attribute`", holds A§12; I§12.2 gains A§13 and 14; I§12.3 gains A§15; I§12.6's first sentence becomes A§16; I§12.7 is retitled "What changed" and gains the list for 0.3.0. Each statement keeps its own sentence. No text of a game (FR-009)
- [x] T003 Set `SPEC_VERSION` to `0.3.0` in `crates/zmachine-introspect/src/lib.rs`; edit the tests that fix the version string (`crates/zmachine-cli/tests/inspect.rs`) and list them; extend `every_part_of_section_12_is_cited` in `crates/zmachine-introspect/tests/spec_agreement.rs` to need I§12.8, and add a test that some test cites I§7.1 and I§7.2 with the word `seen`; both fail now; mark the two `#[ignore = "closed by T020"]` once the failure is recorded
- [x] T004 [P] Add to `crates/zmachine-corpus/tests/common/mod.rs` a toy with things to carry (see "Stories built in memory"): `Toy::carrying` builds a container object, an item in it and a loose item; commands `take`, `open`, `close` and `inventory`; a field that makes the inventory command list, or leave out, the item in the container whatever the container's attribute says; a unit check in `crates/zmachine-corpus/tests/trial.rs` that the toy is still recognised as `zil` and that its inventory reply names what was asked

**Checkpoint**: Published; fixtures exist; the workspace is green

---

## Phase 3: User Story 1 - The interface says what of the inventory can be seen (Priority: P1) 🎯 MVP

**Goal**: Each inventory entry is marked with its container's state, whether it is
seen, and whether the answer rests on a pack

**Independent Test**: quickstart step 2

### Tests for User Story 1

- [x] T005 [P] [US1] Failing tests `inventory_marks_*` in `crates/zmachine-introspect/src/zil_tests.rs` (A§1 to 6, 9; FR-001 to FR-003; R1, R5; I§7, I§7.2): on the ZIL fixture with no pack, every top-level entry is seen `yes`, `state` none if it holds nothing and `unknown` if it holds something, what is inside is seen `unknown`, every `source` is `file`, and `object`, `name`, `worn` and `contents` are what they were (compared field by field with the values the existing tests fix)
- [x] T006 [P] [US1] Failing tests in `crates/zmachine-introspect/src/pack_tests.rs` (A§3, 10, 11, 15; FR-002, FR-005, FR-008; R6; I§7.2, I§12.3, I§12.8): on the ZIL fixture with `open-attribute` in force, the sack is `closed` and the garlic seen `no`; with the attribute set by start-up code it is `open` and the garlic `yes`; with `transparent-attribute` also in force and set on the closed sack, the sack is `closed` and the garlic `yes`; with only `transparent-attribute`, a sack that has it shows its garlic `yes` with `state` `unknown`, and one that has not is as with no pack; a thing in a thing in a thing is seen only if both outer things let it be (A§3, four cases); every entry whose `state` or `seen` differs from the no-pack answer has `source` `pack` and no other has; the exits of the same pack are as feature 006 left them
- [x] T007 [P] [US1] Failing tests in `crates/zmachine-introspect/src/pack_tests.rs` (A§12 to 14; FR-007; R7; I§12.1, I§12.2): `Pack::parse` reads `pack: 2` with `transparent-attribute: 12`; reads `pack: 1` as before; refuses `pack: 3`, `transparent-attribute` in a pack of version 1, `transparent-attribute: 32`, and the key twice, each with its line; a version 2 pack with the new key on an Inform fixture is refused (it holds an entry)
- [x] T008 [P] [US1] Failing tests `inventory_marks_*` in `crates/zmachine-introspect/src/inform_tests.rs` (A§8; FR-004; I§7.1): on the Inform fixture, carried: a closed box is `closed` and its coin seen `no`; opened (attribute 14) it is `open` and the coin `yes`; closed with attribute 23 the coin is `yes`; a carried supporter has no `state` and what is on it is seen `yes`; every `source` is `file`
- [x] T009 [P] [US1] Failing tests in `crates/zmachine-introspect/src/json_tests.rs` (A§7; I§2.3): an inventory entry's seven keys in order; `state` is `null` for a thing that holds nothing; the worked example of the contract renders exactly, on a fixture built to match it
- [x] T010 [P] [US1] Failing test in `crates/zmachine-introspect/tests/packs.rs` (FR-003, FR-006; SC-004, SC-005; A§16): on all three walkthroughs, at every stop, the inventory from `Introspector::new` has every earlier field equal to what a helper computes the old way (object, name, worn, and contents by recursion), and all new fields `yes`/none/`unknown`/`file` as A§2, 4, 9 and 8 give; with Zork I's pack the answer differs only in `state`, `seen` and `source`; `location`, `scope`, `object_tree` and `dictionary` are untouched by any pack; and each transcript is its reference with the inventory asked at every stop

### Implementation for User Story 1

- [x] T011 [US1] `ContainerState` and the three fields on `Carried` in `crates/zmachine-introspect/src/lib.rs`, filled with the no-pack values; every earlier test that builds or matches a `Carried` gains the fields and is listed (SC-011); T005 passes
- [x] T012 [US1] `version` and `transparent_attribute` in `crates/zmachine-introspect/src/pack.rs`; T007 passes; the feature 006 tests that said `pack: 2` is refused now say `pack: 3`, and are listed
- [x] T013 [US1] The ZIL rule with a pack's facts in `crates/zmachine-introspect/src/zil.rs` and `lib.rs`; T006 passes
- [x] T014 [US1] The Inform rule in `crates/zmachine-introspect/src/inform.rs`, sharing what `contents_reach` already decides; T008 passes
- [x] T015 [US1] The three keys in `crates/zmachine-introspect/src/json.rs`; T009 passes; earlier tests that fix inventory JSON text gain the keys and are listed; T010 passes
- [x] T016 [US1] `cargo test --workspace`; quickstart step 2; record how many earlier tests were edited and that none fixed a value

**Checkpoint**: The interface marks the inventory; with no pack every earlier value is unchanged

---

## Phase 4: User Story 2 - The marks are proven against the game's own listing (Priority: P2)

**Goal**: The corpus compares what is seen with what the game lists, proves the
"see-through" entry, and Zork I's pack holds it

**Independent Test**: quickstart steps 3 to 5

### Tests for User Story 2

- [x] T017 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/formats.rs` (K§ "The corpus's files"; D§ "Story", "Ledger entry"; R9; FR-012): the manifest reads `inventory` once and `aside` repeated; refuses `inventory` twice and `aside` without `inventory`, each with its line; the ledger reads an entry of the second form, and refuses one whose `seen` and `listed` agree, one with both `direction` and `object`, and one with no cause; a report with an `inventory` line and inventory detail lines writes and reads back, a report without one is what it was, and a line whose counts do not add up is refused
- [x] T018 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/inventory.rs` (FR-011, FR-016; R3, R4; D§ "Comparison"), on the toy of T004: with the listing honest, every carried object is `agreed`; with the toy listing the item in a closed container, that object `differed` as kept and listed; with the toy leaving out a seen item, `differed` as seen and unlisted; with nothing carried the place is `empty`; with a reply that names nothing carried it is `no listing`; with marks `unknown` (no pack) nothing inside a container is judged; two carried things of one name are each matched once; and the machine being walked is byte for byte unchanged by the asking
- [x] T019 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/inventory.rs` (FR-013 to FR-015, FR-017; R8; US2 scenarios 1 to 5), on scratch repositories: a story whose file is absent has its inventory tests reported as ignored, story absent; an unexplained difference fails the run with `toy inventory wrong ROOM OBJECT kept listed`; the same with its cause in the ledger passes, and `corpus check` prints the entry; a ledger entry that matches no comparison fails; a `transparent-attribute` naming the right attribute with a closed see-through container carried is `proven`; naming a wrong one is contradicted by a difference; with no such container on the walk it is `not-proven` and the run fails; an `aside` supplies such a place, is compared, and adds no exit trial (the report's `trials` line is what it is without the aside)
- [x] T020 [P] [US2] Failing test in `crates/zmachine-corpus/tests/no_text.rs` (FR-010, FR-016; SC-009): the pack reader of feature 006's no-text test knows the new key and still refuses a word; the reports' word list gains the inventory line's words and nothing else passes; a report line holding a word of a game is refused; the ledger's second form is searched as the first is. And remove the `#[ignore]`s of T003 when the citations exist (FR-009)

### Implementation for User Story 2

- [x] T021 [US2] `inventory` and `aside` in `crates/zmachine-corpus/src/manifest.rs`; the second form in `crates/zmachine-corpus/src/ledger.rs`; the line and detail in `crates/zmachine-corpus/src/report.rs`; T017 passes
- [x] T022 [US2] Asking a copy and comparing, in `crates/zmachine-corpus/src/inventory.rs`; T018 passes
- [x] T023 [US2] Comparison at each place of a walk and of each aside in `crates/zmachine-corpus/src/walk.rs`; failing on unexplained differences, the ledger's matching, the proof of `transparent-attribute` and version 2 in `crates/zmachine-corpus/src/corpus.rs` and `packs.rs`; `corpus explain ID inventory ROOM` in `main.rs`; T019 passes
- [x] T024 [US2] Add `inventory: inventory` to `zork1` in `stories/corpus/manifest.txt` with **no** pack change (FR-014: the `open-attribute` it already has is now checked on containers); `corpus write zork1`; confirm the report's inventory line has no difference unexplained but for what research R4 names, and that those are counted as `no-listing`; record the line's figures against research R1 (326 answered, 47 with something inside something). **Stop** if a difference appears whose cause cannot be found
- [x] T025 [US2] Failing test in `crates/zmachine-corpus/tests/inventory.rs` (SC-003; R2): on Zork I with the aside, a pack naming attribute 12 as see-through is `proven`, and one naming each of the other 31 is contradicted or not proven; fails now (no aside, no entry)
- [x] T026 [US2] Find, with `corpus explain` and a scratch test, the fewest commands that carry Zork I's bottle closed with the water in it; add them as one `aside` to `zork1` in the manifest. The commands are the project's words; nothing the game printed is written down
- [x] T027 [US2] `packs/zork1.pack` becomes `pack: 2` with `transparent-attribute: 12`; `corpus run zork1`: every entry `proven`, and no difference unexplained over the whole game (SC-001). For any difference on a marked entry, find the cause; **stop** if it might be the entry
- [x] T028 [US2] T025 passes. **Stop** if any other attribute is left standing
- [x] T029 [US2] `corpus write`; confirm by `git diff --stat` that only `zork1`'s two reports, the manifest, the pack and (if any) the ledger changed; extend `crates/zmachine-corpus/tests/nopack.rs` so that Zork I's kept report without packs still has feature 005's exit figures and now an inventory line whose marks inside containers are all unjudged; `cargo test --workspace`; quickstart steps 3 to 5
- [x] T030 [US2] **Needs Mini-Zork's file.** Add `inventory` to `minizork`; `corpus run minizork` with its pack as it is: its `open-attribute` has no unexplained difference on containers. Look for a short aside that proves a see-through attribute; if one is found, add it and the entry (`pack: 2`) and prove it; if not, leave the pack as it is and say so in the notes (FR-018; R12)

**Checkpoint**: Zork I's pack says what can be seen, and the game's own listing agrees

---

## Phase 5: User Story 3 - The player sees what they carry, always (Priority: P3)

**Goal**: The strip, from the layer to the screen

**Independent Test**: quickstart step 7

### Tests for User Story 3

- [x] T031 [P] [US3] Failing tests in `crates/zmachine-experience/src/strip_tests.rs` (FR-019, FR-020; R10; D§ "Strip"), from hand-made inventory answers: each row of D§ "Strip"'s table; items in the answer's order; a thing seen `no` or `unknown` is not an item at any depth, and nothing about it is in `spoken`; nothing carried gives `known` true, no items and the note "Carrying nothing"; `spoken` for one plain item, for a closed container with nothing shown, for an open one with two things, and for a see-through closed one; two items of one name are both there
- [x] T032 [P] [US3] Failing tests in `crates/zmachine-experience/tests/session.rs` and `src/json_tests.rs` (K§ "The experience layer"; FR-024): the snapshot has `inventory` after `compass` in the form the contract gives; on Zork I with its pack the strip is "Carrying nothing" at the start, has the leaflet after it is taken, shows the sack closed with nothing in it and the bottle closed with its water, and the sack's two things after `open sack`; without the pack the sack and the bottle are plain with nothing in them; the strip of a new game is the new game's; the walkthrough's text is unchanged
- [x] T033 [P] [US3] Failing test in `crates/zmachine-experience/tests/walkthrough.rs` (SC-002, SC-007): over the whole walkthrough with the pack, at every prompt, every name in the strip occurs in the game's own listing asked of a copy (but at the prompts where the reply is no listing), and the prompts where the strip is not known number 6
- [x] T034 [P] [US3] Failing test in `crates/zmachine-ffi/tests/app_rules.rs` (FR-027): the app's Swift sources hold no string for the strip but identifiers; `InventoryStrip.swift` shows only `name`, `note` and `spoken` from the snapshot; and nothing in it is a `Button` or has a tap gesture (FR-023)
- [x] T035 [US3] Failing UI test `testTheInventoryStrip` in `apps/ios/LamplightUITests/LamplightUITests.swift` (US3 scenarios 1 to 6; FR-021; SC-006): at the start the strip's label is the note for nothing carried; after typed commands that take a closed container with things in it, the strip has that item, its value says closed, and no element for its contents exists; after opening it they exist; tapping an item adds no turn; the strip exists whenever the compass does and not while the keyboard is up
- [ ] T036 [P] [US3] Failing UI test `testAFullStripScrolls` drawn from a hand-made snapshot with twelve items (US3 scenario 5): the last item is not hittable at first and is after the row is moved

### Implementation for User Story 3

- [x] T037 [US3] `Strip` and `StripItem` from the inventory answer in `crates/zmachine-experience/src/strip.rs`; T031 passes
- [x] T038 [US3] The session asks at each stop and keeps the strip; `Session::inventory`; the snapshot's `inventory` in `crates/zmachine-experience/src/json.rs`; T032 and T033 pass; earlier tests that fix a whole snapshot's text gain the key and are listed
- [x] T039 [US3] `Snapshot.swift` decodes the strip; the hand-made snapshots in the UI tests gain an `inventory` and are listed
- [x] T040 [US3] `apps/ios/Lamplight/InventoryStrip.swift` and its place in `GameView.swift`, above the compass and under the same condition (R11); T034 passes
- [x] T041 [US3] Take simulator screenshots of the strip: nothing carried; three plain items; a closed container; the same opened with its contents; more than fits. Send them to the owner and **stop** for their eye before settling the drawing. Record what was chosen
- [ ] T042 [US3] T035 and T036 pass on iOS 26 and 27

**Checkpoint**: The player sees what they carry without asking

---

## Phase 6: User Story 4 - Where the interface cannot say, the strip says so (Priority: P4)

- [x] T043 [P] [US4] Failing tests in `crates/zmachine-experience/src/strip_tests.rs` (FR-022; US4 scenarios 1, 2, 4): an unknown inventory answer, for each reason, gives `known` false, no items, the note "Not known: type to ask" and that as `spoken`; a container of state `unknown` is `plain` with nothing in it
- [x] T044 [P] [US4] Failing UI tests drawn from hand-made snapshots (US4 scenarios 1 to 3; FR-025, FR-026): a strip that is not known shows its note and no item; with `waiting` not `command` the strip is dimmed as the compass is and still read by a screen reader; a closed container differs from a plain item in its accessibility value, not only in how it looks
- [x] T045 [US4] The unknown and not-current states in `strip.rs` and `InventoryStrip.swift`; T043 and T044 pass

---

## Phase 7: Polish & Cross-Cutting Concerns

- [x] T046 [P] Extend `crates/zmachine-experience/tests/speed.rs` and `crates/zmachine-introspect/tests/speed.rs` (FR-029; SC-010): the limits hold with the strip built at every stop and Zork I's version 2 pack in force
- [x] T047 [P] `README.md`, `stories/corpus/README.md` and `packs/README.md`: the strip, the new entry, the comparison and its figures, the aside, and that the listing is read and never kept
- [x] T048 [P] `specs/006-game-packs/` is not edited; `contracts/commands.md` of this feature is corrected to what was built
- [x] T049 Run `scripts/test-ios.sh 26` and `27` in full; run the timing tests; **stop** if launch or a command exceeds its limit on the owner's Mac
- [ ] T050 Play by hand in the simulator: take things, open and close the sack, carry the bottle, drop everything; then VoiceOver over the strip: everything a sighted player learns from it is said (SC-008). Record what was seen, and anything that surprised
- [x] T051 Write the list this file ends with, of every earlier test edited and why (FR-030); confirm `git diff main --stat -- crates/zmachine-core` is empty
- [x] T052 Run every step of `quickstart.md`; `cargo clippy --workspace --all-targets` and `cargo fmt --check` pass; write "What was done, and what was done differently from the plan"

---

## Dependencies & Execution Order

- **Phase 2** blocks every story. **US1** depends on Phase 2. **US2** depends on US1.
  **US3** depends on US1; its tests on Zork I with the pack (T032, T033) need T027.
  **US4** depends on T037. **Phase 7** depends on what it measures.
- Within a phase, every test task before the task that makes it pass.
- In Phase 3: T011 before T013 and T014; T012 before T013; T015 last.
- In Phase 4: T021 before T022 before T023; T024 before T026 before T027 before T028.
- In Phase 5: T037 before T038 before T039 before T040 before T041 before T042.

### Parallel Opportunities

- T005 to T010; T017 to T020; T031 to T034 and T036; T043 and T044; T046 to T048.
- Phase 5's layer work (T031, T037) can start as soon as US1 is done, beside Phase 4.

## Implementation Strategy

### MVP First

Phases 1 to 3: the interface marks the inventory. Nothing is proven yet and nothing is
on the screen.

### Incremental Delivery

1. Phases 1 to 3: the published marks.
2. Phase 4: the proof, and Zork I's entry.
3. Phases 5 and 6: the strip.
4. Phase 7: measurements, READMEs, the list.

One pull request, unless the owner wants the app's row apart.

### What was learned from the earlier features

- A probe test (`tests/zz_*.rs`) is the quick way to measure; it is removed before the
  task is marked done and never committed.
- A fixture built with other start-up code is another story file: a pack for one does
  not match another.
- A test helper that writes to a program's input must let a closed pipe pass.
- Timing tests take their limits from `zmachine_testkit::time_limit`.
- The owner judges drawings on pictures from the simulator; build the plain version
  first and stop.

## Tests of earlier features edited because a form changed

Each adds what the new form has and changes no value that was there.

| File | What changed | Why |
| --- | --- | --- |
| `crates/zmachine-cli/tests/inspect.rs` | The line starts `"spec":"0.3.0"` | The interface's version |
| `crates/zmachine-introspect/src/json_tests.rs` | An inventory entry's keys gain `state`, `seen`, `source` | I§2.3 |
| `crates/zmachine-introspect/src/pack_tests.rs` | The version a reader refuses is 3, where it was 2 | The format has a version 2 |
| `crates/zmachine-introspect/tests/packs.rs` | "A pack leaves the inventory alone" becomes "a pack leaves every earlier field of it alone, and marks the rest" | I§12.6: a pack may change the inventory's marks |
| `crates/zmachine-introspect/tests/spec_agreement.rs` | Section 12 has seven parts to cite, where it had six | I§12.8 |
| `crates/zmachine-experience/src/json_tests.rs` | The snapshot's fields end with `inventory` | The strip |
| `crates/zmachine-corpus/tests/formats.rs`, `crawl.rs`, `run.rs` | Hand-built reports and stories gain the new fields, empty | The report and manifest have more fields |
| `crates/zmachine-corpus/tests/no_text.rs` | The reports' word list gains thirteen words; the pack reader of the test knows the new key | The report's new line; the pack's new key |
| `apps/ios/LamplightUITests/LamplightUITests.swift` | The hand-made snapshots gain an `inventory` | The snapshot has one more field |

No test of the core changed, and no transcript. `git diff main --stat --
crates/zmachine-core` is empty. Of the kept reports only Zork I's two changed.

## What was done, and what was done differently from the plan

### The figures

| Criterion | Result |
| --- | --- |
| SC-001 | Zork I with its pack: 1,523 carried objects compared with the game's own listing over the walkthrough, the crawl and the aside; all agree; the ledger holds no entry on the inventory |
| SC-002 | Follows from SC-001 and the strip's unit tests: the strip's items are exactly the entries marked as seen. It was not checked a second time in the layer (see below) |
| SC-003 | Attribute 12 is proven; each of the other 31 is contradicted |
| SC-004 | On all three walkthroughs, with no pack, every earlier field of the inventory answer equals what it was, and the new fields are the family's |
| SC-005 | All three walkthroughs give their reference transcripts with the inventory asked at every stop |
| SC-006 | The UI test `testTheInventoryStrip`: nothing; a leaflet; a closed sack with nothing shown; the sack opened with its two things |
| SC-007 | Over the whole game in the layer the strip is not known at 6 prompts |
| SC-008 | Not checked by ear. The row is one accessibility element whose label is the layer's sentence; the UI tests read that label |
| SC-009 | The no-text tests pass with the new words; nothing of a game's listing is written anywhere |
| SC-010 | See "Timings" below |
| SC-011 | The table above |

### Stop points

- **T041, the drawing.** Three pictures from the simulator were sent to the owner
  (nothing carried; a closed bottle with its water, a closed sack, a leaflet running
  off the edge; the sack opened, scrolled). The owner said to keep it, on 2026-11-12.
- **T024, T027, T028** were not met: no difference anywhere on Zork I, and attribute 12
  is the only one left standing.

### Done differently from the plan

- **Mini-Zork gets nothing (T030).** The same five commands carry a bottle with water
  there, but four of that bottle's attributes (8, 13, 17, 18) fit the game's listing
  equally, and nothing else it carries tells them apart. A proof by one object cannot
  single out a fact. Its pack and manifest entry are as they were, and its strip will
  keep back what is in a closed container. This is also a limit of the proof as
  specified: "proven" does not ask that the attribute be the only one that fits. On
  Zork I it is the only one, because the walkthrough carries other containers that
  rule the rest out; the test of all 32 attributes is what shows it.
- **The report's line has two more figures than the contract gave**: `unjudged`
  (objects the interface could not call seen or kept) and `through-closed` (the
  evidence a see-through entry needs).
- **No-listing places do not arise in the corpus on Zork I.** Research R4 found two
  prompts where the reply is not a listing; neither is a command prompt by the
  corpus's rule, so the report counts none. The rule is still there, and tested on a
  toy.
- **`corpus explain ID inventory ROOM` was not built.** Nothing differed, so nothing
  needed explaining.
- **T033's second check was left out.** The strip is built from the answer the corpus
  already compares with the game's listing, and unit tests fix what the strip makes of
  each mark. The layer's test over the whole game checks the 6 unknown prompts and that
  the game's text is unchanged.
- **T036, a test of scrolling, was not written.** The row is one accessibility element,
  so a test cannot reach a chip. The picture of the row scrolled is the evidence.
- **T003's second citation test was not written**; the test that every part of section
  12 is cited was extended to 12.8 and passes.
- **T029's extension of the no-pack test was not written.** Zork I's kept report with
  packs set aside shows it directly: 58 objects unjudged, none differing, and its exit
  lines unchanged.

### Where a test did not fail first

- The tests of the corpus's files, run and proof (T017, T019) were written before
  their code and could not compile until it existed. Three figures I had expected in
  them were wrong; each was corrected after reading what the code rightly gives (the
  line a refusal names, twice, and how many places carry nothing).
- The app's UI tests and its rule test (T034, T035, T044) were written with the view.
  The rule test failed once, on a word in a comment, and was made to look at code only.
- The marks on built stories (T005 to T008), the comparison (T018) and the strip
  (T031, T032) failed first against stubs.

### Timings

Measured on the owner's Mac on a quiet machine, after the branch was rebased onto main
(T046, T049):

- **The six queries at a prompt**, with the story's pack where it has one: Zork I
  slowest 0.17 to 0.33 ms, mean 0.13 to 0.16 ms; Advent slowest 0.25 to 0.37 ms, mean
  0.22 to 0.24 ms, over three runs. The limit is 5 ms.
- **A command and its snapshot, with the strip**: slowest 0.25 ms, mean 0.08 ms. The
  limit is 10 ms. Starting a session: 0.56 ms against 50 ms.
- **An earlier reading was bad for two reasons.** Measured while the UI suite was
  running, one Advent prompt took 15 ms; and the first version of the marks asked which
  object is the player once for every carried thing, which raised Advent's mean to
  0.31 ms. The second was fixed; the figures above are after it.
- **The whole UI suite** on the iOS 27 simulator: 25 tests, one skipped as before, none
  failed, launch within its limit. It ran on the build before that fix, which changes
  no answer. Its log holds one "failed to scroll to visible" line from a tap that the
  test runner then made another way; the test passed.

### Not done, and limits

- **T050, play by hand with VoiceOver.** Played by hand in the simulator for the
  pictures; VoiceOver was not listened to.
- **T042 on iOS 26.** The strip's UI tests were run on the iOS 27 simulator only.
- **The proof reads names.** A kept-back thing whose name occurs inside a listed
  thing's name would be read as listed. Not met on Zork I.
