# Data Model: Faithful Z-Machine Version 3 Core

**Date**: 2026-11-11 (revised after independent review) | **Plan**: [plan.md](plan.md)

Section numbers refer to the Z-Machine Standard 1.1 unless marked Quetzal.

## Story file

The game program as loaded. Immutable after loading; the machine keeps the original for
restart and for save compression.

| Field | Meaning | Rule |
| --- | --- | --- |
| version | Header byte 0 | Must be 3, otherwise load is refused (FR-002) |
| release | Header word at 0x02 | Part of the story identity |
| serial | Six bytes at 0x12 | Part of the story identity |
| checksum | Header word at 0x1C | Part of the story identity; used by `verify`, not by load |
| high memory base, initial PC, dictionary, object table, globals, static memory base, abbreviations | Header words (§11) | Each must lie inside the file; static base must be at least 64 |
| length | Header word at 0x1A, times 2 | Used by `verify` only; a mismatch with the file size is not a load error |

**Identity** is (release, serial, checksum). A save belongs to exactly one identity.

## Machine

Everything that changes while a game runs.

| Part | Contents | Notes |
| --- | --- | --- |
| Dynamic memory | Bytes from 0 to the static memory base | The only writable memory (§1.1). Writes elsewhere are faults. |
| Call stack | Frames, innermost last | See Frame. Limited to 1024 frames; exceeding it is a fault. |
| Program counter | Address of the next instruction | |
| Random state | `A`, `interval`, `counter` | [research.md](research.md) R2. Not part of a save. |
| Stream state | Which output streams are selected | §7; stream 2 mirrors Flags 2 bit 0 |
| Screen buffer | Stream 1 text since the host last drained it | Unwrapped; no input echo |
| Transcript buffer | Stream 2 text since the host last drained it | Includes echoed input |
| Status line | Latest status line content, with a changed flag | See Status line |
| Diagnostics | Tolerated illegal operations since last drained | See Diagnostic |
| Run state | Running, awaiting a line, awaiting a save result, awaiting restore data, awaiting a transcript answer, finished, faulted | See State transitions |

## Frame

One routine call (§6.4).

| Field | Meaning |
| --- | --- |
| return address | Where execution resumes |
| result variable | Where the routine's value is stored |
| locals | Up to 15 words |
| evaluation stack | Words pushed during this call; popping below its base is a fault |
| argument count | Recorded for Quetzal |

## Status line

Version 3 only (§8.2). Produced before each line input and by `show_status`.

| Field | Meaning |
| --- | --- |
| location | Short name of the object in the first global variable |
| kind | Score game or time game, from header Flags 1 bit 1 |
| first, second | Score and turns, or hours and minutes |

## Event

What `run` returns to the host. Exactly one per successful call.

| Event | Meaning | Host answer |
| --- | --- | --- |
| Line wanted | The game is waiting for input; carries the maximum length (text buffer byte 0 minus 1) | Provide a line |
| Save requested | Carries the Quetzal bytes | Report stored or failed |
| Restore requested | The game wants a saved position | Provide bytes, or decline |
| Transcript requested | The game set Flags 2 bit 0 | Accept or decline |
| Finished | The game executed `quit` | None; no further runs |
| Fault | Carries a Fault | None; no further runs |
| Budget exhausted | The instruction budget ran out mid-turn | Call `run` again, or stop |

## Fault

An illegal operation that stops the game (FR-018). Each carries the address of the
instruction that caused it.

Kinds: unknown instruction; division by zero; stack underflow; call stack overflow; read
outside the file; write outside dynamic memory; missing property on `put_prop`; invalid
variable number; text that runs past the end of the file.

## Diagnostic

An illegal operation the core tolerates ([research.md](research.md) R12). Carries the
instruction address and the kind. The only kind in this feature is an instruction given
object 0, which does nothing or yields 0.

## Saved position

Quetzal 1.4 ([research.md](research.md) R7).

| Chunk | Contents | Rule on restore |
| --- | --- | --- |
| IFhd | Story identity and program counter; 13 bytes plus a pad byte. For version 3 the program counter points at the branch data of the `save` instruction (Quetzal §5.8.1) | Identity must equal the loaded story's, otherwise refused |
| CMem | Dynamic memory, XORed with the original and run-length encoded | Decoded data shorter than dynamic memory is accepted, the rest unchanged; longer is refused (Quetzal §3.4, §3.5) |
| UMem | Dynamic memory, uncompressed | Must be exactly the dynamic memory size (Quetzal §3.6) |
| Stks | A dummy frame holding the top-level evaluation stack, then the real frames (Quetzal §4.11) | Must parse completely |

A refused restore leaves the machine exactly as it was (FR-017). A successful restore
resumes inside the saved `save` instruction with its branch taken, re-applies the
interpreter's header fields and keeps the current Flags 2 (§6.1.2). The random state is
not restored.

## Command script

A text file, one input line per line, used in order. Lines are passed to the game without
the line terminator. A line is also consumed as a file name when the game starts a
transcript, saves or restores. A line must not begin with a backslash, which dfrotz
treats as a command of its own. Reaching the end while the game wants input ends the run
normally.

## Transcript

The file a host writes from the transcript stream. Format in
[contracts/transcript-format.md](contracts/transcript-format.md).

## Divergence ledger entry

| Field | Meaning |
| --- | --- |
| id | Stable identifier, for example `D-001` |
| description | What differs, in plain words |
| standard section | The section that justifies our behavior |
| rule kind | `transcript` or `czech` |
| transcript rule | Story digest, index of the command after which the difference appears, reference text, our text |
| czech rule | Name of the czech test whose expectation conflicts with Standard 1.1 |

An entry that matches nothing is an error (SC-008).

## State transitions

```text
Running ──run()──▶ Awaiting line ──provide line──▶ Running
Running ──run()──▶ Awaiting save result ──stored/failed──▶ Running
Running ──run()──▶ Awaiting restore data ──bytes/decline──▶ Running
Running ──run()──▶ Awaiting transcript answer ──accept/decline──▶ Running
Running ──run()──▶ Running            (budget exhausted)
Running ──run()──▶ Finished           (quit)
Running ──run()──▶ Faulted            (illegal operation)
```

Calling `run` while an answer is owed, or after Finished or Faulted, returns a host error
and changes nothing. Restart is an instruction the game executes: it reloads dynamic
memory from the story file while keeping Flags 2 (§6.1.3), clears the call stack,
re-applies the interpreter's header fields, reseeds the generator from the host seed, and
continues in Running.
