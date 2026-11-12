# Implementation Plan: Game Packs

**Branch**: `006-game-packs` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-game-packs/spec.md`

## Summary

A game pack is a small published text file of facts about exactly one story file. This
feature defines the format in `docs/introspection.md` (version 0.2.0), with two kinds of
entry: the attribute that means a door is open, and a variable whose stored value is not
to be trusted. With a pack in force a door exit is reported as a door, open or closed,
with the room behind it; an exit that hangs on an untrusted variable is reported as
conditional; and each such answer is marked. The corpus proves every entry by trials or
fails. Zork I and Mini-Zork get the first packs, and the Lamplight compass shows a
closed door as closed.

The introspection crate gains a pack parser, a second constructor and three fields on an
exit. The corpus crate gains three reported values, a proof step and a second run with
packs set aside. The experience layer gains one compass state, the C interface one
function, the app one resource and one way of drawing a place. The core is not touched.
Findings are in [research.md](research.md).

## Strongest objection

**Every exits answer changes its written form, for every game, to serve two.** The
interface's own rule is that a field is never left out, so `door`, `state` and `source`
appear on every exit whether or not a pack exists (research R4). No value that was
there changes, and the interface's versioning calls this a minor change. But the JSON
text of every exits answer is different after this feature, the tests that fix that
text are edited, and "absent means unchanged" then rests on reading "unchanged" as
"every earlier field has its earlier value". If the owner reads it more strictly, the
alternative is to leave the three fields out when no pack is in force, which breaks
the specification's rule 2.2 instead. The plan takes the first reading and lists every
test it edits (SC-012).

Three lesser ones:

- **The Mini-Zork pack may not be provable.** Its proof needs a door seen open, which
  needs a few commands that open one (research R12). If none are found, the entry is
  "not proven" and the pack cannot be kept; the feature would then ship one pack.
- **The introspection crate gains its first dependency besides the core**, `sha2`, and
  the app hashes its story at every launch (research R2). The cost is to be measured.
- **Whether a ledger entry's cause "is the entry" is judged by a person.** The check can
  only print such entries for the owner to read (research R10).

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app, as in feature 004

**Primary Dependencies**: `sha2` 0.10 moves from a development dependency of
`zmachine-introspect` to a dependency. Nothing new enters the workspace.

**Storage**: Plain text. Packs under `packs/`; the app's copy in its resources; a second
set of corpus reports under `stories/corpus/reports/nopack/`.

**Testing**: `cargo test`, with stories built in memory by `zmachine-testkit` for each
rule and the committed stories for the proofs; the app's UI tests through
`scripts/test-ios.sh`.

**Target Platform**: Linux and macOS for the Rust workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app, in the
existing workspace.

**Performance Goals**: Every query within feature 003's limit with a pack in force
(SC-011). The app's launch within feature 004's two seconds with the story hashed. The
committed stories' corpus run within ten minutes on CI, with Zork I run twice.

**Constraints**: The core knows nothing of packs (FR-034). No code for one game
(FR-035). No answer changes without a pack in force (FR-013). Transcripts are identical
with and without a pack (FR-014). No query can fail because of a pack (FR-005). No pack
holds text of a game (FR-003). No test downloads a story (FR-020).

**Scale/Scope**: Two packs of one to three entries. About 250 lines in the
introspection crate, 300 in the corpus crate, 60 in the experience layer and the C
interface, 80 in the app, each with its tests; one new section of the published
specification.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

Checked against version 2.0.0, which is proposed in pull request 8 and not yet merged.
**This feature cannot be merged before that amendment is.** Under version 1.x it fails
Principle IV outright.

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core is not changed and does not learn of packs. |
| II. Test-First | Each statement of the format (contracts/pack-format.md, 1 to 26) has a test on a story built in memory, written and seen to fail before its code. Each proof outcome has a test that fails first. Where a test cannot be seen to fail first, the task says so. |
| III. Typed Commands | A closed door on the compass types the direction, like any place. Nothing new changes a game's state. |
| IV. Read-Only Introspection | See the table below, rule by rule. |
| V. Strict Layering | Packs enter at the introspection layer as text from the host. The experience layer passes the text through; the C interface passes a string. No new `unsafe` beyond one checked pointer in the new C function. |
| VI. Story File Licensing | A pack holds identity and numbers, no text; a test enforces the grammar. Mini-Zork's file stays fetched and untracked; its pack's tests are reported as ignored where it is absent. No download. |

**Principle IV's rules for a pack:**

| Rule | How |
| --- | --- |
| Identity | Digest, release and serial are computed by the interface from the file and all three must match (research R2, R3). |
| Facts only | Two kinds of entry, each a number. The grammar has no place for a word, a condition or a script (research R1). |
| Published | Section 12 of `docs/introspection.md`, complete enough to read a pack from it alone; every statement cited by a test (research R14). |
| Verified | Each entry is proven by corpus trials in `corpus run`; contradicted, not proven and unneeded all fail (research R11). The check refuses a pack for a story the corpus does not hold. |
| Marked | `source: pack` on each exit that a pack changed, and a `pack` word on the host's line (research R4). |
| Absent means unchanged | `Introspector::new` is untouched; a pack not in force changes nothing; a second corpus run with packs set aside is kept and compared (research R11). See the strongest objection for what "unchanged" is read to mean. |
| No presentation content | None. The app's drawing of a closed door is the app's own, the same for every game. |
| A pack does not license a guess | An open door is the door's stored attribute, read at the moment of the query. Nothing is tried or run. |

**Readings the owner confirmed on 2026-11-12**: a wrong report on a marked answer may
be in the ledger when its cause is not the entry; an "open" entry needs trials in both
states; a pack whose story CI does not hold counts as verified by a test that CI
reports as ignored (by choosing Mini-Zork).

**Result**: Pass under version 2.0.0, with the one reading named in the strongest
objection. No deviation to justify.

**Post-design re-check**: Pass. The design added no query, no dependency on the corpus
crate, no file reading in the introspection crate, and nothing in the core.

## Project Structure

### Documentation (this feature)

```text
specs/006-game-packs/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── pack-format.md   # the draft of section 12 and the changed answers
│   └── commands.md      # functions, programs, files, and the tests whose form changes
└── tasks.md             # written by /speckit-tasks
```

### Source Code (repository root)

```text
docs/introspection.md               # version 0.2.0: section 12; changes to 2.3, 2.4, 8, 8.2, 10.4

