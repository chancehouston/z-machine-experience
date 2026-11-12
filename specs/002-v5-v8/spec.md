# Feature Specification: Z-Machine Versions 5 and 8

**Feature Branch**: `002-v5-v8`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "Start the next feature: versions 5 and 8." Scope decisions
made by the project owner on 2026-11-11: the upper window is delivered to the host as a
grid of text, with bold, italic, reverse and fixed-pitch styles reported on both windows
and colour reported as unavailable; timed input is reported as unavailable; the reference
game is Advent, built for version 5 and for version 8, subject to its license.

## User Scenarios & Testing *(mandatory)*

As with the version 3 core, the "users" are the project's developers and the layers that
will be built on the interpreter. Versions 5 and 8 matter because nearly every freely
distributable game is one or the other, so the shipped app depends on them.

### User Story 1 - Play a version 5 game from the command line (Priority: P1)

A developer points the command-line host at a version 5 story file and plays it: the
opening text appears, they type commands, the game responds, and they can quit.

**Why this priority**: Nothing else in this feature can be measured until a version 5 game
runs. Version 5 is where almost all the new behavior is.

**Independent Test**: Start the host with a version 5 build of the reference game, type a
handful of commands, and confirm each response is the game's correct reply.

**Acceptance Scenarios**:

1. **Given** a valid version 5 story file, **When** the developer starts the host with it,
   **Then** the opening text is shown and the host waits for a command.
2. **Given** a running version 5 game, **When** the developer types commands, **Then** the
   game's responses are shown, including text the game prints in bold or italic.
3. **Given** a game that asks for a single keypress (to page through a menu or to
   continue), **When** the developer presses a key, **Then** the game receives it and
   continues.
4. **Given** a game that asks whether timed input is offered, **When** it checks,
   **Then** it is told no and falls back to ordinary input.
5. **Given** a version 4, 6 or 7 story file, **When** the developer starts the host with
   it, **Then** the host refuses it and says which versions it runs.

---

### User Story 2 - Play a version 8 game (Priority: P2)

A developer plays a version 8 story file, including one larger than a version 5 file can
be.

**Why this priority**: Version 8 is version 5 with larger addresses. It is cheap once
version 5 works and it is what larger modern games use.

**Independent Test**: Play the version 8 build of the reference game with the same
commands as the version 5 build; the game text is the same.

**Acceptance Scenarios**:

1. **Given** the reference game built for version 8, **When** the same commands are
   entered as for the version 5 build, **Then** the game's text is the same apart from
   anything the game itself prints about its version.
2. **Given** a version 8 story file larger than 256 kilobytes, **When** it runs code and
   prints text located beyond that point, **Then** it behaves correctly.

---

### User Story 3 - See what the game draws in its upper window (Priority: P3)

A version 5 game draws its own status line, and may draw menus and boxed quotations, in an
area above the main text. A host can obtain that area as a grid of characters with their
styles, and the command-line host can show it. A boxed quotation is drawn in a window the
game enlarges and then shrinks again before it next asks for input, so what was drawn
stays readable until that input arrives.

**Why this priority**: Without it a player cannot see the status line, a menu or a
quotation, and the phone interface will need exactly this information.

**Independent Test**: Run a game that draws a status line and a boxed quotation; the grid
the host receives contains them at the positions the game chose.

**Acceptance Scenarios**:

1. **Given** a game that draws a status line, **When** the game waits for input,
   **Then** the host can read the status line's text from the upper window grid.
2. **Given** a game that prints in reverse video, bold, italic or fixed pitch, in either
   window, **When** the host reads the text, **Then** each run of text carries the styles
   in force when it was printed.
3. **Given** a game that sets colours, **When** it does so, **Then** the request is
   accepted without effect, and the game was told beforehand that colour is unavailable.
4. **Given** a game that splits, resizes, clears or unsplits the upper window, **When**
   it does so, **Then** the grid the host reads reflects it.
6. **Given** a game that draws a boxed quotation and shrinks the window back before
   asking for input, **When** the host reads the grid at that input, **Then** the
   quotation is still there, marked as lying outside the current window; after the input
   is given it is gone.
5. **Given** a game that moves the cursor outside the upper window or prints past its
   edge, **When** it does so, **Then** nothing is corrupted and the game continues.

---

### User Story 4 - Pass the conformance test programs (Priority: P4)

A developer runs praxix, and czech built for version 5 and for version 8. Every test
reports a pass.

**Why this priority**: praxix is named in the constitution as part of the definition of a
faithful core, and it was deferred from the first feature because it is a version 5
program. Together with czech it pinpoints which instruction is wrong.

**Independent Test**: Run each program to completion and read its own summary.

