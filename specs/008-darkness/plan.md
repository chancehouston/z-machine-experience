# Implementation Plan: Darkness

**Branch**: `008-darkness` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/008-darkness/spec.md`

## Summary

Let the interface say whether the player is in the dark, and whether a way leads into
the dark, for games of the ZIL family, from one fact in a game pack: the attribute set
on rooms that are lit of themselves and on things that give light. The room is lit if
it has the attribute or a light can be seen in it; a way is dark ahead if the room it
leads to would be dark for the player as they are. The corpus proves the fact against
the game by asking throwaway copies to describe the room, and stops counting exit
reports made in the dark as wrong. The app says "It is dark" beside the compass and
marks ways into darkness.

The core is not touched. What was measured on Zork I is in [research.md](research.md).

## Strongest objection

**"Dark ahead" is the first thing the project tells a player about a place they have
not been, and it is a prediction.** Measured on Zork I it was right 783 times in 784:
the one miss was a character taking the player's light in the turn they walked in.
That is a good rate, and the miss has a cause. But a prediction shown on the compass
invites trust that a present-tense fact has earned and a forecast has not. A room lit
by the game's own code would be marked dark; a light that goes out on the way would
leave a dark room unmarked. The corpus checks every mark it can by walking the way on a
copy, and the mark's spoken words say "into darkness", not "you will die". The owner
chose to show it.

Three lesser ones:

- **The mark's reach grows.** With the entry in force, every exit with a known room is
  `source: pack`, since its `dark_ahead` rests on the pack (research R4). The mark
  stays true to its rule and says less for being nearly everywhere.
- **The pack format reaches version 3** (research R5).
- **"Made in the dark" forgives.** A wrong exit report at a dark place is no longer
  counted as wrong. That is honest only because the interface now says it is dark; a
  real misreading of a dark room's table would also be forgiven. Confirmed reports are
  still counted, so the report shows how many there were.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new.

**Storage**: Plain text: the pack, the corpus manifest, ledger and reports.

**Testing**: `cargo test`, on built stories for each rule and on the committed stories
for the proofs; the app's UI tests through `scripts/test-ios.sh`. Timing tests take
their limits from `zmachine_testkit::time_limit`.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: The limits of features 003 and 004 hold (FR-024).

**Constraints**: The core does not change. No code for one game. With no pack, and for
Inform stories, every earlier value is unchanged. No transcript changes. Nothing of a
game's reply is kept. Unknown light shows nothing in the app.

**Scale/Scope**: About 150 lines in the introspection crate, 250 in the corpus crate,
60 in the experience layer, 60 in the app, each with tests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

Checked against version 2.0.0.

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core is not changed. |
| II. Test-First | Each statement of contracts/answers.md has a test on a built story, seen to fail before its code; so has each verdict and proof outcome. Where a test cannot fail first, the task says so. |
| III. Typed Commands | Touching a marked way types the direction. The corpus's `look` is a typed line given to copies. |
| IV. Read-Only Introspection | Light and dark ahead are read from the object tree at the moment of asking; nothing is tried. Where the rule cannot say, the answer is unknown and the app shows nothing. |
| V. Strict Layering | The rule in the interface; the note and the marks in the experience layer; drawing in the app. |
| VI. Story File Licensing | No story is added. Nothing of a game's reply is kept. |

**Principle IV's rules for a pack, for the new entry:** identity unchanged; one number;
published in I§12.9 and I§12.10; verified by the game's own behaviour at every place
and on every way a trial walks; marked on `location`, `scope` and each exit; with no
pack every earlier value unchanged.

**"A pack supplies a fact; it does not license a guess."** Dark ahead is the nearest
this project has come to that line. It is on the right side of it: the answer is
worked out from stored facts by a published rule, is `unknown` where a fact is
missing, and is checked by trials. It is not a guess; it is a forecast, and the spec
says so.

**Result**: Pass. No deviation to justify.

**Post-design re-check**: Pass.

## Project Structure

### Documentation (this feature)

```text
specs/008-darkness/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/answers.md, contracts/commands.md
└── tasks.md             # written by /speckit-tasks
```

### Source Code (repository root)

```text
docs/introspection.md               # 0.4.0: sections 6, 6.2, 8, 9.2, 2.3, 12

packs/zork1.pack                    # version 3, lit-attribute

crates/zmachine-introspect/src/
├── lib.rs                          # Location.source, Scope.source, Exit.dark_ahead
├── pack.rs                         # version 3; lit-attribute
├── zil.rs                          # the light rule, for the room and for a way's room
└── json.rs, and tests beside each

crates/zmachine-experience/src/
├── compass.rs                      # dark_ahead on a place and an extra
├── light.rs                        # NEW: the snapshot's light
└── lib.rs, json.rs

crates/zmachine-corpus/
├── src/
│   ├── light.rs                    # NEW: ask a copy to look; compare; verdicts
│   ├── manifest.rs, ledger.rs, report.rs, verdict.rs
│   ├── walk.rs, corpus.rs          # compare at each place and after each trial
│   └── packs.rs                    # proof; version 3
└── tests/light.rs                  # NEW; others extended

stories/corpus/manifest.txt, ledger.txt, reports/zork1.txt, reports/nopack/zork1.txt

apps/ios/Lamplight/
├── CompassView.swift               # the badge
├── GameView.swift                  # the note
└── Snapshot.swift
apps/ios/LamplightUITests/
```

## Order of work

1. **Publish** the changes to `docs/introspection.md`.
2. **User Story 1**: the pack's version 3 and key; the light rule and its unknowns on
   built stories; the marks; JSON.
3. **User Story 3**: dark ahead on built stories. (Before User Story 2, because the
   proof checks both.)
4. **User Story 2**: the manifest's key; the two comparisons and their verdicts on toy
   stories; "in the dark"; the ledger's forms; the report's lines; proof. Then Zork I:
   its command, its entry, the test of all 32 attributes, the 39 ledger entries
   removed. If a difference appears whose cause cannot be found, stop.
5. **User Story 4**: the layer's light and marks; the app's note and badge, with
   pictures for the owner before they are settled; the UI test.
6. **Mini-Zork**, where its file is.
7. **Measure**; READMEs; the list of earlier tests whose form changed.

## Decisions for the project owner

Made in research and open to being overruled before tasks are written:

1. **The attribute is read, not the game's own flag** (research R8).
2. **Where the player cannot be told, light and dark ahead are unknown** unless
   something other than what is carried settles them (research R2).
3. **Every exit with a known room becomes `source: pack`** once the entry is in force
   (research R4).
4. **The pack format becomes version 3** (research R5).
5. **The game "described the room" if its reply to the look command names the room**,
   and nothing of the reply is kept (research R6).
6. **At a dark place, a report that would be wrong is counted "in the dark"**; a
   confirmed one stays confirmed (research R7).
7. **The note reads "It is dark"; a marked way is spoken "... into darkness"; the mark
   is a small dark badge on the way's disc** (research R10), to be judged on pictures.

## Pull requests

One.

## Complexity Tracking

No violation of the constitution to justify.
