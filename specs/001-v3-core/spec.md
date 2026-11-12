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
the commands in order without a person present. When the script turns on the game's own
transcript feature, the host writes a transcript file of everything the game printed and
everything that was typed. Running it again produces the identical transcript.

**Why this priority**: Every fidelity test in this project is a scripted run compared
against an expected transcript. Repeatability is what makes those tests meaningful.

**Independent Test**: Run the same story file, command script and seed twice and compare
the two transcripts; they must be identical.

**Acceptance Scenarios**:

1. **Given** a story file, a command script that turns transcription on, and a seed,
   **When** the host runs the script, **Then** it writes a transcript file containing the
   game's text and the echoed commands, and exits when the script or the game ends.
2. **Given** the same story file, script and seed, **When** the run is repeated any number
   of times, on any supported platform, **Then** every transcript is identical byte for
   byte.
3. **Given** a script that ends while the game is still waiting for input, **When** the
   host reaches the end of the script, **Then** it stops cleanly and the transcript
   contains everything printed up to that point.
4. **Given** any terminal width setting, **When** a scripted run is performed, **Then**
   the transcript file is unaffected: it holds unwrapped text, one paragraph per line.

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
Frotz) with the same seed. Each writes a transcript file through the game's own
transcript feature, with wrapping off. The two files are identical byte for byte, except
for differences recorded in the divergence ledger.

**Why this priority**: This is the project's headline definition of "faithful". It comes
last because it only passes once everything else is right.

**Independent Test**: Produce both transcript files from the same script and seed and
compare them; the comparison reports no unledgered difference.

**Acceptance Scenarios**:

1. **Given** the committed Zork 1 story file and the walkthrough script, **When** both interpreters
   run the script with the same seed, **Then** the transcript files match byte for
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
exactly where it was when saved. They can also restart the game from the beginning. Saves
are interchangeable with the reference interpreter's.

**Why this priority**: Required for a complete version 3 interpreter and needed by the
mobile app later, but no other story in this feature depends on it.

**Independent Test**: Take a save at a known point. In a fresh run with a fixed seed,
restore it and play a fixed set of commands; the transcript matches the reference
interpreter doing the same with the same save. Repeat with a save written by the reference
interpreter.

**Acceptance Scenarios**:

1. **Given** a game in progress, **When** the player saves, **Then** the host is handed
   the saved position to store and the game reports the save succeeded.
2. **Given** a saved position from the same story file, **When** the player restores it,
   **Then** the game's memory and call stack are exactly as they were at the moment of
   saving. Random events afterwards depend on the current random state, which a save does
   not carry.
6. **Given** a save written by the reference interpreter, **When** this interpreter
   restores it, **Then** it is accepted and play continues; and the reverse also holds.
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
- The player enters an empty line: it is passed to the game, which gives its own reply.
- The player enters characters the game's character set cannot represent: each is
  replaced as the Standard directs for input.
- A story file applies an instruction to object number 0, which the Standard forbids but
  some published games do: the instruction has no effect or yields zero, the game
  continues, and the host can read a diagnostic. Nothing is added to the game's text.
- A story file's stated length or checksum does not match its contents: the game's own
  verify command reports failure; loading is not refused for this reason alone.
- A story file runs without ever asking for input (an endless loop): the host can stop it.
- A game seeds the generator itself: the sequence that follows is the one the reference
  interpreter produces for the same request.
- A word is longer than the dictionary's word length; a dictionary lookup falls on the
  first or last entry.
- Text contains abbreviations, shift characters and the multi-part escape for arbitrary
  characters, including at the very end of a string.
- A score game and a time game: the status line content reflects the kind of game.
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
  transcript text, status line content, player input, save storage, and randomness are
  all exchanged with the host through defined interfaces.
- **FR-007**: The interpreter MUST deliver game text to the host and MUST accept a line
  of player input when the game asks for one.
- **FR-008**: The interpreter MUST give the host the status line content (location name,
  and score and turns or the time) whenever the game updates it.
- **FR-009**: The interpreter MUST deliver game text unwrapped, leaving layout to the
  host. The command-line host MUST wrap its terminal output at a width the user can set.
- **FR-029**: The interpreter MUST support the game's transcript feature: while the game
  has transcription on, the host receives the game's text together with each line of
  player input, and can decline the request to start.
- **FR-030**: The interpreter MUST let the game select and deselect its output streams as
  the Standard specifies for version 3.
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
- **FR-031**: Saves written by this interpreter MUST be restorable by the reference
  interpreter, and saves written by the reference interpreter MUST be restorable by this
  one.

**Errors**

- **FR-018**: When a story file performs an illegal operation, the interpreter MUST stop
  that game and report the kind of fault and where it occurred. The host MUST remain
  running and able to load another game. The one exception is an instruction applied to
  object 0, which is tolerated and reported as a diagnostic.

**Command-line host**

- **FR-019**: A command-line host MUST allow a story file to be played interactively.
- **FR-020**: The command-line host MUST allow a story file to be played from a command
  script with a chosen seed, writing a transcript file when the script turns the game's
  transcript on. The same script MUST be usable with the reference interpreter.
- **FR-032**: The command-line host MUST be able to stop a story file that runs beyond a
  given amount of work without asking for input, ending with a fault status.
- **FR-021**: The command-line host MUST indicate through its exit status whether the run
  ended normally or with a fault.

