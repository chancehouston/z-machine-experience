# Implementation Plan: Faithful Z-Machine Version 3 Core

**Branch**: `001-v3-core` | **Date**: 2026-11-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-v3-core/spec.md`

## Summary

Build a Z-machine version 3 interpreter as a dependency-free Rust library, plus a small
command-line host. The library is a resumable state machine: the host calls `run`, gets
back an event (input wanted, save, restore, finished, fault, budget exhausted), answers
it, and calls `run` again. Fidelity is proven three ways: per-instruction tests citing the
Standard, the czech conformance program, and a full Zork 1 walkthrough whose transcript
matches dfrotz 2.55 byte for byte. To make the last possible, the seeded random number
generator reproduces Frotz's sequence. Details and evidence are in
[research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.99 stable, edition 2024

**Primary Dependencies**: None for the core. The CLI depends only on the core. `sha2` as a
development dependency of the fidelity tests.

**Storage**: None in the core. The CLI stores saves as Quetzal files on disk.

**Testing**: `cargo test`. Unit tests build story images in memory with a test-support
builder. Integration tests run czech and the Zork 1 walkthrough against dfrotz 2.55.

**Target Platform**: macOS and Linux for this feature. The core must stay free of anything
that would block iOS and Android targets later.

**Project Type**: Library plus CLI, in one Cargo workspace

**Performance Goals**: Full Zork 1 walkthrough in under five seconds on a development
laptop (SC-005)

**Constraints**: No I/O and no `unsafe` in the core; bit-identical output for the same
story file, script and seed on every platform; only story files licensed for
redistribution are committed

**Scale/Scope**: The version 3 instruction set (roughly 70 instruction forms), Quetzal
save and restore, one CLI with two modes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
| --- | --- | --- |
| I. Faithful Core | Standard 1.1 is the definition; czech passes; Zork 1 matches dfrotz; divergences ledgered; no global mode; deterministic | Pass. Matching Frotz's random sequence is permitted by the Standard, not a divergence (research R2). Ledger format in R10. |
| II. Test-First | Failing test before implementation; every instruction test cites the Standard | Pass. The in-memory story builder (R9) exists so tests can be written first. Tasks order tests before code. |
| III. Typed Commands | State changes only through input text, save, restore, restart | Pass. The event interface offers no other way to alter state. |
| IV. Introspection | Not in this feature | Not applicable. The core keeps its state readable so the next feature can add queries without changing execution. |
| V. Strict Layering | Core has no UI, no I/O, no `unsafe`; core does not know about the CLI | Pass. `#![forbid(unsafe_code)]` and an empty dependency list in the core. Wrapping lives in the CLI (R6). |
| VI. Story File Licensing | Only redistributable story files committed, each with license, source and digest; others local and skipped when absent | Pass. Zork 1 (MIT) is committed with its license record; czech (no stated license) is fetched into a gitignored directory. |

**Note on Principle VI**: Research R3 found that Zork 1 was relicensed under MIT in
November 2025. The constitution was amended to version 1.1.0 on 2026-11-11 to decide by
each file's license, and this plan follows the amended text.

**Post-design re-check**: No change. The design adds no layer, dependency or mode that the
gates above did not already consider.

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
└── tasks.md             # created by /speckit-tasks
```

### Source Code (repository root)

```text
Cargo.toml                      # workspace
crates/
├── zmachine-core/
│   ├── Cargo.toml              # no dependencies
│   ├── src/
│   │   ├── lib.rs              # public API: Machine, Event, Fault, StatusLine
│   │   ├── header.rs           # header fields, load-time validation
│   │   ├── memory.rs           # dynamic/static/high memory, bounds rules
│   │   ├── text.rs             # ZSCII, Z-character decode/encode, abbreviations
│   │   ├── objects.rs          # object tree, attributes, properties
│   │   ├── dictionary.rs       # dictionary lookup, input tokenisation
│   │   ├── decode.rs           # instruction decoding
│   │   ├── exec.rs             # instruction execution, calls, stack
│   │   ├── random.rs           # Frotz-compatible generator
│   │   ├── quetzal.rs          # save and restore
│   │   └── testkit.rs          # in-memory story builder (test support)
│   └── tests/                  # per-area tests, each citing the Standard
└── zmachine-cli/
    ├── Cargo.toml              # depends on zmachine-core
    ├── src/
    │   ├── main.rs             # argument handling, exit status
    │   ├── driver.rs           # run loop answering core events
    │   └── dumbterm.rs         # wrapping and transcript conventions
    └── tests/
        ├── czech.rs
        ├── determinism.rs
        ├── fidelity_zork1.rs
        ├── save_restore.rs
        └── malformed.rs
tests/fidelity/
├── zork1.walkthrough           # our command script
└── zork1.reference.txt         # dfrotz transcript for the pinned seed and width
docs/
└── divergences.md              # the ledger
stories/
├── zork1/
│   ├── zork1.z3                # from historicalsource/zork1, MIT
│   ├── LICENSE                 # that repository's license text
│   └── SOURCE.md               # URL, revision, SHA-256, release and serial
└── local/                      # gitignored; czech and any non-redistributable file
scripts/
└── fetch-stories.sh            # downloads czech into stories/local/
.github/workflows/ci.yml        # macOS and Linux
```

**Structure Decision**: One workspace with two crates. The core is a library with no
knowledge of the CLI; the CLI is the only place that touches files, the terminal or the
clock. Later features add crates beside these (C bindings, introspection) without
changing the core's manifest.

## Implementation Approach

Work proceeds in the order of the spec's user stories, each ending in a test that passes:

1. **Foundations**: workspace, the test-support story builder, header and memory, the
   instruction decoder. Nothing runs yet, but every later test depends on these.
2. **Story 1, play from the command line**: instruction execution, text, objects,
   dictionary and tokenisation, the event loop, the interactive CLI.
3. **Story 2, scripted and repeatable**: script mode, the Frotz-compatible generator, the
   dumb-terminal formatter, the determinism test.
4. **Story 3, czech**: fetch script, czech in the suite, fixes for whatever it finds.
5. **Story 4, match dfrotz on Zork 1**: characterize dfrotz output, write the walkthrough,
   commit the reference transcript, the comparison with first-difference reporting, the
   ledger, CI.
6. **Story 5, save, restore, restart**: Quetzal, the save and restore events, the five
   save points in the walkthrough.

The walkthrough is written against dfrotz at the pinned seed before our interpreter can
finish the game, so it serves as the specification our interpreter then has to meet.

## Risks

- **dfrotz output conventions are undocumented** (input echo, status line, end of input).
  Mitigation: the first fidelity task observes and records them before any formatter code
  is written. If dfrotz emits something that cannot be reproduced from the event stream,
  that is raised with the project owner, not worked around.
- **czech may lack a version 3 build or conflict with Standard 1.1** (R4). Mitigation:
  compile from source with Inform 6, and ledger conflicts.
- **Writing a winning Zork 1 walkthrough for one seed is real work**: the thief and combat
  outcomes depend on the seed. Mitigation: build it incrementally against dfrotz; choose
  the seed for a cooperative run.
- **Frotz's generator is described from a reading of its source**, not yet observed (R2).
  Mitigation: a dedicated comparison test before the walkthrough depends on it.

## Complexity Tracking

No constitution violations to justify.
