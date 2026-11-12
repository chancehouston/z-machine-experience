# Implementation Plan: First Demo on a Phone

**Branch**: `004-first-demo` | **Date**: 2026-11-11 (revised after independent review) | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-first-demo/spec.md`

## Summary

Put the whole stack on an iPhone for the first time. Zork I runs inside an app called
Lamplight; the screen shows the game's text, its status line, a compass whose lit
directions are the exits the introspection interface reports, and a text field. A tap on
a lit direction sends the game the word for that direction, shown to the player as if
typed.

Three pieces are new. An **experience layer** in Rust decides everything: what the
compass shows, what a tap sends, what the game is waiting for. A small **C binding**
crate exposes it as eight functions. A **SwiftUI app** draws a snapshot the layer hands
it and passes taps and typed lines back; it decides nothing.

Behaviour is proven in Rust, without a phone: the Zork I walkthrough is replayed with a
tap wherever the compass offered the move, and what the layer recorded must equal the
reference transcript, from the first command to the end, byte for byte. The app itself is tested in the simulator by automated UI tests and by
screenshots. Findings are in [research.md](research.md).

## Revision after review

Two independent reviews found the first version of this plan was not ready for task
writing. One read it against the spec, the constitution and the code. The other built a
throwaway app outside the repository to test the toolchain claims. The main changes:

- **The rule for telling a question from a command prompt was wrong.** Zork I prints its
  prompt before yes-or-no questions too. By the project owner's decision the rule is now
  where in its code the game is waiting (research R6), which needs one read-only
  accessor added to the core.
- **The headline comparison could not be made as described.** The reference is the
  game's transcript stream, which the layer declines. The comparison is now of what the
  layer records, unaltered, against the reference from its first command on (R14).
- **A late tap was not really refused.** Commands now carry the turn they were made
  against, and the layer refuses a stale one.
- **A dark compass no longer says "no exit".** There is a state for "not offered now".
- **The app no longer composes any text about the game.** Labels and spoken texts come
  from the layer, which is what makes "no game logic in the app" checkable.
- **"New game" is a new machine with a fresh seed**, so successive games differ.
- **The compass check asks a second machine**, so it is not the layer agreeing with
  itself.
- **Timing criteria were reworded** to what can be measured in a simulator.
- **The build approach was confirmed end to end**, with four corrections: the Swift
  actor needs an isolated deinitialiser; the simulator destination must name the system
  version; simulator builds exclude x86_64; and numbers in SwiftUI text must be written
  verbatim.
- **iOS details that a task-writer would have had to invent are now in the screen
  contract**: the keyboard, large text, the lifecycle, how the story file is bundled, and
  what the UI tests can see.

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
the bindings, 500 of Swift, one C header of eight functions, and one accessor of a few
lines added to the core.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core's behaviour is not changed. One read-only accessor is added (`Machine::waiting_at`), with tests first; the introspection crate is not changed. The game is driven only through the core's public interface. SC-004 replays the walkthrough through the new layer and requires the reference text; every earlier test passes unchanged. |
| II. Test-First | Each behaviour of the experience layer and each binding has a failing test before its code. The app's UI tests are written against the screen contract before the views exist. Where a test cannot be seen to fail first (a view's appearance), the task says so and what stands in for it. |
| III. Typed Commands | A tap produces one line of text, a direction word, given to the game through the same private function as a typed line, and shown to the player in the transcript (FR-010, FR-011). The only other action is starting a new game, which loads the story file afresh. Declining a save, restore or transcript is an answer the core's interface already offers a host; it changes nothing a player could not. Typing is always available while the game waits for a line. |
| IV. Read-Only Introspection | The layer uses the introspection interface as published, by shared reference. When it answers "unknown" the compass lights nothing and says so (FR-014, FR-021). A direction is lit only when an exit is reported. The rule for telling a command prompt from a question is the experience layer's, not the introspection interface's; it is a stated heuristic chosen by the project owner, with its two failure cases written down (research R6), and where it is unsure the compass offers nothing. |
| V. Strict Layering | App → bindings → experience layer → introspection → core; no arrow points back. `unsafe` appears only in `zmachine-ffi`, each use with a comment stating its invariant. The Swift code is presentation; a test reads its source for anything else (SC-011). |
| VI. Story File Licensing | Zork I's license permits bundling; its license text is shown in the app (FR-029). Advent is not bundled. "Zork" is not used in the app's name, icon or own text (FR-028), and a test enforces it. |

**Result**: Pass. No deviation. The prompt rule is a heuristic about what the game is
waiting for, not a guess about the game's world; the constitution's "MUST NOT guess"
(Principle IV) governs the introspection queries, which are used as published.

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
│   │   ├── lib.rs              # Session: new, send_line, tap, new_game, snapshot
│   │   ├── compass.rs          # places, matching exits to places, the word a tap sends
│   │   ├── transcript.rs       # turns: the game's text and the commands given
│   │   └── json.rs             # the snapshot as JSON
│   └── tests/
│       ├── walkthrough.rs      # the replay with taps (SC-004, SC-005, SC-006)
│       ├── session.rs          # waiting states, new games, declined requests, stale taps, faults
│       ├── speed.rs            # time per command and snapshot (SC-003, SC-007)
│       └── crate_rules.rs      # no unsafe, no I/O, no game named
├── zmachine-ffi/               # NEW: the C bindings (staticlib; the only unsafe)
│   ├── include/zmx.h           # the C header, written by hand
│   ├── src/lib.rs
│   └── tests/abi.rs            # every function as C would call it
├── zmachine-core/              # one accessor added: Machine::waiting_at
└── (introspect, cli, testkit unchanged)

apps/ios/                       # NEW: the iPhone app
├── project.yml                 # XcodeGen; the .xcodeproj is generated, not committed
├── Lamplight/
│   ├── LamplightApp.swift
│   ├── Engine.swift            # an actor that owns the session and calls the bindings
│   ├── Snapshot.swift          # Codable types matching the snapshot
│   ├── GameView.swift          # the screen: status, transcript, compass, field
│   ├── CompassView.swift
│   ├── CompassLayout.swift     # where each of the twelve places goes; the app's only table
│   ├── AboutView.swift         # the license notice
│   └── Resources/              # story.z3 and story-license.txt, copied by the build script; not committed
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

Tests come before the code they test at every step. The last two features each let this
slip; the task list must name, for each implementation task, the failing test that
precedes it.

1. The core's accessor: failing tests, then the accessor. Every earlier test unchanged.
2. The experience layer's tests on built story files (compass mapping, waiting states,
   stamps, new games, declined requests, snapshot JSON), written against stubs and seen
   to fail; and the walkthrough replay (research R14), also failing. Then the layer.
3. The bindings: the header and tests that call it as C would, failing; then the code.
4. The build script, the generated project, and the UI-test target with its first test
   (launch; the opening text is shown), failing because the app shows nothing; then the
   least app that passes it. This proves the pieces link and run (User Story 1).
5. For each part of the screen, a failing UI test from the screen contract, then the
   view: status, transcript, compass and taps, the field, the keyboard, questions,
   endings, a missing story file, leaving and returning, large text.
6. The About sheet, the name check, the source check for SC-011.
7. Timing, screenshots, CI, README.

Where a test cannot be seen to fail first, because what it checks is how something
looks, the task says so and names the screenshot that stands in for it.

## Complexity Tracking

No constitution violations to justify.
