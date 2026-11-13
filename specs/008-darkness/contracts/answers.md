# Contract: The Published Interface and the Pack Format

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

What `docs/introspection.md` is to say, at version 0.4.0. Each numbered statement
becomes a sentence there with a test that cites it.

## The pack (changes to I§12)

1. `lit-attribute: L`, a decimal number 0 to 31, at most once, says that in this story
   attribute L is set on a room that is lit of itself and on a thing that gives light.
2. The format's version is 3. A reader of version 3 reads versions 1, 2 and 3. A pack
   that holds `lit-attribute` states `pack: 3`; in an earlier version the key is a
   fault.
3. A pack may change `location`, `scope`, `exits` and `inventory`, and no other query.

## Light (to be I§12.9, and changes to I§6.2 and I§9.2)

4. A light is an object with attribute L, under the player's room in the object tree,
   such that every object between it and the room is the player, is open, or can be
   seen through (I§7.2).
5. With `lit-attribute` in force, `location`'s `lit` is `yes` if the room has attribute
   L or a light is present, and `no` otherwise, except as statements 6 and 7 say.
6. If the player cannot be told (I§7.2), the room has not attribute L, and no light is
   seen outside the objects that tied, `lit` is `unknown`.
7. If an object with attribute L is inside another and no `open-attribute` is in force,
   and nothing else makes the room lit, `lit` is `unknown`.
8. `scope`'s `light` is `lit`, `dark` or `unknown` as `location`'s `lit` is `yes`, `no`
   or `unknown`. Its contents are listed as before.
9. `location` and `scope` each have a field `source`: `pack` if the light would be
   different with no pack in force, and `file` otherwise.

## Dark ahead (to be I§12.10, and changes to I§8 and I§2.3)

10. Every exit has a field `dark_ahead`: `yes`, `no` or `unknown`.
11. For an exit with a `destination`, with `lit-attribute` in force: `no` if that room
    has attribute L, or a light lies in it by statement 4 taken from that room, or the
    player carries a light; `yes` if none of those; `unknown` where statement 6 or 7
    would apply.
12. For an exit with no `destination`, and wherever no `lit-attribute` is in force,
    `dark_ahead` is `unknown`.
13. An exit whose `dark_ahead` is `yes` or `no` has `source` `pack` (I§12.5).
14. In JSON an exit's keys are, in order: `direction`, `words`, `kind`, `destination`,
    `destination_name`, `door`, `state`, `dark_ahead`, `source`. `location`'s are
    `room`, `name`, `holder`, `lit`, `source`; `scope`'s are `light`, `source`,
    `player`, `objects`.
15. `dark_ahead` says what the room would be for the player as they are at the moment
    of the query. The game has the last word (I§8.3).

## What changed from 0.3.0

- `location` and `scope` have one more field, `source`; an exit has one more,
  `dark_ahead`.
- The pack format has a version 3, with one more key.
- A pack may change `location` and `scope`.
