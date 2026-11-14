# Implementation Plan: Touch the Room to Look

**Branch**: `013-room-look` | **Date**: 2026-11-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/013-room-look/spec.md`

## Summary

The experience layer finds, once for a story, the word its dictionary has for
looking. The map's snapshot says when the player's place can be touched and what the
touch sends. A touch goes to the layer by an entry of its own, beside the one for a
way, and the layer sends the word as a tapped line. The app makes the player's place
a control and holds no word.

One stage, one pull request, stacked on the map's stage 3 (pull request 21).

## Strongest objection

**"The dictionary has the word `look`" is taken to mean "this word looks around".**
Nothing in a story file says what a verb does. Every story the project has met
means it so, and the worst case is a visible command with a visible reply, which the
player can judge. But it is the nearest this layer has come to assuming a meaning,
and Principle IV is about exactly that. The corpus test records how many stories have
the word; it cannot record whether it means what we think.

**Second: two words in two specifications changed in one day.** The specification
first said the layer gives the spoken label. Planning found that changing the label
breaks every test that reads the place's name, against SC-004. The plan keeps the
name as the label and gives the layer's words as the hint. The specification was
corrected (User Story 4, FR-008, FR-010) and the owner should see it.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new.

**Storage**: None new. A touch is kept as the tap line the kept game already has.

**Testing**: `cargo test` on built stories, Zork I, Adventure and the corpus; the
app's UI tests on iOS 26 and 27; the owner's phone.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: The limits of features 003, 004 and 011 hold (FR-013). The
dictionary is read once for a story, not at each stop.

**Constraints**: The app holds no command word. The map is unchanged by a look.

**Scale/Scope**: About 60 lines in the layer and 30 of tests; one binding; about 40
lines in the app; three or four UI tests.

## Constitution Check

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | The core is not touched. **Pass.** |
| II. Test-First | Each clause of the contract is a test, seen failing first; the record says what happened. **Pass**, to be shown. |
| III. The Engine Takes Typed Commands | The touch sends one line, visible in the transcript and kept as a tap. **Pass.** |
| IV. Introspection Is Read-Only | The word is read from the story's dictionary; where there is none, nothing is offered. The meaning of the word is assumed: see the strongest objection. No pack is changed and nothing is authored. **Pass**, with that said. |
| V. Strict Layering | The word, when it is offered, and the words spoken are the layer's. The app draws a control and sends a touch. A rule test holds the app to it. **Pass.** |
| VI. Story File Licensing | Nothing of a story is added to the repository. **Pass.** |

## Corrections made to the specification in planning

1. **User Story 4 and FR-008**: the place's spoken label stays its name. The
   layer's words say what a touch does and are the control's hint.
2. **FR-010**: the touch area is 44 points tall and as wide as the box allows
   (80 points) without overlapping the east and west ways' controls. "44 points
   square" could not be had for the small inset places without overlap.

## Project Structure

```text
specs/013-room-look/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/look.md
├── checklists/requirements.md
└── tasks.md

crates/zmachine-experience/src/
├── look.rs, look_tests.rs     # the word for looking, from a dictionary
├── map.rs, map_tests.rs       # the look, in the view
├── json.rs, json_tests.rs     # the look, in the snapshot
└── lib.rs                     # the word found at opening; Session::tap_place
crates/zmachine-experience/tests/map.rs     # Zork I and Adventure; coming back
crates/zmachine-corpus/tests/map.rs         # every story: the word or no touch
crates/zmachine-ffi/src/lib.rs, include/zmx.h, tests/abi.rs, tests/app_rules.rs

apps/ios/Lamplight/
├── Snapshot.swift             # the look
├── Engine.swift, Game.swift   # the touch, sent
├── MapDrawing.swift           # the player's place as a control
├── MapBand.swift, GameView.swift
apps/ios/LamplightUITests/LamplightUITests.swift
README.md, apps/ios/README.md
```

**Structure Decision**: the existing workspace; one small new module in the layer.

## Decisions for the owner in this plan

1. The two corrections above.
2. **The layer's words for the hint**: "Look around".
3. **A tap, not a button, in the app**: the place answers a touch by a tap gesture
   and is marked as a button for a screen reader, so that a drag beginning on it
   still moves the sheet (research R4).

## Complexity Tracking

No violation to justify.
