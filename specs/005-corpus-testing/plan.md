# Implementation Plan: Corpus Testing of Introspection

**Branch**: `005-corpus-testing` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/005-corpus-testing/spec.md`

## Summary

Measure, across about ten games, how often the introspection interface is right about
exits. At every place a game is brought to, each direction is tried on a throwaway copy
of the game, and what the copy did is set against what introspection reported: leads to
a room, not an exit, conditional, or unknown. A report that was wrong fails the test
until its cause is written in a ledger. The result is a report for each game and one
table across them, in numbers only.

Almost everything needed exists. The core's machine can already be copied with its
random state; introspection already gives the exits and the words for each direction;
the core already says where in its code a game is waiting. What is new is a crate of
testing tools, `zmachine-corpus`, that nothing else depends on, three small text formats
(a manifest of stories, a ledger, reports), and a script that fetches a named story
into a folder the repository ignores. Neither the core, the introspection interface, the
experience layer nor the app is changed. Findings are in [research.md](research.md).

## Strongest objection

**The instrument asks introspection where the player is.** A trial is judged by the room
before and after, and the room comes from the interface being tested. If introspection
is wrong about the room in the same way before and after, a wrong exit report can look
confirmed. Feature 003 tested the room separately against hand-made expectations on two
games; on the other eight there is no such check, and this plan does not add one. The
figures for those games are therefore "consistent with itself", which is weaker than
"right". The plan accepts this, says so in the reports' covering note, and leaves an
independent check of the room (the game's own status line, for example) as later work.

Two lesser ones: the crawl only walks, so on games with no walkthrough it may see little
of the map (the report says how little); and the nine proposed downloads rest on the
archive's presumption of personal use, which is a reading of its terms, not a grant from
each author.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); POSIX shell for the fetch script

**Primary Dependencies**: None new. The corpus crate uses `sha2` 0.10, which two crates'
tests already use. The fetch script uses `curl` and `shasum`, as `scripts/build-ios.sh`
does.

**Storage**: Plain text files in the repository under `stories/corpus/` (manifest,
ledger, reports). Fetched story files under `stories/local/corpus/`, which the
repository already ignores.

**Testing**: `cargo test`. Unit tests on stories built in memory with `zmachine-testkit`;
integration tests on the committed stories; the corpus itself as a test target with its
own `main`, so that an absent story is reported as ignored and not as passed (research
R16).

**Target Platform**: The platforms the project's CI tests on: Linux and macOS.

**Project Type**: A library and a small command-line program of testing tools, in the
existing workspace.

**Performance Goals**: The committed stories within ten minutes on CI; the whole first
corpus within thirty minutes on the owner's Mac (SC-009). Estimated at under a minute a
story in an optimised build (research R12), to be measured.

**Constraints**: Trials exist only in the corpus crate, which nothing depends on
(FR-005). A game being walked is unchanged by trials (FR-001, FR-002). No test and no CI
job downloads anything (FR-020). No fetched story and no text of one is tracked
(FR-019). Reports hold numbers and identifiers only (FR-026). Introspection's rules are
not changed (FR-015). Every earlier test passes unchanged (SC-010).

**Scale/Scope**: About ten stories. One new crate of roughly 1,200 lines with its tests,
one shell script of about 80 lines, three file formats, one CI step.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core is not changed. Games are driven only through its public interface: copy, give a line, run. |
| II. Test-First | Each rule (outcome, verdict, crawl order, budget, ledger, each format's refusals) has a failing test on a story built in memory before its code. The command-prompt rule is proven on the two walkthroughs before anything is built on it (research R3). Where a test cannot be seen to fail first, the task says so and the notes record it. |
| III. Typed Commands | A trial gives the copy one line of text: a direction word from the game's own dictionary (FR-003). Nothing else changes any game's state. |
| IV. Read-Only Introspection | Introspection is used as published and is not changed (FR-015). Trials run on copies; the game being walked is proven unchanged (FR-001, FR-002). Introspection never sees a trial's result (FR-005), so it still answers "unknown" where it cannot say and never guesses. See "Per-game data" below. |
| V. Strict Layering | `zmachine-corpus` depends on introspection and the core; nothing depends on it, and a test proves that. No `unsafe`. The corpus crate reads files and the fetch script uses the network; the core still does no I/O. |
| VI. Story File Licensing | A committed story must have its license text, source and digest beside it, and the manifest check enforces it. Fetched stories and their text are never tracked; reports are numbers only, and a test searches them for the committed stories' names. A test needing an absent story is reported as ignored, not passed. Zork II and III are added only with the owner's approval for each file, with Zork I's caveat. |

**Per-game data.** Principle IV prohibits "per-game code and per-game data files". The
manifest holds, for each story, a seed, sometimes opening lines and a budget, and the
ledger holds known wrong cases for each story. These are test fixtures of the kind the
project already keeps (walkthroughs, seeds, hand-made exit expectations): they are read
only by the testing tools, and introspection's answers do not depend on them. The plan
reads the prohibition as governing what introspection and the layers above it may
consult, which is how features 001 to 004 have read it. The owner has said per-game
metadata for the engine is wanted later; that would need an amendment and is not part
of this feature.

**Development workflow.** The constitution's phases (core, introspection, first demo)
are complete, so work on a further feature may begin.

**Result**: Pass. No deviation.

**Post-design re-check**: Pass. The design added no dependency on the corpus crate, no
query to the introspection interface (research R10 changed the spec instead), and no
network code to the workspace.

## Project Structure

### Documentation (this feature)

```text
specs/005-corpus-testing/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── formats.md      # the manifest, the ledger, a report, the summary
│   └── commands.md     # the corpus program, the fetch script, the tests
└── tasks.md            # written by /speckit-tasks
```

### Source Code (repository root)

```text
crates/
└── zmachine-corpus/            # NEW: testing tools; nothing depends on it
    ├── Cargo.toml              # publish = false; the corpus test target has harness = false
    ├── src/
    │   ├── lib.rs
    │   ├── manifest.rs         # read and check the manifest; find present stories
    │   ├── ledger.rs           # read and check the ledger; match cases
    │   ├── trial.rs            # copy, one word, run, outcome (research R2, R3)
    │   ├── directions.rs       # a story's directions and their words (R4)
    │   ├── verdict.rs          # reported + outcome -> verdict (spec FR-006)
    │   ├── walk.rs             # walkthrough places and the crawl (R5, R6)
    │   ├── report.rs           # a story's report and the summary: write, read, compare
    │   └── main.rs             # the corpus program: check, run, write
    └── tests/
        ├── trial.rs            # outcomes, on stories built in memory
        ├── verdict.rs          # the table of FR-006
        ├── crawl.rs            # order, budget, branches not followed, the same twice
        ├── ledger.rs           # unexplained fails; known passes; stale and causeless fail
        ├── formats.rs          # each format's refusals, with line numbers
        ├── prompts.rs          # the command-prompt rule gives the known counts (R3)
        ├── untouched.rs        # walkthrough transcripts identical with trials (SC-002)
        ├── no_text.rs          # reports by grammar; committed stories' names by search
        ├── crate_rules.rs      # no crate depends on zmachine-corpus; no unsafe
        ├── check.rs            # the manifest and ledger in the repository are well formed
        └── corpus.rs           # harness = false: one line for each story (R16)

