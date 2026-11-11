# Feature Specification: Faithful Z-Machine Version 3 Core

**Feature Branch**: `001-v3-core`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "Faithful Z-machine version 3 core. A library that loads a version 3 story file and runs it: text in, text out, no UI and no I/O of its own. Complete version 3 instruction set, object tree, properties, dictionary and tokenisation, text encoding and abbreviations, status line, seedable random numbers, save, restore and restart. A minimal command-line host for interactive and scripted play. Done when it passes czech for version 3, a scripted Zork 1 walkthrough matches Frotz byte for byte (ledgered divergences excepted), and runs are deterministic. Out of scope: versions 5 and 8, praxix, introspection, experience layer, C bindings, mobile shells."

## User Scenarios & Testing *(mandatory)*

The "users" of this feature are the project's own developers and the later layers of the
product. Nothing here is shipped to players on its own; it is the foundation everything
else is measured against.

### User Story 1 - Play a version 3 game from the command line (Priority: P1)

A developer points the command-line host at a version 3 story file and plays it: the
opening text appears, they type commands, the game responds, and they can quit.

**Why this priority**: Nothing else can be built or measured until a story file runs. This
is the smallest slice that proves the interpreter works at all.

**Independent Test**: Start the host with a version 3 story file, type a handful of
commands, and confirm each response is the game's correct reply.

**Acceptance Scenarios**:

1. **Given** a valid version 3 story file, **When** the developer starts the host with it,
   **Then** the game's opening text is shown and the host waits for a command.
2. **Given** a running game waiting for input, **When** the developer types a command the
   game understands, **Then** the game's response is shown and the host waits again.
3. **Given** a running game, **When** the developer types a word the game does not know,
   **Then** the game's own "I don't know that word" style response is shown and play
   continues.
4. **Given** a running game, **When** the game ends or the developer quits, **Then** the
   host exits cleanly and reports success.
5. **Given** a file that is not a version 3 story file (a version 5 file, a truncated
   file, an unrelated file), **When** the developer starts the host with it, **Then** the
   host refuses to run it and states the reason.

---

### User Story 2 - Run a scripted session and get a repeatable transcript (Priority: P2)

A developer supplies a story file, a file of commands, and a random seed. The host plays
the commands in order without a person present and writes a transcript of everything the
game printed. Running it again produces the identical transcript.

**Why this priority**: Every fidelity test in this project is a scripted run compared
against an expected transcript. Repeatability is what makes those tests meaningful.

**Independent Test**: Run the same story file, command script and seed twice and compare
the two transcripts; they must be identical.

**Acceptance Scenarios**:

1. **Given** a story file, a command script and a seed, **When** the host runs the script,
   **Then** it writes a transcript containing all game output and exits when the script or
   the game ends.
2. **Given** the same story file, script and seed, **When** the run is repeated any number
   of times, on any supported platform, **Then** every transcript is identical byte for
   byte.
3. **Given** a script that ends while the game is still waiting for input, **When** the
   host reaches the end of the script, **Then** it stops cleanly and the transcript
   contains everything printed up to that point.
4. **Given** a fixed screen width setting, **When** a scripted run is performed, **Then**
   line wrapping in the transcript follows that width.

---

### User Story 3 - Pass the standard conformance test program (Priority: P3)

A developer runs the czech conformance test program built for version 3. Every test it
contains reports a pass.

**Why this priority**: czech exercises the instruction set systematically and pinpoints
which instruction is wrong, which a full game cannot do.

**Independent Test**: Run the czech version 3 story file to completion and read its own
summary line.

**Acceptance Scenarios**:

1. **Given** the czech test program for version 3, **When** it is run to completion,
   **Then** it reports zero failed tests.
2. **Given** the czech test program, **When** it is run as part of the automated test
   suite, **Then** any failed test causes the suite to fail and names the failing test.

---

### User Story 4 - Match the reference interpreter on a full game (Priority: P4)

A developer runs a scripted walkthrough of Zork 1, from the opening to winning the game,
through both this interpreter and the reference interpreter (the dumb-terminal build of
Frotz) with the same seed and screen width. The two transcripts are identical byte for
byte, except for differences recorded in the divergence ledger.

**Why this priority**: This is the project's headline definition of "faithful". It comes
last because it only passes once everything else is right.

**Independent Test**: Produce both transcripts from the same script, seed and width and
compare them; the comparison reports no unledgered difference.

**Acceptance Scenarios**:

1. **Given** the committed Zork 1 story file and the walkthrough script, **When** both interpreters
   run the script with the same seed and width, **Then** the transcripts match byte for
   byte apart from ledgered differences.
2. **Given** the transcripts differ, **When** the comparison finishes, **Then** it reports
   the position of the first difference and the command that preceded it.
3. **Given** the reference interpreter is not installed, **When** the test suite runs,
   **Then** the walkthrough is still compared against the committed reference transcript,
   and only the check that regenerates that transcript is reported as skipped.
4. **Given** the walkthrough has run to its end, **When** the final transcript is
   inspected, **Then** it shows the game was won with the maximum score.

