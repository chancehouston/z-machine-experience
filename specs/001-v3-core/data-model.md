# Data Model: Faithful Z-Machine Version 3 Core

**Date**: 2026-11-11 | **Plan**: [plan.md](plan.md)

Section numbers refer to the Z-Machine Standard 1.1.

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
| length | Header word at 0x1A, times 2 | If non-zero must not exceed the file size |

**Identity** is (release, serial, checksum). A save belongs to exactly one identity.

## Machine

Everything that changes while a game runs.

| Part | Contents | Notes |
| --- | --- | --- |
| Dynamic memory | Bytes from 0 to the static memory base | The only writable memory (§1.1). Writes elsewhere are faults. |
| Call stack | Frames, innermost last | See Frame |
| Program counter | Address of the next instruction | |
| Random state | `A`, `interval`, `counter` | See [research.md](research.md) R2 |
| Output buffer | Text printed since the host last drained it | Unwrapped |
| Status line | Latest status line content, with a changed flag | See Status line |
| Run state | Running, awaiting a line, awaiting a save result, awaiting restore data, finished, faulted | See State transitions |

## Frame

One routine call (§6.4).

| Field | Meaning |
| --- | --- |
| return address | Where execution resumes |
| result variable | Where the routine's value is stored |
| locals | Up to 15 words |
| evaluation stack | Words pushed during this call; cannot be popped past its base (underflow is a fault) |
| argument count | Recorded for Quetzal |

## Status line

Version 3 only (§8.2). Produced before each line input and by `show_status`.

| Field | Meaning |
| --- | --- |
| location | Short name of the object in the first global variable |
| kind | Score game or time game, from header flags 1 bit 1 |
| first, second | Score and turns, or hours and minutes |

## Event

What `run` returns to the host. Exactly one per call.

| Event | Meaning | Host answer |
| --- | --- | --- |
| Line wanted | The game is waiting for a line of input; carries the maximum length | Provide a line |
| Save requested | Carries the Quetzal bytes | Report stored or failed |
| Restore requested | The game wants a saved position | Provide bytes, or decline |
| Finished | The game executed `quit` | None; the machine accepts no further runs |
| Fault | Carries a Fault | None; the machine accepts no further runs |
| Budget exhausted | The instruction budget ran out mid-turn | Call `run` again, or stop |

## Fault

An illegal operation by the story file (FR-018). Each carries the address of the
instruction that caused it.

Kinds: unknown instruction; division by zero; stack underflow; call stack overflow; read
outside the file; write outside dynamic memory; invalid object number; missing property
on `put_prop`; invalid variable number; text that runs past the end of the file.

## Saved position

Quetzal 1.4 ([research.md](research.md) R7).

| Chunk | Contents | Rule on restore |
| --- | --- | --- |
| IFhd | Story identity, program counter | Identity must equal the loaded story's, otherwise refused |
| CMem or UMem | Dynamic memory, compressed or not | Length must equal the dynamic memory size |
| Stks | Frames | Must parse completely |

A refused restore leaves the machine exactly as it was (FR-017).

## Command script

A text file, one input line per line, used in order. Lines are passed to the game without
the line terminator. Reaching the end while the game wants input ends the run normally.

## Transcript

The bytes a run wrote, at a stated width. Format in
[contracts/transcript-format.md](contracts/transcript-format.md).

## Divergence ledger entry

| Field | Meaning |
| --- | --- |
| id | Stable identifier, for example `D-001` |
| description | What differs, in plain words |
| standard section | The section that justifies our behavior |
| story digest | SHA-256 of the story file the rule applies to |
| after command | Index of the command in the script after which the difference appears |
| reference text, our text | The exact bytes to substitute |

An entry that matches nothing in a comparison is an error (SC-008).

## State transitions

```text
Running ──run()──▶ Awaiting line ──provide line──▶ Running
Running ──run()──▶ Awaiting save result ──stored/failed──▶ Running
Running ──run()──▶ Awaiting restore data ──bytes/decline──▶ Running
Running ──run()──▶ Running            (budget exhausted)
Running ──run()──▶ Finished           (quit)
Running ──run()──▶ Faulted            (illegal operation)
```

Calling `run` while an answer is owed, or after Finished or Faulted, is a host error and
is rejected without changing the machine. Restart is an instruction the game executes; it
reloads dynamic memory from the story file, clears the call stack, reseeds the generator
with the host seed, and continues in Running.