scripts/
└── fetch-corpus.sh             # NEW: fetch a named story, or propose one

stories/
├── corpus/                     # NEW
│   ├── manifest.txt
│   ├── ledger.txt
│   ├── README.md               # what the figures mean, and their limits
│   └── reports/
│       ├── <id>.txt
│       └── summary.txt
├── zork2/                      # NEW, with the owner's approval: zork2.z3, LICENSE, SOURCE.md
├── zork3/                      # NEW, the same
└── local/corpus/               # ignored already: fetched stories

.github/workflows/ci.yml        # the corpus runs with `cargo test --workspace`; one step
                                # added to run it optimised if the plain run is slow
Cargo.toml                      # the new member
```

**Structure Decision**: One new crate in the existing workspace, and data under
`stories/corpus/`. The crate is a library so that its tests, its program and the corpus
test target share one implementation. Feature 003's test helpers in
`crates/zmachine-introspect/tests/common` are left as they are (research R15).

## Order of work

1. **Prove the instrument on what is known.** The command-prompt rule on both
   walkthroughs (research R3); the trial and its outcomes; that trials leave a
   walkthrough's transcript untouched. If the prompt rule fails on Advent, stop and
   revisit it.
2. **User Story 1**: verdicts and a report for Zork I along its walkthrough.
3. **User Story 2**: the ledger. Zork I's and Advent's wrong cases are read one by one
   and given causes; none is entered without one.
4. **User Story 3**: the crawl and its budget; measure the time (research R12).
5. **User Story 4**: the manifest check, the fetch script, the skip. Zork II and III are
   fetched and recorded, each on the owner's approval; then the proposed stories, each
   on the owner's approval, replacing any that cannot be used.
6. **User Story 5**: the summary, and `stories/corpus/README.md` saying what the figures
   mean and what they do not (the objection above).

## Decisions for the project owner

Made in research and open to being overruled before tasks are written:

1. **No total of a story's rooms** (research R10): the spec was amended to report rooms
   visited and rooms named but not reached, since a total would need a new query in the
   published interface.
2. **Nine stories are proposed for fetching** (research R14), three of the ZIL family
   and six of the Inform family, each needing approval before it is downloaded.
3. **The digest of a fetched story is recorded at its first fetch.** There is no
   published digest to check the first download against; what is checked afterwards is
   that the file never changes.
4. **The room is taken from introspection** (the objection above).

## Complexity Tracking

No violation of the constitution to justify.