**Acceptance Scenarios**:

1. **Given** praxix, **When** it is run with its "all" command, **Then** it reports that
   all tests passed.
2. **Given** czech built for version 5 and for version 8, **When** each is run, **Then**
   it reports zero failed tests and its printed output matches czech's published expected
   output for that version, apart from the lines that describe the interpreter.
3. **Given** any of these runs as part of the automated suite, **When** a test fails,
   **Then** the suite fails and names it.

---

### User Story 5 - Match the reference interpreter on a full game, in both versions (Priority: P5)

A developer runs a scripted walkthrough of the reference game, from the opening to
winning, through this interpreter and through Frotz 2.55 with the same seed, once for the
version 5 build and once for the version 8 build. Each pair of transcript files is
identical byte for byte, except for differences recorded in the divergence ledger.

**Why this priority**: This is the project's definition of "faithful", applied to the
versions the shipped app will actually use.

**Independent Test**: Compare this interpreter's transcript with the committed reference
transcript for each version; the comparison reports no unledgered difference.

**Acceptance Scenarios**:

1. **Given** the reference game built for version 5, its walkthrough and its seed,
   **When** this interpreter plays it, **Then** its transcript matches the committed
   reference transcript byte for byte apart from ledgered differences.
2. **Given** the version 8 build, **When** the same is done, **Then** the same holds.
3. **Given** either transcript, **When** it is inspected, **Then** it shows the game was
   won with its maximum score.
4. **Given** the reference interpreter is installed, **When** the suite's optional checks
   are run, **Then** it still produces the committed reference transcripts.

---

### User Story 6 - Undo, save and restore (Priority: P6)

A player takes back their last move with the game's undo command. They can also save and
restore, and saves interchange with the reference interpreter as they do for version 3.

**Why this priority**: Undo is the feature of later versions that players use most, and
save and restore work differently in version 5 from version 3.

**Independent Test**: Make a move, undo it, and confirm the game is where it was; take
saves in both interpreters and restore each in the other.

**Acceptance Scenarios**:

1. **Given** a game in progress, **When** the player makes a move and then undoes it,
   **Then** the game is in the position before the move and says so.
2. **Given** several moves, **When** the player undoes repeatedly, **Then** each undo goes
   back one more move, and when nothing is left to undo the game says so.
3. **Given** nothing has been done yet, **When** the player tries to undo, **Then** the
   game reports that it cannot.
4. **Given** a save taken in a version 5 or 8 game by either interpreter, **When** the
   other interpreter restores it and plays a fixed continuation, **Then** the transcript
   matches the reference interpreter restoring the same save.
5. **Given** a saved position from another story file or a damaged save, **When** the
   player restores it, **Then** it is refused and play continues unharmed.

---

### Edge Cases

- A version 3 story file: it runs exactly as it did before this feature (FR-003).
- A story file for version 1, 2, 4, 6 or 7, or with a version byte above 8: refused at
  load with a message naming the supported versions.
- A game asks for a single keypress while input comes from a script: it gets the next
  unused character of the current script line, and a new line is read when that one is
  used up; an empty line is the return key. Characters left unused are dropped when a
  whole line is next asked for. These are the reference interpreter's rules.
- The script ends while the game waits for a keypress: the run ends normally, as it does
  for a line of input.
- A game redirects its output into memory, nests such redirections, or closes more than
  it opened.
- A game redirects output into a table that runs outside dynamic memory: a fault, never a
  crash. The Standard gives such a table no size to check, so "too small" is not
  detectable short of that.
- A game prints a character it has no glyph for, asks whether a character can be
  printed, or supplies its own alphabet or character table.
- A game throws to a stack frame that no longer exists: a fault.
- A game calls a routine with more arguments than the routine has locals, or checks how
  many arguments it was given.
- A game asks the interpreter to tokenise text against a dictionary of its own, sorted or
  unsorted.
- A game copies a table over itself, forwards or backwards, or with a zero or negative
  length.
- A game restores a save that was taken while output was redirected into memory.
- A game supplies an input buffer already holding text the player may edit.
- A game sets a list of keys that end a line of input.
- An object number, property number or attribute number that version 5 allows and
  version 3 does not, at the top of its range.
- A game sets the cursor before splitting the window, or to row or column zero.
- A game plays a sound, changes font, or asks for the piracy check: each has a defined,
  harmless answer.

## Requirements *(mandatory)*

### Functional Requirements

**Versions**

- **FR-001**: The interpreter MUST load and run story files of versions 3, 5 and 8.
- **FR-002**: The interpreter MUST refuse any other version and MUST name the versions it
  supports.
