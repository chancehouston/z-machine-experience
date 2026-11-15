# Research: Untangling the Map

- **R1. Around a place.** The player's place is the anchor and keeps its position.
  The places it joins are taken in this order: ways walked out of it, in the
  compass's order; then ways walked into it from a place not yet put, each put
  opposite. Each is put at the first free position one, two or three steps along.
  Then the same is done from each of those, and so outward, a ring at a time.
  A place no ring reaches is put at the nearest free position to the anchor.
- **R2. Tidying the rest.** With the anchor and the places it joins held, each other
  place in the order found is tried at every position within the sheet's bounds
  and one step beyond, swapping with any place there that is not held; the best
  strictly better layout is kept; passes repeat until one changes nothing or the
  budget of candidates is spent. A layout is judged by: joins bent; pairs of joins
  crossing; the summed length of joins in steps.
- **R3. What it gave** (2026-11-15, outside the repository, a wandered Zork I sheet
  of 13 positioned places and 21 joins, 11 of them bent): the figures in the
  specification's table; the tidying took 744 to 1,767 candidates.
- **R4. Attached places.** A place attached by up or down keeps its inset relative
  to its parent: when the parent's position changes by some steps, its inset
  changes by the same. If the anchor is itself attached, its parent is treated as
  the anchor for R1, so that the player's place does not move either.
- **R5. Keeping.** The kept game lists inputs in order. A redraw is a line `redraw`
  among them: it stands after the inputs answered before the press. The session
  keeps, for each redraw, how many inputs had been answered. Bringing back gives
  the inputs in order and applies each redraw when that many have been given. The
  redraw is always of the sheet and around the place the player is in at that
  moment, so the line needs to say nothing else.
- **R6. The same every time.** R1 and R2 use no chance and no clock.
- **R7. The offer.** The map's view has `redraw`, the layer's words, when the game
  waits for a command, the player's place is known, and their sheet has at least
  two places.
- **R8. The screen.** The app sends a press by its own entry and asks for the
  snapshot again; the turn count has not changed, so nothing is spoken and no turn
  is added. A place's box is positioned from its coordinates and a change is
  animated; a join is a shape with animatable ends, so it follows its places.
