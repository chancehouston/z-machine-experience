# Feature Specification: An Authored Map for Zork I

**Feature Branch**: `017-authored-map`

**Created**: 2026-11-15

**Status**: Draft

**Input**: User description: "We have put a lot of work into the map system. I found maps online, including a map created by a Zork implementor. Can we use these maps as a guide without switching to static maps?" And then: "Amend the constitution then start the authored map experience for Zork I."

## What made this possible

The constitution's version 3.0.0, approved by the owner on 2026-11-15, adds Principle
VII: how a game is drawn may be authored by hand, per game, in a **presentation
file**, held to drawing only. This is the first feature to use it.

## Decisions the owner has made (2026-11-15)

1. **The map stays self-revealing.** A room is drawn only once the player has been in
   it. The presentation file says where it goes, never whether it is shown.
2. **Gaps are fine.** A room sits at its authored place; the empty space where a room
   not yet found belongs may show.
3. **The table wins.** A sheet whose places are authored has no redraw control
   (feature 016).
4. **Sheets and regions may be named** by the presentation file. No other text.
5. **A trial first**: Zork I above ground and the first region below, for the owner to
   judge on a phone, before the rest of its 110 rooms are placed.
6. **Published maps are a guide and are not copied.** The positions are the project's
   own, on its own grid.

## Strongest objection

**The map now knows the shape of a game the player has not seen.** Until today every
position on the page came from the player's own steps. With a presentation file the
layout of all of Zork I is in the app before the first move. Nothing is shown early,
and that rule is the heart of this feature and of Principle VII; but a gap is a hint,
a sheet's name is a hint, and the way a room appears far from its neighbour tells the
player there is ground between. The owner weighed this and chose it. It reverses
feature 012's "no foresight" for a game that has a file.

**Second: whose map is it.** A player's notebook is theirs, wrong turns and all. An
authored layout is somebody's idea of the right map. It will be tidier than anything
the player would have drawn, and less their own.

**Third: it is a layout chosen, not found.** Published maps of Zork I disagree with
each other, and the forest's paths do not fit a flat page. Some joins will still not
be straight. The measure is whether a player reads it easily, which only the owner's
trial can say.

**Fourth: copying.** The published maps are other people's work. This specification
requires the project's own positions and a record of what was consulted. Whether that
is enough is a legal question the project has not had answered.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The rooms I find appear where a mapmaker would put them (Priority: P1)

The player wanders the forest and then walks round the house. Each place appears, as
they enter it, at its authored position: the house's four sides make a ring, the
forest lies round it, and nothing has to be redrawn.

**Why this priority**: it is the cure for the tangle that works without the player
doing anything.

**Independent Test**: on Zork I with its presentation file, after any walk above
ground, every place visited is at the position the file gives, and no place not
visited is drawn.

**Acceptance Scenarios**:

1. **Given** a story with a presentation file, **When** the player first stands in a
   place the file has, **Then** it is drawn on the sheet and at the position the file
   gives.
2. **Given** the same, **Then** a place the player has not been in is not drawn, and
   nothing of it is shown: no box, no name, no line, no mark.
3. **Given** a place the file does not have, **Then** it is put by the rules the map
   has today, from the place it was reached from.
4. **Given** two places joined by a way walked, **Then** the join is drawn straight
   or bent by the same rule as today, from the positions they have.
5. **Given** the same game without the file, **Then** the map is exactly what it is
   today.

---

### User Story 2 - Sheets are regions with names (Priority: P1)

The file groups rooms into sheets by region and may name each. The opened-out map
shows the name with the sheet's number.

**Acceptance Scenarios**:

1. **Given** a place the file puts on a named sheet, **When** the player has been on
   that sheet, **Then** its label carries the name in place of the first place found.
2. **Given** a sheet the player has not been on, **Then** its name is not shown
   anywhere, and the count of sheets counts only sheets with a place drawn.
3. **Given** a walk from one authored sheet to another, **Then** the way between is
   shown as a way that leaves the sheet, as today.

---

### User Story 3 - An authored sheet is not redrawn (Priority: P1)

**Acceptance Scenarios**:

1. **Given** the player is on a sheet whose places are authored, **Then** the map
   offers no redraw.
2. **Given** a kept game holding a redraw of a sheet that is now authored, **When**
   it is brought back, **Then** the game comes back, with the authored positions; the
   redraw is passed over.
3. **Given** a sheet with no authored place (another story, or a place off the
   file), **Then** redraw is offered as today.

---

### User Story 4 - The file is honest, and ours (Priority: P1)

**Acceptance Scenarios**:

1. **Given** a presentation file, **Then** it names its story by digest, release and
   serial, and is not used for any other file.
