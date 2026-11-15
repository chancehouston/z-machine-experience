# Feature Specification: The Other Sheets of the Map

**Feature Branch**: `015-map-sheets`

**Created**: 2026-11-14

**Status**: Draft

**Input**: User description: "A way to look at the sheets of the map you are not standing on. Zork I has eleven and today only one can be seen."

## Decisions the owner has made (2026-11-14)

1. **Moving between sheets**: "Follow the ways, plus arrows". A touch on a way that
   leaves the sheet turns the map to the sheet it leads to. Two arrows step to the
   sheet before and the sheet after, in the order the sheets were found.
2. **Where**: "Opened-out map only". The band always shows where the player is.
   Closing the map returns to the player's own sheet.
3. **What a sheet is called**: "Number and a place": its number, how many there
   are, and the first place the player found on it.

## Strongest objection

**A sheet the player is not on is a picture of the past, and the map will not say
so.** The places and ways drawn on it are as they were when the player last stood
there. The map has always worked this way for places off the current sheet, but
until now those could not be looked at. A door drawn closed on sheet 3 may be open
now. The feature shows what was witnessed and claims nothing about now, and nothing
on another sheet can be touched to act.

**Second: the arrows go by the order sheets were found, which is not the shape of
the world.** Sheet 4 is not "next to" sheet 3. The ways that leave a sheet are the
true connections; the arrows are only so that no sheet is out of reach.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - I turn to the sheet a passage leads to (Priority: P1)

The player is in the Living Room with the map opened out. A short line leaves the
room, labelled as leading down to the Cellar, which is on another sheet. They touch
it. The map turns to that sheet and shows the Cellar and the places about it.

**Why this priority**: it is how a notebook is used: follow the passage to its page.

**Independent Test**: on Zork I, go down to the Cellar and back up, open the map,
touch the way that leads down: the sheet shown has the Cellar on it and its label
says so.

**Acceptance Scenarios**:

1. **Given** the map is opened out on the player's sheet, with a walked way that
   leaves it, **When** that way is touched, **Then** the sheet it leads to is shown,
   moved so that the place it leads to is in the middle.
2. **Given** another sheet is shown, with a walked way that leaves it, **When** that
   way is touched, **Then** the sheet it leads to is shown, which may be the
   player's own.
3. **Given** the band, not opened out, **Then** a way that leaves the sheet cannot
   be touched, as now.
4. **Given** any touch that turns the sheet, **Then** nothing is sent to the game:
   the turn, the moves and the transcript are as they were.

---

### User Story 2 - I step through every sheet (Priority: P1)

Two arrows on the opened-out map step to the sheet before and the sheet after. From
the last, the next is the first.

**Why this priority**: a sheet reached only by a way never walked back, or after a
restart, has no way drawn to it; the arrows are how it is reached.

**Independent Test**: with three sheets, the next arrow pressed three times shows
each sheet once and returns to the first.

**Acceptance Scenarios**:

1. **Given** more than one sheet, **When** the next arrow is touched, **Then** the
   sheet after is shown; from the last, the first. The arrow before goes the other
   way.
2. **Given** only one sheet, **Then** there are no arrows.
3. **Given** a sheet shown by an arrow, **Then** it is moved so that the first place
   found on it is in the middle.

---

### User Story 3 - I always know which sheet I am looking at, and can get back (Priority: P1)

The opened-out map says which sheet is shown. On a sheet the player is not on, the
player's marker is not drawn, no way can be walked, and a control returns to where
they are.

**Why this priority**: without it the player can take a remembered sheet for the
place they are standing in.

**Independent Test**: on another sheet there is no lamp, no way control and no
touchable place; the label names the sheet; the return control shows the player's
own sheet with the lamp on their place.

**Acceptance Scenarios**:

1. **Given** the map opened out, **Then** it shows the sheet's label: its number, how
   many sheets there are, and the first place found on it that has a name; or the
   numbers alone where no place on it has a name yet.
