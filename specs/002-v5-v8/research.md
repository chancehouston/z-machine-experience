# Research: Z-Machine Versions 5 and 8

**Date**: 2026-11-11 (revised after independent review the same day) | **Plan**: [plan.md](plan.md)

Evidence is labelled as in the first feature: **Observed** means run on this machine with
dfrotz 2.55 (git commit `acf205585a9472d27c07c0fe62da4b8bc89d1ec7`) or Inform 6.44;
**Read** means taken from a primary source and not yet exercised. A task that first relies
on a Read item must confirm it. Section numbers refer to the Z-Machine Standard 1.1.

## R1. The reference game: Advent, and what its license allows

**Finding (Read, from the files installed with Inform 6.44)**: `advent.inf` ("Advent
060321", Release 9, serial 060321) ships in the Inform 6 package's demos. The file itself
states no license. The package's `COPYING` says the status of the demos "is what it says
in each file", and quotes the Inform Designer's Manual: copyright on Inform's "example
games" is retained by Graham Nelson, who is "happy for it to be freely distributed to
anybody who wants a copy, provided that: (a) distributed copies are not substantially
different from those archived by the author, (b) this and other copyright messages are
always retained in full, and (c) no profit is involved." The Inform library, which is
compiled into the story file, is under that license or the Artistic License 2.0.

**This is not the public domain the spec assumed.** It permits what this feature needs
and restricts what a later one might want:

- Committing the unmodified source and story files compiled from it to a public
  repository, with the notices kept, for no profit: permitted.
- Bundling Advent in an app that is sold: not permitted without the author's agreement.

**Decision**: Use Advent as the reference game for tests. Commit `stories/advent/` with
the unmodified `advent.inf`, the two compiled story files, the license text quoted above,
and a `SOURCE.md` with digests and the compile commands. `SOURCE.md` gives a URL and
revision for the source (the Inform 6 distribution's repository, not "as installed"),
says plainly that this directory is not under the repository's MIT license, and records
that the game is a test fixture not cleared for bundling in a shipped app. The README's
story-file section says the same.

**The reading is an inference, and weaker than it first looked** (reviewer's points, all
fair): the Inform package sends the reader to the file for its license and the file is
silent, so the fallback to the Designer's Manual's "example games" clause is our
interpretation; the grant is Graham Nelson's, while this release also contains later
changes by Roger Firth and is a port of Crowther and Woods's game; and the clause speaks
of distributing copies not substantially different from the author's, while we distribute
new builds and full transcripts.

**Observed**: Inform 6.44 with library 6.12.8 compiles it for version 5 (152,064 bytes)
and version 8 (156,160 bytes). Compiling twice gives identical files. Both run under
dfrotz. The game's maximum score is 350.

**Alternatives considered**: A game under a plainly open license (none surveyed has
Advent's combination of size, a known winning route and a version 5 and 8 build from one
source); writing our own test game (large, and proves less than a real game).

**Accepted by the project owner on 2026-11-11** as a test-only reference game under these
terms. The constitution's Principle VI requires that the license "permits
redistribution", which it does, with conditions.

## R2. praxix and czech

**Finding (Read)**: `praxix.inf` begins "Praxix - a Z-Machine unit test, by Zarf and
Dannii. Public domain, but please share your changes so we can improve the tests", and
notes that its indirect test is Copyright (C) 2003 Amir Karger, whose czech license
permits redistribution with its notice. The IF Archive's `praxix.zip` holds `praxix.inf`
and `praxix.z5`.

**Decision**: Commit `stories/praxix/` with both files from the archive, unmodified, and a
`SOURCE.md`. Use the archive's `praxix.z5` as is; its source says it has one canonical
build. For czech, compile the already-committed `czech.inf` for versions 5 and 8 and add
the archive's `czech.out5` and `czech.out8`.

**To confirm at the task**: praxix is interactive; find the command that runs every test
and the text it prints when all pass.

## R3. Transcripts in version 5

**Observed** with Advent and with a small test program:

- The transcript file works as in version 3: lower-window text, unwrapped, with each line
  of input echoed after the prompt. The quit question echoes its answer the same way.
- Upper-window text is not in the transcript. Styles are not in the transcript.
- A single keypress is not echoed into the transcript.
- The game's `version` command prints "Standard interpreter 1.1 (1F)": the Standard
  revision, then the interpreter number and version letter from the header. dfrotz
  reports number 1 and letter F. A byte-for-byte match therefore needs the same two
  header values (R8), and the walkthrough includes `version` so that this is exercised.
  The opening banner does not print them.
- With `buffer_mode 0`, Frotz writes text to the transcript a character at a time
  instead of a word at a time. The Inform library never turns buffering off, so Advent is
  unaffected.
- The transcripts of `advent.z5` and `advent.z8` for the same script are identical, and
  do not depend on the screen width.

The full set of rules is in [contracts/transcript-format.md](contracts/transcript-format.md).

## R4. Single keypresses from a script

**Observed**: dfrotz reads input a line at a time. A request for a keypress takes the
next unused character of the current line; when the line is used up it reads another. So
the script line `abc` answers three keypress requests with a, b and c, and three lines
`a`, `b`, `c` do the same. A line of input requested after that read a fresh line.

Also observed: an empty line gives the return key; characters left unused are dropped
when a whole line is next requested, and a key request drops anything left over from a
line; spaces and case are kept. dfrotz translates backslash escapes inside a line into
special keys, and treats a line that begins with a backslash and a lower-case letter as a
command to itself.

**Decision**: `zm` follows the same rules and the same escapes, and refuses the lines
dfrotz would treat as its own commands, so one script means the same to both.

## R5. Undo

**Observed**: Advent's `undo` works repeatedly under dfrotz, each one going back a turn.
`save_undo` returns 1. dfrotz keeps 25 positions: its manual says twenty, but its source
defines 25, and thirty moves followed by thirty undos gave twenty-five successes.

**Decision**: Keep the 25 most recent positions; drop the oldest. The number is a host
setting, and 0 makes undo unavailable, in which case `save_undo` stores -1. An undo position is
dynamic memory, the call stack and the address of the `save_undo` instruction's store
byte. It does not include the random state, matching save files.

## R6. Version differences are data, not branches

**Decision**: One `Version` value, fixed at load, answers every question that differs
between versions: the packed-address multiplier (2, 4, 8) and file-length multiplier (2,
4, 8) (§1.2.3, §11.1.6); the maximum story size (128K, 256K, 512K) (§1.1.4); the object
table layout (§12.3); the dictionary key size (§13.3); the property size encoding
(§12.4.2); whether routines carry initial local values (§5.2.1); and which instruction
each opcode number means (§14).

**Rationale**: Version 3 behavior must not change (FR-003). Keeping every difference
behind one type makes each one visible and testable, and makes "version 8 is version 5
with two numbers changed" literally true in the code.

**Alternatives considered**: Separate interpreters per version (duplicates the 68 shared
instructions); scattered `if version >= 5` tests (the differences become invisible).

## R7. The output interface

**Decision**: Internally, lower-window output becomes a list of runs, each with its text
and its style (reverse, bold, italic, fixed pitch). Two ways to drain it:

- `take_output()` still returns plain text, exactly as now.
- `take_styled_output()` returns the runs.

Either call empties the same buffer. What the game draws outside the main text is a
canvas of cells the host reads with `screen()`, with a flag saying whether it changed
([contracts/screen-model.md](contracts/screen-model.md)). Version 3's status line
interface is unchanged.

**Rationale**: Every version 3 test reads `take_output()` and must keep passing unchanged.
The phone UI will need the runs.

## R8. What the header tells the game

**Decision**: At load and after every restart, restore and undo (§6.1.2.2), for versions
5 and 8 the interpreter writes the fields listed in data-model.md: Flags 1 with bold,
italic and fixed pitch available and colour and timed input unavailable; the interpreter
number and version letter; the screen size in lines and characters, and again in units
with a 1 by 1 font (§8.4.3); the Standard revision 1.1. In Flags 2, the bits for
pictures, mouse and sound are cleared and the rest, including the transcript bit, keep
their current values across restore and undo (§6.1.2).

The interpreter number and version letter come from the host, defaulting to 1 and `Z`.
The fidelity tests pass `F`, because games print these in their banners (R3) and the
reference transcript was made by Frotz.

**Rationale**: The Standard leaves both values to the interpreter, and number 1 is what
the reference interpreter's plain-terminal build reports.

## R9. Save and restore in version 5

**Read** (Quetzal §5.8.2, Standard §15): `save` and `restore` are extended instructions
that store a result. The saved program counter points at the store-variable byte of the
`save`. A successful restore stores 2 in the variable that byte names and continues after
it. Frames record whether a call discards its result and how many arguments it was given
(needed by `check_arg_count`).

**Decision**: The existing Quetzal reader and writer already carry all of this. The
version decides only what is done with the saved program counter on restore. The optional
table, bytes and name operands of `save` and `restore` are answered with 0, failure
(FR-021).

**Observed**: Advent saves and restores under dfrotz from a script, as Zork I did.

## R10. The test image builder

**Decision**: `zmachine-testkit` takes a version. For 5 and 8 it writes 14-byte object
entries with 48 attributes and 63 property defaults, six-byte dictionary keys, routine
headers without initial values, packed addresses divided by 4 or 8 with matching
alignment, and the version 5 header. New helpers encode the extended form, the two
type-byte form for eight operands, and the version 5 text buffer.

Its encoders are checked as before: against hand-computed vectors, and against the
dictionary and object table of a real story file compiled by Inform (Advent).

## R11. Checking the upper window against Frotz

**Observed**: dfrotz's terminal output is its screen, redrawn. For a small test program
the upper window's first row appeared in it, mixed into the lower window's lines.

**Decision**: Screen behavior is specified and tested against the Standard (§8.7), with
ten named test programs listed in the screen contract. An optional test, run only where
dfrotz is installed, uses `dfrotz -r cn -r ln -r lt -h 24 -w 80`, which the reviewer
found prints the whole screen with numbered rows before each input, and compares the
upper-window rows' text with our canvas for three of those programs and for the opening
of Advent. It is a cross-check, not a definition, and skips the rows where the contract
knowingly chose differently from Frotz.

## R12. Praxix needs undo, colour no-ops and the memory stream

**Observed by the reviewer**: the script `all`, `quit` runs every praxix test with no
keypresses or file prompts, and under dfrotz ends "All tests passed." Its tests require
`save_undo` to store 1 and `restore_undo` to resume with 2; issue `set_colour` with
colour numbers version 5 does not define and `set_true_colour` with -1, which must be
silent; and require the default extra characters to pass through a memory stream intact.

**Decision**: Undo's instructions are built with the rest of the instruction set, before
praxix is run, not left to the last user story.

## R13. The header does not match Frotz's, by choice

**Observed by the reviewer**: dfrotz 2.55 writes Flags 1 as 0x80 for version 5: timed
input available, and bold, italic and fixed pitch not. This interpreter writes the
opposite, on the project owner's decisions. dfrotz measures timed input against the wall
clock, so input from a script never times out there either.

**Consequence**: a game that tests those bits may behave differently in the two. The
Inform 6 library does not test them in version 5, so Advent is unaffected. Recorded in
the spec's Assumptions.

## R14. Keeping version 3 unchanged

The first draft of this plan promised that no version 3 test would change, and then
described interface changes that would have altered about twenty-five of them. Revised:

- **No existing type or function changes.** New settings go through a new `load_with`;
  pre-loaded input is read with a new accessor; `Event::LineWanted` and `LoadOptions`
  keep their shape.
- **Three tests change, and only these.** Two assert that a version 5 file is refused
  (`load_refuses_what_is_not_a_version_3_story` in the core's `tests/events.rs`, and
  `files_that_are_not_version_3_stories_are_refused` in the CLI's `tests/play.rs`); each
  switches to version 4, which is still refused. One (`tests/coverage.rs`) asserts the
  decoder knows exactly 68 instructions; it becomes a table per version, with the version
  3 table unchanged.
- **Unit tests that call internal functions** keep their expectations; where a function
  gains a version parameter the test passes version 3.
- **The testkit's existing interface stays as it is.** Version 5 images are built with new
  types beside it, and version 3 images come out byte for byte as before.

## R15. What stays the same

The event-driven core, the divergence ledger, the fidelity test harness, the regeneration
script and CI all carry over. Each gains cases; none changes shape. The transcript rules
gain nine. The seeded random number generator is untouched.

## Sources

- Z-Machine Standards Document 1.1: <https://inform-fiction.org/zmachine/standards/z1point1/index.html>
- Quetzal 1.4: <https://inform-fiction.org/zmachine/standards/quetzal/index.html>
- Inform 6.44 package (`COPYING`, `share/inform/std/demos/advent.inf`, library `COPYING`), as installed by Homebrew
- praxix source: <https://github.com/curiousdannii/if/blob/master/tests/praxix.inf>
- IF Archive, interpreter tools: <https://ifarchive.org/indexes/if-archive/infocom/interpreters/tools/>
- Frotz 2.55: <https://gitlab.com/DavidGriffith/frotz/-/tree/2.55>
