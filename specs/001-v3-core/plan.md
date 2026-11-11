# Implementation Plan: Faithful Z-Machine Version 3 Core

**Branch**: `001-v3-core` | **Date**: 2026-11-11 (revised after independent review) | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-v3-core/spec.md`

## Summary

Build a Z-machine version 3 interpreter as a dependency-free Rust library, plus a small
command-line host. The library is a resumable state machine: the host calls `run`, gets
back an event (line wanted, save, restore, transcript, finished, fault, budget
exhausted), answers it, and calls `run` again. Fidelity is proven three ways:
per-instruction tests citing the Standard, the czech conformance program, and a full
Zork 1 walkthrough whose transcript file matches the one Frotz 2.55 writes, byte for
byte. To make the last possible, the seeded random number generator reproduces Frotz's
sequence. Details and evidence are in [research.md](research.md).

## Revision after review

Two independent reviews found the first version of this plan was not executable. The
main changes:

- **Comparison method.** The reference is now Frotz's transcript file with wrapping off,
  not dfrotz's standard output, which turned out to be a diff of a 24-row screen
  (research R1). No screen emulation is needed.
- **Save and restore test.** A save does not carry random state, so "restore and match
  the uninterrupted run" cannot hold. The test is now: a fresh run that restores a save
  and plays a fixed continuation matches Frotz doing the same, for saves written by
  either interpreter (R7).
- **czech is committed**, compiled for version 3 from its source; its license permits it
  (R4).
- **Tests of internals live inside the crate**; a separate testkit crate builds images;
  the CLI gains a library target (R8).
- **Object 0 operations are tolerated with a diagnostic**, not fatal (R12).
- **Risky unknowns come first**: the generator comparison and the walkthrough start in
  the first two phases.

## Technical Context

**Language/Version**: Rust 1.99 stable, edition 2024

**Primary Dependencies**: None for the core or the testkit. The CLI depends only on the
core. `sha2` as a development dependency.

**Storage**: None in the core. The CLI writes transcript and Quetzal save files.

**Testing**: `cargo test`. Unit tests inside modules for internals; integration tests
through the public API; czech; Zork 1 against committed reference artefacts; ignored
tests that regenerate those artefacts with dfrotz 2.55.

**Target Platform**: macOS and Linux for this feature. The core must stay free of
anything that would block iOS and Android targets later.

**Project Type**: Library plus CLI, in one Cargo workspace

**Performance Goals**: Full Zork 1 walkthrough in under five seconds in a release build
(SC-005)

**Constraints**: No I/O and no `unsafe` in the core; identical output for the same story
file, script and seed on every platform; only story files licensed for redistribution
are committed

**Scale/Scope**: The 68 instructions of version 3, Quetzal save and restore, output
streams 1 and 2, one CLI with two modes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
| --- | --- | --- |
| I. Faithful Core | Standard 1.1 is the definition; czech and praxix pass; Zork 1 transcript matches Frotz; divergences ledgered; no global mode; deterministic | Pass, with one recorded deviation: praxix is deferred (see Complexity Tracking). Matching Frotz's random sequence is permitted by the Standard (R2). |
| II. Test-First | Failing test before implementation; every instruction test cites the Standard | Pass. Types are stubbed first so tests compile and fail for the right reason; the testkit's encoders have their own vector tests. |
| III. Typed Commands | State changes only through input text, save, restore, restart | Pass. The event interface offers no other way. |
| IV. Introspection | Not in this feature | Not applicable. |
| V. Strict Layering | Core has no UI, no I/O, no `unsafe`; core does not know about the CLI | Pass. Checked by a test of the core's manifest and source. |
| VI. Story File Licensing | Only redistributable files committed, each with license, source and digest beside it | Pass. Zork 1 and czech each have a record; reference transcripts and saves sit beside Zork 1's. The uncertainty over the compiled Zork file is stated in R3 and was accepted by the owner. |

**Post-design re-check**: No change after the revision.

## Project Structure

### Documentation (this feature)

```text
specs/001-v3-core/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── core-api.md
│   ├── cli.md
│   └── transcript-format.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml                          # workspace
crates/
├── zmachine-core/
│   ├── Cargo.toml                  # no dependencies; dev-dependency on the testkit
│   ├── src/
│   │   ├── lib.rs                  # public API: Machine, Event, Fault, StatusLine, ...
│   │   ├── header.rs               # header fields, load validation
│   │   ├── memory.rs               # memory map and write rule
│   │   ├── text.rs                 # ZSCII, Z-characters, abbreviations
│   │   ├── objects.rs              # object tree, attributes, properties
│   │   ├── dictionary.rs           # lookup and tokenisation
│   │   ├── decode.rs               # instruction decoding
│   │   ├── exec.rs                 # execution loop, calls, stack, events
│   │   ├── ops_math.rs             # arithmetic and logic instructions
│   │   ├── ops_object.rs           # object instructions
│   │   ├── ops_io.rs               # output, streams, status line, read
│   │   ├── random.rs               # Frotz-compatible generator
│   │   └── quetzal.rs              # save and restore
│   └── tests/                      # public-API tests: events.rs, streams.rs,
│                                   #   transcript.rs, random.rs, save.rs,
│                                   #   robustness.rs, coverage.rs; fixtures/ from dfrotz
├── zmachine-testkit/
│   ├── Cargo.toml                  # no dependencies
│   └── src/lib.rs                  # in-memory story image builder
└── zmachine-cli/
    ├── Cargo.toml                  # depends on zmachine-core
    ├── src/
    │   ├── lib.rs                  # driver, wrapping, file handling, ledger parsing
    │   └── main.rs                 # argument handling, exit status
    └── tests/                      # play.rs, script.rs, czech.rs, fidelity_zork1.rs,
                                    #   save_restore.rs, malformed.rs, ledger.rs