2. **Given** a sheet the player is not on, **Then** no place carries the lamp, no
   way of any place can be touched to walk it, and no place can be touched to look.
3. **Given** such a sheet, **When** the return control is touched, **Then** the
   player's sheet is shown with their place in the middle. On their own sheet there
   is no return control.
4. **Given** another sheet is shown, **When** the map is closed, or a new turn
   comes, **Then** what is shown is the player's own sheet.
5. **Given** a sheet the player is on, reached by an arrow or a way, **Then** it is
   their sheet as always: the lamp, the ways and the touchable place.

---

### User Story 4 - A listener can use it (Priority: P2)

**Acceptance Scenarios**:

1. **Given** a screen reader, **Then** the arrows, the return control and each way
   that leaves a sheet are buttons with the layer's words; and the map's spoken
   summary says which sheet is shown and that the player is not on it.

### Edge Cases

- A way walked only toward a sheet, never back: the sheet it came from has the way
  drawn; the sheet it leads to has none back. The arrows reach it.
- Two ways from one sheet lead to the same other sheet: each turns to that sheet,
  centred on its own far place.
- A sheet whose first place was only ever seen in the dark: the label uses the
  first place on it that has a name, or none.
- The sheet shown is dragged, then turned: the new sheet starts centred.
- Reduce Motion: the turn is not animated.
- The player's sheet number changes while another is shown (a group of four moves to
  its own sheet at the new turn): the new turn returns to the player's sheet anyway.
- Largest text size: the arrows, the label and the return control stay on screen
  and do not overlap the open/close control.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The layer MUST give any sheet of the map on request, in the form the
  player's sheet has in the snapshot, with only what was witnessed.
- **FR-002**: A sheet the player is not on MUST carry no marker of where the player
  is, no way that can be sent, and nothing that can be touched to act on the game.
- **FR-003**: Each sheet MUST carry its label and its spoken summary, in the layer's
  words: its number, the count of sheets, and the first place found on it that has a
  name.
- **FR-004**: Each way that leaves a sheet MUST say which sheet it leads to and which
  place there, so that the sheet can be turned to and centred on it.
- **FR-005**: The sheets are ordered as they were found; that order does not change
  as the game goes on, save that a new sheet is added at its end.
- **FR-006**: Turning the sheet MUST send nothing to the game and change nothing of
  it: not the turn, the kept game, or the map itself.
- **FR-007**: The app MUST show another sheet only in the opened-out map. The band
  always shows the player's sheet.
- **FR-008**: The app MUST return to the player's sheet when the map is closed and
  at any new turn.
- **FR-009**: The app composes no text and decides no order: the label, the spoken
  words, which sheet is before and after, and where a way leads are the layer's.
- **FR-010**: The limits of earlier features hold. A sheet is given within the time
  a command is allowed.

### Key Entities

- **Sheet**: one page of the map: its number, its label, its places, lines, ways
  that leave it, and (on the player's sheet only) the player's place and its ways.
- **Lead**: a walked way that leaves a sheet, with the sheet and the place it
  leads to.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At the end of Zork I's walkthrough every one of its sheets can be
  shown, and together they hold every place on the map exactly once.
- **SC-002**: Every way that leaves a sheet, on every sheet, leads to a sheet that
  holds the place it names.
- **SC-003**: Asking for any sheet leaves the kept game and the snapshot identical,
  at every sheet of both walkthroughs.
- **SC-004**: Every test that passed before this feature passes after it.
- **SC-005**: The owner, on their phone, finds a place they visited earlier on
  another sheet, and gets back to where they are.

## Assumptions

- A sheet is asked for when it is wanted, not sent with every snapshot.
- Nothing can be done from another sheet: no walking to a place, no looking.
- Sheets have no names of their own; a later feature may let the player name them.

## Out of Scope

An overview of all sheets at once; naming sheets; walking to a place on another
sheet; showing where things are; marking a sheet's contents as out of date.
