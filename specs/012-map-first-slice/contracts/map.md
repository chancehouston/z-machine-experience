# Contract: The Map

Four parts: what is witnessed (W), where places go (P), the snapshot (S), the screen
(A). Each clause is a test.

## W. What is witnessed

| | What | Is on the map when |
| --- | --- | --- |
| **W§1** | A place | The player was in it at a stop where the game waited for a command |
| **W§2** | A place's name | The player was in it at such a stop and the layer did not report it dark |
| **W§3** | A place's ways (stubs) | As W§2; they are the compass's places and extras at that stop whose state is open, uncertain or door |
| **W§4** | A way's state, and its mark for leading into the dark | As it was at the last such stop in that place |
| **W§5** | A line, in a direction | The player sent a direction word in a place, and at the next command stop was in another place, and the game had not jumped, and W§9 does not apply |
| **W§6** | A line, by a word | As W§5, with a command that is not a direction word. It is marked with the command as sent |
| **W§7** | The name of the place a way leads to | The line has been walked, by W§5. Until then no name is shown or spoken for it |
| **W§8** | A refused way | The player sent a direction word the compass did not offer as open or door, and at the next command stop was in the same place, and the game had not jumped |
| **W§9** | A carried move | The player sent a direction word; the compass gave that way as open with a named destination; and they arrived somewhere else. Drawn as W§6, marked with the command |
| **W§10** | A way that varies | Two lines by the same direction or word from one place lead to different places. Both are kept; the way is marked |

- **W§11** Nothing else is on the map. No place the player has not been in; no way of
  such a place; no line not walked.
- **W§12** A place seen only in the dark has no name and no stubs. Light reported as
  unknown counts as not dark.
- **W§13** Where the layer cannot say where the player is, nothing is added, the
  marker is "not known", and the place the next line starts from is not known either:
  the next arrival is recorded with no line.
- **W§14** When the game jumped (core counter, research R3) nothing is added but the
  place arrived at, if new, on a new sheet; no line is drawn.
- **W§15** Places with the same name are numbered from 1 in the order first reached.
  A name is numbered once a second place has it; the first then becomes 1.
- **W§16** A way that leads back to the place it leaves draws no line and is marked
  as returning.
- **W§17** A stop where the game waits for an answer or a key adds nothing. A move is
  judged from one command stop to the next, with the command sent at the first.

## P. Where places go

- **P§1** The first place is at (0, 0) of sheet 1.
- **P§2** By a compass direction: on the sheet of the place left, at one, two or three
  steps along it, the first free; else the nearest free position to one step, by
  rings, preferring positions on the side the direction points to; ties by lowest y,
  then lowest x.
- **P§3** By up or down: on the same sheet, nearest free to (+1, −1) for up and
  (−1, +1) for down from the place left, by rings, passing over positions on a ray of
  a compass way that place has. The place is attached to the place left.
- **P§4** A place reached by compass, in or out from an attached place joins its
  group. When a group has four places it moves to a new sheet, keeping the positions
  of its places relative to each other, with the place first attached at (0, 0). It
  is then not a group. No other placed place ever moves.
- **P§5** By in or out: the nearest free position to the place left, on its sheet.
- **P§6** By any other word, or carried (W§9), or after a jump (W§14): a new sheet at
  (0, 0); unless the place left was itself put by this clause on its sheet, when the
  new place goes at the nearest free position to one step below it.
- **P§7** No two places of a sheet share a position.
- **P§8** A compass line is straight when the far place is exactly along the
  direction from the near one, the same holds for the direction walked back if it
  was, and no place lies between. Otherwise it is bent.
- **P§9** For one story, one chance and one sequence of inputs, every place's sheet
  and position, and every line's being straight or bent, are the same every time.
- **P§10** Nothing of P§1 to P§8 uses a place the player has not been in.

## S. The snapshot

A key `map` after `start_over`. `null` when the layer does not recognise the game.

