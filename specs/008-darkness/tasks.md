# Tasks: Darkness

**Input**: Design documents from `/specs/008-darkness/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started. Where code must exist for a test to compile, it is a stub
that answers nothing. Where a test passes before its code exists, or is written with
its code, the task's notes say so and say what was done to show it can fail.

Tasks cite what they serve: "FR-" and "SC-" are the spec's; "R" a section of
`research.md`; "A§" a numbered statement of `contracts/answers.md`; "K§" a section of
`contracts/commands.md`; "D§" a section of `data-model.md`; "I§" a section of
`docs/introspection.md`.

**The guard**: no file under `crates/zmachine-core` is edited (FR-023). After every
phase `cargo test --workspace` passes. A test of features 001 to 007 is edited only
where this feature changes the written form it fixes, and every such edit is listed at
the end of this file (FR-025). A test that fixes a *value* and fails is a stop.

**No text of a game is kept.** A game's reply to the look command is read in memory and
dropped. No task writes a line of it into a tracked file, a commit message or these
notes.

**Stop points**: stop for the project owner if a test of an earlier feature that fixes
a value fails; if light at a place differs from the game on Zork I and the cause cannot
be found, or might be the entry (T027); if more attributes than 19 are left standing on
Zork I (T029); if removing the darkness entries leaves a wrong report at a dark place
that is not explained by "in the dark" (T030); if a timing limit is exceeded on the
owner's Mac (T046); and at T040, where the note and the mark wait for the owner's eye.

**No downloads.** The Mini-Zork task needs its file in `stories/local/corpus/`; where
it is absent the task is left open.

**Stories built in memory**: the ZIL fixture of `crates/zmachine-introspect/src/fixtures.rs`
has a kitchen, a hall and a cellar, a lamp, a sack and commands that set attributes;
tests give rooms and the lamp a "lit" attribute as they need. The corpus's toy needs a
room that is dark unless a lamp with the attribute is carried, a `look` command that
names the room only when the toy's own rule says it is lit, and a way to make that
rule disagree with the attribute on purpose.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create `crates/zmachine-experience/src/light.rs` and `crates/zmachine-corpus/src/light.rs`, each holding a module comment that cites this feature, and declare them; `cargo test --workspace` passes

---

## Phase 2: Foundational

**⚠️ CRITICAL**: No user story starts until this phase is complete

- [x] T002 In `docs/introspection.md`: version line to `0.4.0`; I§6's table gains `source`, and I§6.2 gains A§5 to 7 by pointing to a new I§12.9; I§9's table gains `source` and I§9.2's "Light is `unknown`" becomes A§8; I§8's table gains `dark_ahead`, with A§10, 12 and 15; I§2.3 gains the key orders of A§14 and the worked examples of I§2.5 gain the new keys; I§12.1's table gains `lit-attribute` and `pack` reads "`1`, `2` or `3`", with A§2 in the paragraph on versions; I§12.6's first sentence becomes A§3; a new I§12.9, "`lit-attribute`", holds A§1 and 4 to 9; a new I§12.10, "Dark ahead", holds A§11 and 13; I§12.7 gains the list for 0.4.0. No text of a game (FR-008)
- [x] T003 Set `SPEC_VERSION` to `0.4.0`; edit and list the tests that fix the version string; extend `every_part_of_section_12_is_cited` to need I§12.9 and I§12.10 (it fails until Phase 3 and 4 cite them; mark it `#[ignore = "closed by T017"]` once the failure is recorded)
- [ ] T004 [P] Extend the corpus's toy in `crates/zmachine-corpus/tests/common/mod.rs`: `Toy::lit_rooms` gives chosen rooms a `LIT_ATTR`; a lamp object the player may `take`, with commands `on` and `off` that set and clear `LIT_ATTR` on it; a `look` command that prints the room's name when the toy's rule says lit (the room has the attribute, or the lamp has it and is carried or in the room) and something else when not; a field that makes `look` lie (always name the room, or never). A check in `tests/trial.rs` that the toy is still `zil`

---

## Phase 3: User Story 1 - The interface says whether it is dark (Priority: P1) 🎯 MVP

### Tests

