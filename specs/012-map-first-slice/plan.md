# Implementation Plan: The Map, First Slice

**Branch**: `012-map-first-slice` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/012-map-first-slice/spec.md`

## Summary

The experience layer gains a map: a record of the places the player has stood in and
the moves they made, and a page on which each place is put as it is first reached and
then left where it is. It is fed at every stop of the game with what was sent and what
the layer then reports, and it is a function of those turns alone, so a game brought
back brings its map with it. The snapshot carries the sheet the player is on. The app
draws that sheet in a band above the game's reply, with the keyboard up, in place of
the compass; the band opens out to the whole sheet.

It is built in three stages, each its own pull request: the layer; the band; the
opened-out sheet and the hand-drawn look.

## Strongest objection

**The specification asks the app to change three things at once that players already
rely on**: where the ways out are shown, how the keyboard behaves, and where the
inventory sits. Each was built and tested as its own feature. If the band turns out
too small to be worth looking at on a real phone, which nobody knows yet, the compass
will already be gone. The stages are the answer to this as far as there is one: stage
1 changes nothing on screen, and the owner holds stage 2 before stage 3 is built. But
stage 2 itself cannot be made smaller: with the keyboard always up there is no room
for both a compass and a band. If the owner's trial of stage 2 goes badly, the remedy
is to change the band before it is merged, not to keep the compass beside it.

**Second: the placement was proved on two walkthroughs.** The spike drew Zork I and
Adventure, each along the one path its walkthrough takes. A player who wanders will
reach rooms in another order, and a placed room never moves. Research R13 adds a
seeded wander to find out; what it shows is not known.

**Third: an attached room can land far from its parent.** The spike showed it. The
ladder still joins them, but the Attic drawn beside the Forest Path reads poorly.
This plan does not solve it.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new.

**Storage**: None. The map is made again from the turns (research R1). Two recorded
pages are kept as test fixtures (research R12).

**Testing**: `cargo test` on built stories, on Zork I and Adventure, and across the
corpus; timing tests on request; the app's UI tests on iOS 26 and 27; the owner's
phone after stages 2 and 3.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: The limits of features 003, 004 and 011 hold with the map
added (FR-039): a command and its snapshot in 10 ms; a kept game of 2,000 turns
brought back within the layer's second.

**Constraints**: The map shows only what the player has witnessed. No foresight. A
placed place does not move, save a group's one move. The app composes no text and
decides no position. Nothing is sent to the game by the map.

**Scale/Scope**: A map module of about 600 lines with as many of tests; one counter
in the core; one snapshot key; in the app about 500 lines new and the compass's two
files removed. Sheets of at most thirty places.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | One read-only counter is added to the core (research R3). No instruction's behaviour changes; the fidelity tests must pass unchanged. **Pass.** |
| II. Test-First | Every clause of the contract is a test, written and seen to fail before its code. Features 009 to 011 did not keep to this strictly and said so; the tasks for this feature must order tests first and the record must say what happened. **Pass**, to be shown. |
| III. The Engine Takes Typed Commands | A touch on a way sends that way's command as a typed line, shown in the transcript (contract A§2). The map sends nothing else. **Pass.** |
| IV. Introspection Is Read-Only | The map reads what the layer already reads at each stop: location and the settled compass. It guesses nothing: a line is a move that happened, and where the layer does not know, the map says so (W§13, S§8). No game pack is changed and no authored content is used. **Pass.** |
| V. Strict Layering | Record, positions, straight or bent, and every word are the layer's. The app draws and sends touches. **Pass.** |
| VI. Story File Licensing | Recorded pages hold room names. They are kept only for Zork I and Adventure, beside the files that already hold those names under those stories' licenses (research R12). The corpus's reports stay free of names. **Pass.** |

**Witnessed only, against Principle IV's "the UI MUST NOT guess"**: the two agree. The
map shows less than the layer knows, never more.

**Re-check after design**: the contract's snapshot gives the place a way leads to
only once walked (S§7), which is stricter than the compass. The `compass` key remains
in the snapshot with its destinations for the command-line program; the app no longer
reads it. A test of the app's source holds it to that. **Pass.**

## Corrections made to the specification in planning

For the owner to see; both are in spec.md with the date.

1. **FR-006 no longer covers a death.** No fact in a story file says the player died.
   Undo, restart and restore are known from the core (research R3). A death is caught
   by the new FR-006a: a direction walked that did not go where the layer said is
   drawn as a move by a word, so the room is not put in a wrong direction for good.
2. **FR-032's spoken labels drop the name of a place not yet walked to.** The compass
   says "West, to Kitchen". The owner decided an untaken way shows no name. The map
   says "West" until the line is walked.

## Project Structure

### Documentation (this feature)

```text
specs/012-map-first-slice/
├── spec.md
├── plan.md              # this file
├── research.md          # R1 to R14
├── data-model.md
├── quickstart.md
├── contracts/
│   └── map.md           # W: witnessed; P: placement; S: snapshot; A: the screen
├── checklists/
│   └── requirements.md
└── tasks.md             # made by /speckit-tasks
```

### Source Code (repository root)

```text
crates/zmachine-core/src/
└── lib.rs, ops_io.rs, undo.rs     # stage 1: the count of jumps

crates/zmachine-experience/
├── src/
│   ├── map.rs                     # stage 1: the record and the rules W
│   ├── map_place.rs               # stage 1: placement P
│   ├── map_tests.rs, map_place_tests.rs
│   ├── lib.rs                     # the observation, fed from advance()
│   ├── json.rs, json_tests.rs     # the map key
│   └── compass.rs                 # labels without a destination
└── tests/
    ├── map.rs                     # the walkthroughs, the recorded pages, coming back
    └── speed.rs                   # the map's cost

crates/zmachine-corpus/tests/map.rs        # stage 1: every story; the seeded wander

stories/zork1/map/walkthrough.txt          # stage 1: recorded page
stories/advent/map/walkthrough.txt         # stage 1: recorded page

apps/ios/Lamplight/
├── Snapshot.swift                 # stage 2: the map
├── MapBand.swift                  # stage 2: the band; stage 3: opening out
├── MapDrawing.swift               # stage 2: places, lines, stubs; stage 3: the hand
├── MapLayout.swift                # stage 2: positions to points; where a way's button goes
├── GameView.swift                 # stage 2: band, keyboard, strip and note moved
├── CompassView.swift              # stage 2: removed
└── CompassLayout.swift            # stage 2: removed; its symbols move to MapLayout

apps/ios/LamplightUITests/LamplightUITests.swift
crates/zmachine-ffi/tests/app_rules.rs     # the app's rules, for the map's files
README.md, apps/ios/README.md
```

**Structure Decision**: The existing workspace. Two new modules in the experience
layer; no new crate. The spike's file is not used and not merged; the placement is
written again in the layer, test-first.

## Decisions for the owner in this plan

1. **Three stages, three pull requests** (research R11), with the phone asked for
   after the second and the third.
2. **The two corrections to the specification** above.
3. **The band is 200 points tall** for the first trial. It is a guess.
4. **The inventory strip and the darkness note move into the band's lower edge.**
5. **The layer's words**, fixed here unless changed:
   - where there is no map: "No map for this game."
   - where the player's place is not known: "Where you are is not known."
   - the spoken summary: "Map. You are in" and the place's name; or "Map. You are
     somewhere dark."

## Complexity Tracking

No violation to justify.