```json
"map": {
  "note": null,
  "spoken": "Map. You are in Kitchen.",
  "sheet": 1,
  "sheets": 2,
  "here": 74,
  "lamp": "lit",
  "places": [
    {"id": 74, "x": 1, "y": 0, "name": "Kitchen", "attached_to": null},
    {"id": 79, "x": 2, "y": -1, "name": null, "attached_to": 74}
  ],
  "lines": [
    {"a": 74, "b": 193, "out": "west", "back": "east", "kind": "straight", "label": null},
    {"a": 74, "b": 79, "out": "up", "back": null, "kind": "ladder", "label": null}
  ],
  "leads": [
    {"from": 193, "way": "down", "label": "down", "to": "Cellar", "sheet": 2}
  ],
  "stubs": [
    {"of": 193, "way": "east", "state": "door", "dark_ahead": false}
  ],
  "ways": [
    {"way": "west", "label": "West", "state": "open", "command": "west",
     "leads_to": "Living Room", "dark_ahead": false, "refused": false, "varies": false,
     "spoken": "West, to Living Room"}
  ]
}
```

- **S§1** `sheet` is the number of the sheet the player is on, from 1; `sheets` how
  many there are. Only that sheet's places, lines, leads and stubs are given.
- **S§2** `here` is the place the player is in, or `null` (W§13). `lamp` is `lit`,
  `out` (the layer reports the place dark) or `unknown`.
- **S§3** A place's `name` is `null` until W§2; it carries its number (W§15) when it
  has one. `x` and `y` are whole numbers, y growing downward. `id` is stable for the
  game.
- **S§4** A line's `kind` is `straight`, `bent` (compass lines, P§8), `ladder` (up or
  down between two places of this sheet) or `word` (W§6, W§9, in, out, between two
  places of this sheet). `out` and `back` are the ways walked, as compass names,
  `up`, `down`, `in`, `out`, or `word`; `label` is the command for a `word`.
- **S§5** `leads` are the walked lines whose far end is on another sheet: the way, the
  words to show for it (a direction's name, or the command), the far place's name if
  it has one, and its sheet.
- **S§6** `stubs` are the ways seen and not walked of every place on the sheet but the
  one the player is in.
- **S§7** `ways` are those of the place the player is in, in the compass's order and
  then the extras: every way that is a stub, a walked line or refused. `command` is
  present when it can be sent now. `leads_to` and the destination in `spoken` are
  given only by W§7. With the game not waiting for a command there are no commands.
- **S§8** `note` is the layer's words where there is something to say in place of the
  map: that the ways here are not known, or that the player's place is not known.
- **S§9** `spoken` is the layer's summary for a screen reader.
- **S§10** The snapshot's other keys are unchanged. `compass` is still given.

## A. The screen

- **A§1** With `map` not null the app shows the band between the status line and the
  transcript: the sheet drawn at the positions given, moved so that `here` is in the
  middle, clipped to the band. With `map` null there is no band.
- **A§2** Each of `ways` with a command is a button over the drawing at its place's
  edge in its direction, with the identifier `compass.<way>` for the twelve and
  `compass.extra.<n>` for the rest, the accessibility label `spoken`, and the value
  its `state`. Touching it sends `command` through the one place a line is sent from.
- **A§3** The app shows no name, state or word the snapshot did not give.
- **A§4** While the game waits for a line the command field has the focus. It gives
  it up only when the map is opened out or the transcript is dragged.
- **A§5** The band's height does not change while the keyboard is up.
- **A§6** The strip of what is carried and the note of darkness are shown at the
  band's lower edge whenever the band is.
- **A§7** The newest reply is shown from its first line.
- **A§8** Pulling the band down, or touching its `button.openMap`, shows the sheet in
  the space of the band and the transcript, with the keyboard down. It can be
  dragged. `button.closeMap`, or touching the command field, returns to the band.
- **A§9** The band and the sheet are one element for a screen reader with the label
  `spoken`, containing the buttons of A§2. Places and lines are not separate
  elements in this slice.
- **A§10** A place's outline is drawn from its `id`, the same each time. New places
  and lines are drawn in; with Reduce Motion they appear at once.
- **A§11** There is no compass view. `button.compass` is gone.
