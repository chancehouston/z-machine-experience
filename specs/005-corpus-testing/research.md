# Research: Corpus Testing of Introspection

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Checked on 2026-11-12 against the code on `main` (after features 001 to 004) and against
the public archives named below. Nothing was downloaded: file sizes on the archive were
asked for by header only, and the archive does not state them.

## R1. Copying a running game

**Checked**: `zmachine_core::Machine` derives `Clone`, and the copy includes the random
number generator (`random: Random`, itself `Clone`). `crates/zmachine-core/tests/clone.rs`
(feature 003) already proves that a copy is independent and waits for the same answer.
Feature 003's test helpers already give a copy one command
(`crates/zmachine-introspect/tests/common/mod.rs`, `ask`).

**Decision**: A trial is `machine.clone()`, one `provide_line`, and `run` until the copy
next stops. No change to the core.

**Rationale**: It exists, it is tested, and it carries the random state, so a trial
shows what the same move would do next in the game itself (spec, Assumptions).

**Alternatives considered**: Saving to Quetzal and restoring into a second machine: more
code, and a save does not hold the random state.

## R2. What a trial does and how its outcome is read

**Decision**: At a place (a machine stopped at a command prompt):

1. Copy the machine.
2. Give the copy one line: the word for the direction (R4).
3. Run the copy until it stops, within a limit of 50 million instructions.
4. Read the outcome from how it stopped. The copy's output is taken and thrown away
   unread.

| The copy stopped | Outcome |
| --- | --- |
| Waiting for a line at the command prompt (R3), location known and a different room | moved (to that room) |
| The same, and the same room | stayed |
| The same, and the location unknown before or after | location unknown |
| Waiting for a line anywhere else, for a key, or for a save, restore or transcript name | question |
| The game quit | ended |
| A fault, or the instruction limit | fault |

**Rationale**: Only where the player is counts (spec, Edge Cases). Reading the text
would bring per-game phrases into the instrument.

## R3. Telling a command prompt from a question

**Checked**: Feature 004 settled this for the experience layer (its research R6): a
prompt is a command prompt when the game is waiting at the same place in its code as at
its first line prompt. The core gives that place by `Machine::waiting_at()`
(`crates/zmachine-core/tests/waiting_at.rs`).

**Decision**: The same rule, applied by the corpus tools themselves: the command address
is `waiting_at()` at the first line prompt after the story's opening lines (R6). A stop
for a line at any other address is a question.

**To be proven by a test before anything is built on it**: along each walkthrough the
project holds, the number of command prompts by this rule equals the number the earlier
features already assert (`the_walkthroughs_have_the_expected_number_of_command_prompts`).

**Alternatives considered**: Depending on `zmachine-experience` for the rule. Rejected:
the instrument would then be judging introspection through the layer whose compass it is
meant to inform, and FR-005 is simpler to prove if the tools depend on nothing above
introspection.

## R4. Directions, and the word a trial sends

**Checked**: `Introspector::exits` lists only the directions a room's table mentions:
fixed, conditional, or unknown. A direction it leaves out is "not an exit".
`Introspector::dictionary` gives every word with `direction: Option<u8>`.

**Decision**:

- **The directions of a story** are the distinct direction numbers among its dictionary
  words, in ascending order. They are found once per story.
- **The word for a direction** is its longest stored word; among equals, the first in
  alphabetical order. (Version 3 stores six letters: `northe` is what is sent, and the
  game reads it as it reads the same letters typed.)
- **A direction with no word** is not tried; it is counted as "wordless".
- **What was reported** for a direction at a place: the entry in `exits` (fixed with its
  destination, or conditional); else the entry in `unknown`; else "not an exit". Where
  the whole answer is unknown, every direction is "unknown".

**Rationale**: The longest word is the least likely to be read as something else. It is
what feature 003's tour already sends.

## R5. Reaching places

**Decision**: Two sources of places, one set of visited rooms.

1. **Walkthrough**: replay the script with the story's seed. Every command prompt is a
   place and has its trials. Its room is marked visited.
2. **Crawl**: a queue, first in first out. It starts with the opening place and, when
   there is a walkthrough, receives places as below. At each place, directions are tried
   in ascending order. A trial whose outcome is "moved" to a room not yet visited marks
   the room visited and puts the trial's own copy on the queue as a new place. Places
   from the queue have their trials run when they are taken off it.

   Along a walkthrough, a trial that moves to an unvisited room is queued the same way,
   once the walkthrough has finished, in the order met; a room the walkthrough later
   reaches itself is by then visited and is dropped from the queue.

**Budget**, the same for every run: 400 rooms from the crawl and 40,000 trials for each
story, unless the manifest gives other figures for it. Reaching either ends the crawl.

**Naming a place**, without text: a walkthrough place by its script line; a crawl place
by the directions taken from where it started.

