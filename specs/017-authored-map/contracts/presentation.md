# Contract: The Presentation File and the Authored Map

Parts: the file (F), the map (M), the snapshot (S), the app (A). Each clause is a
test.

## F. The file

- **F§1** The first four lines that are not skipped are `presentation: 1`, `sha256:`,
  `release:` and `serial:`, in that order, as a pack's identity lines are.
- **F§2** `sheet: N` or `sheet: N Name` declares a sheet, N from 1 to 99, once. A
  name is up to 40 characters and is the only text the file holds.
- **F§3** `place: ROOM SHEET X Y` puts a room on a declared sheet at a position; X
  and Y are whole numbers from -99 to 99. A room is given once.
- **F§4** No two places are in one position of one sheet.
- **F§5** Anything else, or a departure from F§1 to F§4, and the file is refused
  whole.
- **F§6** A file is used only for the story whose digest, release and serial it
  names, and only if every place is a room of that story.
- **F§7** `presentations/zork1.present` holds no word of the game: its lines are the
  keys, digits, and its sheets' names.

## M. The map

- **M§1** A place the file has is put, when the player is first in it, on the map's
  sheet for its file's sheet and at its position; it is not inset.
- **M§2** At no turn is a place drawn that the player has not been in.
- **M§3** A place the file does not have is put by the present rules and never at a
  position the file gives to a place of that sheet.
- **M§4** The same game played in another order of discovery has every authored
  place at the same position.
- **M§5** A sheet the file names is labelled "Sheet N of M · Name"; N and M count
  the sheets the player has reached, in the order reached.
- **M§6** No redraw is offered or made on a sheet with an authored place; a kept
  redraw of such a sheet is passed over when the game is brought back.
- **M§7** With no file, or a file refused, every position, sheet, line and label is
  what it is without one.
- **M§8** The transcript of a game is the same with and without the file.

## S. The snapshot

- **S§1** Each of `map.places` has `authored`, after `attached_to`.
- **S§2** A file refused is told once in `notice`: "The map's layout could not be
  used."

## A. The app

- **A§1** The build puts the story's presentation file in the bundle beside its
  pack, after checking its digest against the story.
- **A§2** The app hands the file's text to the layer when it opens a game and reads
  none of it.
