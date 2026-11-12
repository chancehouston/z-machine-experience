# Implementation Plan: Introspection Interface

**Branch**: `003-introspection` | **Date**: 2026-11-11 (revised after independent review) | **Spec**: [spec.md](spec.md)

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

## Revision after review

Two independent reviews found the first version of this plan was not ready for task
writing: one read it against the spec and constitution, the other re-tested every claim
in the research against the story files. The main changes:

- **Three research rules were wrong and are corrected.** The ZIL recognition rule
  rejected Zork I (its `ROOMS` object carries a direction property); the rule telling
  Inform routines from strings misread two compiler-internal properties and, on the
  version 5 build, classed every routine exit as a refusal; and `compass` is named in the
  library's language file, not its parser (research R1, R2, R7).
- **Inform recognition now checks attribute numbering**, not only property numbering, so
  a game that shifts the attributes is unrecognised and not misread (R1 rule 5).
- **The inventory check is redesigned.** The first version asked a discarded copy of the
  machine, and the core had no way to make one. The core gains `Clone`.
- **The location check no longer reads what the query reads.** For Zork the status line
  comes from the same variable as the answer; the check now uses the room headings the
  game prints.
- **Four requirements, not one, were revised for ZIL scope**, with the project owner's
  agreement, and SC-002 to SC-006 now say what counts as passing.
- **A known gap is recorded as a deviation**, not passed silently: a change of player in
  an Inform game (Complexity Tracking).
- **The family is a fact that goes with each answer, not a seventh query.**
- **"Unknown" has one form everywhere**, for a whole answer or a single field, in Rust
  and in JSON; no field that can be unknown is ever just left out.
- **`--inspect` says when its line is written** and how to tell it from `zm`'s other
  messages.
- **Every success criterion now has a named measurement**, and the specification's
  skeleton is written first so that tests can cite it.

The project owner decided four questions the reviews raised; research R12 records them.

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
state gives the same answers on every platform. The core's public interface gains two
things and changes nothing: `Machine::view` and `Clone for Machine`.

**Scale/Scope**: Six queries, two adapters, about 280 objects and 1,800 dictionary words
in the largest committed game. One new crate of roughly 1,500 lines, about 150 lines
added to the core, about 200 to the CLI.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core's behaviour is not changed. One read-only module is added. SC-001 requires both walkthrough transcripts and saves to be byte-identical with every query asked at every prompt, and FR-041 that every earlier test passes unchanged. |
| II. Test-First | Each task that adds behaviour is preceded by a failing test, and each query's tests cite the section of `docs/introspection.md` they verify. Feature 002 broke this rule for its fidelity and conformance tests. Here the order is fixed (see "Order of work"): the specification's skeleton, then the walkthrough driver and the hand-made expectations, then the failing tests, then the adapters. |
| III. Typed Commands | Nothing here gives the engine input. The interface reports the words that name an exit; turning them into a command is the experience layer's job. |
| IV. Read-Only Published Spec | The contract is `docs/introspection.md`, versioned, written for another implementer (FR-026 to FR-031). Queries take a shared reference. No game code is run. Adapters are per family; a test fails if any source file of the new crate contains a game's name, serial number or checksum. Every answer can be "unknown". **One deviation from "MUST NOT guess"** is recorded under Complexity Tracking. |
| V. Strict Layering | New crate `zmachine-introspect` sits between the core and the hosts. The core does not depend on it or know of it. No `unsafe`. No I/O in the core or the new crate. |
| VI. Story File Licensing | No new story file. Expected answers derived from the games (room names, object names, exits) are committed beside each story file and fall under its recorded license. Zork I's source is read, not committed (research R12). |

**Result**: Pass with one justified deviation (Complexity Tracking), approved by the
project owner on 2026-11-11.

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
│       ├── common/mod.rs         # the walkthrough driver and the game-text checks
│       ├── read_only.rs          # transcripts and saves unchanged (SC-001)
│       ├── robustness.rs         # damaged files (SC-009)
│       ├── speed.rs              # all six queries under 5 ms, ignored unless --release (SC-010)
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

A test driver in the new crate's tests replays a walkthrough through the core, and at
every stop for input asks all six queries. It keeps the game's own text for each turn:
the lower window's output and, for versions 5 and 8, the screen canvas. A stop is a
**command prompt** when the game wants a line and its text ends with `>`; other stops
(yes-or-no questions, keys, file names) are queried for SC-001 and SC-009 but not held
to the checks below, which are about commands.

