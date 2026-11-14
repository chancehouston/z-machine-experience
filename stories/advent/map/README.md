# The map of this story's walkthrough

`walkthrough.txt` is the map the experience layer has drawn when the walkthrough in
`../fidelity/walkthrough.txt` has been played to its end: one line for each place the
player stood in, each line walked between two places, and each way tried and refused,
sorted (specs/012-map-first-slice, research R12).

It is a recorded result. A test compares the map with it, so that a change to the
map's rules shows as a change to this file, to be read before it is accepted:

```bash
UPDATE_PAGES=1 cargo test -p zmachine-experience --test map
```

```bash
git diff stories
```

The file holds the names of this story's rooms, which are the story's text, and is
kept here under the story's own license, beside the other files that hold them. No
such file is kept for a story the repository may not hold.

Reading a line:

- `place 74 sheet 1 at 1, 0 seen Kitchen`: room 74 is on sheet 1 at position (1, 0),
  x to the right and y downward, and has been seen lit. `inset` in place of `at` means
  it is attached to another place and drawn in the gap at that position's lower-right
  corner. `unseen` means it has only been stood in dark, and has no name.
- `line 74 west -> 193, back east straight`: walked west from 74 to 193, and back
  east. `straight` or `bent` for a compass line; `ladder` for up or down; `word` for
  in, out or a command; `across sheets` when the two places are on different sheets.
