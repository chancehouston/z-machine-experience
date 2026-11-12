# Data Model: Z-Machine Versions 5 and 8

**Date**: 2026-11-11 (revised after independent review) | **Plan**: [plan.md](plan.md)

What is added to, or changes in, the data model of the first feature
(`specs/001-v3-core/data-model.md`). Anything not mentioned is unchanged. Section numbers
refer to the Z-Machine Standard 1.1 unless marked Quetzal. Where a rule is this
interpreter's choice and not the Standard's, it says so.

## Version

Fixed when a story file is loaded. Everything that differs between versions is a
property of it (research R6).

| Property | Version 3 | Version 5 | Version 8 |
| --- | --- | --- | --- |
| Packed address times (§1.2.3) | 2 | 4 | 8 |
| Header length field times (§11.1.6) | 2 | 4 | 8 |
| Largest story file (§1.1.4) | 128K | 256K | 512K |
| Object entry (§12.3.1, §12.3.2) | 9 bytes, links are bytes | 14 bytes, links are words | as 5 |
| Attributes (§12.3.1, §12.3.2) | 32 | 48 | as 5 |
| Property defaults (§12.2) | 31 | 63 | as 5 |
| Property size (§12.4.1, §12.4.2) | one byte: length and number | one or two bytes; up to 64 bytes of data | as 5 |
| Dictionary key (§13.3, §13.4) | 4 bytes, 6 Z-characters | 6 bytes, 9 Z-characters | as 5 |
| Routine header (§5.2.1) | count, then initial values | count only; locals start at 0 | as 5 |
| Status line (§8.2) | drawn by the interpreter | drawn by the game | as 5 |
| `save`, `restore` (§15) | branch | store a result | as 5 |
| Line input (§15 `read`) | `sread`: no result, zero terminator | `aread`: stores the ending key, length byte | as 5 |
| Instruction set (§14) | 68 | 98 | as 5 |

Abbreviation addresses are word addresses in every version (§3.3); they do not take the
packed-address multiplier.

## Instructions

Version 5 has 98 instructions (§14). 63 opcode numbers mean the same as in version 3; 35
instructions are new.