- **FR-003**: Version 3 behavior MUST NOT change: for a version 3 story file, everything a
  host can observe is what the version 3 feature produced. Every test of the version 3
  feature MUST continue to pass with its expectations unchanged, with two kinds of
  exception that the plan lists test by test: a test whose expectation was that version 5
  is refused, and a test that counts the instructions the decoder knows.
- **FR-004**: The interpreter MUST implement every instruction the Z-Machine Standard 1.1
  defines for version 5, with the behavior the Standard specifies, except where this
  specification states that a facility is reported as unavailable.
- **FR-005**: For version 8 the interpreter MUST apply that version's rules for the size
  of a story file and the meaning of packed addresses, and otherwise behave as version 5.
- **FR-006**: The interpreter MUST implement the version 5 forms of the object table,
  properties, dictionary, text encoding and routine calls, which differ from version 3.

**Screen and text**

- **FR-007**: The interpreter MUST maintain the upper window as a grid of characters whose
  size the host sets, and MUST let the host read it, with the cursor position, whenever
  the host has control.
- **FR-008**: Every run of text delivered to the host, in either window, MUST carry the
  styles in force when it was printed: reverse video, bold, italic and fixed pitch.
- **FR-009**: The interpreter MUST tell the game that bold, italic and fixed-pitch styles
  are available and that colour, timed input, sound and pictures are not.
- **FR-010**: A request to set colours, play a sound or change font MUST be accepted
  without stopping the game, with the answer the Standard prescribes for an interpreter
  that does not offer the facility.
- **FR-011**: The host MUST be able to tell the interpreter the size of the screen in
  characters, and the game MUST see that size.
- **FR-012**: The interpreter MUST support redirecting output into memory, including
  nested redirection to the depth the Standard requires.
- **FR-013**: The interpreter MUST honor a game's own alphabet table and its own table of
  extra characters, and MUST answer whether a given character can be printed and read.
- **FR-014**: The transcript stream MUST continue to record lower-window text and player
  input only. Upper-window text MUST NOT appear in a transcript.

**Input**

- **FR-015**: The interpreter MUST support line input in its version 5 form. Text already
  in the input buffer is kept and what the player types follows it. A host MUST be able to
  end a line with any key the game has listed as ending one, and the game MUST be told
  which key it was.
- **FR-016**: The interpreter MUST support single-keypress input, delivered by the host.
- **FR-017**: A request for input with a time limit MUST behave as input with no time
  limit, consistent with the game having been told timed input is unavailable.

**Game state**

- **FR-018**: The interpreter MUST support undo: the game can record its position and
  later return to it, to the same depth as the reference interpreter, which is 25
  positions.
- **FR-019**: Save and restore MUST work in their version 5 form, in which the
  instruction stores a result instead of branching, and a restored game resumes with the
  result that signals a restore.
- **FR-020**: Saves MUST use the same standard portable format as version 3, and MUST
  interchange with the reference interpreter in both directions.
- **FR-021**: A request to save or restore an arbitrary region of memory to a named file
  MUST be answered as a failure, which the Standard permits.
- **FR-022**: The interpreter MUST support the instructions for catching and throwing to
  an earlier point in the call stack.

**Errors**

- **FR-023**: Every illegal operation by a version 5 or 8 story file MUST stop that game
  with a report of the kind of fault and where it occurred, except where the Standard
  itself says an interpreter should ignore it, or where version 3 already tolerates it
  with a diagnostic. The host MUST remain running.
- **FR-024**: No story file or save file of any version, however damaged, may crash the
  host.

**Command-line host**

- **FR-025**: The command-line host MUST play version 5 and 8 story files interactively
  and from a script, with the same options as for version 3.
- **FR-026**: The command-line host MUST be able to show the upper window, and MUST
  render bold, italic and reverse text distinguishably on a terminal that can.
- **FR-027**: A script MUST be able to supply single keypresses, including the return,
  escape, delete, arrow and function keys, and the same script MUST be usable with the
  reference interpreter.

**Fidelity and testing**

- **FR-028**: The automated suite MUST include praxix and MUST fail if praxix reports any
  failed test.
- **FR-029**: The automated suite MUST include czech for versions 5 and 8, checked as
  czech for version 3 is.
- **FR-030**: The automated suite MUST include the reference game's walkthrough for
  version 5 and for version 8, each compared with a committed reference transcript, and
  MUST fail on any difference not recorded in the divergence ledger.
- **FR-031**: Every instruction new in version 5, and every instruction whose behavior
  differs from version 3, MUST have tests that cite the section of the Standard they
  verify.
- **FR-032**: The reference game, its walkthrough, and the reference transcripts and saves
  MUST be committed, beside a record of the game's license, source and digest. This
  depends on the game's license permitting redistribution.
