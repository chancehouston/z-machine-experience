# Z-Machine Experience Constitution

"Z-Machine Experience" is a working name. Renaming the project is a PATCH amendment.

## Core Principles

### I. Faithful Core

The core is a Z-machine interpreter in Rust: text in, text out, nothing else.

- The Z-Machine Standard 1.1 is the definition of correct behavior.
- Frotz (dumb-terminal build, fixed screen width) is the transcript oracle. A scripted
  Zork 1 walkthrough MUST produce a transcript that matches Frotz byte for byte.
- The core MUST pass the czech and praxix test programs.
- Where Frotz and the Standard disagree, the core follows the Standard. Each such case
  MUST be recorded in a divergence ledger (`docs/divergences.md`) citing the Standard
  section, and the transcript comparison MUST exclude only the ledgered bytes.
- If a real story file depends on a non-standard behavior, it is supported through a
  named quirk flag that is off by default, individually tested, and listed in the ledger.
  There is no global "Frotz mode" or "strict mode".
- The core MUST be deterministic: the same story file, input script and random seed
  produce the same output on every platform.

Rationale: the interpreter is the foundation the product stands on. A single, citable
definition of correct keeps the test matrix small; a second global mode would double it
for the sake of a handful of edge cases.

### II. Test-First (NON-NEGOTIABLE)

- A failing test MUST exist before the implementation it covers is written.
- Every opcode, and every introspection query, has tests that cite the section of the
  spec they verify.
- A change that alters the Frotz transcript comparison or the czech/praxix results MUST
  NOT be merged unless the difference is ledgered under Principle I.
- Tests MUST NOT be weakened, skipped or deleted to make a change pass.

Rationale: fidelity is only meaningful if it is measured continuously.

### III. The Engine Takes Typed Commands

- The only way to change game state is to give the core the text a player could have
  typed, plus save, restore and restart.
- Taps and gestures are translated into command text by the experience layer
  (tap lantern → "take lantern"; tap exit → "north"). The translated command MUST be
  visible to the user.
- Typing MUST always remain available. Anything reachable by touch MUST also be
  reachable by typing.

Rationale: games behave exactly as their authors wrote them, and every session is
reproducible from its command script.

### IV. Introspection Is a Read-Only, Published Spec

- The introspection interface exposes live internals: object tree, player location,
  objects in scope, exits, inventory, dictionary.
- It MUST NOT write to Z-machine memory or otherwise alter execution. Running with
  introspection on or off MUST produce identical transcripts.
- The interface is defined by a Markdown spec in this repository, written so another
  interpreter could implement it. The spec is the contract; our core is one
  implementation of it.
- Location, exits and scope are library conventions, not Z-machine concepts. They are
  provided by adapters per compiler family (Infocom/ZIL, Inform 6 library). Per-game
  code and per-game data files are prohibited.
- Every query MUST be able to answer "unknown". When an adapter cannot answer, the UI
  degrades to typing; it MUST NOT guess.

Rationale: a generic, honest interface is what makes the experience layer portable to
other games and, eventually, other interpreters.

### V. Strict Layering

Dependencies point one way: shells → experience layer → introspection interface → core.

- The core MUST NOT depend on, or know about, any UI, platform or the experience layer.
- The core performs no I/O of its own. Files, screen, time and randomness are supplied
  by the host through explicit interfaces.
- `unsafe` Rust is permitted only in the C-binding crate, and each use carries a comment
  stating the invariant it relies on.
- Logic that both platforms need lives in Rust. The SwiftUI and Jetpack Compose shells
  contain presentation only.

Rationale: one tested implementation of behavior, two thin native shells.

### VI. Story File Licensing

- Commercial story files (including Zork) MUST NOT be committed to the repository or
  included in any build artifact. They are used locally for development and testing.
- Tests that need a commercial story file locate it through a local path and are
  reported as skipped, not passed, when it is absent.
- Only story files whose licenses permit redistribution may be bundled in a shipped app.
  Each bundled file's license and source are recorded in the repository.

Rationale: Zork remains Activision's property; shipping it is not ours to decide.

## Scope & Technology Constraints

- Z-machine versions 3, 5 and 8 are supported. Version 6 is out of scope. Versions 1, 2,
  4 and 7 are out of scope until an amendment adds them.
- The core, introspection implementation and experience logic are written in Rust and
  exposed to the shells through C bindings.
- Shells are SwiftUI (iPhone) and Jetpack Compose (Android).
- The introspection spec stays in this repository as Markdown until it stabilizes.
  Moving it to its own repository is a MINOR amendment.

## Development Workflow

- Work follows Spec Kit: specify → plan → tasks → implement. Every plan includes a
  Constitution Check against the principles above.
- Phases are delivered in order: (1) faithful core, (2) introspection interface,
  (3) first demo: Zork boots, room text shows, and a tappable compass of exits works end
  to end on a phone. Work on a later phase does not begin until the earlier phase's
  acceptance tests pass.
- When a requirement is unclear, ask and wait for an answer; do not guess.
- Reviews and proposals lead with the strongest objection, and state whether agreement
  is positive agreement or merely the absence of a found fault.

## Governance

- This constitution takes precedence over other project documents, including the
  original handoff document, where they conflict.
- Amendments are made by editing this file through `/speckit-constitution`, with the
  project owner's approval, and recording the change in the version line below.
- Versioning is semantic: MAJOR for removing or redefining a principle, MINOR for adding
  a principle or section or materially expanding one, PATCH for clarifications.
- Every plan and every review MUST verify compliance. A deviation MUST be written down
  in the plan with its justification and the simpler alternative that was rejected.

**Version**: 1.0.0 | **Ratified**: 2026-11-11 | **Last Amended**: 2026-11-11
