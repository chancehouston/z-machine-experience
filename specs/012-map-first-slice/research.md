# Research: The Map, First Slice

Each entry: the decision, why, and what else was weighed. The spike is
`crates/zmachine-corpus/examples/map_spike.rs` on branch `spike-map-layout` (not
committed); its figures are from Zork I's and Adventure's walkthroughs on 2026-11-13.

## R1. The map is a fold over the turns

**Decision**: a new module `map` in the experience layer holds the record and the
page. It is fed once at every stop of the game, from inside `Session::advance`, with
what was just sent and what the layer now reports. It keeps no clock and reads no
randomness. For one story, one chance and one sequence of inputs it gives one map.

**Why**: feature 011 brings a game back by giving it its inputs again, through the
same path. A map that is a function of those turns comes back with the game for
nothing (FR-037), and is the same on every platform (FR-015). It also makes the tests
plain: play a script, compare the page.

**What follows**: no map is stored. Nothing in the map may depend on the order of a
hash map or on anything outside the session; ordered maps are used throughout.

## R2. What the map is given at each stop

**Decision**: an observation, made after the compass, the light and the status are
worked out:

- the input just sent, if any: its text and whether typed or tapped;
- what the game is waiting for;
- the layer's `location`: the room's number, its name, whether lit (yes, no, unknown);
  or that it is unknown;
- the compass as settled (feature 009), with its extras;
- whether the game jumped since the last stop (R3).

The map takes the room's number as the place's identity (FR-003). It never asks the
introspection interface anything itself.

**Why the compass and not the exits**: the compass is what the player was shown. A way
the look-ahead settled as no way is absent from it, which is FR-018 without more work,
and the states and spoken labels are already the layer's words.

## R3. Knowing that the game jumped (FR-006)

**Decision**: the core gains one read-only counter, `Machine::jumps()`: the number of
times the game has restarted itself, restored a saved position, or undone a turn. The
layer compares it before and after a command. If it rose, the move is recorded as no
move: the marker goes to where the player now is and nothing joins the two places.

**Why in the core**: only the core knows. The layer sees a line go in and a stop come
out. Without this, typing `undo` in a version 5 game draws a way named "undo".

**Why a counter**: it adds no event and changes no behaviour, so the fidelity tests
and every transcript are untouched (Principle I).

**Death cannot be told, and the specification is corrected.** FR-006 as first written
included "put the player elsewhere on their death". No fact in a story file says that
a player has died; the game prints some words and moves them. The review said as
much. So FR-006 now covers undo, restart and restore only, and a death is handled by
R4. The specification is changed with this plan, and the change is listed in plan.md
for the owner.

## R4. A direction walked that did not go where the way said

**Decision**: when the command is a direction, and the compass at the place left gave
that way as open to a named place, and the player arrives somewhere else, the move is
recorded as **carried**: a short line from the place left, marked with the command,
to the place reached, with no claim about direction. It does not place the new room
in that direction and does not count as walking that way.

**Why**: this is what a death, a slide, or a river looks like from outside. The corpus
found 183 such cases, all the game's code overriding its own table. Recording them as
compass lines would put a room in the wrong place for good, since a placed room never
moves.

**When the way gave no destination** (uncertain, or a door) the move is taken as
walked: there is nothing to contradict.

## R5. Witnessed, as rules

**Decision**: the table is in `contracts/map.md`, part W, and each row is a test
(FR-008). The rulings behind it:

- A place exists once the player is in it at a stop where the game waits for a
  command.
