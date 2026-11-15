# Feature Specification: Untangling the Map

**Feature Branch**: `016-map-untangle`

**Created**: 2026-11-15 (written again the same day, after the owner changed its
direction; the first version is in the history of this file)

**Status**: Draft

**Input**: User description: "Let's keep improving the map. The tangle gets very confusing." And then: "The rooms that actually connect to the room I'm in are really the only priority. ... Maybe with a button press, we relay out the rooms around the person. Every room is expected to move except the room the person is in, so that the connecting rooms are clear."

## How this came to be what it is

The owner named two faults on a picture of their phone: rooms in the wrong place,
and lines that cross and curl. A first specification laid a whole sheet out again
by itself whenever that left fewer joins bent, and drew what stayed bent as short
named marks. Its plan found that the search the app could afford did less than the
pictures had shown.

The owner then saw the matter differently: what a player needs is the rooms round
the one they are in. Far rooms may tangle. This specification is that idea.

## What was measured (2026-11-15, outside the repository)

One sheet of Zork I after a wander of 400 moves: 16 places, 21 joins. Laid out
around each of five rooms in turn:

| Standing in | Its own joins straight, today | After |
| --- | --- | --- |
| South of House | 2 of 3 | 3 of 3 |
| West of House | 2 of 3 | 3 of 3 |
| Forest Path | 0 of 4 | 4 of 4 |
| Clearing | 2 of 4 | 4 of 4 |
| Forest | 2 of 4 | 3 of 4 |

With the far rooms then tidied while the near ones were held, the sheet had between
6 and 9 bent joins (11 today) and at most one crossing.

## Decisions the owner has made (2026-11-15), from pictures

1. **The sheet is laid out around the room the player is in.** That room does not
   move. The rooms it joins are put where the ways walked say. Every other room may
   move.
2. **It happens when the player presses a button**, and at no other time. Between
   presses a room stays where it is, as in feature 012.
3. **The far rooms are tidied too**: after the player's room and the rooms it joins
   are set, the rest are moved to leave fewer joins bent and crossing.
4. **A join that still cannot be straight is a curved line, as today.** The named
   marks of the first version are dropped.
5. **A redrawn sheet is kept with the game** and comes back as it was left.

These replace, for a sheet the player has redrawn, feature 012's rule that a placed
place never moves. Still standing: only what the player has walked is used; the map
shows only what was witnessed.

## Strongest objection

**One room's clarity is bought with every other room's place.** Press the button in
the kitchen and the forest is somewhere new; walk to the forest, press again, and
the kitchen has moved. The player gives up a page they can learn by heart for one
that is clear where they stand. The owner chose this, and the button makes it the
player's own act each time, never a surprise. But a player who presses often will
never see the same sheet twice.

**Second: it cannot always be done.** Where two ways out of the player's room lead
to the same room, or a way leads back to the room itself, not every join of the
room can lie along its way. In the measure, one room of five kept one bent join.

**Third: the kept game stops being only what was sent to the game.** A press is
recorded in it, the first thing there that the game never saw. It has to be, for
the map to come back the same.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - I press a button and the rooms round me are clear (Priority: P1)

The player is in South of House, drawn off to one side with a curved line to West
of House. They press the map's redraw button. South of House stays where it is on
the screen; West of House is now directly west of it, the forest directly south,
Behind House to the north-east, each joined by a straight line.

**Why this priority**: it is the whole of what was asked for.

**Independent Test**: on Zork I, after a walk that leaves South of House's joins
bent, a press leaves every way walked out of South of House lying along the page.

**Acceptance Scenarios**:

1. **Given** the player is in a place on the map, **When** they press redraw,
   **Then** that place has not moved, and each place it is joined to by a walked
   compass way is exactly along that way from it, one to three steps off.
2. **Given** two ways out of the player's place lead to the same place, **Then** the
   first of them in the compass's order decides where that place is.
3. **Given** a place joined to the player's only by a way walked toward it, **Then**
   it is put opposite to that way.
4. **Given** a press, **Then** nothing is sent to the game: the turn, the moves and
   the transcript are as they were.
5. **Given** a sheet of one place, or the player's place not known, or the game not
   waiting for a command, **Then** there is no redraw to press.
6. **Given** the player walks on after a press, **Then** no place moves until they
   press again; a new place is put by feature 012's rules.

---

### User Story 2 - The rest of the sheet is as tidy as it can be made (Priority: P1)

**Why this priority**: the owner chose it over leaving far rooms where they fall.

**Independent Test**: on the measured wander, after a press in any room, the sheet
has fewer bent joins and crossings than the same sheet with the far rooms left
where the first step put them.

**Acceptance Scenarios**:

1. **Given** a press, **Then** the places that are neither the player's nor joined
   to it are moved to leave fewer joins bent, then fewer crossing, without moving
   the player's place or the places it joins.
