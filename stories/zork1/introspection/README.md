# Zork I: hand-made exit expectations

`exits.txt` lists, for 59 chosen moments of the walkthrough in
`../fidelity/walkthrough.txt`, the exits the player's room should be reported to have.
It is an independent answer key for judging exit introspection.

The list was made by reading the game's ZIL source and the reference transcript
(`../fidelity/reference.scr`, written by Frotz). It was made without reference to the
interpreter's introspection output: the introspection code was not read and the
interpreter was not run.

## Source read

- Repository: <https://github.com/historicalsource/zork1>
- Revision: commit `97b7b3d68c075dd9af7da499c3e9690ada3471fd` (master, 2025-11-20), read on
  2026-11-11.
- Files: `1dungeon.zil` (room definitions and initial flag values), `1actions.zil`,
  `gverbs.zil` and `gmain.zil` (where the flags are set).

The source is not marked with a release number. The story file is release 119. The
source's room names, the routes the walkthrough takes and the flag-dependent room text
all agree with the release 119 transcript, but the exit tables of the story file itself
were not inspected, so a difference between the two cannot be ruled out.

## File format

```
# Lines starting with # are comments.
at line 3 | West of House
  north: fixed North of House
  west: fixed Forest
  southwest: conditional
```

- `at line N` is the moment the game is waiting for the command on line N of
  `walkthrough.txt` (lines numbered from 1; line 3 is the first real command, so
  `at line 3` is the opening position). The name after `|` is the room the player is in
  at that moment, as shown by the transcript.
- Each indented line is one direction that is an exit, in this fixed order when present:
  north, east, west, south, northeast, northwest, southeast, southwest, up, down, in,
  out, land.
- A room name is the room's `(DESC "...")` string exactly. Several rooms share a name
  (Forest, Clearing, Maze, Coal Mine, Cave, Mirror Room).
- Entries are separated by a blank line.

How each ZIL exit form is expected to appear:

| ZIL form | Expected |
| --- | --- |
| `(DIR TO ROOM)` | `fixed <room name>` |
| `(DIR "message")` | not listed |
| `(DIR PER ROUTINE)` | `conditional` |
| `(DIR TO ROOM IF FLAG [ELSE "message"])` | `fixed <room name>` while the flag is true; not listed while it is false |
| `(DIR TO ROOM IF OBJECT IS OPEN)` | `conditional`, whatever the object's state |

Every direction a room's definition mentions is either listed or left out for one of
those reasons; the comment above an entry says which.

## What the entries exercise

There are 59 entries covering 49 different rooms. Every entry has at least one
unconditional exit. The other forms:

| Form | Entries (line numbers) |
| --- | --- |
| Refusal message only (not listed) | 3, 4, 6, 20, 25, 46, 54, 66, 67, 105, 142, 200, 330, 331 |
| Routine (`PER`) | 13 and 309 (Living Room down), 28 (Studio up), 290 (Maze down) |
| Door object | 10 and 11 (Behind House west, in; window closed, then open), 12 (Kitchen east, out), 20 (Cellar up) |
| Exit back to the same room | 207 (Coal Mine east), 289 (Maze north) |

Flag exits:

| Flag | False (exit not listed) | True (exit listed) | How the value is known |
| --- | --- | --- | --- |
| `WON-FLAG` | 3 West of House | 331 West of House | Set when the score reaches 350; the transcript prints the whispered voice at line 325 |
| `FALSE-FLAG` | 12 Kitchen (down) | never | Never set anywhere |
| `MAGIC-FLAG` | 13 Living Room, 298 Cyclops Room | 299 Cyclops Room, 309 Living Room | Set by `odysseus` (line 298) |
| `CYCLOPS-FLAG` | 298 Cyclops Room | 299 Cyclops Room | Set by `odysseus` |
| `TROLL-FLAG` | 21 The Troll Room | 22 The Troll Room | Set when the troll dies (line 21) |
| `DOME-FLAG` | 44 Dome Room | 45 Dome Room | Set by tying the rope (line 44) |
| `COFFIN-CURE` | 53 Altar | 173 Altar | See below |
| `RAINBOW-FLAG` | 70 End of Rainbow | 71 End of Rainbow, 142 Aragain Falls | Set by waving the sceptre (line 70) |
| `LOW-TIDE` | 102 Reservoir South | 104 Reservoir South, 107 Reservoir North | Set when the reservoir empties; the transcript reports it after line 103 |
| `LLD-FLAG` | 175 Entrance to Hades | 181 Entrance to Hades | Set by reading the book (line 179) |
| `EMPTY-HANDED` | 234 Timber Room | 236 Drafty Room | See below |

Two flags are not changed by an event but recomputed by a room's action routine at the
start of every command typed in that room, so between commands they hold a value from
the past:

- `COFFIN-CURE` (Altar, down) is set to "the player is not carrying the coffin" at the
  start of each command at the Altar. At line 53 no command has yet been processed there,
  so it has its initial value, false. At line 173 two commands have been processed there
  without the coffin, so it is true.
- `EMPTY-HANDED` (Timber Room west; Drafty Room east and out) is recomputed at the start
  of each command in those two rooms. At line 234 the player has just arrived in the
  Timber Room for the first time, so it has its initial value, false. At line 236 the
  player has just passed through the narrow passage, which required it to be true, and
  nothing has changed it since. Other moments in these rooms were left out because the
  value depends on object weights.

## Rooms left out

- The Frigid River rooms and the Shore while the player is in the boat (lines 115 to
  127): the player is inside a vehicle.
- "Inside the Barrow" (after line 332) is not a room; it is text printed by the Stone
  Barrow's action routine as the game ends.
- Timber Room and Drafty Room at moments other than lines 234 and 236, as explained above.
