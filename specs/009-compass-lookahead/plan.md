# Implementation Plan: Compass Look-Ahead

**Branch**: `009-compass-lookahead` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/009-compass-lookahead/spec.md`

## Summary

At each command prompt, the experience layer tries every way the interface could not
settle on three throwaway copies of the game: one with the game's own draws of chance,
two with others. Where all three walked to the same room, the compass shows an open way
to that room, with its darkness if a pack makes that known. Where all three stayed, the
compass shows no way. Anywhere else the question mark stays. The corpus sets each
finding against a trial of its own with yet other draws, and an unexplained miss fails.

Measured on the corpus before any code was kept ([research.md](research.md)): 482 of
495 such ways are settled, 13 turn on chance and keep their mark, and none of the 482
was contradicted by four further copies.

## Strongest objection

**Four ways in ten that are settled are settled by vanishing.** Of the 482 findings,
184 are refusals, and by the owner's choice a refused way is drawn as no way at all. In
Zork I that is 141 of 405. Some of those are the trap door under the rug, which ought
to stay hidden. Others are ways the game shows plainly and guards: a player who sees a
troll before a passage will find no passage on the compass, and reads the game's reason
only by typing the direction. The compass will be right about what happens next and
silent about what could happen later. The owner weighed this on 2026-11-13 and chose
it; the tasks put pictures of two such places before the owner, with a stop, before the
app's tests are fixed to it.

Three lesser ones:

- **The core changes.** One method lets the host give a machine a new value for chance
  (research R3). It is small and no game can observe it, but feature 008 could say "the
  core is not touched" and this one cannot.
- **A copy with the game's own draws is an oracle.** It knows what the next move will
  do. The two other copies are there to keep it from telling the player the dice
  (research R2); the design depends on them, and a test must show a way decided by
  chance keeps its mark.
- **The corpus has no fatal or questioning way among its unsettled ones** (research R1).
  User Story 3 rests on built stories alone.
- **Ten milliseconds is tight for the slower games.** Three copies of one way cost up
  to 2.6 ms in the Inform build of Advent on a Mac (research R7). The total limit will
  leave some ways unsettled there before it lets a turn run long. Zork I has room to
  spare.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app's tests

**Primary Dependencies**: None new. The corpus crate gains a dependency on the
experience layer.

**Storage**: Plain text: the corpus's reports and ledger.

**Testing**: `cargo test`, on built stories for each rule and on the committed stories
for the proof; the app's UI tests through `scripts/test-ios.sh`. Timing tests take
their limits from `zmachine_testkit::time_limit`.

**Target Platform**: Linux and macOS for the workspace; iOS 26 and 27 for the app.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: A command and its snapshot within 10 ms with look-ahead on, as
feature 004 set; the limits of feature 003 and of launch unchanged (FR-016, FR-017).

**Constraints**: The player's game is never changed, down to its next draw. Every
transcript is unchanged. The interface's answers are unchanged. The snapshot's form is
unchanged. No code for one game. Nothing a copy prints is kept. The same prompt always
gives the same compass.

**Scale/Scope**: About 15 lines in the core, 250 in the experience layer, 200 in the
corpus crate, a section of `docs/introspection.md`, and tests for each. No app code.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

Checked against version 2.0.0.

| Principle | How this plan complies |
| --- | --- |
| I. Faithful Core | The core gains one method the host may call; unless it is called, nothing differs. Every fidelity test runs unchanged and must give the same bytes. |
| II. Test-First | Each statement of the contracts has a test on a built story, seen to fail before its code; so has each verdict of the proof. Where a test cannot fail first, the task says so. |
| III. Typed Commands | A copy is given the text a touch would type, and nothing else. Touching a settled way types the direction. |
| IV. Read-Only Introspection | The interface runs nothing and its answers do not change. Look-ahead is made above it, on copies; the game being played is not written to, and a test compares it before and after, byte for byte and draw for draw. |
| V. Strict Layering | Chance is supplied by the host through an explicit method. Look-ahead is in the shared layer. The app holds no logic for it. |
| VI. Story File Licensing | No story is added. The corpus keeps numbers only. |

**"The UI MUST NOT guess."** Principle IV says that when an adapter cannot answer, the
interface degrades to typing and does not guess. A finding is not a guess: it is what
the game itself did, three times, when given the command. Where the three differ, the
way keeps its question mark and the player types. This plan reads the principle as
kept. A stricter reading was possible, that only stored facts may reach the compass;
it was put to the owner, who answered on 2026-11-13 that look-ahead complies. No
amendment is made.

**Re-check after design**: the contracts add nothing that changes this table. The
snapshot keeps its form (research R6), so the layering between layer and app is as it
was.

## Project Structure

### Documentation (this feature)

```text
specs/009-compass-lookahead/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── lookahead.md     # what the layer does, and what the published spec is to say
│   └── corpus.md        # the report's new line, the ledger's new entry, the proof
└── tasks.md             # made by /speckit-tasks
```

### Source Code (repository root)

```text
crates/zmachine-core/src/
├── lib.rs               # one method: give this machine a new value for chance
└── random.rs            # the generator takes the value; predictable mode is kept

crates/zmachine-experience/src/
├── lookahead.rs         # new: one way tried on three copies; the finding
├── lookahead_tests.rs   # new: built stories for each statement of the contract
├── compass.rs           # a compass is settled by findings before it is returned
└── lib.rs               # the session asks for findings at a command prompt

crates/zmachine-experience/tests/
└── speed.rs             # the 10 ms limit, now with look-ahead on

crates/zmachine-corpus/src/
├── walk.rs              # at each place: the layer's finding, and a trial apart from it
├── report.rs            # one new line for each story, family and all
├── ledger.rs            # a cause for a finding not borne out
└── trial.rs             # unchanged, save that a trial may be given a value for chance

crates/zmachine-testkit/src/
└── programs.rs          # built stories: a way by chance, a fatal way, a way that asks

docs/introspection.md    # a section on settling a conditional answer by trial
stories/corpus/          # reports rewritten with the new line; README's figures
apps/ios/LamplightUITests/  # the living room, before and after the trap door
```

**Structure Decision**: The existing workspace. One new module in the experience
layer; no new crate (research R4).

## Complexity Tracking

No violation to justify. The one matter of judgement, the reading of Principle IV
above, was decided by the owner on 2026-11-13.
