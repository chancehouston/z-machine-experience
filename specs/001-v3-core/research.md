# Research: Faithful Z-Machine Version 3 Core

**Date**: 2026-11-11 (revised after independent review the same day) | **Plan**: [plan.md](plan.md)

Each item records what was decided, why, and what else was considered. Evidence is
labelled: **Observed** means run on this machine with dfrotz 2.55; **Read** means taken
from a primary source and not yet exercised. A task that first relies on a Read item must
confirm it.

## R1. The reference and how transcripts are compared

**Decision**: The reference interpreter is dfrotz from Frotz 2.55 (Homebrew bottle;
`dfrotz -v` reports git commit `acf205585a9472d27c07c0fe62da4b8bc89d1ec7`, "Official
release"). The reference transcript is the **transcript file Frotz writes** when the game
turns transcription on, with wrapping disabled:

```text
dfrotz -m -q -w 80 -S 0 -s <seed> <story> < <script>
```

The command script begins with the game's own `script` command followed by a line giving
the transcript file name. Everything after that is recorded.

**Observed**:

- The transcript file is plain game text. Each paragraph is one unwrapped line. The
  status line does not appear in it.
- Each command is echoed on the same line as the game's `>` prompt, followed by a line
  feed. Answers to in-game questions are echoed the same way (`...(Y is affirmative): >y`).
- File names given to the transcript, save and restore prompts are read from the next
  line of standard input and do not appear in the transcript.
- When input ends while the game is waiting, the file ends with the bare prompt and no
  line feed; `EOT` goes to standard error and the exit status is 0.
- Two runs with the same seed give byte-identical transcript files; a different seed
  gives a different file once combat occurs.
- Frotz appends `.scr` to the transcript name and `.qzl` to a save name without one.

**Rationale**: The original plan compared dfrotz's standard output. Review, then
observation, showed that output is a diff of a 24-row screen grid: the status row is
reprinted after most commands, runs of blank lines collapse, long turns gain a blank
line, and commands are not echoed. Matching it would mean emulating Frotz's screen in our
host, which tests a terminal, not an interpreter. The transcript file contains exactly
what the game printed and what the player typed. It is also the literal reading of the
handoff's "transcript matching Frotz byte for byte", and it exercises a real version 3
feature (output stream 2).

**Alternatives considered**: dfrotz standard output with `-r ch1 -h 255` to hide the
status row and height effects (still a screen diff, with a fixed extra first line); full
emulation of the screen grid (large, and outside what "faithful interpreter" means).

**Also found**: `-p` (plain ASCII) has no effect in dfrotz 2.55, because the flag sets a
variable the output code never reads. It is not used. Zork 1's text is ASCII.

## R2. Matching the reference random sequence

**Decision**: The core's generator reproduces the behavior of Frotz 2.55
(`src/common/random.c`), reimplemented from its description, not copied:

- State is one 32-bit value `A` plus `interval` and `counter` for predictable mode.
- **The host seed goes straight into `A`**, whatever its value, at start and on every
  restart. It does not enter predictable mode.
- A game's own `random` with a zero or negative argument reseeds and returns 0. With
  argument 0, `A` is taken from the host seed again. With magnitude 1 to 999, predictable
  mode is entered: `counter` is reset to 0 and `interval` set to the magnitude. With
  magnitude 1000 or more, `A` is set to the magnitude and predictable mode is left. The
  magnitude of -32768 is 32768, so the argument is widened before negating.
- `random` with a positive argument `range`: in predictable mode the raw value is
  `counter`, which then increments and wraps to 0 on reaching `interval`. Otherwise
  `A = 0x015a4e35 * A + 1` in wrapping 32-bit arithmetic and the raw value is
  `(A >> 16) & 0x7fff`. The result is `(raw % range) + 1`.
- Restore does not touch the generator.

**Evidence**: Read, confirmed line by line by an independent reviewer against the 2.55
source, and Observed: a probe story printing 260 results of `random 100` (200 from the
host seed, then 20 after each of `random -5`, `random -1000` and `random 0`) was run
under dfrotz at seeds 0, 1, 999, 1000, 123456 and 2147483647. A model of the rules above
reproduces all six outputs exactly. The outputs are committed as
`crates/zmachine-core/tests/fixtures/random/seed_<n>.txt`.

**Rationale**: Standard §2.4 and the `random` opcode leave the generator to the
interpreter, so this is a free choice. Without it a game with combat and a wandering
thief takes a different course and no byte-for-byte comparison is possible.

**Seed range**: dfrotz reads `-s` with `atoi` into a signed integer and treats -1 as
"use the clock". Seeds for comparison are therefore limited to 0 to 2147483647.

**Licensing note**: Frotz is GPL-2.0-or-later and this project is MIT. Constants and
algorithms are facts and are restated here in our own words. Frotz source may be read to
learn how the reference behaves; none of its text is copied into this repository.

## R3. Zork 1 story file and its license

**Finding**: On 2025-11-20 Microsoft, with Xbox and Activision, relicensed the Zork I, II
and III source under the MIT License. `historicalsource/zork1` reports license MIT and
contains `COMPILED/zork1.z3`: 86,838 bytes, version 3, release 119, serial 880429, header
checksum valid, SHA-256
`37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79` (Observed). The
license commit excludes trademarks.

**Decision**: That file is the acceptance story file, committed at
`stories/zork1/zork1.z3` with the repository's license text and a `SOURCE.md` recording
URL, commit, digest, release and serial. Walkthrough, reference transcripts and reference
saves live under `stories/zork1/fidelity/`, beside that record, as Principle VI requires.
The project owner approved committing it on 2026-11-11, and the constitution was amended
to version 1.1.0.

**Known uncertainty, accepted**: The license text covers "this software and associated
documentation files" and the repository's README describes the compiled file as an
Infocom artefact found on the original drive. Whether the grant covers the compiled file
is a legal reading, not a fact, and no task can settle it. The owner approved committing
with this stated. If the reading proves wrong, the file, transcripts and saves are
removed from history and the fidelity tests revert to a local path.

## R4. czech

**Finding** (Read, from the archive's contents): `czech_0_8.zip` contains `czech.inf`,
`README.txt`, `CHANGES.txt`, `czech.z5`, and expected outputs `czech.out3`, `.out4`,
`.out5`, `.out8`. There is no version 3 build. The README's license (Copyright 2003 Amir
Karger) permits use, distribution and modification provided the notice is kept. The
version 3 expected output ends "Passed: 349, Failed: 0, Print tests: 19".

**Decision**: Commit `stories/czech/` containing `czech.inf`, `README.txt` (the license
notice), `czech.out3`, a `czech.z3` compiled once with Inform 6 (`inform6 -v3`, Homebrew
`inform6` 6.44), and a `SOURCE.md` with URL, digests and the compile command. The test
checks the summary line and compares the print-test output with `czech.out3`.

**Rationale**: With the license permitting redistribution, committing removes a fetch
step and a skip path. Inform 6 is needed once, by whoever builds `czech.z3`, not by
everyone who runs the tests.

**To confirm at the task**: the license wording, read directly from `README.txt`; and how
much of `czech.out3` is interpreter-specific (header and formatting lines) and so must be
left out of the comparison. czech targets Standard 1.0 with some 1.1 fixes; an
expectation that conflicts with Standard 1.1 is ledgered, not obeyed.

## R5. How the host drives the core

**Decision**: The core is a resumable state machine. `run(budget)` executes instructions
until something needs the host, then returns an event: line wanted, save requested,
restore requested, transcript requested, finished, fault, or budget exhausted. The host
answers and calls `run` again. Game text, transcript text, status line updates and
diagnostics are collected by the core and drained by the host.

**Rationale**: A phone UI cannot block a thread inside the interpreter waiting for a tap,
and C bindings are far simpler over "call, get event, answer" than over callbacks into
Swift or Kotlin. The instruction budget satisfies FR-010 without threads or signals.

**Alternatives considered**: Host traits with blocking `read_line` (wrong shape for the
shells this core exists to serve); async Rust (pulls a runtime concept into a crate that
must have no dependencies).

**Reviewed**: An independent reviewer found no version 3 instruction needing host
interaction beyond these events.

## R6. Output streams, input echo and wrapping

**Decision**:

- The core emits screen text (stream 1) unwrapped. It does not echo input into it; the
  host has already shown what the player typed.
- The core produces transcript text (stream 2) while Flags 2 bit 0 is set: the same game
  text, plus each input line echoed and followed by a line feed. This is what R1 compares.
- When the game sets Flags 2 bit 0, the core raises a transcript-requested event before
  the next output or input. If the host declines, the core clears the bit.
- `output_stream` selects and deselects streams as Standard §7 specifies for version 3.
  Deselecting stream 1 suppresses screen text.
- Wrapping to a width is done by the host. The CLI wraps its terminal output at word
  boundaries for readability; that output is not compared with anything.

**Rationale**: A version 3 header has no screen width field, so the game never sees one.
The phone UI will lay out text itself.

## R7. Save format

**Decision**: Quetzal 1.4.

- `IFhd`: release, serial, checksum, and the program counter; 13 bytes plus a pad byte.
  For version 3 the saved program counter points at the branch data of the `save`
  instruction (Quetzal §5.8.1).
- `CMem`: dynamic memory XORed with the original and run-length encoded. On restore,
  decoded data **shorter** than dynamic memory is accepted and the remainder is unchanged;
  longer is an error (§3.4, §3.5). Frotz writes short `CMem` chunks deliberately.
- `UMem` is accepted on restore and must be exactly the dynamic memory size (§3.6).
- `Stks`: begins with the dummy frame holding the top-level evaluation stack (§4.11),
  then the real frames.
- A successful restore resumes inside the saved `save` instruction with its branch taken;
  the `restore` instruction's own branch is never taken on success in version 3.
- After restore and after restart, the interpreter re-applies the header fields it owns,
  and Flags 2 survives (Standard §6.1.2, §6.1.3).
- Restore is refused when `IFhd` does not match the loaded story or a chunk is malformed.

**Evidence**: Read. dfrotz save and restore driven from a script were Observed to work,
which makes cross-checking possible.

**Rationale**: The standard portable format (FR-016). Compatibility is tested in both
directions against Frotz, because a round trip through our own code would pass even with
a wrong convention on both sides.

## R8. Toolchain, crates and where tests live

**Decision**: Rust 1.99 stable, edition 2024, one Cargo workspace with three crates:

- `zmachine-core`: no dependencies, `#![forbid(unsafe_code)]`.
- `zmachine-testkit`: builds version 3 story images in memory. Depends on nothing. Used
  only as a development dependency.
- `zmachine-cli`: a library (`lib.rs`) holding the driver and formatting, and a thin
  `main.rs`. Depends on the core only.

Tests of internals (decoder, memory rules, text, objects, dictionary) are unit tests
inside the module they test. Tests of behavior visible through the public API are
integration tests in each crate's `tests/` directory. `sha2` is a development dependency
for checking committed files against their recorded digests.

**Rationale**: The public API deliberately hides memory and objects, so tests of those
must sit inside the crate. A separate testkit crate lets the core and the CLI share the
image builder without a feature flag on the core.

## R9. Testing instructions without a compiler

**Decision**: The testkit builds an image from a header description, objects, a
dictionary and hand-assembled instruction bytes, with helpers for each operand form and
for Z-encoded strings. Its encoders are themselves tested against known vectors taken
from the Standard and from the committed Zork 1 file, so an encoder bug cannot be
mirrored unseen by the implementation.

## R10. Divergence ledger

**Decision**: `docs/divergences.md` is the single ledger. Each entry has an identifier, a
description, the Standard section, and a fenced rule of one of two kinds:

- `transcript`: story digest, index of the command after which the difference occurs,
  reference text, our text.
- `czech`: the name of a czech test whose expectation conflicts with Standard 1.1.

The fidelity and czech tests parse these rules and apply exactly them. A rule that
matches nothing fails the test (SC-008). The parser and the substitution are tested with
synthetic inputs, since the real ledger may well stay empty.

## R11. Reference artefacts are committed

**Decision**: The walkthrough, its seed, the reference transcript, and the reference
saves and continuation transcripts are committed under `stories/zork1/fidelity/`. The
fidelity tests compare against these files and need no dfrotz. Separate tests, marked
`#[ignore = "needs dfrotz 2.55"]`, regenerate each artefact with dfrotz and fail if it
differs; CI runs them with `--ignored` after installing dfrotz and checking its version.

**Rationale**: Rust's test harness cannot mark a test skipped at run time. With Zork 1
and czech committed, the only tests with an optional prerequisite are the regeneration
checks, and a static `#[ignore]` reports them honestly as ignored, never as passed.

## R12. Illegal operations on object 0

**Decision**: Instructions given object 0 do not stop the game. They do nothing or return
0, as the common interpreters do, and the core records a diagnostic that the host can
read. Nothing is added to the game's text.

**Rationale**: The Standard calls these operations illegal but several published games
perform them, and an interpreter that stops is unusable for those games. Frotz prints a
warning into the game text and continues. If the Zork 1 walkthrough triggers one, the
reference transcript will contain Frotz's warning and ours will not; that would be a
ledger entry. An early task runs the walkthrough under `dfrotz -Z 2` to find out.

**Observed**: the finished walkthrough was run under `dfrotz -Z 2`. No warning appears and
the transcript is identical to the one made without it, so Zork I performs no object 0
operation on this route and the ledger needs no entry for one.

**Observed**: dfrotz stops with a fatal error when a version 3 game asks for input while
its first global variable, the location shown on the status line, is object 0. This
interpreter reports an empty location name and a diagnostic, and continues.

**Alternatives considered**: Fault, as first planned (strict, but would stop real games
the reference plays); printing Frotz's warning text into the game output (imitates a
Frotz message that is not game text).

## R13. Continuous integration

**Decision**: GitHub Actions on macOS and Linux runs `cargo test --workspace`, then
installs dfrotz, checks it reports 2.55, and runs the ignored regeneration tests and the
timing test in a release build. The committed reference files being identical on both
platforms is the cross-platform evidence for SC-003.

## Sources

- Z-Machine Standards Document 1.1: <https://inform-fiction.org/zmachine/standards/z1point1/index.html>
- Quetzal 1.4: <https://inform-fiction.org/zmachine/standards/quetzal/index.html>
- Frotz 2.55: <https://gitlab.com/DavidGriffith/frotz/-/tree/2.55>
- Homebrew formula for frotz: <https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/f/frotz.rb>
- czech 0.8 contents: <https://unbox.ifarchive.org/?url=/if-archive/infocom/interpreters/tools/czech_0_8.zip>
- Zork 1 source repository: <https://github.com/historicalsource/zork1>
- Relicensing coverage: <https://www.theregister.com/2025/11/21/microsoft_zork_source/>, <https://hackaday.com/2025/11/21/microsoft-open-sources-zork-i-ii-and-iii/>
