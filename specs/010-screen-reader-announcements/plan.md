# Implementation Plan: Screen Reader Announcements

**Branch**: `010-screen-reader-announcements` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/010-screen-reader-announcements/spec.md`

## Summary

The experience layer makes, for each turn, the text a screen reader is to speak: the
game's reply without its prompt character, then "It is dark" on the turn the dark
falls, then the words for an ended game on the turn it ends; or a fault's message. The
snapshot carries it in one new field. The app posts that text to the system when a
turn arrives and a screen reader is running, marked to cut off speech in progress, and
composes nothing itself.

Measured before any code was kept ([research.md](research.md)): every one of the 502
turns of the two walkthroughs at which the game waits for a line ends with the prompt
character, so the rule for leaving it out is one line.

## Strongest objection

**Nothing in this repository can hear.** The feature is done when a listener hears the
reply, and the tests can show only that the app asked the system to speak it. Whether
the system then speaks it, whether it cuts off the reply before, and whether the
screen reader's own speech for the button just pressed cuts off ours instead, are all
outside what a test here can see (research R5). The same was true of the labels the
earlier features gave, and two listening checks from those features are still open.
The plan answers this the only way it can: the first thing built is the smallest
version that can be heard, and it goes to the owner's phone, with a stop, before the
rest is built on it.

Two lesser ones:

- **A launch argument that makes the app pretend.** The app's tests need it to see
  anything. It is the second such argument, and each one is a way the tested app
  differs from the shipped one. The element it adds does not exist without it.
- **`message` gains a second meaning.** It was a fault's message; it becomes also the
  words for an ended game (research R4). Simple, and it changes tests that fixed it as
  null.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new.

**Storage**: None.

**Testing**: `cargo test`, on built stories and on the committed stories; the app's UI
tests through `scripts/test-ios.sh` on iOS 26 and 27; the owner's ear on a phone.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: The limits of features 003 and 004 hold (FR-018).

**Constraints**: The app composes no text. Nothing is sent to the game. No transcript
changes. No label, value or identifier an earlier feature gave an element changes.
With no screen reader running, nothing is posted. The C interface does not change.

**Scale/Scope**: About 80 lines in the experience layer and 60 in the app, each with
tests. The core, the introspection crate and the corpus crate are not touched.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

Checked against version 2.0.0.

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core is not changed. |
| II. Test-First | Each statement of the contract has a test, seen to fail before its code. Where a test cannot fail first, the task says so. Statement 22 is a person's to check. |
| III. Typed Commands | Nothing is sent to the game. The player's commands reach it as before. |
| IV. Read-Only Introspection | The interface is not touched. The announcement is made from text the game printed and from the light the interface already answers. |
| V. Strict Layering | The words and their order are the layer's. The app posts a field and holds no sentence. |
| VI. Story File Licensing | No story is added. Tests on Zork I hold no more of its text than the earlier ones do. |

**Re-check after design**: the contract moves one sentence, "The game has ended.", out
of the app and into the layer, which is in Principle V's direction. Nothing else
bears on the table.

## Project Structure

### Documentation (this feature)

```text
specs/010-screen-reader-announcements/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── announcements.md   # the layer, the snapshot, the app, and the owner's ear
└── tasks.md               # made by /speckit-tasks
```

### Source Code (repository root)

```text
crates/zmachine-experience/src/
├── announce.rs          # new: the reply, and the announcement for a turn
├── announce_tests.rs    # new
├── lib.rs               # the session keeps "was dark" and the announcement
└── json.rs              # the snapshot's last key; `message` when the game has ended

crates/zmachine-experience/tests/
├── session.rs           # announcements through a real session, on Zork I
└── walkthrough.rs       # every turn's announcement over the walkthrough

crates/zmachine-ffi/tests/  # the snapshot's keys, where a test fixes them

apps/ios/Lamplight/
├── Snapshot.swift       # decodes `announcement`
├── Announcer.swift      # new: posts to the system; counts for tests
├── Game.swift           # asks the announcer when a turn is shown
└── GameView.swift       # shows `message` for an ended game; the tests' element

apps/ios/LamplightUITests/  # what was asked to be spoken
```

**Structure Decision**: The existing workspace. One new module in the experience
layer and one new file in the app.

## Complexity Tracking

No violation to justify.