---

### User Story 5 - Save, restore and restart (Priority: P5)

A developer playing a game saves their position, keeps playing, and restores: the game is
exactly where it was when saved. They can also restart the game from the beginning.

**Why this priority**: Required for a complete version 3 interpreter and needed by the
mobile app later, but no other story in this feature depends on it.

**Independent Test**: Play some commands, save, play different commands, restore, then
play a fixed set of commands; the output matches a run that never left the saved position.

**Acceptance Scenarios**:

1. **Given** a game in progress, **When** the player saves, **Then** the host is handed
   the saved position to store and the game reports the save succeeded.
2. **Given** a saved position from the same story file, **When** the player restores it,
   **Then** play continues exactly as it would have from the moment of saving.
3. **Given** a saved position from a different story file or a damaged save, **When** the
   player tries to restore it, **Then** the restore is refused, the game reports failure,
   and play continues from the current position unharmed.
4. **Given** a game in progress, **When** the player restarts, **Then** the game returns
   to its opening state and prints its opening text.
5. **Given** the host declines or fails to store a save, **When** the player saves,
   **Then** the game reports the save failed and play continues.

---

### Edge Cases

- The story file performs an illegal operation (unknown instruction, division by zero,
  stack underflow, a read or write outside permitted memory, an invalid object number):
  the interpreter stops that game with a diagnostic naming the fault and where it
  occurred. It never takes the host down with it.
- The player types more characters than the game's input buffer allows, or more words
  than its parse buffer allows: the excess is dropped as the Standard describes.
- The player enters an empty line, or characters outside the game's character set.
- A story file's stated length or checksum does not match its contents: the game's own
  verify command reports failure; loading is not refused for this reason alone.
- A story file runs without ever asking for input (an endless loop): the host can stop it.
- A game asks for a random number in a predictable mode (seeds the generator itself).
- A word is longer than the dictionary's word length; a dictionary lookup falls on the
  first or last entry.
- Text contains abbreviations, shift characters and the multi-part escape for arbitrary
  characters, including at the very end of a string.
- A game prints to the status line with a score game and with a time game.
- Restoring a save taken at a different point in a nested routine call.

## Requirements *(mandatory)*

### Functional Requirements

**Loading and running**

- **FR-001**: The interpreter MUST load a version 3 story file supplied by the host and
  run it from its starting point.
- **FR-002**: The interpreter MUST refuse any file that is not a well-formed version 3
  story file, and MUST state why (wrong version, too short, unreadable header).
- **FR-003**: The interpreter MUST implement every instruction the Z-Machine Standard 1.1
  defines for version 3, with the behavior the Standard specifies.
- **FR-004**: The interpreter MUST implement the version 3 memory model, routine calls
  and stack, variables, the object tree with attributes and properties, the dictionary,
  input tokenisation, and text decoding and encoding including abbreviations.
- **FR-005**: The interpreter MUST set the header fields that an interpreter is
  responsible for, so a game can tell what the interpreter supports.

**Input and output**

- **FR-006**: The interpreter MUST perform no input or output of its own. Game text,
  status line content, player input, save storage, and randomness are all exchanged with
  the host through defined interfaces.
- **FR-007**: The interpreter MUST deliver game text to the host and MUST accept a line
  of player input when the game asks for one.
- **FR-008**: The interpreter MUST give the host the status line content (location name,
  and score and turns or the time) whenever the game updates it.
- **FR-009**: The host MUST be able to set the screen width the game sees, and line
  wrapping MUST honor that width.
- **FR-010**: The host MUST be able to stop a running game at any time, including one
  that never asks for input.

**Randomness and determinism**

- **FR-011**: The host MUST be able to supply a seed. With the same story file, input and
  seed, output MUST be identical on every run and every supported platform.
- **FR-012**: With a given seed, the sequence of random numbers MUST be the same sequence
  the reference interpreter produces for that seed, so that a game with random events
  follows the same course in both.
- **FR-013**: The interpreter MUST honor a game's own request to seed the generator or
  switch to predictable mode, as the Standard describes.
- **FR-014**: Without a seed from the host, random numbers MUST be unpredictable from run
  to run.

**Save, restore, restart**

- **FR-015**: The interpreter MUST support save, restore and restart. A saved position is
  handed to the host to store and is requested from the host to restore.
- **FR-016**: Saved positions MUST use the standard portable save format for Z-machine
  interpreters, so they are not tied to this interpreter's internals.
- **FR-017**: A restore MUST be refused when the save belongs to a different story file
  or is damaged, leaving the game in progress untouched.

**Errors**

- **FR-018**: When a story file performs an illegal operation, the interpreter MUST stop
  that game and report the kind of fault and where it occurred. The host MUST remain
  running and able to load another game.

**Command-line host**

- **FR-019**: A command-line host MUST allow a story file to be played interactively.
- **FR-020**: The command-line host MUST allow a story file to be played from a command
  script, with a chosen seed and screen width, writing a transcript of the game's output.
