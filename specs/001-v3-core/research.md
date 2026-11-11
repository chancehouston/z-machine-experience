# Research: Faithful Z-Machine Version 3 Core

**Date**: 2026-11-11 | **Plan**: [plan.md](plan.md)

Each item records what was decided, why, and what else was considered. Items marked
**Verify** rest on a source that was read but not yet exercised on this machine; the task
that first touches them must confirm the fact before building on it.

## R1. Reference interpreter and how it is invoked

**Decision**: dfrotz from Frotz 2.55, installed with Homebrew (`brew install frotz`, which
builds the `frotz`, `dumb` and `sdl` targets). The reference transcript is dfrotz's
standard output with the command script on standard input, run as
`dfrotz -m -p -q -w 80 -s <seed> <story>`.

**Rationale**: 2.55 is the current Homebrew stable and the version whose source was read
for R2. `-m` removes MORE prompts, `-p` forces plain ASCII, `-q` removes the startup
banner, `-w` fixes the width and `-s` fixes the seed, so the output depends only on the
story file and the script.

**Alternatives considered**: Using the in-game SCRIPT command to produce the transcript
(adds commands to the walkthrough and a second wrapping rule, `-S`); the curses build
(output is screen-addressed, not a byte stream).

**Verify**: dfrotz is not installed on this machine yet. The manual page does not say how
dfrotz treats piped input (whether it echoes the command), how it renders the version 3
status line, or what it writes at end of input. The first fidelity task characterizes
these by observation and records them in
[contracts/transcript-format.md](contracts/transcript-format.md).

## R2. Matching the reference random sequence

**Decision**: The core's generator reproduces the behavior of Frotz 2.55
(`src/common/random.c`), reimplemented from its description, not copied:

- State is one 32-bit value `A` plus `interval` and `counter` for predictable mode.
- Seeding with a value from 1 to 999 enters predictable mode: `counter` is reset to 0 and
  `interval` is set to the value. Seeding with 1000 or more sets `A` to the value and
  leaves predictable mode. Seeding with 0 takes `A` from the host's seed source and leaves
  predictable mode.
- `random` with a zero or negative argument reseeds with the absolute value and returns 0.
- `random` with a positive argument `range`: in predictable mode the raw value is
  `counter`, which then increments and wraps to 0 on reaching `interval`. Otherwise
  `A = 0x015a4e35 * A + 1` in wrapping 32-bit arithmetic and the raw value is
  `(A >> 16) & 0x7fff`. The result is `(raw % range) + 1`.
- At start and on every restart the generator is seeded with 0, so the host seed applies.

**Rationale**: Standard §2.4 and the `random` opcode leave the generator to the
interpreter, so this is a free choice. Without it a game with combat and a wandering thief
takes a different course and no byte-for-byte comparison is possible.

**Alternatives considered**: A modern generator with a walkthrough restricted to
non-random passages (cannot reach the end of Zork 1); recording Frotz's random outputs and
replaying them (couples tests to a recording, and the core would still need a replay
hook).

**Licensing note**: Frotz is GPL-2.0-or-later and this project is MIT. The constants and
the algorithm are facts and are restated here in our own words; no Frotz source text is
copied into this repository.

**Verify**: Confirm against dfrotz by running a tiny story file that prints a run of
`random` results under several seeds, including one below 1000 passed with `-s`.

## R3. Zork 1 story file and its license

**Finding**: The handoff and constitution Principle VI treat Zork as commercial and
Activision's. That changed on 2025-11-20: Microsoft, with Xbox and Activision, relicensed
the Zork I, II and III source under the MIT License. The public repository
`historicalsource/zork1` reports license MIT and contains `COMPILED/zork1.z3` (86,838
bytes). The license covers the code; the Zork trademark and packaging are not included.

**Decision**: The acceptance story file is `COMPILED/zork1.z3` from
`historicalsource/zork1`, taken at a specific commit. It is committed at
`stories/zork1/zork1.z3` with that repository's license text and a `SOURCE.md` recording
the URL, commit, SHA-256 digest, release and serial number. The project owner approved
this on 2026-11-11 and the constitution was amended to version 1.1.0 to allow it.

**Rationale**: A committed, pinned file makes the headline test reproducible by anyone
and runnable in CI with no fetch step, and lets the reference transcript be committed too.

**Alternatives considered**: A copy supplied by the project owner (not reproducible by
others); fetching at test time (works, but adds a network dependency and forces a
fingerprint in place of a readable reference transcript).

**Verify**: Whether the MIT grant in that repository is understood to cover the compiled
file as well as the source. The file sits in the licensed repository beside the license,
which is the basis for committing it; if that reading proves wrong, the file and
transcript are removed and the test reverts to a local path.

## R4. czech

**Decision**: czech 0.8 (`czech_0_8.zip`, IF Archive, Amir Karger, 2003) is fetched by
a script into the gitignored `stories/local/` and not committed.

**Rationale**: The archive listing states no license, so redistribution rights are
unknown; treating it as a local story file complies with Principle VI either way.

**Verify**: That the archive contains a version 3 build (`czech.z3`) and an expected
output file for it. If it contains only source, the task compiles it with Inform 6 for
version 3, or the project owner is asked. czech targets Standard 1.0 with some 1.1 fixes;
any czech expectation that conflicts with Standard 1.1 is ledgered, not obeyed.