**New in version 5** (the version they first appear in is the Standard's):

- 2OP: `call_2s`, `call_2n`, `set_colour`, `throw`
- 1OP: `call_1s`, `call_1n`
- 0OP: `catch`, `piracy`
- VAR: `call_vs2`, `erase_window`, `erase_line`, `set_cursor`, `get_cursor`,
  `set_text_style`, `buffer_mode`, `read_char`, `scan_table`, `not`, `call_vn`,
  `call_vn2`, `tokenise`, `encode_text`, `copy_table`, `print_table`, `check_arg_count`
- EXT: `save`, `restore`, `log_shift`, `art_shift`, `set_font`, `save_undo`,
  `restore_undo`, `print_unicode`, `check_unicode`, `set_true_colour`

**Opcode numbers whose meaning changes.** Each needs a test in both versions.

| Opcode | Version 3 | Version 5 |
| --- | --- | --- |
| 1OP:15 | `not` | `call_1n`; `not` is VAR:24 |
| 0OP:5, 0OP:6 | `save`, `restore`, branching | not instructions; replaced by EXT:0 and EXT:1, which store |
| 0OP:9 | `pop` | `catch`, which stores |
| 0OP:12 | `show_status` | not an instruction, but §15 says to treat it as doing nothing |
| 0OP:14 | not an instruction | the prefix of the extended form (§4.3.4) |
| 0OP:15 | not an instruction | `piracy`, which always branches (§15) |
| VAR:0 | `call` | `call_vs`; the `vs2` and `vn2` forms carry up to seven arguments in two type bytes (§4.4.3.1) |
| VAR:4 | `sread` | `aread` |
| VAR:10 | `split_window`, clearing the window | does not clear (§15) |
| VAR:19 | `output_stream number` | adds a table operand and stream 3 |
| VAR:21 | `sound_effect` | adds operands; still answered by doing nothing |

In version 5 an extended opcode number above 28 is ignored, not faulted (§14.2.1). In
version 3 every opcode that was a fault stays a fault, and streams 3 and 4 stay ignored.

## Story file

| Field | Rule |
| --- | --- |
| version | Must be 3, 5 or 8; otherwise load is refused, naming those three |
| size | For versions 5 and 8, must not exceed the version's largest story file. Version 3 is not checked, as before |
| terminating characters table (0x2E), alphabet table (0x34), header extension table (0x36) | Read only for versions 5 and 8. Each is optional (0); if present it must lie inside the file |
| Unicode translation table (§3.8.5.2) | Word 3 of the header extension table, if that table is long enough |

## What the interpreter writes into the header

For versions 5 and 8, at load and again after every restart, restore and undo (§6.1.2.2,
§11.1). Version 3 is as before.

| Field | Value |
| --- | --- |
| Flags 1 (0x01) | bit 0 colours: clear. bits 2, 3, 4, bold, italic, fixed pitch: set. bit 7 timed input: clear |
| Flags 2 (0x10) | bit 4, the game wants undo: left as the game set it. Kept across restore and undo (§6.1.2). Bits 3, 5, 7 (pictures, mouse, sound): cleared |
| Interpreter number and version (0x1E, 0x1F) | From the host; defaults 1 and `Z` |
| Screen height and width in characters (0x20, 0x21) | From the host; default 24 and 80 |
| Screen width and height in units (0x22, 0x24), font width and height (0x26, 0x27) | The same sizes, with a font of 1 by 1 (§8.4.3) |
| Standard revision (0x32) | 1.1 |

The default colour bytes (0x2C, 0x2D) are left as the file has them, colour being
unavailable. This differs from the reference interpreter, which reports timed input
available and the three styles unavailable (research R13).

## Machine

| Part | Contents | Notes |
| --- | --- | --- |
| Styled output | Lower-window text since last drained, as runs of (text, style) | `take_output` still yields plain text |
| Current style | Reverse, bold, italic, fixed pitch | One style for all text, as in Frotz; `set_text_style` adds to it and 0 clears it (§15) |
| Current font | 1 | See `set_font` in [contracts/screen-model.md](contracts/screen-model.md) |
| Screen | See Screen | |
| Memory stream stack | Up to 16 tables being written (§7.1.2.1.1) | While any is open, text goes only to the innermost (§7.1.2.2) |
| Buffering | On or off, set by `buffer_mode` | Decides how text reaches the transcript ([contracts/transcript-format.md](contracts/transcript-format.md)) |
| Undo positions | Up to 25, newest last | See Undo position |
| Run state | adds: awaiting a key | |

**What restart, restore and undo do to interpreter state.** Restart returns the screen to
unsplit and blank, clears the style, selects the lower window, closes every memory
stream, turns buffering on, and forgets nothing else. Restore and undo replace game state
only: the screen, style, selected window, streams and buffering stay as they are, and
Flags 2 keeps its current bits.

## Frame

| Field | Meaning |
| --- | --- |
| result | `None` also means the call was one of the forms that discard the result (§6.4.1) |
| arg_count | Read by `check_arg_count` (§15) |
| locals | Up to 15; arguments beyond the routine's locals are discarded (§6.4.4.1) |

`catch` stores the number of routine calls in progress, not counting the main code, which
is the value Frotz uses and so the value found in saves (Quetzal §6.2). Observed with
dfrotz: 1 in a routine called from the main code, 2 one call deeper, and so on. `throw` returns
from the frame so numbered; a number with no such frame is a fault. A call in a
discarding form made between the `catch` and the `throw` stores nothing.

## Screen

The upper window is modelled as a canvas the size of the whole screen, because a version
5 game may draw below the bottom of its upper window and Inform's boxed quotations rely
on it.

| Field | Meaning | Rule |
| --- | --- | --- |
| width, rows | The screen size the host set | Fixed for the session |
| height | Rows currently split off as the upper window | Set by `split_window`, never more than `rows` |
| cells | `rows` × `width`, each a character and a style, or empty | Only cells the game has written are non-empty |
| cursor | Row and column, from 1 | Anywhere on the canvas |

Cells in rows below `height` are an **overlay**: what the game drew over the lower window.
They are readable until the game is next given input, and are then emptied. Details and
the instruction-by-instruction rules are in the screen contract.

## Style

A set of four flags: reverse video, bold, italic, fixed pitch (§8.7.1). Fixed pitch is
also in force while Flags 2 bit 1 is set (§8.1).

## Event

Adds one event:

| Event | Meaning | Host answer |
| --- | --- | --- |
| Key wanted | The game is waiting for a single keypress (§10.6, §15 `read_char`) | Provide a key |

`Line wanted` is unchanged. Text a game placed in the buffer beforehand is read with a
separate call.

## Key

A character the game's character set can represent, or one of: return, delete, escape,
the four arrows, function keys 1 to 12, keypad 0 to 9 (§3.8.4). Each maps to a ZSCII
code. A character with no code is refused, and the host asks again.

## Fault

Adds: throw to a frame that does not exist; a seventeenth nested memory stream; a memory
stream write outside dynamic memory. The existing kinds apply to versions 5 and 8.

Not faults: closing a memory stream when none is open, which does nothing; `set_cursor`
outside the screen, which is brought to the nearest cell; a colour number or font number
the interpreter does not know.

## Outcomes for the spec's edge cases

Where the spec names a situation and leaves the outcome to the Standard, this is it.

| Situation | Outcome |
| --- | --- |
| Closing a memory stream when none is open | Nothing happens |
| A memory stream's table | Word 0 receives the number of characters when the stream is closed; the characters follow as ZSCII bytes, a new-line as 13 (§7.1.2.1). A character with no ZSCII code is stored as `?` |
| Printing a character with no glyph | The core passes every character it is asked to print to the host as Unicode; a ZSCII code with no meaning on output prints `?`, as in version 3 |
| `print_unicode` | Prints that character (§3.8.5.4) |
| `check_unicode` | Bit 0, can be printed: set for any Unicode scalar value. Bit 1, can be read: set if the character has a ZSCII input code (§15) |
| A game's own alphabet table or Unicode table | Used in place of the defaults for decoding, encoding and input (§3.5.5, §3.8.5.2) |
| `tokenise` with a dictionary of the game's own | Used in place of the main one. A negative entry count means the entries are unsorted and are searched in order (§13.2). With its flag operand set, words not found leave their parse-buffer entries untouched (§15) |
| `copy_table` | Second table 0: the first is zeroed. Positive size: copied so that overlapping tables come out right. Negative size: copied forwards byte by byte regardless of overlap (§15). Size 0: nothing |
| `print_table`, `scan_table`, `encode_text` | As §15 specifies; `scan_table`'s form operand defaults to words |
| Restoring, or undoing to, a position taken while a memory stream was open | The streams are the interpreter's state, not the game's, and are left as they are now |
| Text already in the input buffer | Kept; typed text follows it ([contracts/core-api.md](contracts/core-api.md)) |
| Keys that end a line | The host may end a line with one the game listed; the game is told which |
| `set_cursor` before any split, or to row or column 0 | Allowed anywhere on the canvas; 0 is treated as 1 |
| `piracy` | Always branches, "the game is genuine" (§15) |
| `sound_effect`, `set_colour`, `set_true_colour`, `input_stream` | Accepted with any operands; nothing happens |
| `show_status` in version 5 | Nothing happens (§15) |
| An extended opcode above 28 | Ignored (§14.2.1) |
| `throw` to a frame that no longer exists | Fault |
| More arguments than the routine has locals | The extras are discarded (§6.4.4.1), but `check_arg_count` counts every argument that was passed. Observed with dfrotz: a routine with three locals called with five arguments reports arguments 1 to 5 as given |
| Object, property and attribute numbers at the top of version 5's range | Object 65535, property 63, attribute 47 are valid; attribute 48 and above is ignored as attribute 32 is in version 3 |

## Undo position

Dynamic memory (compressed against the original, as in a save), the call stack, and the
address of the `save_undo` instruction's store byte. `save_undo` stores 1. `restore_undo`
with a position available resumes at that address with 2 stored, keeping the current
Flags 2 (§6.1.2) and re-applying the interpreter's header fields; with none it stores 0.
The random state, the screen and the output streams are not part of a position.

## Saved position

The Quetzal format is unchanged. For versions 5 and 8 the saved program counter points at
the store-variable byte of the `save` (Quetzal §5.8.2); a successful restore stores 2
there and continues after it. A version 5 build and a version 8 build of the same game
have different checksums and so different identities.

## Command script

As before, with keypresses: a request for a key takes the next unused character of the
current script line, reading another when it is used up; an empty line is the return key;
unused characters are dropped when a whole line is next requested, and the reverse.
Special keys are written with the reference interpreter's escapes ([contracts/cli.md](contracts/cli.md)).
A comparison script starts a transcript at most once.

## Reference artefacts

- `stories/advent/fidelity/`: one walkthrough and seed, which includes the game's
  `version` command so the interpreter's letter is exercised; a reference transcript for
  each of `advent.z5` and `advent.z8`; for at least three save points in each version,
  the scripts, saves and continuation transcripts in the form used for Zork I; and an
  undo script of thirty moves and thirty undos with its reference transcript.
- `crates/zmachine-core/tests/fixtures/`: transcripts dfrotz wrote for version 5 probe
  programs, as for version 3.