2. **Given** a place attached to another (up or down), **Then** it keeps its corner
   of that place wherever that place goes.
3. **Given** any press, **Then** no two places share a position.
4. **Given** the tidying would take longer than the time allowed, **Then** it stops
   with the best it has found; the player's place and its joins are right all the
   same.

---

### User Story 3 - I can follow the page as it changes (Priority: P1)

**Acceptance Scenarios**:

1. **Given** a press, **Then** each place glides from where it was to where it is,
   with its lines, over a short moment, and the player's place stays in the middle.
2. **Given** Reduce Motion, **Then** the places are at their new positions at once.

---

### User Story 4 - The sheet I redrew is there when I come back (Priority: P1)

**Acceptance Scenarios**:

1. **Given** a game in which sheets were redrawn, **When** it is closed and brought
   back, **Then** every place is where it was left.
2. **Given** a kept game written before this feature, **Then** it is brought back as
   it always was.
3. **Given** the same game with the same presses at the same turns, **Then** the map
   is the same every time.

---

### User Story 5 - A listener can press it (Priority: P3)

**Acceptance Scenarios**:

1. **Given** a screen reader, **Then** the control is a button with the layer's
   words, and pressing it is not announced as a turn.

### Edge Cases

- Pressed twice in the same place: the second press changes nothing.
- Pressed on a sheet where every join of the player's place is already straight:
  the far rooms may still be tidied.
- The player's place is an attached one (up a tree): it does not move; the place it
  hangs from is put so that it keeps its corner.
- A new game, or starting over: no press is carried into it.
- Looking at another sheet (feature 015): the control redraws the player's own
  sheet, and is not shown on a sheet they are not on.
- Largest text size: the control is at least 44 points and overlaps no way's
  control.

## Requirements *(mandatory)*

### Functional Requirements

**The redraw**

- **FR-001**: The map MUST say when the player's sheet can be redrawn, and give the
  words for the control.
- **FR-002**: A redraw MUST leave the player's place where it is.
- **FR-003**: A redraw MUST put each place joined to the player's by a walked compass
  way exactly along that way, at the first free position of one, two or three steps;
  for a place joined by more than one way, by the first in the compass's order; for
  a place joined only by a way walked toward the player's, opposite to it.
- **FR-004**: A redraw MUST then move the other places of the sheet to leave fewer
  joins bent, then fewer crossing, holding still the player's place and the places
  it joins. This is bounded in the work it may do.
- **FR-005**: A redraw uses only the places the player has been in and the ways they
  have walked.
- **FR-006**: A redraw MUST send nothing to the game and add no turn.
- **FR-007**: Between redraws no place moves; a new place is put by feature 012's
  rules.
- **FR-008**: The same game with the same redraws at the same turns MUST give the
  same positions, every time and on every machine.

**Keeping**

- **FR-009**: Each redraw MUST be kept with the game, at its place among the inputs,
  so that a game brought back has the positions it was left with.
- **FR-010**: A kept game written before this feature MUST still be brought back.

**The screen**

- **FR-011**: The app MUST show the control wherever the map is shown and the
  snapshot offers it, send a press to the layer, and show the snapshot that comes
  back. It works out no position.
- **FR-012**: A place whose position changes MUST glide to it with its lines; at
  once with Reduce Motion.

**Limits**

- **FR-013**: A press and its snapshot MUST be answered within 50 ms on both
  walkthroughs and on the measured wander. It is the player's own act and not a
  command of the game, so the 10 ms limit of a command is not the measure.
- **FR-014**: The cost of a command and its snapshot is unchanged: nothing of this
  runs unless the control is pressed.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On the measured wander through Zork I, a redraw in each of the sheet's
  positioned rooms in turn leaves, for at least four rooms in five, every join of
  that room straight; and for every room, no fewer than it had.
- **SC-002**: On that wander, after a redraw in any room the sheet has no more bent
  joins than it has today.
- **SC-003**: A game with redraws, brought back, has the same map as the game never
  closed, in 100 of 100 trials of one fixed game; and Zork I's walkthrough brought
  back at every tenth turn is as it is today.
- **SC-004**: With no press, every recorded page and every test of the map that
  passed before this feature passes unchanged.
- **SC-005**: Across the corpus's wander of every recognised story, a redraw at the
  end leaves no two places in one position and faults nowhere.
- **SC-006**: The owner, on their phone, wanders the house and forest, presses the
  button where the page confuses them, and says whether the rooms round them are
  now clear.

## Assumptions

- The control is shown in the band and in the opened-out map.
- A redraw is of the sheet the player is on; other sheets are untouched.
- There is no way to undo a redraw but to walk elsewhere and press again.

## Out of Scope

Laying out by itself; moving a room by hand; named marks in place of curves; mazes;
foresight; undoing a redraw.
