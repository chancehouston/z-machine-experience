# Implementation Plan: Z-Machine Versions 5 and 8

**Branch**: `002-v5-v8` | **Date**: 2026-11-11 (revised after independent review) | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-v5-v8/spec.md`

## Summary

Extend the version 3 interpreter to versions 5 and 8 without changing how version 3
behaves. Every difference between versions is put behind one `Version` value fixed at
load. Version 5 adds 35 instructions and changes what eleven opcode numbers mean, and
brings a new object table and dictionary
format, an upper window the game draws in, text styles, single-key input, undo, output
redirected into memory, and save and restore that store a result. Version 8 differs from
5 only in two multipliers and a size limit.

Fidelity is proven as before: per-instruction tests citing the Standard, the conformance
programs (praxix, and czech for both versions), and a full walkthrough of a real game,
Advent, whose transcript must match Frotz 2.55 byte for byte in both versions. The upper
window, which no transcript records, is tested directly against the Standard. Evidence
is in [research.md](research.md).

## Revision after review

Two independent reviews found the first version of this plan was not ready for task
writing. The main changes:

- **Version 3 is kept unchanged by not touching the existing interface**, and the three
  tests that must change are named (research R14). The first draft's interface changes
  would have altered about twenty-five tests while claiming to alter none.
- **The upper window is a canvas the size of the screen, with an overlay.** Inform's
  boxed quotations are drawn in a window that is shrunk again before input; a grid of
  exactly the window's height would have lost them (screen contract).
- **Undo is built with the instruction set, not last.** praxix requires it, and Advent's
  library records an undo position every turn (R12).
- **Undo keeps 25 positions**, which is what dfrotz does; its manual says 20 (R5).
- **The transcript rules for version 5 are written down** as a contract of their own,
  including that buffering off changes them.
- **Scripted keypresses follow dfrotz exactly**, including its escapes for special keys.
- **The screen contract cites the right sections.** The first draft cited the version 3
  screen model throughout, and gave `set_font` the wrong result.
- **The full instruction list is in the data model**: 98, of which 35 are new.

## Technical Context

**Language/Version**: Rust 1.99 stable, edition 2024

**Primary Dependencies**: None for the core or the testkit. The CLI depends only on the
core. `sha2` remains a development dependency.

**Storage**: None in the core. Undo positions are held in memory. The CLI writes
transcript and Quetzal save files as now.

**Testing**: `cargo test`, as in the first feature: unit tests inside modules,
integration tests through the public API, conformance programs, a walkthrough against
committed reference artefacts, robustness tests on damaged files, and ignored tests that
regenerate the artefacts with dfrotz 2.55.

**Target Platform**: macOS and Linux. The core stays free of anything that would block
iOS and Android targets.

**Project Type**: Library plus CLI, in the existing Cargo workspace

**Performance Goals**: Each full Advent walkthrough in under five seconds in a release
build (SC-009)

**Constraints**: Version 3 behavior and every version 3 test's expectations unchanged;
no I/O and no `unsafe` in the core; identical output for the same story file, script and
seed on every platform; only redistributable story files committed

**Scale/Scope**: 35 new instructions and eleven opcode numbers whose meaning changes;
three story-file formats; one new output model; a screen canvas; undo

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
| --- | --- | --- |
| I. Faithful Core | Standard 1.1 is the definition; czech and praxix pass; transcript matches Frotz; divergences ledgered; no global mode; deterministic | Pass. praxix, deferred from the first feature, is delivered here. Reporting timed input, colour and sound as unavailable is permitted by the Standard, not a divergence. The interpreter number and version letter are host settings because the Standard leaves them to the interpreter (R8). |
| II. Test-First | Failing test before implementation; every instruction test cites the Standard; tests are not weakened | Pass, with the three test changes named in Complexity Tracking. The testkit learns versions 5 and 8 first, with vector tests, so instruction tests can be written before the instructions. The per-version coverage table is created in the foundations and grows with each instruction, so a missing test is caught when the instruction is added and not at the end. |
| III. Typed Commands | State changes only through input, save, restore, restart | Pass. Undo is the game's own instruction acting on positions the game recorded. A keypress is input. Nothing new lets a host alter state. |
| IV. Introspection | Not in this feature | Not applicable. The upper window is what the game drew, not a view of its internals. |
| V. Strict Layering | Core has no UI, no I/O, no `unsafe`; knows nothing of the CLI | Pass. Styles are data on text; the terminal escape codes that render them are in the CLI. |
| VI. Story File Licensing | Only redistributable files committed, each with license, source and digest beside it | Pass. praxix is public domain. Advent may be freely redistributed unmodified and not for profit (R1), which permits committing it as a test fixture and does not permit bundling it in a sold app; the owner accepted it on those terms. |

**Post-design re-check**: No change.

## Project Structure

### Documentation (this feature)

```text
specs/002-v5-v8/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── core-api.md         # what changes in the core's interface
│   ├── screen-model.md     # the upper window and styles
│   ├── cli.md              # what changes in zm
│   └── transcript-format.md  # transcript rules 13 to 21
├── checklists/requirements.md
└── tasks.md                # created by /speckit-tasks
```

### Source Code (repository root)

Changes to the existing layout. Files not listed are untouched or gain only tests.

```text
crates/
├── zmachine-core/src/
│   ├── version.rs          # NEW: Version and everything that differs by version
│   ├── header.rs           # versions 5 and 8 header fields; what the interpreter writes
│   ├── decode.rs           # extended form, eight operands, per-version opcode tables
│   ├── text.rs             # nine Z-character keys, alphabet and Unicode tables
│   ├── objects.rs          # the version 4+ object table layout
│   ├── dictionary.rs       # version 5 buffers; tokenise against any dictionary
│   ├── exec.rs             # call variants, catch and throw, argument counts
│   ├── ops_io.rs           # styles, memory stream, key input, version 5 read
│   ├── ops_table.rs        # NEW: copy_table, scan_table, print_table, encode_text
│   ├── screen.rs           # NEW: the upper window grid and styled runs
│   ├── undo.rs             # NEW: undo positions
│   └── quetzal.rs          # unchanged format; version decides the resume rule
├── zmachine-testkit/src/   # new types for version 5 and 8 images beside the existing
│                           #   ones; version 3 images byte for byte as before
└── zmachine-cli/src/
    ├── lib.rs              # key input from scripts, upper window display
    └── term.rs             # NEW: terminal rendering of styles