**Fidelity and testing**

- **FR-022**: The automated test suite MUST include the czech version 3 run and MUST fail
  if czech reports any failed test or if its printed-output tests differ from czech's
  published expected output, ledgered conflicts with the Standard excepted.
- **FR-023**: The automated test suite MUST include the Zork 1 walkthrough comparison
  against the reference interpreter, and MUST fail on any difference not recorded in the
  divergence ledger.
- **FR-024**: A divergence ledger MUST exist. Each entry MUST identify the difference,
  cite the section of the Standard that justifies it, and be specific enough that the
  comparison excludes only that difference.
- **FR-025**: Tests that need a story file that is not in the repository MUST locate it
  through a local path outside version control and MUST be reported as skipped when it is
  absent.
- **FR-026**: The Zork 1 story file, the czech test program, the walkthrough command
  script and the reference transcripts and saves MUST be committed, each beside a record
  of the story file's license, source and digest. No story file or transcript whose license does not permit redistribution may
  be committed.
- **FR-027**: Every instruction MUST have tests that cite the section of the Standard
  they verify.
- **FR-028**: The walkthrough comparison MUST run against the committed reference
  transcript, so it does not require the reference interpreter to be installed. When the
  reference interpreter is installed, the suite MUST also confirm that it still produces
  the committed transcript.

### Key Entities

- **Story file**: The game program. Identified by its version, release number, serial
  number and checksum. Read-only once loaded, apart from its dynamic memory.
- **Game state**: Everything that changes as a game runs: dynamic memory, the call stack,
  and the point of execution. This is what a save captures. The random number state is
  not part of it.
- **Saved position**: A portable snapshot of game state, tied to the story file it came
  from.
- **Command script**: An ordered list of player input lines used for unattended runs.
- **Transcript**: The file recording a run through the game's own transcript feature: the
  game's text, unwrapped, with each line of player input.
- **Seed**: The value that fixes the sequence of random numbers for a run.
- **Divergence ledger**: The record of every place this interpreter intentionally differs
  from the reference interpreter, each justified by the Standard.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The czech conformance program for version 3 reports zero failed tests.
- **SC-002**: A scripted Zork 1 walkthrough, from the opening to winning with the maximum
  score, produces a transcript file with zero unledgered byte differences from the
  reference interpreter's transcript file for the same seed.
- **SC-003**: Ten consecutive runs of the same story file, command script and seed
  produce ten identical transcripts, on each supported development platform.
- **SC-004**: Every one of the instructions the Standard defines for version 3 is covered
  by at least one test citing the relevant section of the Standard.
- **SC-005**: The full Zork 1 walkthrough completes in under five seconds on a current
  development laptop in an optimized build, so fidelity tests are cheap enough to run on
  every change.
- **SC-006**: For saves taken at no fewer than five points in the walkthrough, a fresh
  run that restores the save and plays a fixed continuation produces a transcript file
  identical to the reference interpreter's for the same save and seed. This holds for
  saves written by this interpreter and for saves written by the reference interpreter.
- **SC-007**: A set of at least twenty deliberately malformed or misbehaving story files
  each produce a diagnostic and none takes the host down.
- **SC-008**: The divergence ledger accounts for every difference the walkthrough
  comparison excludes; an entry with no corresponding difference is flagged.

## Assumptions

- **Constitution applies.** The project constitution fixes that the core is written in
  Rust, that the Standard 1.1 outranks Frotz, and that tests come first. This spec does
  not restate those.
- **Reference interpreter.** "Frotz" means the dumb-terminal build of Frotz 2.55, run with
  a fixed seed. "Transcript" means the file Frotz writes through the game's transcript
  feature with wrapping off, not Frotz's terminal output, which is a redrawn screen and
  includes the status line.
- **Matching the reference random sequence is permitted by the Standard.** The Standard
  does not prescribe how random numbers are generated, so producing the same sequence as
  the reference interpreter (FR-012) is a choice, not a deviation. Without it, a
  byte-for-byte match on a game with combat and a wandering thief is not achievable.
- **One Zork 1 release.** The walkthrough targets the compiled story file published in
  the MIT-licensed `historicalsource/zork1` repository, pinned by revision and digest.
  Other releases are not part of the acceptance test.
- **The walkthrough script is our own.** Its commands are entered by us while playing
  against the reference interpreter; maps and hints may be consulted, but no published
  command list is copied. It is tuned to the pinned seed, since random events change
  which commands are needed.
- **Zork 1 is redistributable.** Its source was relicensed under the MIT License on
  2025-11-20, so the story file and transcripts of it are committed under constitution
  Principle VI (version 1.1.0). The Zork trademark is not covered by that license.
- **czech.** Its license permits redistribution with its notice, so it is committed. The
  archive has no version 3 build, so one is compiled once from its source and committed.
- **Status line.** It is not part of a transcript file. It is delivered to the host
  separately and is verified by its own tests.
- **A save does not carry random state**, in the standard format or in the reference
  interpreter. Restoring returns the game to a position, not to a fixed future.
- **No sound, no timed input, no extra windows.** These do not exist in version 3 as used
  by the target games; the sound effect instruction, where present, is accepted and
  ignored.
- **Supported development platforms** are macOS and Linux.
- **Out of scope:** versions 5 and 8 and the praxix test program (the next feature), the
  introspection interface, the experience layer, C bindings and the mobile shells.