- Its name and ways are seen when the player is there and the layer does not report
  it dark: lit, or light unknown (the owner's confirmation, point 3).
- A way's state is as last seen from the place.
- A way's destination is never shown or spoken until the line is walked. This changes
  the spoken label: the compass says "West, to Kitchen"; the map says "West" until
  the player has walked it. FR-032 asked for the compass's labels; they are the
  compass's labels less the name of a place not yet reached. The specification is
  corrected to say so.
- The mark for a way into the dark is kept (point 1).
- A way typed and refused is drawn as refused (point 2): the command was a direction
  word, the compass did not offer it or offered it as uncertain, and the place did
  not change and the game did not jump.

## R6. Placement

**Decision**: the rules of FR-009 to FR-014 as the spike drew them under "attached",
turned to work as the player goes:

- Positions are whole numbers on a sheet. A sheet is a sparse grid.
- **Compass**: one, two or three steps along the direction, the first that is free;
  else the nearest free position by rings around one step, preferring the side the
  direction points to; ties broken by a fixed order.
- **Up or down**: on the same sheet, in the nearest free position to above-right (up)
  or below-left (down) of the place left, passing over any position that lies on a
  compass way that place has (FR-028). The new place is marked as attached to the
  place left, and joins its **group**. Places reached from a group by compass, in or
  out, join the group.
- **A group that reaches four places** moves to a new sheet, once: every place of it
  keeps its position relative to the others; the group ends, and its places are
  ordinary places of the new sheet. This is the one move FR-010 allows.
- **In or out**: the nearest free position to the place left.
- **Any other command, or carried (R4)**: a new sheet; unless the place left is on a
  sheet that was started that way and the place left was itself reached by such a
  move, in which case the nearest free position below it on that sheet.

**What the spike gave under these rules**: Zork I, 83 places on 11 sheets, 49 of 60
compass lines straight, none crossing; Adventure, 65 places on 7 sheets, 44 of 51.

**Known weakness**: an attached room can land well away from its parent when the near
positions are taken (the Attic, in the spike's Zork I page). The drawing shows the
ladder, so it reads, but not well. It is left for the owner's eye at SC-009.

**Not used**: any knowledge of places not stood in (FR-011). The spike measured what
foresight would buy: two bent lines fewer in Zork I and three in Adventure.

## R7. Straight or bent

**Decision**: the layer says, for each line, whether it is straight. A compass line is
straight when the far place lies exactly along the direction walked from the near
one, the same holds for the way back if it was walked, and no other place lies
between. Otherwise it is bent, and the app draws it leaving each end in the direction
walked.

**Why in the layer**: it is a fact about the page, and both platforms must agree.

## R8. What the snapshot carries

**Decision**: one new key, `map`, after `start_over`. It holds the sheet the player
is on, whole: its places with positions, its lines, its leads to other sheets, the
ways of the place the player is in, and the layer's words. Its form is in
`contracts/map.md`, part S.

**Why the whole sheet each time**: a sheet is at most thirty places (SC-004) and
comes to a few kilobytes. Sending changes only would need the app to keep a copy in
step, which is a second place for the map to be wrong.

**The cost is measured** (FR-039): the time to make the snapshot at the largest sheet
of each walkthrough, and its size.

**The `compass` key stays** in the snapshot for now. The command-line program and the
tests of earlier features read it. The app stops reading it.

## R9. How the app draws

**Decision**: SwiftUI shapes, one view for each place and each line, in a container
that is moved and clipped; not a canvas.

**Why**: a sheet is at most thirty places. Shapes animate by themselves (a line's
`trim` gives the pencil drawing in, FR-029), and each is an accessibility element
without a parallel tree (FR-032). The research found that a canvas offers neither.

**The hand**: each outline's wobble is worked out from the place's number, so it is
the same every time (FR-030). The layer gives the number; the app has no other use
for it.

**The ways of the place the player is in** are buttons laid over the drawing, with
the identifiers the compass's buttons had (`compass.north` and so on), so that the
tests of earlier features that walk by tapping need no change but their expectations
of labels.

## R10. The keyboard

**Decision**: the command field takes the focus when the game waits for a line and
keeps it. Sending a command does not give it up. Two things lower the keyboard: the
map being opened out, and a drag on the transcript. Touching the field, or sending a
command, raises it.

**The inventory strip and the note of darkness** move into the band's lower edge,
since with the keyboard up there is no other room (FR-024).

**The band's height** is fixed at 200 points for the trial on the owner's phone; it is
the first thing SC-009 is to judge.

## R11. Stages

**Decision**: three stages, each its own pull request, each leaving the app working.

1. **The layer.** The core's counter; the map module with its rules, placement and
   snapshot key; the recorded pages and all the layer's tests. The app is untouched
   and still shows the compass.
2. **The band.** The app draws the band from the `map` key, the keyboard stays up,
   the compass view is removed, the strip and the note move, the spoken labels are
   carried over. Plain lines, no animation.
3. **The sheet and the hand.** Opening the band out; the pencil outline; drawing in;
   the lamp marker; reduced motion and larger text.

**Why**: the specification is the largest in the project and replaces a finished
control. Stage 1 can be proven wholly by tests before anything on the screen changes.
Stage 2 is where the owner first holds it. Stage 3 cannot hide a fault in the first
two.

**The owner's phone** is asked for after stage 2 and again after stage 3.

## R12. Where recorded pages are kept (FR-046)

**Decision**: `stories/zork1/map/walkthrough.txt` and
`stories/advent/map/walkthrough.txt`, beside the `introspection/` folders that
already hold those stories' room names under their licenses. No other story has a
recorded page. The corpus's reports stay free of names, as now: for the other stories
the tests assert counts and FR-040 only.

**The form** is text, one line for each place and each line of the map, sorted, so
that a change shows as a readable difference.

## R13. A second order of exploring (FR-043)

**Decision**: a test in `zmachine-corpus`, which already depends on the experience
layer and has the crawl: each of Zork I and Adventure is walked by a fixed-seed
wander (at each prompt, one of the offered ways chosen by a seeded draw, for 400
moves), and the measures of SC-003 are printed and recorded in tasks.md, with FR-040
asserted.

## R14. What changes in tests that exist

- The snapshot's key list gains `map`; checks of the last key move to it.
- Every app test that reads a compass place's `label` for a destination ("West, to
  Kitchen") changes: the destination is spoken only once walked.
- App tests that depend on the compass being hidden by the keyboard, or shown when it
  is lowered (`button.compass`), change or go: there is no such state.
- `crates/zmachine-ffi/tests/app_rules.rs`: the rules that name `CompassView.swift`
  and `CompassLayout.swift` are rewritten for the map's files.
- Each change is listed in tasks.md as it is made (FR-045).