**Rationale**: Breadth-first with a fixed order is the simplest crawl that is the same
on every run (FR-011) and visits the nearest rooms first when the budget is short.

**Known limit**: the crawl walks only. On a story with no walkthrough it reaches what
the opening connects to. The report says how many rooms were named as destinations and
never reached (R10).

## R6. Starting a story

**Checked**: The corpus stories do not all open at a command prompt; a game may ask for
a key or a line first.

**Decision**: The manifest may give a story **opening lines**: answers given in order to
whatever the game asks before its first command prompt (a line for a line, its first
character for a key). The command address (R3) is taken after them. A story with a
walkthrough needs none: the walkthrough's own first lines do it.

**Rationale**: This is test data, of the same kind as a walkthrough or a seed. It is
read only by the corpus tools.

## R7. Where the code lives

**Decision**: A new crate, `crates/zmachine-corpus`, `publish = false`.

| | |
| --- | --- |
| Depends on | `zmachine-core`, `zmachine-introspect`; `sha2` (already used by two crates' tests) |
| Depended on by | nothing |
| Holds | the manifest, ledger and report readers and writers; the trial; the crawl; verdicts |
| Tests | its own unit tests on built stories (`zmachine-testkit`), and integration tests on the corpus |
| Program | `corpus`: check the manifest, run stories, write reports (contracts/commands.md) |

A test in the crate reads every other crate's `Cargo.toml` and fails if any names
`zmachine-corpus` (FR-005).

Fetching is a shell script, `scripts/fetch-corpus.sh`, using `curl` and `shasum`, as
`scripts/build-ios.sh` does. The Rust workspace gains no network code.

**Alternatives considered**: Putting it in `zmachine-testkit`. Rejected: the testkit
builds story images and depends on nothing; every crate's tests depend on it. Putting it
in `zmachine-introspect/tests`: rejected, because a program and committed reports need a
library, and feature 003's helpers there stay as they are (SC-010).

## R8. Formats

**Decision**: Plain text, one fact to a line, read by code in the corpus crate. No new
dependency. The three formats are in [contracts/formats.md](contracts/formats.md).

| File | Where |
| --- | --- |
| Manifest | `stories/corpus/manifest.txt` |
| Ledger | `stories/corpus/ledger.txt` |
| Report for a story | `stories/corpus/reports/<id>.txt` |
| Summary | `stories/corpus/reports/summary.txt` |

**Rationale**: The project already keeps its hand-made expectations as line-oriented
text (`stories/*/introspection/exits.txt`) and writes its JSON by hand. A report that is
plain lines of keywords and numbers also makes FR-026 checkable by grammar (R9).

## R9. Proving a report holds no game text

**Decision**: Two checks.

1. **By grammar, for every report**: each line must match the report format exactly,
   whose only free fields are numbers, digests and the story's id. A line that does not
   match fails the test.
2. **By search, for committed stories** (SC-006): every object's printed name and every
   dictionary word of five letters or more is searched for in that story's report and in
   the summary; a hit fails the test. Keywords of the format are exempt by listing them.

Ledger causes are written by a person and are prose. A cause may name a routine or a
rule; the check in 2 runs over the ledger too, so a cause cannot quote a room's name.

## R10. Counting a story's rooms

**Checked**: The introspection interface has no query for "how many rooms". The ZIL
adapter knows the object that holds the rooms, but does not publish it; the Inform
adapter has no such object.

**Decision**: The report gives **rooms visited** and **rooms named and not reached**:
rooms that some fixed exit, reported at a place visited, leads to, and that were never
visited. No total is claimed.

**The spec was amended to match** (FR-024, SC-007 and User Story 3, scenario 5, which
asked for "the number of rooms the story has where the family's rules can count them").
A total would need a new query in the published interface, and FR-015 keeps this feature
from changing it.

## R11. Where fetched stories live

**Checked**: `.gitignore` already ignores `/stories/local/` and every story-file
extension.

**Decision**: `stories/local/corpus/<file name>`. A file is used only if its SHA-256 is
in the manifest (FR-017). The tests never download (FR-020).

## R12. Time

**Estimate, to be measured in the tasks**: Zork I's walkthrough has about 330 command
prompts and 12 directions, so about 4,000 trials; a command with a copy of an 87 KB
story is well under a millisecond (feature 004 measured 0.35 ms for a command with its
snapshot). The crawl's 40,000-trial limit bounds a story at well under a minute in an
optimised build. SC-009's ten and thirty minutes leave a wide margin; the corpus tests
run in the optimised test profile if the plain one proves slow.

## R13. Stories that may be committed

| Story | File | Source | Standing |
| --- | --- | --- | --- |
| Zork I | `stories/zork1/zork1.z3` | already recorded | Committed (MIT, with the recorded caveat) |
| Advent | `stories/advent/advent.z5`, `advent.z8` | already recorded | Committed as a test fixture |
| Zork II | `zork2.z3`, 92,524 bytes | <https://github.com/historicalsource/zork2>, `COMPILED/zork2.z3` | MIT License in the repository, dated 2025-11-20. To be fetched with the owner's approval and recorded as Zork I is |
| Zork III | `zork3.z3`, 87,984 bytes | <https://github.com/historicalsource/zork3>, `COMPILED/zork3.z3` | The same |

**Checked**: of the Infocom titles in that collection, only `zork1`, `zork2` and `zork3`
carry a license. No other is touched.

## R14. Stories proposed for fetching

**The archive's terms** (<https://ifarchive.org/misc/license.html>, read 2026-11-12):
files are the property of their creators; material with a license is under that
license; "Any material with no attached license is presumed to be licensed for personal
use only." Downloading a file to test an interpreter on one's own machine is personal
use. Redistribution is not granted, so none of these is committed, and neither is any
transcript of its text (constitution, Principle VI).

Each is proposed, not approved. The project owner approves each file before it is
fetched. The archive does not state sizes; the format limits them to 128 KB (version 3),
256 KB (version 5) and 512 KB (version 8). The digest is recorded at the first fetch,
from the file the owner approved.

All are under `https://ifarchive.org/if-archive/`.

| # | Path | What it is | Why it is wanted |
| --- | --- | --- | --- |
| 1 | `infocom/demos/minizork.z3` | Mini-Zork I, a demo Infocom gave away | ZIL family, Infocom's own compiler, not one of the three Zorks |
| 2 | `infocom/demos/sampler1_R55.z3` | The 1984 Infocom Sampler, release 55 | ZIL family; several small games in one file, which may confuse the player rule |
| 3 | `games/zcode/advent.z3` | Colossal Cave in ZIL, by Tara McGrew, built with ZILF | ZIL family from a modern compiler |
| 4 | `games/zcode/curses.z5` | Curses, by Graham Nelson, release 16 | Inform family; large; an early library |
| 5 | `games/zcode/Adventureland.z5` | Adventureland, ported to Inform by Graham Nelson | Inform family; small |
| 6 | `games/zcode/Balances.z5` | Balances, by Graham Nelson | Inform family; magic and unusual movement |
| 7 | `games/zcode/905.z5` | 9:05, by Adam Cadre | Inform family; small and modern |
| 8 | `games/zcode/anchor.z8` | Anchorhead, by Michael Gentry | Inform family; version 8; large |
| 9 | `games/zcode/Advent_Crowther.z8` | Crowther's original Adventure, ported to Inform 7 | Inform 7's library: introspection may not recognise the family, which is itself a finding |

Nine are proposed so that the six FR-023 requires remain if the owner declines some or a
file proves unusable. Each address answered on 2026-11-12. None states a license on its
archive entry, so each is under the archive's presumption of personal use.

**Not proposed**: any Infocom title other than the three Zorks and the two demos Infocom
itself gave away; anything not on a public archive that offers it for download.

## R15. What exists already and is left alone

Feature 003 has, in `crates/zmachine-introspect/tests/common`, a walkthrough driver, a
check that moves along the walkthrough arrive where exits said, and a tour of 200 steps
through fixed exits. They stay as they are and keep passing (SC-010). The corpus crate
has its own driver, because test helpers of one crate cannot be used by another, and
because the corpus must run on stories that have no entry in those helpers.

Feature 003 also found that a game's code may move the player on at arrival to another
room of the same printed name (its specification, section 8.3). Such a case will appear
here as a "wrong" verdict and go to the ledger with that cause.

## R16. Reporting a story as skipped

**Checked**: Rust's standard test runner cannot mark a test skipped while it runs
(feature 001, research). Every story the earlier features use is committed, so the
project has had no absent story to report until now.

**Decision**: The corpus runs as a test target with its own `main` (`harness = false`
in the crate's `Cargo.toml`). It prints, for each story of the manifest, a line in the
standard runner's form, with `ignored, story absent` for an absent one, and counts it
under "ignored" in the totals. `cargo test` runs it with every other test.

**Rationale**: The constitution requires "skipped, not passed" (Principle VI). A test
that returned early would be counted as passed.

**Alternatives considered**: One `#[test]` per story generated by a build script, with
`#[ignore]` decided at build time: a build script that reads the file system to decide
what is tested is harder to follow than a short `main`. A crate that adds run-time
skipping: a new dependency for one line of output.

## Still open, for the task list

1. Whether the command-prompt rule (R3) gives the known prompt counts on Advent as well
   as Zork I. If not, the rule is revisited before anything else is built.
2. The measured time (R12), and whether the budget (R5) needs changing.
3. Whether each proposed story (R14) loads, is recognised, and opens at a command prompt
   or needs opening lines (R6). A story that cannot be used is replaced from the list.
