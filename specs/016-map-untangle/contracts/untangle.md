# Contract: Untangling the Map

Parts: the redraw (R), keeping (K), the snapshot (S), the screen (A). Each clause is
a test.

## R. The redraw

- **R§1** The player's place is where it was.
- **R§2** Each place joined to the player's by a compass way walked out of it is
  exactly along that way, at the first free of one, two or three steps; where two
  ways lead to one place, the first in the compass's order decides.
- **R§3** A place joined only by a way walked into the player's place is put
  opposite to that way, by the same rule.
- **R§4** The other places are moved to leave fewer joins bent, then fewer crossing;
  the player's place and the places it joins are not moved by this.
- **R§5** A place attached to another keeps its corner of it. Where the player's
  place is attached, it and its parent are both where they were.
- **R§6** No two places share a position, and no two an inset; every place is on
  the sheet it was on.
- **R§7** Only places the player has been in and ways they have walked are used.
- **R§8** A second redraw with nothing walked between changes nothing.
- **R§9** The tidying stops at its budget.
- **R§10** Nothing is sent to the game: the turns, the moves and the transcript are
  as they were.

## K. Keeping

- **K§1** A kept game with redraws is of form 2 and has a line `redraw` after the
  inputs answered before each press.
- **K§2** Brought back, it has the positions of the game never closed.
- **K§3** A kept game of form 1 is brought back as before. A kept game with no
  redraw is written as form 1 was, byte for byte but for the form's number.
- **K§4** A `redraw` line where none can be made (no place known, a sheet of one)
  makes the kept game one that cannot be brought back.

## S. The snapshot

- **S§1** `map.redraw` follows `map.back`: the layer's words, or `null`.
- **S§2** It is present when the game waits for a command, the player's place is
  known and their sheet has at least two places.

## A. The screen

- **A§16** With `map.redraw` present the map has the control `button.map.redraw`,
  in the band and opened out, at least 44 points square, with those words spoken.
  It is absent on a sheet the player is not on.
- **A§17** A press calls `zmx_session_redraw_map` and shows the snapshot that comes
  back. No turn is added and nothing is spoken.
- **A§18** A place whose position changes glides to it, and its lines with it; with
  Reduce Motion they are there at once.
- **A§19** The app works out no position, and only `Engine.swift` calls the binding.
