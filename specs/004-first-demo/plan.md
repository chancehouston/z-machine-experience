# Implementation Plan: First Demo on a Phone

**Branch**: `004-first-demo` | **Date**: 2026-11-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-first-demo/spec.md`

## Summary

Put the whole stack on an iPhone for the first time. Zork I runs inside an app called
Lamplight; the screen shows the game's text, its status line, a compass whose lit
directions are the exits the introspection interface reports, and a text field. A tap on
a lit direction sends the game the word for that direction, shown to the player as if
typed.

Three pieces are new. An **experience layer** in Rust decides everything: what the
compass shows, what a tap sends, what the game is waiting for. A small **C binding**
crate exposes it as seven functions. A **SwiftUI app** draws a snapshot the layer hands
it and passes taps and typed lines back; it decides nothing.

Behaviour is proven in Rust, without a phone: the Zork I walkthrough is replayed with a
tap wherever the compass offered the move, and the transcript must equal the reference
byte for byte. The app itself is tested in the simulator by automated UI tests and by
screenshots. Findings are in [research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024) for the experience layer and bindings;
Swift 6.4 with SwiftUI for the app

**Primary Dependencies**: None added to Rust. The app uses only Apple's frameworks.
Build tools, not dependencies: XcodeGen 2.46 to generate the Xcode project, `xcodebuild`
to build and test.

**Storage**: None. The story file is a resource inside the app. Nothing is written.

**Testing**: `cargo test` for the experience layer and the bindings; XCUITest in the iOS
Simulator for the app; simulator screenshots for appearance

**Target Platform**: iOS 26 and later, iPhone, portrait. Verified in the simulator on
iOS 26.0 and iOS 27.2.

**Project Type**: Mobile app over shared Rust libraries, in the existing workspace

**Performance Goals**: Opening text on screen within 2 seconds of a cold launch
(SC-001); reply and compass within 100 ms of a command (SC-003, SC-007)

**Constraints**: The app decides nothing about the game (FR-026, SC-011); `unsafe` only
in the binding crate; no "Zork" in the app's name, icon or own text; every earlier test
passes unchanged; nothing done that would prevent a build for a physical iPhone

**Scale/Scope**: One screen. About 600 lines of Rust in the experience layer, 250 in
the bindings, 500 of Swift, one C header of seven functions.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core and the introspection crate are not changed. The game is driven only through the core's public interface. SC-004 replays the walkthrough through the new layer and requires the reference transcript. |
| II. Test-First | Each behaviour of the experience layer and each binding has a failing test before its code. The app's UI tests are written against the screen contract before the views exist. Where a test cannot be seen to fail first (a view's appearance), the task says so and what stands in for it. |
| III. Typed Commands | A tap produces one line of text, a direction word, given to the game through the same call as a typed line, and shown to the player in the transcript (FR-010, FR-011). The only other action is starting a new game. The layer has no other way to reach the game. Typing is always available while the game waits for a line. |
| IV. Read-Only Introspection | The layer uses the introspection interface as published, by shared reference. When it answers "unknown" the compass lights nothing and says so (FR-014, FR-021). A direction is lit only when an exit is reported. |
| V. Strict Layering | App → bindings → experience layer → introspection → core; no arrow points back. `unsafe` appears only in `zmachine-ffi`, each use with a comment stating its invariant. The Swift code is presentation; a test reads its source for anything else (SC-011). |
| VI. Story File Licensing | Zork I's license permits bundling; its license text is shown in the app (FR-029). Advent is not bundled. "Zork" is not used in the app's name, icon or own text (FR-028), and a test enforces it. |

**Result**: Pass. No deviation.

**Post-design re-check**: Pass. The snapshot gives the app finished facts, so the app has
no game logic to hold; a tap names a place, not a command, so the app cannot send what
the layer did not offer.

## Project Structure

### Documentation (this feature)

```text
specs/004-first-demo/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── experience-api.md    # the Rust layer: session, snapshot, commands
│   ├── c-abi.md             # the seven C functions and their rules
│   └── screen.md            # what the app shows for each part of a snapshot
└── tasks.md                 # written by /speckit-tasks
```

### Source Code (repository root)

```text
crates/
├── zmachine-experience/        # NEW: the experience layer (no unsafe, no I/O)
│   ├── src/
│   │   ├── lib.rs              # Session: new, send_line, tap, restart, snapshot
│   │   ├── compass.rs          # places, matching exits to places, the word a tap sends
│   │   ├── transcript.rs       # turns: the game's text and the commands given
│   │   └── json.rs             # the snapshot as JSON
│   └── tests/
│       ├── walkthrough.rs      # the replay with taps (SC-004, SC-005, SC-006)
│       ├── session.rs          # waiting states, restart, save/restore/script, faults
│       └── crate_rules.rs      # no unsafe, no I/O, no game named
├── zmachine-ffi/               # NEW: the C bindings (staticlib; the only unsafe)
│   ├── include/zmx.h           # the C header, written by hand
│   ├── src/lib.rs
│   └── tests/abi.rs            # every function as C would call it
└── (core, introspect, cli, testkit unchanged)

apps/ios/                       # NEW: the iPhone app
├── project.yml                 # XcodeGen; the .xcodeproj is generated, not committed
├── Lamplight/
│   ├── LamplightApp.swift
│   ├── Engine.swift            # an actor that owns the session and calls the bindings
│   ├── Snapshot.swift          # Codable types matching the snapshot
│   ├── GameView.swift          # the screen: status, transcript, compass, field
│   ├── CompassView.swift
│   ├── AboutView.swift         # the license notice
│   └── Resources/              # the story file and its license, copied at build
└── LamplightUITests/
    └── LamplightUITests.swift

scripts/
└── build-ios.sh                # builds the bindings for simulator and device, makes the XCFramework
```

**Structure Decision**: Two new crates in the existing workspace and one app directory.
The experience layer is where the constitution puts logic both platforms need; the
bindings are separate so that `unsafe` stays in one small crate; the app is a consumer
that could be deleted and rewritten without touching a rule of the game.

## Order of work

1. The experience layer's contract tests, then the layer: compass mapping, waiting
   states, transcript, snapshot, restart, the declined requests.
2. The walkthrough replay with taps. Red until step 1 is complete.
3. The bindings: header, tests that call it as C would, then the code.
4. The build script and the generated project, with an app that shows only the opening
   text: proof that the pieces link and run in the simulator (User Story 1).
5. UI tests for the screen contract, then the views: transcript, status, compass, field.
6. Endings, faults, the About sheet, text sizes, the name check.
7. Timing, screenshots, CI, README.

## Complexity Tracking

No constitution violations to justify.
