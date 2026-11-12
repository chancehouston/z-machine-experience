# Implementation Plan: Introspection Interface

**Branch**: `003-introspection` | **Date**: 2026-11-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-introspection/spec.md`

## Summary

Add a read-only window onto a running game: six queries (object tree, dictionary,
location, inventory, exits, objects in scope) defined by a written specification that
another interpreter could implement, with this project's implementation of it and an
option on `zm` that prints the answers at each prompt.

Two families of story file are recognised from their contents alone, Inform 6 with its
standard library and Infocom's ZIL, each by rules that hold for the family and name no
game. The queries are computed from the bytes of the game's memory; none runs the game's
code, and every one takes the machine by shared reference, so the compiler enforces that
nothing is altered. Where a story file's contents do not settle an answer, the answer is
"unknown".

Evidence, as in the earlier features, is measurement against real games: both
walkthroughs are replayed with every query asked at every prompt, and the transcripts and
saves must not change by a byte; the location, inventory and exits reported are checked
against what the games themselves then say and do. Findings are in
[research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99 stable, edition 2024

**Primary Dependencies**: None. The new crate depends only on `zmachine-core`. The CLI
depends on both. JSON is written by hand.

**Storage**: None. Expected answers for tests are committed text files beside the story
files.

**Testing**: `cargo test`: unit tests for each adapter rule on small images built by the
testkit; integration tests on the four committed story files; walkthrough replays with
every query at every prompt; robustness tests on damaged files; a check that the written
specification and the implementation list the same queries.

**Target Platform**: macOS and Linux now; nothing that would block iOS and Android.

**Project Type**: Library plus CLI, in the existing Cargo workspace

**Performance Goals**: All six queries at one prompt in under 5 milliseconds for Advent
(SC-010)

**Constraints**: No write to the machine by any path; no game code run; no per-game code
or data; no `unsafe`; every test from features 001 and 002 passes unchanged; the same
state gives the same answers on every platform.

**Scale/Scope**: Six queries, two adapters, about 280 objects and 1,800 dictionary words
in the largest committed game. One new crate of roughly 1,500 lines, about 150 lines
added to the core, about 200 to the CLI.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core's behaviour is not changed. One read-only module is added. SC-001 requires both walkthrough transcripts and saves to be byte-identical with every query asked at every prompt, and FR-041 that every earlier test passes unchanged. |
| II. Test-First | Each task that adds behaviour is preceded by a failing test, and each query's tests cite the section of `docs/introspection.md` they verify. Feature 002 broke this rule for its fidelity and conformance tests; this plan orders the measurement tests (walkthrough replays, expected exits) before the adapters they measure, and the task list will say so task by task. |
| III. Typed Commands | Nothing here gives the engine input. The interface reports the words that name an exit; turning them into a command is the experience layer's job. |
| IV. Read-Only Published Spec | The contract is `docs/introspection.md`, versioned, written for another implementer (FR-026 to FR-031). Queries take `&Machine`. No game code is run. Adapters are per family; a test fails if any source file of the new crate contains a game's name, serial number or checksum. Every answer can be "unknown". |
| V. Strict Layering | New crate `zmachine-introspect` sits between the core and the hosts. The core does not depend on it or know of it. No `unsafe`. No I/O in the core or the new crate. |
| VI. Story File Licensing | No new story file. Expected answers derived from the games (room names, object names, exits) are committed beside each story file and fall under its recorded license. Zork I's source is read, not committed (research R12). |

**Result**: Pass, with two matters recorded for the project owner in research R12: a
requirement revised in planning (FR-023), and a known weakness in finding the player in
ZIL files that measurement must judge.

**Post-design re-check**: Pass. The design adds no global mode, no quirk flag and no
write path.

## Project Structure

### Documentation (this feature)

```text
specs/003-introspection/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── queries.md        # what each query answers; becomes docs/introspection.md
│   ├── rust-api.md       # the new crate's interface and the core's addition
│   └── cli.md            # zm --inspect, --tree, --dictionary
└── tasks.md              # written by /speckit-tasks
```

### Source Code (repository root)

```text
docs/
└── introspection.md              # NEW: the published specification, version 0.1.0

crates/
├── zmachine-core/
│   └── src/
│       ├── inspect.rs            # NEW: read-only View over memory, shipped image, header
│       └── lib.rs                # Machine::view(&self)
├── zmachine-introspect/          # NEW crate
│   ├── src/
│   │   ├── lib.rs                # Introspector, the six queries
│   │   ├── answer.rs             # Answer, Unknown reasons, Exactness
│   │   ├── table.rs              # objects, properties, attributes, text, dictionary, from bytes
│   │   ├── family.rs             # recognition (R1, R2)
│   │   ├── inform.rs             # the Inform 6 adapter (R3, R6, R7, R9, R10)
│   │   ├── zil.rs                # the ZIL adapter (R4, R5, R9, R10)
│   │   └── json.rs               # the JSON rendering the specification defines
│   └── tests/
│       ├── machine_level.rs      # tree and dictionary on all four story files (SC-007)
│       ├── families.rs           # recognition, and czech and praxix unrecognised (SC-008)
│       ├── zork1.rs              # location, inventory, exits, scope against the game
│       ├── advent.rs             # the same for both builds
│       ├── read_only.rs          # transcripts and saves unchanged (SC-001)
│       ├── robustness.rs         # damaged files (SC-009)
│       └── spec_agreement.rs     # document and implementation agree (SC-011)
├── zmachine-cli/
│   ├── src/lib.rs                # --inspect, --tree, --dictionary
│   └── tests/inspect.rs
└── zmachine-testkit/             # builders gain what the adapter unit tests need

stories/
├── zork1/introspection/          # NEW: expected exits and positions, with a README
└── advent/introspection/         # NEW: the same
```

**Structure Decision**: One new library crate between the core and the hosts, as the
constitution's layering describes. The core gains a small read-only module and nothing
else. The specification is a top-level document, not a feature artefact, because it
outlives this feature.

## How the evidence is gathered

A test driver replays a walkthrough through the core, and at every stop for input asks
all six queries. From the same run it has the game's own text. That gives, with no
hand-written expectations:

- **Location (SC-002)**: after a move, the game prints the room's name on a line of its
  own; that line must equal the reported location's name. The kinds of exception
  (darkness, a vehicle, a move that failed) are listed in the specification and
  recognised by rule.
- **Exits (SC-003)**: when the command was a word of a reported fixed exit and the
  location changed, the new location must be the reported destination.
- **Inventory (SC-005)**: the walkthrough is replayed with `inventory` inserted before
  each command, on a copy of the machine's state that is discarded. The copy is made by
  save and restore into a second machine, so the machine under test is never given the
  extra command.

Expected exits (SC-004) and scope positions (SC-006) are hand-made files, checked by
test.

## Complexity Tracking

No constitution violations to justify.