**Location (SC-002, FR-037)**, by a check that does not read what the query reads:

- *Room headings*: the set of all rooms' printed names is known from the object tree. A
  reply whose first non-empty line is one of them is a room heading. Whenever a reply
  has a room heading, it must be the reported location's printed name.
- *Changes*: whenever the reported room differs from the one at the previous command
  prompt, the reply must have a room heading, or must say the room is dark (Zork I: "It
  is pitch black"; Advent: the location's `lit` is `no` and the status row reads
  "Darkness").
- *Inform only, in addition*: the first row of the canvas, which the library draws from
  its own variable, begins with the reported printed name, or with "Darkness" when `lit`
  is `no`. For Zork I the version 3 status line is drawn from the very variable the
  adapter reads, so it is not used.
- The location must never be unknown at a command prompt.

**Exits (SC-003)**: when the command, cut to the dictionary's word length, is one of the
words of a reported fixed exit and the reported room then changed, the new room must be
the exit's destination. At least 100 such moves must occur in each game, or the test
fails as vacuous.

**Inventory (SC-005)**: at each command prompt the driver clones the machine, gives the
clone the game's inventory command, runs it to its next stop and discards it. The
machine under test is never given the command. The reply is compared with the reported
inventory by two rules: every reported object's printed name occurs in the reply, and the
number of item lines in the reply (lines indented under the heading, not themselves
ending in a colon) equals the number of reported objects, counting contents the game
shows. An empty report must meet the game's empty reply ("You are empty-handed." or
"You're carrying nothing."). A prompt where the inventory is unknown is counted, not
compared; the count must be within SC-005's limit.

**Save files (SC-001)**: the committed save scripts (`saves/make_n.txt`) are run with and
without every query asked at every stop; the save files must be equal byte for byte. The
walkthroughs themselves are compared by transcript.

**Expected exits (SC-004) and scope positions (SC-006)** are hand-made files beside each
story file, each entry naming the walkthrough step at which it holds, since a stored
value can open an exit. They are written from the games' sources before the adapters
exist. Zork I's source is read at the revision `stories/zork1/SOURCE.md` names.

**Speed (SC-010)**: a test, ignored unless run with `--release`, times all six queries
at every prompt of the Advent walkthrough and requires the slowest to be under 5 ms.

**Blind reading (SC-012)**: once the specification is complete, an independent reviewer
with no access to the new crate's source is given the document and the story files,
predicts all six answers at five positions in each game, and the predictions are compared
with `zm --inspect`. The outcome is recorded in tasks.md. If predictions differ, the
document is wrong or unclear and is corrected; the implementation is not bent to it.

## Order of work

The task list must keep this order, so that no test is written after the code it tests:

1. `docs/introspection.md` as a skeleton: every section, numbered, with one sentence
   each. Tests cite these numbers from the start.
2. The core's `view` and `Clone`, each with tests first; the earlier features' tests
   run unchanged.
3. The walkthrough driver, and the hand-made expectations for exits and scope.
4. Machine-level queries (object tree, dictionary): failing tests on all story files,
   then the code.
5. Recognition: failing tests (both games recognised; czech and praxix not; small built
   images that break each rule in turn), then the code.
6. Location, inventory, exits, scope, in that order, each as failing walkthrough and
   unit tests, then the adapter code for both families.
7. Read-only and robustness tests. These can only pass once queries exist, but they are
   written with step 3's driver and are red until step 6 completes.
8. `zm --inspect`, `--tree`, `--dictionary`: failing CLI tests, then the code.
9. The specification filled in, the agreement check, speed, and the blind reading.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Principle IV says the interface "MUST NOT guess". For Inform games the player is taken to be `(self object)`, and a game that makes another object the player is detected only when no variable still holds the original player's number. In the remaining case the inventory, holder and carried part of scope are the original player's, labelled exact. | The library's `player` variable has no value in the file as shipped, so no rule found so far locates it without running code or knowing the library version's layout. Advent, the only Inform game in the repository, never changes the player, so the gap cannot be exercised or measured here. | Labelling every Inform player-dependent answer "approximate" was rejected by the project owner: it would blur the one real approximation (scope) with answers that are right in every game that can be tested. Answering "unknown" always would remove inventory for the whole family. The gap is stated in the published specification as a known limit, to be closed before its version 1. |