- **FR-021**: The command-line host MUST indicate through its exit status whether the run
  ended normally or with a fault.

**Fidelity and testing**

- **FR-022**: The automated test suite MUST include the czech version 3 run and MUST fail
  if czech reports any failed test.
- **FR-023**: The automated test suite MUST include the Zork 1 walkthrough comparison
  against the reference interpreter, and MUST fail on any difference not recorded in the
  divergence ledger.
- **FR-024**: A divergence ledger MUST exist. Each entry MUST identify the difference,
  cite the section of the Standard that justifies it, and be specific enough that the
  comparison excludes only that difference.
- **FR-025**: Tests that need a story file that is not in the repository MUST locate it
  through a local path outside version control and MUST be reported as skipped when it is
  absent.
- **FR-026**: The Zork 1 story file, the walkthrough command script and the reference
  transcript MUST be committed, with the story file's license, source and digest recorded
  beside it. No story file or transcript whose license does not permit redistribution may
  be committed.
- **FR-028**: The walkthrough comparison MUST run against the committed reference
  transcript, so it does not require the reference interpreter to be installed. When the
  reference interpreter is installed, the suite MUST also confirm that it still produces
  the committed transcript.
- **FR-027**: Every instruction MUST have tests that cite the section of the Standard
  they verify.

### Key Entities

- **Story file**: The game program. Identified by its version, release number, serial
  number and checksum. Read-only once loaded, apart from its dynamic memory.
- **Game state**: Everything that changes as a game runs: dynamic memory, the call stack,
  and the point of execution. This is what a save captures.
- **Saved position**: A portable snapshot of game state, tied to the story file it came
  from.
- **Command script**: An ordered list of player input lines used for unattended runs.
- **Transcript**: The complete text a game printed during a run, at a stated screen width.
- **Seed**: The value that fixes the sequence of random numbers for a run.
- **Divergence ledger**: The record of every place this interpreter intentionally differs
  from the reference interpreter, each justified by the Standard.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The czech conformance program for version 3 reports zero failed tests.
- **SC-002**: A scripted Zork 1 walkthrough, from the opening to winning with the maximum
  score, produces a transcript with zero unledgered byte differences from the reference
  interpreter's transcript for the same seed and screen width.
- **SC-003**: Ten consecutive runs of the same story file, command script and seed
  produce ten identical transcripts, on each supported development platform.
- **SC-004**: Every one of the instructions the Standard defines for version 3 is covered
  by at least one test citing the relevant section of the Standard.
- **SC-005**: The full Zork 1 walkthrough completes in under five seconds on a current
  development laptop, so fidelity tests are cheap enough to run on every change.
- **SC-006**: A position saved at any of at least five points in the walkthrough, then
  restored, yields the same remaining transcript as an uninterrupted run.
- **SC-007**: A set of at least twenty deliberately malformed or misbehaving story files
  each produce a diagnostic and none takes the host down.
- **SC-008**: The divergence ledger accounts for every difference the walkthrough
  comparison excludes; an entry with no corresponding difference is flagged.

## Assumptions

- **Constitution applies.** The project constitution fixes that the core is written in
  Rust, that the Standard 1.1 outranks Frotz, and that tests come first. This spec does
  not restate those.
- **Reference interpreter.** "Frotz" means its dumb-terminal build, run with a fixed seed
  and a fixed screen width of 80 columns. The exact version used is pinned and recorded
  during planning, because the comparison is only meaningful against a known version.
- **Matching the reference random sequence is permitted by the Standard.** The Standard
  does not prescribe how random numbers are generated, so producing the same sequence as
  the reference interpreter (FR-012) is a choice, not a deviation. Without it, a
  byte-for-byte match on a game with combat and a wandering thief is not achievable.
- **One Zork 1 release.** The walkthrough targets the compiled story file published in
  the MIT-licensed `historicalsource/zork1` repository, pinned by revision and digest.
  Other releases are not part of the acceptance test.
- **The walkthrough script is our own.** It is written for this project and committed.
  It is tuned to the pinned seed, since random events change which commands are needed.
- **Zork 1 is redistributable.** Its source was relicensed under the MIT License on
  2025-11-20, so the story file and transcripts of it are committed under constitution
  Principle VI (version 1.1.0). The Zork trademark is not covered by that license.
- **czech availability.** czech is assumed to be obtainable as a version 3 story file.
  Its archive states no license, so it is not committed; it is fetched locally and its
  test is skipped when absent.
- **Status line in transcripts.** The transcript comparison covers whatever the reference
  interpreter writes to its transcript. The status line is delivered to the host
  separately and is verified by its own tests.
- **Save compatibility with other interpreters** follows from using the standard format
  but is not an acceptance test for this feature.
- **No sound, no timed input, no extra windows.** These do not exist in version 3 as used
  by the target games; the sound effect instruction, where present, is accepted and
  ignored.
- **Supported development platforms** are macOS and Linux.
- **Out of scope:** versions 5 and 8 and the praxix test program (the next feature), the
  introspection interface, the experience layer, C bindings and the mobile shells.