stories/
├── zork1/
│   ├── zork1.z3                    # historicalsource/zork1, MIT
│   ├── LICENSE
│   ├── SOURCE.md                   # URL, commit, SHA-256, release, serial
│   └── fidelity/
│       ├── walkthrough.txt         # our command script
│       ├── seed.txt
│       ├── reference.scr           # Frotz transcript of the walkthrough
│       └── saves/                  # reference saves, continuation scripts and transcripts
├── czech/
│   ├── czech.inf, README.txt       # source and license notice
│   ├── czech.z3                    # compiled once with Inform 6
│   ├── czech.out3                  # expected output from the archive
│   └── SOURCE.md
└── local/                          # gitignored
docs/
└── divergences.md                  # the ledger
scripts/
└── regenerate-references.sh        # rebuilds fidelity artefacts with dfrotz
.github/workflows/ci.yml            # macOS and Linux
```

**Structure Decision**: One workspace, three crates. The core knows nothing of the
others. Instruction groups are split across `ops_*.rs` files so that work on them does
not collide in one file.

## Implementation Approach

1. **Setup**: workspace, committed story files with their records, the ledger.
2. **Foundations and early evidence**: testkit with vector tests, stub types, header,
   memory, decoder; and, needing only dfrotz, the generator fixtures and the remaining
   transcript-format observations.
3. **Walkthrough** (runs alongside steps 4 to 6; needs only dfrotz): written in sections
   against Frotz until the game is won, then the reference transcript is generated.
4. **Story 1, play from the command line.**
5. **Story 2, scripted and repeatable**: the generator, the transcript stream, script
   mode.
6. **Story 3, czech.**
7. **Story 4, match Frotz on Zork 1**: the comparison, the ledger, CI.
8. **Story 5, save, restore, restart**: Quetzal, both directions against Frotz.
9. **Polish.**

## Risks

- **A winning walkthrough at one seed is real work.** It is written in sections with a
  stop for the owner if a section cannot be completed, and it starts early.
- **czech's expected output may contain interpreter-specific lines.** The task classifies
  them before the comparison is written.
- **Zork 1 may perform object 0 operations on the route**, in which case Frotz's warning
  text appears in the reference. Checked early with `dfrotz -Z 2`.
- **The Homebrew bottle is assumed to match the 2.55 source that was read.** The
  regeneration tests record and check the version string.
- **Legal reading of the Zork license for the compiled file** (R3), accepted by the owner.

## Complexity Tracking

| Deviation | Why | Simpler alternative rejected because |
| --- | --- | --- |
| Principle I requires czech and praxix; this feature delivers czech only | praxix is a version 5 program and cannot run on a version 3 interpreter | Building version 5 first would put the headline Zork 1 test behind a larger feature. praxix is the acceptance test of the next feature (versions 5 and 8). |
