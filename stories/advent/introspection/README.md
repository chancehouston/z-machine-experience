# Advent exit expectations

`exits.txt` is a set of hand-made expectations for the exits of rooms in
Advent (the Inform 6 port of Colossal Cave) at chosen moments of the fidelity
walkthrough. It is meant for judging software that reports a room's exits.

## Provenance

Every entry was worked out by reading `stories/advent/advent.inf` (the game's
source) and `stories/advent/fidelity/reference_z5.scr` (the transcript Frotz
wrote for `stories/advent/fidelity/walkthrough.txt`). It was made without
reference to the interpreter's introspection output: the introspection code
was not read and no interpreter was run.

## Format

```
# Lines starting with # are comments.
at line 3 | At End Of Road
  north: fixed In Forest
  east: fixed Inside Building
  in: fixed Inside Building
```

- `at line N` is the moment the game is waiting for the command on line N of
  `walkthrough.txt`. Lines are numbered from 1; line 1 is `script`, line 2 is
  the transcript file name, line 3 is the first real command, so `at line 3`
  is the opening position.
- After `|` is the printed name of the room the player is in at that moment:
  the room's `short_name` if that is a string, otherwise the quoted name in
  its object header. Maze rooms therefore appear as `Maze` and `Dead End`.
- Then one indented line per direction that is an exit, in this fixed order
  when present: north, south, east, west, northeast, northwest, southeast,
  southwest, up, down, in, out.
- Entries are separated by a blank line.

Each direction property the room provides (`n_to` ... `out_to`, including
those inherited from a class) is classified by its value:

| Value of the property            | Expectation                      |
|----------------------------------|----------------------------------|
| a room object                    | `fixed <printed name of room>`   |
| a door object                    | `conditional`                    |
| a routine (inline or named)      | `conditional`                    |
| any other object                 | `conditional`                    |
| a string                         | a refusal: direction not listed  |

A routine that only prints a message and returns true is still a routine and
is `conditional`. Directions the room does not provide are not listed.
`cant_go` and `before` routines are not direction properties and are ignored.

## Classes

Two classes in `advent.inf` supply a direction property, each a string, so
each is an inherited refusal: `FissureRoom` (`d_to`) and `MazeRoom`
(`out_to`). `Room`, `Aboveground`, `DeadendRoom` and `DiffmazeRoom` supply
none. `MazeRoom`, `DeadendRoom` and `DiffmazeRoom`
supply the `short_name` strings "Maze" and "Dead End".

## What the entries exercise

There are 50 entries, one per moment, for 50 different rooms.

- Opening room: line 3. Late game (repository): lines 167, 168.
- Plain room exits only: lines 3, 9, 14, 17, 18, 20, 23, 26, 29, 35, 37, 38,
  39, 56, 61, 62, 64, 71, 75, 77, 83, 84, 91, 98, 99, 121, 125, 129, 139, 167.
- Door object as the value: line 13 (`u_to Grate`), line 168
  (`d_to RepositoryGrate`).
- Routine as the value: line 11 (down, returns the grate), 21 (down),
  22 (up), 57 (north, east, out), 78 (north, returns the rusty door),
  86 and 89 (named routine `CrossRicketyBridge`), 114 (east), 116 (west),
  136 (south).
- String as the value, so omitted: line 5 (`in_to`), 10 (`d_to`, `in_to`),
  21 (`w_to`), 40 (`e_to`), 63 (`u_to`).
- String inherited from a class, so omitted: line 40 (`d_to` from
  `FissureRoom`), lines 43, 44, 45 (`out_to` from `MazeRoom`).
- `in_to` present: lines 3, 98 (rooms), 5, 10 (strings).
- `out_to` present: lines 5, 46, 99, 139 (rooms), 57 (routine),
  43, 44, 45 (inherited string).
- Printed name differs from the header (no quoted name; `short_name` from a
  class): lines 43, 44, 45 (`Maze`), 46 (`Dead End`). Maze rooms as
  destinations: lines 37, 39, 43, 44, 45, 46.
- A room whose exit leads to itself: line 43 (north).
- Two different rooms with the same printed name as destinations:
  line 45 (`Dead End` twice), lines 61/62 (`Secret N/S Canyon`).

## Notes

- Darkness: the walkthrough is never in the dark. At line 14 the lamp is not
  yet on, but In Cobble Crawl has the `light` attribute; at line 121 the
  player is in The Dark Room (no `light`) with the lamp on.
- The only direction properties changed at run time are the two fissure
  sides (`e_to`/`w_to`, when the rod is waved). The walkthrough never waves
  the rod, so at line 40 `e_to` is still its original string.
- Left out on purpose: Low N/S Passage (its `n_to` leads to a room whose
  header name is `"At ~Y2~"`, containing an escaped quote); At Witt's End
  (its only direction property is a string, giving an entry with no lines);
  line 173 (the game is over and waiting at the restart prompt).
