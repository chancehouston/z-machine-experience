# Implementation Plan: Resume the Game

**Branch**: `011-resume-game` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/011-resume-game/spec.md`

## Summary

The app keeps a small file for the game in progress: which story it is, the chance
the game was started with, and every input the player gave. When the app opens, the
shared layer starts the story with that chance and gives it those inputs again. The
game, the transcript, the ways out and everything else the screen shows come out of
that as they did the first time, because the core is a function of its story, its
chance and its inputs. A digest of the transcript, kept beside the inputs, is checked
afterwards; if it does not match, or anything else is wrong with the file, the player
gets a new game and is told once.

The layer makes the bytes and reads them. The app holds them and hands them back.
A "Start over" button in the About sheet asks before giving up a game in progress.

## Strongest objection

**A kept game is only as lasting as the interpreter's behaviour.** A game is brought
back by playing its inputs again, so it comes back the same only while the core
answers those inputs the same. An update to the app that changes what a game printed
in the turns a player took (a defect in the core put right) makes that player's kept
game fail its check, and it is given up with the message. That is the honest outcome,
and FR-018 requires it over showing a game that may be wrong. It should be rare: the
core's behaviour is held against Frotz by the fidelity tests. But it is a real cost of
this design, and keeping the machine's state instead would not have it. Research R1
says why that was still set aside. The owner may weigh this differently.

**Second: nothing has been measured on a phone.** The figures in research R1 are from
the development Mac, and the 2,000-turn figure is worked out, not measured. They leave
a wide margin (about 120 ms against two seconds), but the tasks must measure it on the
simulator and on the owner's phone before the feature is called done.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new. The digest is `zmachine-introspect`'s, which
already uses `sha2` 0.10 (research R4, as changed in building).

**Storage**: One file per story in the app's Application Support directory, written
whole and atomically after each turn, left out of backups (research R5, R7).

**Testing**: `cargo test`, on built stories and on the committed stories; timing tests
run on request; the app's UI tests on iOS 26 and 27; a hand check on the owner's
phone.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: A kept game of 331 turns and one of 2,000 turns on screen
within two seconds of launch (FR-020). The limits of features 003 and 004 for a
command and its snapshot hold (FR-021).

**Constraints**: The app composes no text and does not read the kept bytes. Nothing
reaches the game but the player's own inputs. The kept file holds no text of any
game. The game's own SAVE and RESTORE are answered as today.

**Scale/Scope**: One new module in the experience layer (about 250 lines with its
tests' helpers), three functions in the C interface, about 120 lines in the app.
Kept file: about 15 bytes a turn.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | The core is not changed. Bringing back relies on its being a function of story, chance and inputs, and SC-002 tests that it is. The game's SAVE and RESTORE are answered as before. **Pass.** |
| II. Test-First | Each contract clause has a test written before the code that meets it; the tasks will order them so. **Pass**, to be shown in tasks. |
| III. The Engine Takes Typed Commands | The only things given to the game are inputs the player gave, in their order (contract L§4). Nothing is added. **Pass.** |
| IV. Introspection Is Read-Only | Not touched. A changed game pack does not stop a game coming back (FR-019). **Pass.** |
| V. Strict Layering | The layer makes and reads the kept bytes and decides the outcome; the app stores bytes and shows the layer's words. The host still supplies the chance. **Pass.** |
| VI. Story File Licensing | The kept file holds the player's inputs and digests, and no text of the game. It is written on the device, outside the repository and the bundle. **Pass.** |

**Re-check after design**: the contracts add two keys to the snapshot and three
functions to the C interface; none moves work across a layer. The kept form (contract
K) holds a digest of the transcript, not the transcript. **Pass.**

**A rule of an earlier feature is replaced**: feature 004's FR-005, "Each launch of
the app MUST start a new game", by this feature's FR-001, on the owner's decision of
2026-11-13.

## Project Structure

### Documentation (this feature)

```text
specs/011-resume-game/
├── spec.md
├── plan.md              # this file
├── research.md          # R1 to R11
├── data-model.md
├── quickstart.md
├── contracts/
│   └── kept-game.md     # K: the bytes; L: the layer; A: the app
├── checklists/
│   └── requirements.md
└── tasks.md             # made by /speckit-tasks
```

### Source Code (repository root)

```text
crates/zmachine-experience/
├── src/
│   ├── kept.rs                # new: the form K, written and read
│   ├── kept_tests.rs          # new
│   ├── lib.rs                 # Session::open, kept, keep_failed; chance, answered, notice
│   ├── json.rs                # + notice, start_over
│   └── json_tests.rs          # key list
└── tests/
    ├── resume.rs              # new: brought back equals never closed
    ├── session.rs             # last-key checks
    └── speed.rs               # + bringing back, keeping, size

crates/zmachine-ffi/src/lib.rs # + zmx_session_open, zmx_session_kept, zmx_bytes_free,
                               #   zmx_session_keep_failed

apps/ios/Lamplight/
├── Engine.swift               # open with kept bytes; kept(); keepFailed()
├── Game.swift                 # read at launch, write after each snapshot, -fresh
├── KeptFile.swift             # new: where the file is, atomic write, no backup
├── Snapshot.swift             # + notice, startOver
├── GameView.swift             # the notice above the command field
└── AboutView.swift            # Start over, with its confirmation

apps/ios/LamplightUITests/LamplightUITests.swift   # -fresh in launch; new tests

README.md, apps/ios/README.md  # what is kept, and where
```

**Structure Decision**: The existing workspace. One new module in the experience
layer; no new crate.

## Decisions for the owner in this plan

1. **Where starting over is put** (research R9): a "Start over" button in the About
   sheet, asking first. Not on the main screen.
2. **The cost named in the strongest objection**: a kept game may be given up after an
   update that changes how the core answers.
3. **The words**, which are the layer's and so are fixed here unless changed:
   - the question: "Start over? The game in progress will be given up."
   - when a game cannot be brought back: "Your earlier game could not be brought
     back. This is a new game."
   - when the place cannot be kept: "Your place is not being kept."

## Complexity Tracking

No violation to justify.