- [x] T005 [P] [US1] Failing tests in `crates/zmachine-introspect/src/pack_tests.rs` (A§1, 2; FR-001; R5; I§12.1, I§12.9): `Pack::parse` reads `pack: 3` with `lit-attribute: 19`; reads versions 1 and 2 as before; refuses `pack: 4`, `lit-attribute` in a pack of version 2, `lit-attribute: 32`, and the key twice, each with its line; the key on an Inform fixture is refused as an entry
- [x] T006 [P] [US1] Failing tests `light_*` in `crates/zmachine-introspect/src/pack_tests.rs` (A§4, 5, 8, 9; FR-002, FR-003, FR-005; R1; I§6.2, I§9.2, I§12.9), on the ZIL fixture with `lit-attribute` and `open-attribute` in force: a room with the attribute is `lit` `yes`; without it and with no light, `no`; with the lamp carried and having the attribute, `yes`; the lamp lying in the room, `yes`; the lamp in the closed sack, `no`; in the open sack, `yes`; in a closed sack that is see-through (with `transparent-attribute`), `yes`; the lamp having the attribute in another room, `no`; `scope`'s `light` is `lit` or `dark` to match, and its objects are as before; `location` and `scope` have `source` `pack` in every one of these cases
- [x] T007 [P] [US1] Failing tests `light_unknown_*` in `crates/zmachine-introspect/src/pack_tests.rs` (A§6, 7; R2): with the player ambiguous (two objects tie, as `zil_tests` already arranges) and the room without the attribute, `lit` is `unknown` and `source` `file`; the same with the room having the attribute is `yes`; the same with a lamp having the attribute lying loose in the room is `yes`; with `lit-attribute` and no `open-attribute`, a lit lamp inside the sack in a dark room is `unknown`, and loose in the room is `yes`
- [x] T008 [P] [US1] Failing tests in `crates/zmachine-introspect/src/json_tests.rs` (A§14; I§2.3): `location`'s keys end `lit`, `source`; `scope`'s begin `light`, `source`
- [x] T009 [P] [US1] Failing test in `crates/zmachine-introspect/tests/packs.rs` (FR-006, FR-007; SC-005, SC-006): on all three walkthroughs, with no pack, `location` and `scope` equal what they were in every earlier field and have `source` `file`; Advent's `lit` is never `unknown` and Zork I's always is; with a pack for Zork I written in the test (`lit-attribute: 19` with the open and see-through entries) `lit` is `yes` or `no` at every stop where the player can be told; `object_tree` and `dictionary` are untouched; each transcript is its reference

### Implementation

- [x] T010 [US1] `version` 3 and `lit_attribute` in `crates/zmachine-introspect/src/pack.rs`; T005 passes; earlier tests that said version 3 is refused now say 4, and are listed
- [x] T011 [US1] `source` on `Location` and `Scope` in `crates/zmachine-introspect/src/lib.rs`, `file` everywhere; the light rule in `crates/zmachine-introspect/src/zil.rs`, used by `location` and `scope`; T006 and T007 pass
- [x] T012 [US1] The keys in `crates/zmachine-introspect/src/json.rs`; T008 passes; earlier tests that fix location or scope JSON gain the key and are listed; T009 passes

**Checkpoint**: The interface says lit or dark

---

## Phase 4: User Story 3 - A way into darkness is marked (Priority: P3, built before P2 because the proof checks both)

### Tests

- [x] T013 [P] [US3] Failing tests `dark_ahead_*` in `crates/zmachine-introspect/src/pack_tests.rs` (A§10 to 13, 15; FR-004; R3; I§8, I§12.10), on the ZIL fixture: a way to a room without the attribute, with no light there and none carried, is `yes`; with a lit lamp carried, `no`; with a lit lamp lying in that room, `no`; to a room with the attribute, `no`; a lit lamp carried inside a closed sack does not count; a way decided by a routine is `unknown`; a door, open or closed, is `yes` or `no` like a fixed exit; with the player ambiguous and the way's room dark of itself, `unknown`; every exit whose `dark_ahead` is `yes` or `no` has `source` `pack`, and a conditional exit keeps `file`; with no `lit-attribute` every `dark_ahead` is `unknown` and every source is what feature 006 gave
- [x] T014 [P] [US3] Failing test in `crates/zmachine-introspect/src/json_tests.rs` (A§14): an exit's nine keys in order
- [x] T015 [P] [US3] Failing test in `crates/zmachine-corpus/tests/verdict.rs` (R4): with a pack that has `lit-attribute`, a fixed exit whose source is `pack` is still reported `fixed R`; a conditional exit on an untrusted variable is still `distrusted`; a plain conditional exit is still `conditional`

### Implementation

- [x] T016 [US3] `dark_ahead` on `Exit`; the rule for a way's room in `crates/zmachine-introspect/src/zil.rs` and `lib.rs`; the key in `json.rs`; T013, T014 and T015 pass; earlier tests that build or fix an exit gain the field and are listed
- [x] T017 [US3] Remove the `#[ignore]` of T003; `cargo test --workspace`; quickstart step 2