packs/                              # NEW
├── zork1.pack
└── minizork.pack

crates/zmachine-introspect/
├── Cargo.toml                      # sha2 becomes a dependency
├── src/
│   ├── pack.rs                     # NEW: Pack, PackError, parse; identity of a story
│   ├── lib.rs                      # with_pack, pack(); Exit's three fields; ExitKind::Door
│   ├── zil.rs                      # the 5-byte and 4-byte rules with a pack
│   └── json.rs                     # the three keys
└── tests/
    ├── packs.rs                    # NEW: statements 1 to 26 on built stories; Zork I's doors
    └── spec_agreement.rs           # section 12's citations

crates/zmachine-experience/src/
├── lib.rs                          # Session::with_pack
└── compass.rs                      # PlaceState::Door

crates/zmachine-ffi/src/lib.rs      # zmx_session_new_with_pack

crates/zmachine-cli/src/lib.rs      # --pack; the pack field

crates/zmachine-corpus/
├── src/
│   ├── packs.rs                    # NEW: find and check packs; proof of an entry
│   ├── verdict.rs                  # open R, closed R, distrusted
│   ├── manifest.rs                 # the key `first`
│   ├── walk.rs                     # run with a pack; `first` commands
│   ├── report.rs                   # the pack line
│   ├── ledger.rs                   # open R and closed R as reported values
│   └── corpus.rs                   # check, run twice, write both
└── tests/
    ├── proof.rs                    # NEW: each outcome; the other 31 attributes
    ├── nopack.rs                   # NEW: counts and transcripts with packs set aside
    ├── no_text.rs                  # packs by grammar
    └── check.rs, formats.rs, verdict.rs, ledger.rs   # extended

stories/corpus/
├── manifest.txt                    # `first` lines for minizork
├── ledger.txt                      # five stale-flag entries removed
└── reports/
    ├── <id>.txt                    # with packs
    └── nopack/zork1.txt, minizork.txt   # NEW

apps/ios/Lamplight/
├── Resources/story.pack            # NEW: a copy of packs/zork1.pack
├── Game.swift, Engine.swift        # pass the pack's text
└── CompassView.swift, Snapshot.swift   # the closed door
apps/ios/LamplightUITests/          # a door seen closed, then open
```

**Structure Decision**: No new crate. The parser lives with the interface that applies
it, since the format is part of that interface's specification. The proof lives in the
corpus crate, which nothing depends on. Packs sit at the repository's root because they
are published data, not test fixtures.

## Order of work

1. **Publish first.** Write section 12 and the changed sections of
   `docs/introspection.md` from contracts/pack-format.md. Nothing is built on a
   statement that is not written down.
2. **User Story 1**: the parser and its refusals; identity; `with_pack` and the pack's
   state; the door rule and the mark, on stories built in memory, then on Zork I. With
   no pack, every earlier test of a value passes untouched.
3. **User Story 2**: the corpus's new reported values and verdicts; the check of
   `packs/`; proof of an `open-attribute` entry; the test of the other 31 attributes;
   Zork I's pack with that one entry.
4. **User Story 3**: the untrusted-variable rule; the second run and its kept report;
   proof of the entry; Zork I's two variables; the five ledger entries removed.
5. **User Story 4**: the `first` key; the commands for Mini-Zork, found on the owner's
   Mac; its pack; the no-text test; the citation test for section 12. If Mini-Zork's
   entry cannot be proven, stop and ask the owner.
6. **User Story 5**: the compass state; the C function; the app's resource and drawing,
   with a picture for the owner before it is settled; the UI test; a play by hand.
7. **Measure**: query time, launch time, the corpus's time on CI.

## Decisions for the project owner

Made in research. The owner confirmed 1, 2 and 4 on 2026-11-12, and did not change the
others:

1. **Three new fields on every exit, with or without a pack** (research R4; the
   strongest objection).
2. **The room behind a closed door is not shown or spoken on the compass** (research
   R7). The interface reports it; the compass keeps it back until the door is open.
3. **Variables are written as the story file stores them**, 16 to 255, not as a global's
   index from 0 (research R6).
4. **The interface hashes the story itself**, making `sha2` a dependency of the
   introspection crate and of the app (research R2).
5. **Packs live in `packs/` at the root**, named by the corpus's id for the story
   (research R9).
6. **A story with a pack keeps two corpus reports**, with and without (research R11).
7. **A new manifest key, `first`**, for commands before the crawl (research R12).

## Pull requests

Two, by the owner's choice on 2026-11-12: the first holds User Stories 1 to 4 (the
interface, the proof and both packs), the second User Story 5 (the compass and the
app). Neither merges before the constitution's amendment.

## Complexity Tracking

No violation of the constitution to justify.