stories/
├── advent/                 # advent.inf, advent.z5, advent.z8, license text, SOURCE.md
│   └── fidelity/           # walkthrough, seed, reference transcripts, saves
├── praxix/                 # praxix.inf, praxix.z5, SOURCE.md
└── czech/                  # adds czech.z5, czech.z8, czech.out5, czech.out8
```

**Structure Decision**: No new crates. Four new modules in the core, each for something
version 3 did not have.

## Implementation Approach

The whole version 3 suite is run after every task in steps 2 to 4 and must stay green.

1. **Setup**: commit Advent, praxix and the czech builds with their records. Run praxix
   and czech under dfrotz to record what a pass looks like.
2. **Foundations**: `Version`; the per-version coverage table; `load_with` and settings;
   the testkit for versions 5 and 8 with vector tests checked against Advent's own tables;
   header, decoder, text, objects and dictionary made version-aware.
3. **Evidence that needs dfrotz**: the version 5 transcript probes and the screen probes,
   built with the new testkit and recorded as fixtures before the code that must match
   them.
4. **Walkthrough** (alongside steps 5 to 8; needs only dfrotz and step 1): written against
   Frotz until Advent is won, with the `version` command in it; then the reference
   transcripts, the undo script and the save points are generated.
5. **Story 1, play a version 5 game**: the 35 new instructions including undo, the version
   5 forms of calls, reads and saves, the memory stream, styled output, key input.
6. **Story 2, version 8**: the multipliers and size limit, and a story file large enough
   to need them; a test that the two Advent builds give the same text.
7. **Story 3, the screen**: the canvas, overlay, cursor and styles; showing them in `zm`.
8. **Story 4, conformance**: praxix, and czech for versions 5 and 8.
9. **Story 5, match Frotz on Advent** in both versions.
10. **Story 6, undo and saves against Frotz**: the undo script, save interchange.
11. **Polish**: thirty named malformed version 5 and 8 files, robustness on damaged ones,
    README.

## Risks

- **Advent's license is conditional** (R1). The owner has accepted it as a test fixture.
  It is not cleared for bundling in a shipped app, so a different game is needed there.
- **A winning Advent walkthrough at one seed** has to cope with wandering dwarves, the
  pirate and a lamp with a limited life. It is written in sections with a stop for the
  owner if a section cannot be completed.
- **Version 3 must not change.** The risk is highest in the decoder and the object table,
  which are rewritten to be version-aware. The version 3 suite, including the Zork I
  transcript match, is the guard and is run after each foundational task.
- **The upper window has no byte-for-byte oracle.** It is tested against the Standard,
  with one optional cross-check against dfrotz's screen (R11).
- **The screen contract makes choices where the Standard is silent**, and some differ
  from Frotz. Each is marked in the contract. A game that depends on Frotz's choice would
  look different, though its transcript would not.
- **Emitting styled runs touches the code that produces every byte of version 3 output.**
  The fifteen version 3 transcript probes and the Zork I match guard it.

## Complexity Tracking

| Deviation | Why | Simpler alternative rejected because |
| --- | --- | --- |
| Three version 3 tests have their expectations changed (Principle II forbids weakening tests; spec FR-003 lists the exceptions) | Two assert that a version 5 file is refused, which this feature exists to reverse; they switch to version 4, which is still refused, so they still test refusal. One asserts the decoder knows exactly 68 instructions; it becomes one table per version with the version 3 table unchanged, so it is stricter than before | Leaving them unchanged is impossible: they assert the absence of this feature |
| The header reports different capabilities from the reference interpreter (Principle I makes Frotz the oracle) | The project owner chose styles available and timed input unavailable; dfrotz reports the reverse. The Standard permits either | Copying dfrotz's header would tell games that timed input works and styles do not, both false for this interpreter |
| A transcript file name is asked for once; Frotz asks each time in version 5 | The Standard says once (§7.1.1.2), and the Standard outranks Frotz | Following Frotz would contradict the Standard for no gain; comparison scripts start one transcript |
