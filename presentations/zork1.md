# zork1.present: how it was made

This note is the record Principle VII asks for beside a presentation file: what was
consulted, and how the positions were made.

## What it holds

Stage 1 of feature 017: 37 of the story's 110 rooms, on two sheets. "Above Ground" has
22: the house, the forest round it, the canyon and the rainbow. "Under the House" has
15: the first rooms below the trap door, as far as the round room and the rooms next
to it. The two sheets' names are the project's own words. The rest of the story's
rooms are not in the file yet and are drawn by the map's own rules when found.

## How the positions were made

1. **The ways were gathered by playing.** The story was played along its walkthrough
   (`stories/zork1/fidelity/walkthrough.txt`), and at every turn each way the layer
   offered was tried on a copy of the game, three rooms deep. That gave the room
   numbers and 281 ways between them. `map_drawio` in `crates/zmachine-corpus/examples`
   does the same gathering.
2. **The positions were worked out by hand from those ways**, on a grid, so that as
   many joins as could be lie straight along a way walked: the four places outside the
   house at the four points of a ring with the rooms of the house inside it; the
   forest's places round that; the canyon to the east.
3. **A drawing was made and looked at** (not kept in the repository: it holds room
   names), and places were moved where a curved join ran over a place or another line.
4. The file was then written by `map_drawio import`, which moves each sheet as a whole
   so that its numbers are small. Only where places are from one another matters.

With every gathered way walked, 36 of the 41 joins between the file's places are
straight. The five that bend are in the forest, whose ways do not agree with one
another: a place cannot be both west of one neighbour and south of another that says
otherwise.

## What was consulted

- The story file itself, by playing it, as above. Every position comes from this.
- One page about published maps of the game was read, as text, when the idea was
  first discussed: Andrew Plotkin, "A treasury of Zork maps", 2023,
  <https://blog.zarfhome.com/2023/02/a-treasury-of-zork-maps.html>. It was read for
  which maps exist and how their makers divided the game into regions.
- **No published map was looked at.** No image of any map was opened, and no position,
  spacing or arrangement in this file was taken from one. The project owner supplied
  links to several published maps; none was opened in making this file.

## Changing it

The file may be changed by hand, or by laying the rooms out again in draw.io
(`presentations/README.md`). Whoever changes it adds here what they consulted. A
published map may be looked at and must not be copied.
