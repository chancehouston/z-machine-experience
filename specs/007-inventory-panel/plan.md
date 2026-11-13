# Implementation Plan: Inventory Panel

**Branch**: `007-inventory-panel` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/007-inventory-panel/spec.md`

## Summary

Show the player what they carry, always, in a row above the compass, and show exactly
what the game's own inventory listing would: a closed sack with nothing in it, a closed
glass bottle with its water. The introspection interface's inventory answer gains three
marks on each entry (whether a container is open, whether a thing inside one is seen,
and whether the answer rests on a pack). Game packs gain one fact, the attribute that
means see-through, and the fact they already hold for "open" is put to work on
containers. The corpus proves both against the game's own listing, asked of throwaway
copies. The experience layer turns the marked answer into a strip; the app draws it.

The core is not touched. Findings, with what was measured on Zork I, are in
[research.md](research.md).

## Strongest objection

**The proof reads the game's own words, and that is new ground.** Until now the corpus
has judged by where the player ended up, which is a number. Here the measure is a
listing the game prints, and the corpus must decide from that text whether each object
was listed. The plan does it by asking only whether an object's own printed name
occurs in the reply (research R3), which needs no knowledge of any game's layout and
keeps nothing of the text. But it is cruder than a walk through a door: a name inside
another name would fool it, and a game that lists by some other name than the object's
would defeat it. On Zork I it agrees at every one of 324 prompts where there is a
listing to compare. On a game where it does not, the differences go in the ledger with
causes, and enough of them would mean the measure does not fit that game.

Three lesser ones:

- **"See-through" is proven on one bottle.** Zork I's walkthrough never carries such a
  thing closed, so the proof is a five-command aside written for it (research R2). It
  is a real proof (every other attribute fails it) but a narrow one.
- **One row is small.** A sack with five things in it, opened, is six chips. The row
  scrolls; whether that is pleasant is for the owner's eye on a picture.
- **The pack format gets a version 2 for one key**, a day after version 1 (research
  R7).

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app, as in feature 004

**Primary Dependencies**: None new.

**Storage**: Plain text: the pack, the corpus manifest, ledger and reports.

**Testing**: `cargo test`, on stories built in memory for each rule and on the
committed stories for the proofs; the app's UI tests through `scripts/test-ios.sh`.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app, in the
existing workspace.

**Performance Goals**: A command and its snapshot within feature 004's 10 ms with the
strip included; the queries within feature 003's 5 ms; launch within 2 s (SC-010).

**Constraints**: The core does not change (FR-028). No code for one game. With no pack
every earlier value of the inventory answer is unchanged (FR-003). No transcript
changes (FR-006). Nothing of a game's listing is kept (FR-016). The strip shows nothing
that is not seen (FR-020). The app composes no text (FR-027).

**Scale/Scope**: About 150 lines in the introspection crate, 250 in the corpus crate,
120 in the experience layer, 120 in the app, each with tests; one changed section and
two changed subsections of the published specification.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

Checked against version 2.0.0.

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core is not changed. |
| II. Test-First | Each statement of contracts/answers.md has a test on a built story, written and seen to fail before its code; so has each verdict and proof outcome of the comparison, and each state of the strip. Where a test cannot fail first, the task says so. |
| III. Typed Commands | Touching the strip sends nothing. The corpus's inventory command and asides are typed lines given to copies. |
| IV. Read-Only Introspection | The answer is still a read. The game's listing is asked of a throwaway copy, by the corpus only, never by the interface. Unknown stays unknown: with no pack fact, marks are `unknown` and the strip keeps contents back. See the pack rules below. |
| V. Strict Layering | Marks in the interface; what to show in the experience layer; drawing in the app. The C interface does not change. No new `unsafe`. |
| VI. Story File Licensing | No story is added. Nothing of a game's reply is kept; a test searches the reports and ledger as before. |

**Principle IV's rules for a pack, for the new entry:**

| Rule | How |
| --- | --- |
| Identity | Unchanged. |
| Facts only | One number. |
| Published | I§12 gains the key and the format's version 2; I§7 gains the marks. |
| Verified | Proven by the comparison with the game's listing; `not-proven` and a contradiction both fail (research R8). |
| Marked | `source: pack` on each inventory entry a pack changed. |
| Absent means unchanged | With no pack the new fields are `unknown`, none and `file`, and every earlier field has its earlier value; a test compares on whole walkthroughs. |

**Result**: Pass. No deviation to justify.

**Post-design re-check**: Pass. The design added no query, no dependency, nothing in the
core, and no game text in any kept file.

## Project Structure

### Documentation (this feature)

```text
specs/007-inventory-panel/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── answers.md       # the inventory answer's marks and the pack's new key
│   └── commands.md      # the snapshot, the screen, the corpus's commands and files
└── tasks.md             # written by /speckit-tasks
```

### Source Code (repository root)

```text
docs/introspection.md               # 0.3.0: sections 7, 7.1, 7.2, 2.3, 12

