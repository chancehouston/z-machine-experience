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
  provided by adapters per compiler family (Infocom/ZIL, Inform 6 library).
- Per-game code is prohibited. Per-game data is permitted only as a **game pack**, and a
  game pack MUST keep every one of these rules:
  - **Identity.** A pack names exactly one story file, by SHA-256 digest, release and
    serial number, and applies only when all three match.
  - **Facts only.** A pack's entries come from a fixed vocabulary that the introspection
    spec defines: which attribute means "open", which means "lit", the property and
    attribute numbers a family's rules assume, a flag whose stored value is not to be
    trusted, and the like. A pack MUST NOT hold conditions, scripts or any text of a
    game.
  - **Published.** The pack format is part of the introspection spec, written so another
    interpreter could read the same packs.
  - **Verified.** Every entry MUST have a test against the story file it names. An entry
    that bears on exits MUST be checked by corpus trials, and an entry that a trial
    contradicts fails the build.
  - **Marked.** An answer that depended on a pack MUST say so.
  - **Absent means unchanged.** Without a pack, every answer is what the introspection
    spec defines without one, including "unknown".
- Authored presentation content (art, map layouts, icons) is not a game pack and is not
  permitted by this principle. It is to be decided by a separate amendment.
- Every query MUST be able to answer "unknown". When an adapter cannot answer, the UI
  degrades to typing; it MUST NOT guess. A pack supplies a fact; it does not license a
  guess.

Rationale: a generic, honest interface is what makes the experience layer portable to
other games and, eventually, other interpreters. Game packs were admitted in version
2.0.0 because corpus testing showed two things: the same stored fact means different
things from game to game, so a rule per family cannot always read it (in Zork I one
attribute predicted all 57 trials of door exits along the walkthrough, and no rule of
the family says which attribute that is); and a pack's entry can be proven by trials
instead of taken on trust. Publishing the format keeps packs portable; verifying each
entry keeps them honest.

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

- A story file, or a transcript of its text, may be committed or included in a build
  artifact only if its license permits redistribution. Its license text, its source
  (URL and revision) and a SHA-256 digest MUST be recorded beside it in the repository.
- Any other story file MUST NOT be committed or included in any build artifact, and
  neither may transcripts of its text. Such files are used locally for development and
  testing.
- Tests that need a story file that is not in the repository locate it through a local
  path and are reported as skipped, not passed, when it is absent.
- A license on a game's code does not grant its trademarks. Names such as "Zork" MUST
  NOT be used in the shipped app's name, icon or store listing without separate
  permission.

Rationale: Zork I, II and III were relicensed under the MIT License on 2025-11-20, so
their story files may be redistributed with attribution; most other Infocom titles
remain commercial. What decides the matter is each file's license, checked and recorded,
not the game's fame.

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

**Version**: 2.0.0 | **Ratified**: 2026-11-11 | **Last Amended**: 2026-11-12
