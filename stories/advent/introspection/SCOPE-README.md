# Hand-made scope expectations (Zork I and Advent)

This note describes two files:

- `stories/zork1/introspection/scope.txt`
- `stories/advent/introspection/scope.txt`

Each one lists, for chosen moments of that game's walkthrough, objects that
must be in the player's scope and objects that must not be. "Scope" here means
what the player could plausibly act on: things in the room, things inside open
or see-through containers or on surfaces there, and things being carried.

## How they were made

By hand, from the reference transcripts only:

- Zork I: `fidelity/walkthrough.txt` and `fidelity/reference.scr`.
- Advent: `fidelity/walkthrough.txt`, `fidelity/reference_z5.scr`, and
  `advent.inf` for object names, attributes (open, transparent, scenery,
  static, found_in) and a few state changes the transcript does not spell out.

They were written without looking at, or running, the interpreter's
introspection code or its output, so they can be used as an independent check
on it. Carried objects were tracked by following every take, drop, put, give
and loss in the transcript up to the line in question; room contents come from
the last description the transcript shows for the room.

## File format

```
# comment
at line 12 | Kitchen
  has: kitchen table, brown sack, glass bottle
  not: small mailbox
```

- `at line N` is the moment the game is waiting for the command on line N of
  that game's `walkthrough.txt` (lines numbered from 1; line 1 is `script`,
  line 2 the transcript file name, line 3 the first real command, so
  `at line 3` is the opening position).
- After `|` comes the room the player is in at that moment, by the heading the
  game prints.
- `has:` is a comma-separated list of object names that must be in scope.
- `not:` is a comma-separated list (at least one) of objects that are in some
  other room at that moment and so must not be in scope.
- Entries are separated by a blank line. Lines starting with `#` are comments.

`has:` is always a subset: anything uncertain was left out, so an object being
in scope but absent from `has:` is not an error. The player, the room itself
and the contents of closed opaque containers are never listed.

## Object names

Names are the object's stored short name without an article.

- Zork I: the name as the game prints it in listings in `reference.scr`
  ("There is a X here.", "The X contains:", "X: Dropped." and the trophy case
  listing), plus "kitchen table" and "small mailbox". Objects only ever
  described in prose (rope, the knives, canary, bauble, map, troll, thief,
  trophy case, the red-hot bell, the machine) are not used.
- Advent: the quoted name in the object's header in `advent.inf`, for example
  "set of keys", "small bottle", "bottled water", "black rod with a rusty star
  on the end", "emerald the size of a plover's egg". No object on the route has
  a `short_name` routine (only rooms do). The magazines are not used because
  their name contains quotation marks.

## What the entries cover

Zork I (38 entries, 31 rooms): the opening; the kitchen table with the sack and
bottle, before and after the sack is opened; the trophy case, open, with one,
four, eight and sixteen treasures; carrying the open coffin with the sceptre in
it; the beached boat and the mine basket as open containers holding things;
the machine with its lid open; many moments with four to eight carried objects;
two moments with nothing carried; the last moves at West of House.

Advent (39 entries, 30 rooms): the opening; the building with its four objects;
carrying the wicker cage with the bird in it (the cage is closed after the
catch but has the `transparent` attribute, so the bird is listed); carrying the
open bottle with water or oil in it; rooms with fixed scenery or static objects
named in the source (spring, sewer pipes, debris, pool of oil, scrawled
inscription, waterfall, boulders, dust, stone tablet, drawings, carpet,
curtains, moss, sign, sleeping dwarves, enormous mirror) and floating
`found_in` objects (well house, stream, steel grate); the treasure pile in the
building at several stages; the Plover Room with nothing carried; and the
repository end game up to the final command.

Not covered: Advent has no dark moment on this route (the lamp is only ever off
inside the lit building), and the only supporter in the source (the Y2 rock) is
not on the route. Wandering or randomly placed things (dwarves, their axe, the
troll, the Zork thief) are never listed.