packs/zork1.pack                    # version 2, transparent-attribute

crates/zmachine-introspect/src/
├── lib.rs                          # Carried's three fields; ContainerState
├── pack.rs                         # version 2; transparent-attribute
├── zil.rs, inform.rs               # the rule for each family
├── json.rs                         # the three keys
└── pack_tests.rs, zil_tests.rs, inform_tests.rs, json_tests.rs

crates/zmachine-experience/src/
├── strip.rs                        # NEW: Strip, StripItem; from the inventory answer
├── lib.rs                          # asks at each stop; Session::inventory
└── json.rs                         # the snapshot's inventory

crates/zmachine-corpus/
├── src/
│   ├── inventory.rs                # NEW: ask a copy, compare, verdicts
│   ├── manifest.rs                 # inventory, aside
│   ├── ledger.rs                   # the second form of entry
│   ├── report.rs                   # the inventory line and detail
│   ├── walk.rs, corpus.rs          # compare at each place; asides; explain
│   └── packs.rs                    # proof of transparent-attribute; version 2
└── tests/
    └── inventory.rs                # NEW; formats.rs, check.rs, nopack.rs, no_text.rs extended

stories/corpus/
├── manifest.txt                    # inventory for zork1, minizork; one aside for zork1
├── ledger.txt                      # only if differences with causes are found
└── reports/zork1.txt, nopack/zork1.txt, minizork.txt, nopack/minizork.txt

apps/ios/Lamplight/
├── InventoryStrip.swift            # NEW
├── Snapshot.swift                  # the inventory
└── GameView.swift                  # the row above the compass
apps/ios/LamplightUITests/          # the strip through a game, and drawn from hand-made snapshots
```

**Structure Decision**: No new crate. The comparison is the corpus's, the marks the
interface's, the strip the experience layer's.

## Order of work

1. **Publish.** Write the changes to `docs/introspection.md` from contracts/answers.md.
2. **User Story 1**: the marks, by family and by pack, on built stories; the pack's
   version 2 and new key; JSON. With no pack every earlier value unchanged, on whole
   walkthroughs.
3. **User Story 2**: the manifest's two keys; the comparison and its verdicts on toy
   stories; the ledger's second form; the report's line; proof. Then Zork I: its
   command, its aside, its entry; the test of all 32 attributes. If the comparison
   differs anywhere without a cause that can be found, stop.
4. **User Story 3**: the strip in the experience layer, from hand-made answers and then
   on Zork I; the snapshot; the app's row, with pictures for the owner before it is
   settled; the UI test.
5. **User Story 4**: the unknown and not-current states, from hand-made snapshots.
6. **Mini-Zork**, where its file is: the command, and an aside if one can be found.
7. **Measure** the timings; READMEs; the list of earlier tests whose form changed.

## Decisions for the project owner

Made in research and open to being overruled before tasks are written:

1. **The game's listing is read by looking for each object's own name in it**, and
   nothing of it is kept (research R3; the strongest objection).
2. **"See-through" is proven on a short aside written for the purpose**, not on the
   walkthrough (research R2).
3. **The inventory answer still lists everything**, with marks; the strip does the
   keeping back (research R5).
4. **A container with nothing in it is a plain item**, with no open or closed mark
   (research R5; the spec's edge case is amended).
5. **The pack format becomes version 2** (research R7).
6. **A closed container in the strip carries the same outlined-panel mark as a closed
   way on the compass** (research R11), to be judged on a picture.
7. **Asides compare the inventory and try no exits**, so Zork I's exit figures do not
   change (research R2).

## Pull requests

One, unless the owner wants the app's row apart from the rest, as the compass was.

## Complexity Tracking

No violation of the constitution to justify.
