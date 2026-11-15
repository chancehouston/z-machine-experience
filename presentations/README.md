# Presentation files

Each file here says how one story's map is drawn: which sheet a room is on, where on
that sheet it sits, and what a sheet is called. It is authored by hand, under the
constitution's Principle VII, and it is not a game pack: a pack holds facts that are
proven against the story, and a presentation file holds an author's choices, which are
not claimed to be facts.

A file is named by its story's id in `stories/corpus/manifest.txt` and applies only to
the story file whose SHA-256 digest, release and serial it states.

## The form

```text
presentation: 1
sha256: <64 lowercase hex digits>
release: <number>
serial: <six letters or digits>

sheet: 1 A Name
sheet: 2
place: ROOM SHEET X Y
```

The first four lines are those, in that order. After them, in any order:

- `sheet: N` or `sheet: N Name` declares a sheet. N is 1 to 99 and is given once. A
  name is at most 40 characters. The numbers are the file's own: the map numbers its
  sheets in the order the player reaches them.
- `place: ROOM SHEET X Y` puts a room, by its object number, on a sheet declared
  above it, at a position. X grows to the east and Y to the south; each is a whole
  number from -99 to 99. A room is given once, and no two rooms share a position of a
  sheet.

Empty lines and lines that start with `#` are skipped. Anything else and the whole
file is refused: the map is then drawn as it is without one, and the player is told
once that the layout could not be used.

## What a file may and may not hold

- **Drawing only.** A file moves what the player has witnessed. A room is drawn when
  the player has been in it and not before, with or without a file; a file shows no
  room, way or name that would not be shown without it.
- **Names of sheets only.** A sheet's name is the only text a file holds that is shown
  or spoken, and only once the player has been on that sheet. A file holds no word of a
  game: not a room's name, not in a comment. A test holds each file to its keys, digits
  and its sheets' names.
- **No behaviour.** No conditions, no commands. A game's transcript is the same with
  and without its file, and a test proves it.
- **Our own work.** The positions are the project's. A published map may be looked at
  while authoring and is never copied. Beside each file, a note of the same name ending
  `.md` records what was consulted and how the positions were made.
- **Checked.** Every room a file names is tested to be a room of the story it names,
  and no two are in one position.
- **Marked.** The snapshot says of each place whether its position came from a file.
- **Absent means unchanged.** A story with no file here is drawn as it always was.

A room the file does not have is still drawn when the player finds it, by the map's
own rules, and never on a position the file gives to another room of that sheet.

## Laying a map out in draw.io

A presentation file can be written by hand, but it is easier to drag boxes about. A
tool turns a story's file into a [draw.io](https://www.drawio.com) diagram and back:

```sh
cargo run --release -p zmachine-corpus --example map_drawio -- export zork1
# open presentations/drawio/zork1.drawio in draw.io, move things, save
cargo run --release -p zmachine-corpus --example map_drawio -- import zork1
```

`export` plays the story to find its rooms and the ways between them, and writes
`presentations/drawio/<story>.drawio`: a page for each sheet, a box for each room where
the file has it, a line for each way found, and a last page, "Not placed", with every
room the file does not have. A line's label says which ways it stands for, as
`n,ne / w`: the ways out of one end, then the ways back from the other. Dashed lines
are ways up and down. A story with no presentation file yet exports as one page of
rooms not placed.

`import` reads the diagram and writes `presentations/<story>.present`. It then says,
for each sheet, how many joins come out straight and which bend, so a layout can be
judged without building the app. If it finds a fault it names the box and writes
nothing.

**The diagram holds the game's names for its rooms and every room of the game**, so
that a person can tell the boxes apart. It is a spoiler, and it is not ours to
publish: `presentations/drawio/` is ignored and a diagram is never committed. Nothing
of a name is read back.

What `import` reads, and so what a change to the diagram can mean:

| In draw.io | In the presentation file |
|---|---|
| Drag a box | The room's position. Positions snap to squares 140 across and 80 down, by the middle of the box. |
| Cut a box and paste it on another page | The room is on that sheet. |
| Move a box from "Not placed" to a page | The room is added to the file. |
| Move a box to "Not placed", or delete it | The room is left to the map's own rules. |
| Rename a page | The sheet's name (40 characters at most). |
| Add a page, or drag the pages into another order | A new sheet; the file's sheets are numbered in page order. The player's sheets are still numbered as they reach them. |

The rules a diagram must keep:

1. **Leave a box's room number alone.** It is the box's `room` property (Edit Data).
   The label is only for reading and may say anything.
2. **One box for a room.** Copying a box makes a second, which is refused.
3. **One box in a square.** Two boxes of a page whose middles fall in the same square
   are refused; move one at least 140 across or 80 down.
4. **Do not group boxes**, and do not put one inside another: a box's position is then
   no longer its own.
5. **Save uncompressed** (File, Properties, untick Compressed), which is draw.io's
   way unless it has been changed.
6. Keep the page called "Not placed". It is not a sheet.

Everything else is ignored and may be changed freely: lines, labels, colours, sizes,
notes, extra shapes with no `room` property. The lines are drawn again from the game at
the next `export`.

After an import, run the tests (`cargo test -p zmachine-experience --test
presentation`) and build the app. The author's note beside the file
(`presentations/<story>.md`) is where to record what was consulted.