2. **Given** any entry, **Then** the place it names is a room of that story, and no
   two entries put places in one position of one sheet.
3. **Given** the file, **Then** it holds numbers and the names of sheets, and no text
   of the game.
4. **Given** the file's folder, **Then** a note beside it says what was consulted in
   making it and that nothing was copied.
5. **Given** what the layer reports, **Then** a place whose position came from the
   file is marked as authored.

---

### User Story 5 - The trial (Priority: P1)

The first file places Zork I's rooms above ground and its first region below. The
owner walks them on a phone before more is placed.

**Acceptance Scenarios**:

1. **Given** the trial file, **Then** every room above ground that the walkthrough or
   a long wander reaches is in it.
2. **Given** a room of the game that is not in the trial file, **Then** the map
   treats it as User Story 1 scenario 3 says, and the game is fully playable.

### Edge Cases

- The file is damaged, or for another story: it is refused whole, and the map is as
  without one.
- A place found in the dark: it is drawn at its authored position with no name, as a
  dark place is today.
- A place attached to another by up or down (the Attic, Up a Tree): the file may
  give it a position like any other; if it does not, it is attached as today.
- A game started before the file existed, brought back with it: the map is made
  again from the turns, with the authored positions.
- Starting over, death, restart: the positions are the same; they do not depend on
  the order of discovery.
- A sheet with both authored and unauthored places: the unauthored are put by
  today's rules and never take an authored position.

## Requirements *(mandatory)*

### Functional Requirements

**The file**

- **FR-001**: A presentation file MUST name exactly one story file by SHA-256 digest,
  release and serial number, and MUST be used only when all three match.
- **FR-002**: It MUST hold, for each place it has, the place's number in the story,
  a sheet and a position; and MAY hold a name for a sheet. It MUST hold nothing else
  that reaches the player.
- **FR-003**: A file that departs from its form, or names what is not a room of its
  story, or puts two places in one position, MUST be refused whole.
- **FR-004**: The file's form MUST be written down in this repository.
- **FR-005**: The record of what was consulted to make a file MUST be kept beside it.

**The map**

- **FR-006**: A place the file has MUST be put on the sheet and at the position the
  file gives, when the player first stands in it. It MUST NOT be shown before.
- **FR-007**: A place the file does not have MUST be put by the map's present rules,
  and MUST NOT take a position the file gives to another place of that sheet.
- **FR-008**: With no file, every position, sheet and line MUST be what it is today.
- **FR-009**: A sheet's name from the file MUST be shown only once the player has
  been on that sheet.
- **FR-010**: What the layer reports MUST say, for each place, whether its position
  is authored.
- **FR-011**: A redraw MUST NOT be offered on a sheet that has an authored place; a
  kept redraw of such a sheet MUST be passed over when a game is brought back.
- **FR-012**: The positions MUST NOT depend on the order in which places were found.

**The app**

- **FR-013**: The app MUST hand the layer the presentation file bundled with its
  story, as it hands the game pack, and MUST read none of it.

**The trial**

- **FR-014**: The first file MUST cover Zork I above ground and its first region
  below. The rest of the game is placed after the owner's trial, as a later stage.

**Limits**

- **FR-015**: The limits of earlier features hold.

### Key Entities

- **Presentation file**: for one story: its identity; sheets, each with an optional
  name; places, each with its number, sheet and position.
- **Authored place**: a place whose position came from the file.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On the wander of earlier features through Zork I above ground, with the
  trial file, the sheet has no crossing and at most a quarter of the bent joins it
  has today without one. The figures are recorded.
- **SC-002**: At every turn of Zork I's walkthrough and of a wander of 400 moves, no
  place is drawn that the player has not been in, and every authored place drawn is
  at its file's position.
- **SC-003**: The same game played in two different orders of discovery ends with
  every authored place at the same position.
- **SC-004**: Without the file, every recorded page and every map test of earlier
  features passes unchanged.
- **SC-005**: Every entry of the file names a room of its story, and no two share a
  position.
- **SC-006**: The owner, on their phone, walks the house and forest and the first
  rooms below, and says whether the page reads as a map of the place, and whether
  the gaps trouble them.

## Assumptions

- Only Zork I gets a file in this feature.
- The file is made with the help of a tool that drafts positions from the ways walked
  on the walkthrough and long wanders, and is then adjusted by hand with published
  maps open beside it.
- A different look for outdoor places, pictures and item marks are later uses of
  Principle VII and are not part of this.

## Out of Scope

The rest of Zork I's rooms (a later stage); other games; pictures or decoration;
showing places not yet visited; letting the player move a room.