- **FR-033**: Upper-window behavior, which no transcript records, MUST be tested directly
  against the Standard, and SHOULD also be checked against what the reference interpreter
  displays for the same test programs.

### Key Entities

- **Story file**: As before, now of version 3, 5 or 8. The version decides how large the
  file may be, how packed addresses are read, and which forms of the object table,
  dictionary and instructions apply.
- **Upper window**: A grid of character cells above the main text, with a height the game
  sets and a cursor the game moves. Each cell holds a character and its styles.
- **Styled text**: Text from the game together with the styles in force when it was
  printed.
- **Keypress**: A single character or special key delivered to a game that asked for one.
- **Undo position**: A snapshot of game state held in memory by the interpreter, to which
  the game can return. Not a file, and not preserved when the host stops.
- **Memory stream**: Output redirected by the game into a table in its own memory instead
  of to the screen.
- **Reference game**: The game whose full walkthrough is compared with the reference
  interpreter, in one build for version 5 and one for version 8.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: praxix reports that all of its tests pass.
- **SC-002**: czech reports zero failed tests for version 5 and for version 8, and its
  printed output matches the published expected output for each.
- **SC-003**: A scripted walkthrough of the reference game, from the opening to winning
  with the maximum score, produces a transcript with zero unledgered byte differences from
  the reference interpreter's, for the version 5 build and for the version 8 build.
- **SC-004**: Every test of the version 3 feature passes with its expectations unchanged,
  apart from the tests the plan lists by name under FR-003. The Zork I transcript match,
  czech for version 3, the fifteen transcript probes and the five save points are not
  among the exceptions.
- **SC-005**: Every instruction of version 5 is covered by at least one test citing the
  relevant section of the Standard.
- **SC-006**: For saves taken at no fewer than three points in each walkthrough, a fresh
  run that restores and plays a fixed continuation matches the reference interpreter, for
  saves written by either interpreter.
- **SC-007**: In a committed script, thirty moves are made and thirty undos attempted. The
  transcript matches the reference interpreter's byte for byte: twenty-five undos succeed
  and the rest are refused.
- **SC-008**: A set of at least thirty deliberately malformed or misbehaving version 5 and
  8 story files each produce a diagnostic, and several thousand randomly damaged story
  files and save files run without crashing the host.
- **SC-009**: Each full walkthrough completes in under five seconds on a current
  development laptop in an optimized build.
- **SC-010**: For a set of test programs that draw in the upper window, the grid the host
  reads equals the expected grid, cell for cell, including styles.

## Assumptions

- **Constitution applies.** The Standard 1.1 outranks Frotz, tests come first, the core
  has no I/O and no dependencies, and only redistributable story files are committed.
- **The reference game is Advent**, the Inform 6 port of Colossal Cave, compiled by this
  project for version 5 and for version 8. This assumes its source may be redistributed;
  research confirms the license before anything is committed. If it may not be, the
  project owner is asked to choose another game.
- **praxix may be committed** if its license allows; otherwise it is fetched locally and
  its test is reported as ignored when absent.
- **The comparison method is unchanged.** Transcripts are the files Frotz 2.55 writes
  through the game's transcript feature with wrapping off. The upper window never appears
  in them, which is why SC-010 and FR-033 exist.
- **Timed input, colour, sound and pictures are out of scope**, by the project owner's
  decision. Each is reported to the game as unavailable, which the Standard allows.
- **Versions 1, 2, 4, 6 and 7 remain out of scope**, as the constitution states.
- **The output interface changes.** Version 3's host received plain text; hosts now
  receive styled text. The command-line host is updated with it. There are no other
  hosts yet.
- **Undo history is held in memory only**, limited to the 25 positions the reference
  interpreter keeps. The oldest is dropped when the limit is reached.
- **The header does not match the reference interpreter's.** By the project owner's
  decision this interpreter reports text styles as available and timed input as
  unavailable; dfrotz reports the opposite. A game that tests those header bits may take
  a different course in the two, so a transcript match is expected only for games that do
  not. Advent, compiled with the Inform 6 library, does not.
- **A transcript is started at most once in a comparison script.** The Standard says the
  file name is asked for once in a session; Frotz asks every time for version 5. This
  interpreter follows the Standard, so a script that started a second transcript would
  feed the two interpreters differently.
- **A walkthrough that wins Advent has to be written**, tuned to one seed, as Zork I's
  was. Advent has random elements (wandering dwarves, the pirate, a limited lamp).
- **Unicode output is limited to what the Standard requires**: the default extra
  characters, a game's own table, and the instruction that prints an arbitrary character.
- **Out of scope:** the introspection interface, the experience layer, C bindings and the
  mobile shells.