## R5. How the host drives the core

**Decision**: The core is a resumable state machine, not a set of blocking callbacks.
`run(budget)` executes instructions until something needs the host, then returns an event:
line input wanted, save requested, restore requested, game finished, fault, or budget
exhausted. The host answers and calls `run` again. Game text and status line updates are
collected by the core and drained by the host. A small blocking driver built on this loop
serves simple hosts such as the CLI.

**Rationale**: A phone UI cannot block a thread inside the interpreter waiting for a tap,
and C bindings are far simpler over "call, get event, answer" than over callbacks into
Swift or Kotlin. The instruction budget satisfies FR-010 without threads or signals. The
machine stays a plain value, which makes determinism and save straightforward.

**Alternatives considered**: Host traits with blocking `read_line` (simplest for a CLI,
wrong shape for the shells this core exists to serve); async Rust (pulls a runtime
concept into a crate that must have no dependencies).

## R6. Where line wrapping and the screen width live

**Decision**: The core emits unwrapped text. Wrapping to a width is done by a
dumb-terminal formatter in the CLI crate, which reproduces dfrotz's conventions.

**Rationale**: A version 3 header has no screen width field (that arrives in version 4),
so the game never sees the width; FR-009 is purely a presentation matter. The phone UI
will lay out text itself and must not inherit 80-column wrapping.

**Alternatives considered**: Wrapping inside the core (forces every host to undo it).

## R7. Save format

**Decision**: Quetzal 1.4: `IFhd` (release, serial, checksum, program counter), `CMem`
(dynamic memory XORed against the original and run-length encoded) and `Stks` (call
frames). `UMem` is accepted on restore. Restore is refused when `IFhd` does not match the
loaded story file or any chunk is malformed.

**Rationale**: The standard portable format (FR-016); written and read by Frotz, which
allows cross-checking saves during development.

**Alternatives considered**: A private snapshot format (smaller code, ties saves to our
internals, and the phone app would need a migration later).

## R8. Toolchain and dependencies

**Decision**: Rust 1.99 stable, edition 2024, one Cargo workspace. `zmachine-core` has no
dependencies and `#![forbid(unsafe_code)]`. `zmachine-cli` depends on the core only;
argument parsing is hand-written (four options). `sha2` is a development dependency, used
to check story file digests against their recorded values.

**Rationale**: A dependency-free core is trivially portable to iOS and Android targets and
keeps Principle V checkable by reading one manifest.

**Alternatives considered**: `clap` for the CLI (more than four options need); a
`no_std` core (attractive, but `alloc`-only discipline buys nothing until a target
needs it; revisit with the C bindings).

## R9. Testing instructions without a compiler

**Decision**: A small test-support module builds version 3 story images in memory: a
header, a few objects, a dictionary, and hand-assembled instruction bytes. Each opcode
test builds an image, runs it, and asserts on memory, stack, output or the event returned.
The same builder produces the malformed images for SC-007.

**Rationale**: Tests can cite a Standard section and exercise one instruction form
exactly, including encodings no compiler would emit. No external assembler is needed and
no story file has to be committed.

**Alternatives considered**: Committing story files compiled with Inform 6 (opaque
binaries, a toolchain to install, and awkward for malformed cases).

## R10. Divergence ledger format

**Decision**: `docs/divergences.md` is the single ledger. Each entry has an identifier, a
plain description, the Standard section, and a fenced rule naming the story file digest,
the index of the command after which the difference occurs, the reference text and our
text. The fidelity test parses these rules, applies exactly those substitutions to the
reference transcript, and fails on any rule that matches nothing (SC-008).

**Rationale**: One file serves both the human reader and the test, so they cannot drift.

**Alternatives considered**: A separate machine-readable file alongside the document
(two sources of truth).

## R11. The reference transcript is committed

**Decision**: `tests/fidelity/zork1.reference.txt` is dfrotz's output for the pinned
story file, script, seed and width, committed as generated. The fidelity test compares
our transcript to it after ledger substitutions and reports the first differing byte. A
second test, skipped when dfrotz is absent, regenerates the transcript with dfrotz and
fails if it no longer equals the committed file.

**Rationale**: The comparison runs anywhere, including CI images without dfrotz, and a
failure can always show where the difference is. The regeneration test catches a changed
dfrotz or an edited reference file.

## R12. Continuous integration

**Decision**: A GitHub Actions workflow runs the test suite on macOS and Linux, fetching
czech with the script. It is the evidence for SC-003's "each supported
platform".

**Rationale**: Determinism across platforms cannot be shown from one laptop.

## Sources

- Frotz 2.55 `src/common/random.c` and `doc/dfrotz.6`: <https://gitlab.com/DavidGriffith/frotz/-/tree/2.55>
- Homebrew formula for frotz: <https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/f/frotz.rb>
- IF Archive, interpreter tools (czech, praxix): <https://ifarchive.org/indexes/if-archive/infocom/interpreters/tools/>
- Zork 1 source repository: <https://github.com/historicalsource/zork1>
- Relicensing coverage: <https://www.theregister.com/2025/11/21/microsoft_zork_source/>, <https://hackaday.com/2025/11/21/microsoft-open-sources-zork-i-ii-and-iii/>