**Checkpoint**: The interface forecasts darkness

---

## Phase 5: User Story 2 - The answer is proven against the game (Priority: P2)

### Tests

- [x] T018 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/formats.rs` (K§ "The corpus"; D§; R7; FR-010): the manifest reads `look`, once; the ledger reads the third and fourth forms and refuses ones whose two words agree, that mix forms, or that have no cause, each with its line; a report with the two new lines and their detail writes and reads back; counts that do not add up are refused; a report without `look` is what it was
- [x] T019 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/light.rs` (FR-009; R6; D§ "Light comparison"), on the toy of T004: asking a copy to look gives "described" in a lit room and "not" in a dark one; with the lamp on and carried, described; with the toy lying, the comparison differs each way; where the interface says `unknown` nothing is judged; the machine being walked is unchanged
- [x] T020 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/light.rs` (FR-012; R7): a trial that walks a way marked dark ahead and arrives where the game does not describe the room agrees; an unmarked way to a lit room agrees; with the toy lying each differs, and the difference names the room and the direction
- [x] T021 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/verdict.rs` and `tests/light.rs` (FR-013; R7; US2 scenario 4): at a place reported dark, a trial that would be `wrong` is `in-the-dark` and needs no ledger entry; a confirmed one stays confirmed; at a place reported lit or unknown nothing changes; the report counts them
- [x] T022 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/light.rs` (FR-011, FR-014, FR-016; R9; US2 scenarios 1 to 3, 6, 7), on scratch repositories: the right attribute, with a room seen both dark and lit, is `proven`; a wrong attribute is `contradicted`, and the run names a place in numbers; a walk that is never dark is `not-proven`; a difference with its cause in the ledger passes and the check prints it; an entry that matches nothing fails; an absent story is ignored
- [x] T023 [P] [US2] Failing test in `crates/zmachine-corpus/tests/no_text.rs` (FR-015; SC-009): the reports' word list gains the two lines' words; the test's pack reader knows `lit-attribute`

### Implementation

- [x] T024 [US2] `look` in `manifest.rs`; the two forms in `ledger.rs`; the lines and detail in `report.rs`; `in-the-dark` in `verdict.rs`; T018, T021 (the verdict part) and T023 pass
- [x] T025 [US2] Asking and comparing in `crates/zmachine-corpus/src/light.rs`; comparison at each place, at each aside, and on each trial's copy in `walk.rs`; T019 and T020 pass
- [x] T026 [US2] Failing on unexplained differences, ledger matching, proof of `lit-attribute` and version 3 in `corpus.rs` and `packs.rs`; T021 and T022 pass
- [x] T027 [US2] Add `look: look` to `zork1` in the manifest with **no** pack change; `corpus write zork1`: with no entry every light is unjudged. Then `packs/zork1.pack` becomes `pack: 3` with `lit-attribute: 19`; `corpus run zork1`. For each difference about light or dark ahead, find the cause on a copy; enter it in the ledger if it is plainly not the entry (the character who takes the player's light is expected, research R3); **stop** if a cause cannot be found or might be the entry. Record the two lines' figures against research R1 and R3, and against SC-001 (light agrees at every place where the player can be told) and SC-004 (every walked way agrees, or has a cause)
- [x] T028 [US2] Failing test, then pass, in `crates/zmachine-corpus/tests/light.rs` (SC-003): on Zork I, attribute 19 is `proven` and each of the other 31 is `contradicted` or `not-proven`
- [x] T029 [US2] **Stop** if any attribute but 19 is `proven` in T028
- [x] T030 [US2] Remove from `stories/corpus/ledger.txt` Zork I's entries whose cause is darkness (FR-017; SC-002); `corpus run zork1` passes with them gone. **Stop** if a wrong report is left at a dark place that "in the dark" does not cover. `corpus write`; confirm only Zork I's two reports, the manifest, the pack and the ledger changed; record the count of entries removed and the report's `in-the-dark` figure
- [x] T031 [US2] **Needs Mini-Zork's file.** Add `look` to `minizork`; try each attribute as `lit-attribute` (a scratch test); if exactly one is left standing, add it (`pack: 3`), prove it and remove its darkness entries; otherwise leave the pack and the manifest entry as they were and say so (R11)

**Checkpoint**: Zork I's pack says what is lit, and the game agrees

---

## Phase 6: User Story 4 - The player is warned (Priority: P4)

### Tests

- [x] T032 [P] [US4] Failing tests in `crates/zmachine-experience/src/light_tests.rs` and `compass_tests.rs` (FR-018, FR-021; R10; D§), from hand-made answers: `lit` `no` gives `dark` true and the note "It is dark"; `yes` gives false and no note; `unknown` gives none and no note; an exit with `dark_ahead` `yes` gives a place with `dark_ahead` true whose `spoken` ends ", into darkness" and whose state, command and `leads_to` are what they would be without it; `no` and `unknown` give false; the same for an extra; in the dark the compass's places are those of the same exits in the light (FR-019)
- [x] T033 [P] [US4] Failing tests in `crates/zmachine-experience/tests/session.rs` and `src/json_tests.rs` (K§ "The experience layer"): the snapshot has `light` after `inventory`; a place has `dark_ahead` after `leads_to`; on Zork I with its pack, in the kitchen with no lamp the way up is dark ahead and nothing else is; after going up `light.dark` is true; with the lamp fetched and switched on, no way is dark ahead and the attic is not dark; without the pack `light.dark` is null and no way is marked; the walkthrough's text is unchanged
- [x] T034 [P] [US4] Failing test in `crates/zmachine-ffi/tests/app_rules.rs` (FR-022): the app's sources hold no string for the note or the mark but identifiers; the note's text is the snapshot's
- [x] T035 [US4] Failing UI test `testDarknessIsWarnedOf` in `apps/ios/LamplightUITests/LamplightUITests.swift` (US4 scenarios 1 to 4; FR-020; SC-007): in the kitchen the way up's label ends ", into darkness"; after going up the element `light.note` reads "It is dark" and the compass's places are enabled as the room's are; back down and west, with the lamp taken and switched on by typed commands, no place's label ends ", into darkness"; up again, `light.note` does not exist
- [x] T036 [P] [US4] Failing UI test drawn from hand-made snapshots (US4 scenario 5; FR-021): with `light.dark` null there is no note and no mark

### Implementation

- [x] T037 [US4] The snapshot's light in `crates/zmachine-experience/src/light.rs`; `dark_ahead` on a place and an extra in `compass.rs`; `lib.rs` and `json.rs`; T032 and T033 pass; earlier tests that fix a whole place or snapshot gain the keys and are listed
- [x] T038 [US4] `Snapshot.swift` decodes them; the hand-made snapshots in the UI tests gain them and are listed
- [x] T039 [US4] The note above the compass in `GameView.swift`; the badge in `CompassView.swift`; T034 passes
- [x] T040 [US4] Take simulator screenshots: a lit room beside a dark one, with the marked way; the dark room, with the note; the same way with the lamp on. Send them to the owner and **stop** for their eye. Record what was chosen
- [ ] T041 [US4] T035 and T036 pass on iOS 26 and 27

---

## Phase 7: Polish

- [x] T042 [P] The timing tests of features 003 and 004 run with Zork I's version 3 pack and hold (FR-024)
- [x] T043 [P] `README.md`, `stories/corpus/README.md`, `packs/README.md`: light, dark ahead, the proof and its figures, the ledger's new count
- [x] T044 [P] Correct this feature's contracts to what was built
- [ ] T045 Play by hand in the simulator: into the attic and the cellar without the lamp and with it; then VoiceOver over the note and a marked way: everything a sighted player learns is said (SC-008). Record what was seen
- [ ] T046 Run `scripts/test-ios.sh 26` and `27` in full and the timing tests on a quiet machine; **stop** if a limit is exceeded
- [x] T047 Write the list this file ends with (FR-025; SC-010); confirm `git diff main --stat -- crates/zmachine-core` is empty
- [ ] T048 Run every step of `quickstart.md`; clippy and `cargo fmt --check` pass; write "What was done, and what was done differently from the plan"

---

## Dependencies & Execution Order

- Phase 2 blocks every story. US1 before US3; US3 before US2 (the proof checks both);
  US4 after US3, and its tests on Zork I with the pack need T027.
- Within a phase, every test task before the task that makes it pass.
- In Phase 5: T024 before T025 before T026; T027 before T028 before T030.
- In Phase 6: T037 before T038 before T039 before T040 before T041.

### Parallel Opportunities

- T005 to T009; T013 to T015; T018 to T023; T032 to T034 and T036; T042 to T044.

## Implementation Strategy

### Incremental Delivery

1. Phases 1 to 4: the published answers.
2. Phase 5: the proof, Zork I's entry, and its ledger entries gone.
3. Phase 6: the warning.
4. Phase 7: measurements, READMEs, the list.

One pull request.

### What was learned from the earlier features

- A probe test (`tests/zz_*.rs`) is the quick way to measure; it is removed before the
  task is marked done.
- A fixture built with other start-up code is another story file; give each build its
  own pack.
- One object cannot single out a fact: a proof is trusted when the test of all 32
  attributes leaves one standing.
- Measure timings on a quiet machine, not while the UI suite runs.
- The formatter rewraps test text; edit by position, not by matching wrapped strings.
- The owner judges drawings on pictures from the simulator; build the plain version
  and stop.

## Tests of earlier features edited because a form changed

Each fixes a written form that this feature changed. None fixed a value that changed.

| Test file | What changed in it |
| --- | --- |
| `crates/zmachine-introspect/src/json_tests.rs` | `location` and `scope` gain `source`; an exit gains `dark_ahead` |
| `crates/zmachine-introspect/src/pack_tests.rs` | three tests that said `pack: 3` is refused now say `pack: 4` |
| `crates/zmachine-introspect/tests/spec_agreement.rs` | section 12 has ten cited parts, not seven |
| `crates/zmachine-cli/tests/inspect.rs` | the new keys in a line of `--inspect`; the interface's version |
| `crates/zmachine-corpus/tests/check.rs` | its example of a key the format lacks was `lit-attribute`; now `wet-attribute` |
| `crates/zmachine-corpus/tests/crawl.rs`, `run.rs`, `common/mod.rs` | hand-built stories gain `look: None`; the toy game gains a lamp and lit rooms |
| `crates/zmachine-corpus/tests/formats.rs` | the manifest's and the ledger's key counts |
| `crates/zmachine-corpus/tests/no_text.rs` | eight more words of the report format; two more keys in its pack reader |
| `crates/zmachine-experience/src/json_tests.rs` | the snapshot ends with `light`; a place has `dark_ahead` after `leads_to` |
| `crates/zmachine-experience/tests/session.rs` | two tests that fix a snapshot's exact text gain the keys |
| `crates/zmachine-ffi/tests/abi.rs` | one place's exact text gains `dark_ahead` |
| `crates/zmachine-ffi/tests/app_rules.rs` | the list of pack keys the app must not hold gains three |
| `apps/ios/LamplightUITests/LamplightUITests.swift` | hand-made snapshots gain `light` and `dark_ahead` |

`git diff main --stat -- crates/zmachine-core` is empty.

## What was done, and what was done differently from the plan

**Done**: the interface says lit, dark or unknown for the player's room, and dark ahead
for each exit (specification 0.4.0, sections 12.9 to 12.11); the corpus proves it by
asking the game to describe the room; Zork I's and Mini-Zork's packs name attribute 19
as lit, and on each no other attribute passes; 51 ledger entries left (Zork I 39,
Mini-Zork 12); the layer's snapshot has `light` and `dark_ahead`; the app shows the
layer's note above the compass and a badge on a way into darkness.

**Differently from the plan**:

- **A second key, `invisible-attribute`** (research R13). The plan had one. With one,
  Zork I's entry was contradicted where a character holds the player's light, hidden.
  The owner chose the second fact at the stop point of T027. The player's own object
  carries the attribute, so the player is looked through regardless.
- **That key is not singled out.** On Zork I 7 passes and so does 30; a test fixes that
  exactly those two do. `packs/README.md` says why 7 is named.
- **Six ledger entries were added**, for ways where the light is taken during the move.
  I§12.10 now says `dark_ahead` is about the moment of the query.
- **Zork I's aside grew**, into the attic without the lamp and with it, so that a room
  is seen both dark and lit. The walkthrough alone has no such room.
- **A closed door is not marked dark ahead**, though the interface says what is behind
  it: the layer does not tell a player what they have not seen. An open way is.
- **Light is said only while the game waits for a command.** At any other stop the
  snapshot's `light.dark` is null.
- **Mini-Zork gained the entry** (T031): one attribute was left standing.
- **Without a pack, Zork I's kept report shows 44 wrong trials with no cause**, where it
  showed 5: the darkness entries are gone, and without the pack nothing tells the
  interface it is dark.

**Timings** (the owner's Mac, quiet): slowest of 20 starts 1.06 ms; 331 commands, mean
69 µs, slowest 190 µs; the whole transcript 77 µs.

**The owner's eye** (T040): shown the kitchen with the marked way, the dark attic with
the note, and the kitchen with the lamp on, the owner kept the note and the badge as
built.

**Not done**: T045, play by hand with VoiceOver.
