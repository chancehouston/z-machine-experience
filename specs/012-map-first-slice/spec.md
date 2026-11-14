# Feature Specification: The Map, First Slice

**Feature Branch**: `012-map-first-slice`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "A self-revealing map that is the main screen." The owner
asked on 2026-11-13 whether the map, the scene and the graphical interface "can't be
one thing", with the map "robust enough to be the thing that the user stares at", and
said "we really want the map feature to be the feature". This is the first slice of
it. What it rests on, all of 2026-11-13: research into earlier maps
(`reports/Self revealing map design research.md`); two sets of sketches the owner
approved; a review by a game developer; a spike that drew Zork I and Adventure from
their walkthroughs; and the owner's decisions, listed under Assumptions with what was
decided by the owner and what by me.

## User Scenarios & Testing *(mandatory)*

The owner played Zork as a child with a notebook beside the computer, drawing the map
by hand. This feature is that notebook, kept by the app: as the player walks, the page
draws itself.

Until now the screen has been the game's text with a compass beneath it: twelve
places, each showing whether there is a way. The compass says what is one step away
and remembers nothing. The map says the same and remembers everything the player has
seen. So the compass is not kept beside the map. It becomes part of it: the ways out
of the room the player is in are the lines that leave that room on the page.

This slice gives the map two depths: a **band** at the top of the screen while the
player types, showing where they are and what is one step away; and the **sheet**,
the whole page they are on, when they open the band out. It draws every place as a
box. It does not yet show what lies in a room, take the player to a far room, keep a
map from one game to the next, or show all sheets at once. Those are later slices.

Words used throughout:

- **Place**: a room of the game, as the game itself tells one room from another.
- **Way**: a direction out of a place that the layer reports, with the state the
  compass gave it: open, uncertain, a door open or closed, no way, not known.
- **Stub**: a way the player has seen and not walked, drawn as a short line.
- **Line**: a connection the player has walked, drawn between two places.
- **Sheet**: one page of the map. Places on a sheet have positions; sheets do not
  pretend to fit together.
- **Witnessed**: seen by the player in this game. The map shows only what is
  witnessed, however much more the layer could read.
- **Band**: the map as shown above the game's reply while the keyboard is up.

### User Story 1 - The map draws itself as I walk (Priority: P1)

A player starts a game. The band shows one box, the place they are in, with a short
line for each way out. They walk north. A line is drawn to a new box above the first,
and the view moves to keep them in the middle. They walk on, and come back, and the
places they passed are where they were drawn. They never tell the map anything.

**Why this priority**: this is the feature.

**Independent Test**: play the first ten moves of Zork I's walkthrough; after each, the
map holds exactly the places stood in so far, each where it was first put, joined by
exactly the lines walked.

**Acceptance Scenarios**:

1. **Given** a new game, **When** it opens, **Then** the map holds one place, named,
   with a stub for each way the layer reports there.
2. **Given** the player walks a compass direction to a place not yet on the map,
   **When** the reply arrives, **Then** that place is drawn one step away in that
   direction, joined by a line, and marked as where the player is.
3. **Given** a place is on the map, **When** the player returns to it by any way,
   **Then** it is where it was, and no second box is drawn for it.
4. **Given** a place the player has stood in, **When** they have not walked one of its
   ways, **Then** that way is a stub, and nothing is drawn beyond it: no box and no
   name.
5. **Given** the player types a command that does not move them, **When** the reply
   arrives, **Then** the map's places and lines are unchanged.
6. **Given** two places with the same name (a maze), **When** both have been stood in,
   **Then** they are two boxes, numbered in the order they were reached.

---

### User Story 2 - I walk by touching the map (Priority: P1)

The ways out of the place the player is in can be touched. Touching one sends its
command, exactly as touching the compass did, and the word is shown in the text as if
typed. Each way shows its state as the compass showed it. There is no compass on the
screen any more.

**Why this priority**: the compass was the app's one way of moving without typing.
Taking it away without this would be a loss.

**Independent Test**: from the opening of Zork I, reach the Kitchen by touching ways
only; each touch sends one command, shown in the transcript, and every state the
compass's tests expect at each place is the state the map shows.

**Acceptance Scenarios**:

1. **Given** the game waits for a command, **When** the player touches a way of the
   place they are in that the layer offers, **Then** its command is sent as one typed
   line, shown in the transcript, and nothing else is sent.
2. **Given** a way the layer reports as uncertain, a door open or closed, no way, or
   leading into the dark, **When** it is drawn, **Then** it can be told from an open
   way by its drawing, and its spoken label is the layer's, as the compass's was.
3. **Given** the layer reports that the ways are not known here, **When** the place is
   drawn, **Then** it is drawn as not known, which can be told from a place with no
   ways.
4. **Given** the game is asking a question, has ended or has faulted, **When** the map
   is drawn, **Then** no way can be touched, as the compass went dark.
5. **Given** a way that is neither on the compass nor up, down, in or out (the
   compass's "extras"), **When** the place is drawn, **Then** it is offered and can be
   touched.
6. **Given** anything the map lets the player do by touch, **Then** it can also be
   done by typing, and what touch sends is shown (constitution Principle III).

---

### User Story 3 - The keyboard stays up (Priority: P1)

While the player plays, the keyboard is on the screen. It does not go down after a
command and come up again. Above it are the command field, the game's reply, and the
band. The player types, reads, and sees where they are, without the screen
rearranging itself.

**Why this priority**: the owner's decision, from playing: "one of the things I don't
like about the game we have now is how the keyboard goes up and down".

**Independent Test**: type five commands in a row; the keyboard is up before, between
and after each, and the band, the reply and the field are all in view throughout.

**Acceptance Scenarios**:

1. **Given** the game waits for a line, **When** the screen is shown, **Then** the
   keyboard is up, with the band, the newest reply and the command field above it.
2. **Given** a command is sent by typing or by touching a way, **When** the reply
   arrives, **Then** the keyboard has not gone down at any moment.
3. **Given** a reply longer than the room it has, **When** it arrives, **Then** it is
   shown from its first line, with a sign that there is more, and the rest is reached
   by scrolling. The band does not change size to make room.
4. **Given** the player scrolls back through earlier replies, **When** they do,
   **Then** the keyboard goes down to give the text room, and comes back when they
   touch the command field or send a command.
5. **Given** what the player carries was shown in a strip when the keyboard was down,
   **When** the keyboard is up, **Then** what they carry can still be seen.
6. **Given** the note of darkness was shown above the compass, **When** the player is
   in the dark, **Then** the note is still shown.

---

### User Story 4 - I can open the map out (Priority: P1)

The band shows one step around the player. To see the whole page they are on, they
pull the band down. The keyboard goes down and the sheet fills the space. They can
move around it. Touching the command field, or pushing the map back up, returns to
the band with the keyboard up.

**Why this priority**: the owner chose it for this slice. A map of which only one step
can ever be seen is not yet a map.

**Independent Test**: walk twelve places; open the map out; every place and line of
the sheet can be brought into view; touch the command field; the band and keyboard
are back and the player's place is in the middle.

**Acceptance Scenarios**:

1. **Given** the band, **When** the player opens it out, **Then** the keyboard goes
   down and the sheet they are on is shown, with their place marked.
2. **Given** the sheet, **When** the player drags it, **Then** it moves, and every
   place on it can be brought into view.
3. **Given** the sheet, **When** the player touches the command field or closes the
   map, **Then** the band returns with the keyboard up, centred on their place.
4. **Given** the sheet, **When** the player touches a way of the place they are in,
   **Then** they walk it as from the band, and the sheet stays open.
5. **Given** the keyboard, **Then** it goes down for exactly two reasons: the map is
   opened out, or the player scrolls back. Never by itself.

---

### User Story 5 - Up, down, and other pages (Priority: P1)

The player climbs a tree, goes up to an attic, goes down into a cellar that turns out
to be the start of everything underground. A lone room up or down is drawn hanging
off the room it was reached from, joined by a ladder. When what is up or down turns
out to be a region of its own, it gets its own sheet, and each sheet shows where its
ways lead on the other.

**Why this priority**: every game in the corpus goes up and down, and the spike showed
that the two simple rules both fail: a new sheet for every stair gives twenty sheets
for Zork I, and counting floors puts the dam beside the house.

**Independent Test**: on Zork I's walkthrough the above-ground places with the attic
and the tree are one sheet; the cellar and what is joined to it are another; and the
whole walk gives no more than twelve sheets.

**Acceptance Scenarios**:

1. **Given** the player goes up or down to a new place, **When** it is drawn, **Then**
   it is on the sheet they came from, joined to the place they came from by a ladder
   mark that cannot be taken for a compass line.
2. **Given** a group of places reached that way grows to four, **When** the fourth is
   reached, **Then** the group moves to a sheet of its own, once, and is not moved
   again. Each end of the way between the sheets shows the direction and the name of
   the place it leads to.
3. **Given** the player moves to a new place by a command that is not a direction (a
   magic word, a boat, "pray"), **When** it is drawn, **Then** it starts a new sheet,
   and the place they left shows a short line marked with the command they typed.
4. **Given** a run of such moves, each from the place the last one reached (a river),
   **Then** the places are on one sheet.
5. **Given** the player goes in or out to a new place, **Then** it is drawn on the
   same sheet, beside the place they came from, with the way marked as in or out.
6. **Given** the player moves to a sheet other than the one shown, **When** the reply
   arrives, **Then** the band shows the sheet they are now on.

---

### User Story 6 - The map is honest about what it does not know (Priority: P2)

The player walks into a dark room. The map shows that they are somewhere, where it
is, and how they got in, and no more: no name and no ways, until they have light. In
a game the layer does not recognise there is no map, and the app says so once and
gives the screen to the text.

**Why this priority**: the owner is comfortable with honest unknowns and not with a
map that misleads or spoils. The map is a claim about the game; where the layer
cannot back the claim it must not make it.

**Independent Test**: on Zork I, walk from the Kitchen up into the Attic without the
lamp: the Attic is drawn with no name and no ways. Take the lamp, light it, go up: it
has its name and its ways, in the same position.

**Acceptance Scenarios**:

1. **Given** the player is in a place the layer reports dark, and has never stood in
   it lit, **When** it is drawn, **Then** it has no name and no stubs: only its
   position and the line they came by.
2. **Given** such a place, **When** the player stands in it lit, **Then** its name
   and stubs appear, and it has not moved.
3. **Given** a game where the layer cannot say whether a place is lit, **Then** the
   place is drawn with its name and stubs.
4. **Given** a turn at which the layer cannot say where the player is, **When** the
   map is drawn, **Then** nothing is added to it, the marker shows that the player's
   place is not known, and typing goes on.
5. **Given** a game the layer does not recognise at all, **When** it is played,
   **Then** there is no band; the text has the space; and the layer's words say once
   that there is no map for this game.
6. **Given** the same way from the same place has led to different places on
   different occasions, **When** it is drawn, **Then** each line walked is kept and
   the way is marked as not always leading to the same place.
7. **Given** the game undoes a turn or restarts itself, **When** the reply arrives,
   **Then** no line is drawn for that: the marker moves, and where the player was is
   not joined to where they now are.
8. **Given** the player walks a way the layer said led to one place and arrives at
   another (they died and were put elsewhere, or were carried), **When** the reply
   arrives, **Then** the move is drawn as a move by a word, marked with what they
   typed, and the place reached is not drawn in that direction.

---

### User Story 7 - It is drawn by hand, and it is alive (Priority: P2)

The page looks kept in pencil. When the player walks to a new place, its line and box
are drawn in as they watch, and the view glides to follow. The mark that shows where
they are is a small lamp. When they are in the dark the lamp is out.

**Why this priority**: the owner approved the sketches as "a notebook kept in pencil",
and the review's judgement was that a first slice drawn as a diagram would have the
feature judged on the wrong look. It is second only because the map must be right
before it is pretty.

**Independent Test**: walk north from the opening: the new line and box appear over a
moment, not at once; walk into the Attic unlit: the marker is shown as out.

**Acceptance Scenarios**:

1. **Given** a new place or line, **When** it is added, **Then** it is drawn in over a
   short moment, and the view moves smoothly to the player's new place.
2. **Given** a place, **When** it is drawn at any time, **Then** its outline is the
   same as every other time: the hand does not shimmer.
3. **Given** the player is in a place the layer reports dark, **Then** the marker is
   shown as out; otherwise as lit.
4. **Given** the system's setting to reduce motion, **Then** places and lines appear
   at once and the view moves without gliding.
5. **Given** the system's larger text sizes, **Then** place names remain readable and
   no way becomes too small to touch.

---

### User Story 8 - Nothing I had is lost (Priority: P1)

A listener using a screen reader could hear each way's state by moving to the
compass. They can still do so on the map. A player who closes the app and opens it
finds the map as it was, with the game.

**Why this priority**: features 007, 010 and 011 are finished and in use. This slice
removes the control two of them were built on.

**Independent Test**: with a screen reader's test hook, every way of the opening
place has the spoken label the compass gave it; play ten moves, end the app abruptly,
open it: the map is the same, each place where it was.

**Acceptance Scenarios**:

1. **Given** a screen reader, **When** the listener moves through the map, **Then**
   each way of the place they are in is an item that speaks the layer's label for it
   and can be acted on, as each place of the compass was.
2. **Given** a screen reader, **Then** the map as a whole has a spoken summary in the
   layer's words: where the player is, and that it is a map.
3. **Given** a reply arrives, **Then** it is announced exactly as before this feature
   (feature 010): the map adds nothing to what is spoken unasked.
4. **Given** a game brought back (feature 011), **When** the app opens, **Then** the
   map is the map that was left: the same places, in the same positions, on the same
   sheets.
5. **Given** the player starts over, **Then** the map is blank but for the opening
   place.

---

### Edge Cases

- **A place cannot go where its direction says**, because another is there. It goes
  further along the same direction if that is free within three steps, and otherwise
  in the nearest free position. Nothing already drawn moves.
- **Directions that contradict the page** (north, then south, arrives somewhere else;
  or east from one place and north from the next lead to the same third). The places
  stay where they are and the line is drawn bent, leaving each end in the direction
  that was walked.
- **A way that leads back to the place it leaves.** No line is drawn; the way is
  marked as returning.
- **A way the look-ahead settled as no way** (feature 009) is not drawn, as the
  compass did not offer it. If the player types it and stays put, it is then drawn as
  a refused way.
- **A door seen closed, then opened elsewhere.** The stub shows the state as last
  seen from that place, until the player stands there again.
- **A way that appears while the player is in the place** (a rug moved, a trap door
  opened). The stub appears. Nothing marks it as new in this slice.
- **A way that goes away.** The stub goes. A line already walked stays: it was walked.
- **The game moves the player while they wait or type something else** (a river). A
  line marked with the command they typed, as for any move by a word.
- **In a vehicle.** The place is the room, as the layer reports it. The vehicle is
  not drawn in this slice.
- **A very small game**, or one of a single room. The band shows what there is. The
  sheet is not drawn larger than its places need.
- **A game of scenes with no ways between them** (9:05, Photopia). Each scene reached
  by a word starts a sheet of one place. The band shows that place.
- **A long room name.** It is wrapped or shortened to fit its box; the whole name is
  spoken, and shown in the status line as today.
- **The band while a place moves to its own sheet** (User Story 5, scenario 2). The
  move is shown once, and the band then shows the sheet the player is on.
- **A game waiting for a single key.** The keyboard stays up; any key answers.
- **A hardware keyboard.** The system hides its own keyboard; the band has the room.
- **Turning the phone.** The app is upright only, as today.

## Requirements *(mandatory)*

### Functional Requirements

**What the map records**

- **FR-001**: The map MUST be made only from what the player has witnessed in the
  game in progress: the place they are in at each command prompt, the ways the layer
  reports there, and each move they made. It MUST NOT hold or show any place the
  player has not stood in, or any way of a place they have not stood in, however
  much more the layer can read.
- **FR-002**: A line MUST be drawn only for a move that happened: the place before a
  command, the command, and the place after it, when the two places differ. The
  layer's report of where a way leads MUST NOT by itself draw a line.
- **FR-003**: Two places are the same place exactly when the layer reports them as
  the same room. Places that share a name MUST be drawn apart and numbered in the
  order they were first reached.
- **FR-004**: A stub MUST carry no name of the place it leads to.
- **FR-005**: A move whose command is a direction word is drawn in that direction. A
  move by any other command is marked with the command as it was sent.
- **FR-006**: When the game undoes a turn, restarts itself or restores a position,
  the change of place MUST NOT be drawn as a line. A death cannot be told from a
  story file, and is not covered by this requirement: see FR-006a. (Corrected in
  planning, 2026-11-13; it first included a death.)
- **FR-006a**: When the player sends a direction, the layer had given that way as
  leading to a named place, and they arrive somewhere else (as on a death, a fall or
  a current), the move MUST be drawn as a move by a word, marked with the command,
  and MUST NOT put the place reached in that direction.
- **FR-007**: When a way from a place has led to more than one place, every line
  walked MUST be kept and the way marked as not always leading to the same place.
- **FR-008**: The rules of FR-001 to FR-007, and of FR-016 to FR-019 below, MUST be
  written as one table in the plan and each row tested in the shared layer.

**Where places go**

- **FR-009**: A new place reached by a compass direction MUST be put on the sheet of
  the place it was reached from: one step in that direction if that position is free;
  else two or three steps in that direction if free; else in the nearest free
  position, preferring the side the direction points to.
- **FR-010**: A place, once put, MUST NOT move, with one exception: FR-012.
- **FR-011**: No knowledge of places the player has not stood in MAY be used to
  choose a position (the owner's decision: no foresight).
- **FR-012**: A new place reached by going up or down MUST be put on the sheet of the
  place it was reached from, attached to it. When the places so attached, with what
  has been reached from them, number four, they MUST be moved together to a new
  sheet, once, keeping their positions relative to each other.
- **FR-013**: A new place reached by going in or out MUST be put on the same sheet,
  in the nearest free position to the place it was reached from.
- **FR-014**: A new place reached by any other command MUST start a new sheet, unless
  the place it was reached from is itself on a sheet started that way and reached by
  such a command, in which case it continues that sheet.
- **FR-015**: Positions, sheets, and which line is straight or bent MUST be decided
  by the shared layer, so that every platform draws the same page. For one game and
  one sequence of inputs they MUST always be the same.

**What is witnessed of light and of states**

- **FR-016**: A place's name and stubs MUST be shown only once the player has stood
  in it while the layer did not report it dark. Until then it is drawn as a place
  with no name and no stubs.
- **FR-017**: Each stub MUST show the state the layer gave that way when the player
  last stood in the place: open, uncertain, a door open, a door closed, not known.
  The mark for a way that leads into the dark is kept.
- **FR-018**: A way the layer does not offer (settled as no way, feature 009) MUST
  NOT be drawn, until the player has typed it there and stayed put; it is then drawn
  as refused.
- **FR-019**: Where the layer reports the ways of a place as not known, the place
  MUST be drawn as not known, distinct from a place with no ways.

**The band, the sheet and the keyboard**

- **FR-020**: While the game waits for a line the keyboard MUST be up, and MUST NOT
  go down except when the player opens the map out or scrolls back through replies.
- **FR-021**: The band MUST show the place the player is in, with every one of its
  ways wholly in view, and whatever else of the sheet falls within the band.
- **FR-022**: The band MUST keep one size while the keyboard is up. The reply MUST be
  shown from its first line, with a sign when there is more than fits.
- **FR-023**: The player MUST be able to open the band out to the sheet they are on,
  move around it, and return. On return the band is centred on their place.
- **FR-024**: What the player carries and the note of darkness MUST be visible with
  the keyboard up.
- **FR-025**: The compass MUST be removed from the screen. Every state it showed and
  every command it sent MUST be available from the map.
- **FR-026**: Touching a way of the place the player is in MUST send exactly that
  way's command, once, shown in the transcript. A second touch before the reply MUST
  NOT send a second command. No touch on the map sends anything else in this slice.

**How it looks**

- **FR-027**: Every place MUST be drawn as a box. Outdoor places are not drawn
  differently in this slice.
- **FR-028**: A way up or down MUST be drawn with a mark that cannot be taken for a
  compass line, and MUST NOT be placed where a compass way of the same place is
  drawn.
- **FR-029**: A new place and its line MUST be drawn in over a short moment, and the
  view MUST move smoothly to the player's place; with reduced motion, both at once.
- **FR-030**: Each place's outline MUST be the same each time it is drawn.
- **FR-031**: The marker of the player's place MUST show lit or out, by what the
  layer reports of the dark.

**Screen readers**

- **FR-032**: Each way of the place the player is in MUST be an item a screen reader
  can move to and act on, speaking the layer's label for it. The labels MUST be those
  the compass spoke, less the name of a place the player has not yet walked to
  (FR-004). (Corrected in planning: the compass's labels name where a way leads.)
- **FR-033**: The map MUST offer a spoken summary in the layer's words. The app
  composes no text.
- **FR-034**: What is announced unasked when a reply arrives MUST be what it was
  before this feature.

**Where the work lies**

- **FR-035**: The record, the positions and every word the map shows or speaks MUST
  come from the shared layer. The app draws at the positions it is given and sends
  back touches (constitution Principle V).
- **FR-036**: The map MUST be made by reading the game, never by changing it, and
  MUST send the game nothing (Principles III and IV).
- **FR-037**: When a game is brought back (feature 011) the map MUST be the map that
  was left. When the player starts over the map MUST begin again.
- **FR-038**: Where the layer does not recognise the game, there MUST be no band; the
  layer's words MUST say once that there is no map for this game.

**Limits**

- **FR-039**: The limits earlier features set for a command and its snapshot, and for
  opening the app on a kept game of 2,000 turns, MUST still hold with the map added.
  What the map adds to each MUST be measured and recorded.
- **FR-040**: No sheet MAY hold two places in one position.

**Proof**

- **FR-041**: There MUST be tests of each row of the table of FR-008 on stories built
  for the purpose.
- **FR-042**: There MUST be tests on Zork I and on Adventure, each played along its
  walkthrough, that fix the whole map at the end (each place's sheet and position,
  each line and whether it is bent), so that a change to the rules shows as a change
  to a recorded page.
- **FR-043**: There MUST be a test on each of those games played in a second order of
  exploring that differs from its walkthrough: the map obeys FR-040 and the measures
  of SC-003 are recorded, without a fixed bar.
- **FR-044**: Every story in the corpus the layer recognises MUST be walked by the
  corpus's own crawl with the map recording, with no fault, and FR-040 holding.
- **FR-045**: There MUST be tests in the app of User Stories 2, 3, 4 and 8 as far as a
  test can see them, and every test of the earlier features MUST pass. Each test that
  was of the compass changes to be of the map, and each such change MUST be listed.
- **FR-046**: Recorded pages and any picture of a map hold the names of a game's
  rooms. They MUST be kept only for stories the repository may hold, and nowhere for
  any other (Principle VI). The plan MUST say where.

### Key Entities

- **Place**: a room the player has stood in: its identity, its name once seen lit,
  its sheet and position, and whether it has been seen lit.
- **Way**: of a place, as last seen from it: its direction or command, its state, and
  whether it leads into the dark.
- **Line**: a move that happened: from, to, the direction or command each way it was
  walked, and whether it is straight or bent.
- **Sheet**: a page: its places, and the ways that lead off it to other sheets.
- **Marker**: where the player is, or that it is not known; lit or out.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can go from the opening of Zork I into the Kitchen, up to the
  Attic and down again by touching the map alone, and the keyboard never goes down.
- **SC-002**: Along the whole of Zork I's walkthrough and the whole of Adventure's,
  the map at every turn holds exactly the places stood in so far and the lines
  walked, and no place has moved except by FR-012.
- **SC-003**: At the end of each walkthrough, of the compass lines joining two places
  on one sheet, at least four in five are straight (the spike gave 49 of 60 for Zork
  I and 44 of 51 for Adventure), and no two straight lines cross.
- **SC-004**: Zork I's walkthrough gives at most twelve sheets and Adventure's at most
  eight (the spike gave eleven and seven), and no sheet holds more than thirty
  places.
- **SC-005**: On Zork I, a place entered only in the dark shows no name and no ways,
  at every turn until it is stood in lit.
- **SC-006**: Every state the compass's tests expected at each place they visit is
  the state the map shows there, and every spoken label is the same but for the name
  of a place not yet walked to.
- **SC-007**: With the map added, every command of Zork I's walkthrough is still
  answered within the earlier limit, and a kept game of 2,000 turns is still brought
  back within its limit.
- **SC-008**: A game closed and brought back at ten points along a walkthrough shows
  the same map each time as the game never closed.
- **SC-009**: The owner plays Zork I on their phone for twenty rooms and says that
  the page reads as the places they walked; that the keyboard stayed put; and what,
  if anything, they would have drawn differently by hand. What they say is recorded.

## Assumptions

**Decided by the owner on 2026-11-13**

- The map and the scene are one screen at different depths.
- The compass stops being a separate control; its ways are drawn on the map.
- Every place is a box; an outdoor look comes later, through game packs.
- The map shows only what is witnessed.
- Maze rooms are drawn apart and numbered.
- An untaken way shows no name of where it leads.
- The keyboard is up by default and goes down only when the map is opened out or the
  player scrolls back.
- No foresight in placing rooms.
- Up and down by the "attached" rule.
- This slice has the band and the opened-out sheet, and no atlas.
- In this slice the map is of the game in progress only. Their decision that the
  notebook survives starting over and death stands, for a later slice.

**Settled by me, for the owner to confirm or change**

- **The mark for a way into the dark is kept** (FR-017). It is something the layer
  knows of a place the player has not stood in, so it bends "witnessed only". I kept
  it because it is a warning the app already gives, and in these games the dark
  kills.
- **A way typed and refused is drawn as refused** (FR-018). The compass hid such
  ways; once the player has tried one, it is witnessed.
- **Where light is not known, names and ways are shown** (User Story 6, scenario 3).
  The alternative hides every room of Zork II and III.
- **The status line stays where it is**, above the band. The review suggested
  pencilling place, score and moves into the band's edge; that is a change of look
  the owner has not seen.
- **A group of four moves to its own sheet** (FR-012). The spike tried three and under
  staying attached; four is that rule stated from the other side.
- **Nothing marks a way as newly appeared, and a line walked one way only has no
  arrowhead**, in this slice.
- **The measures of SC-003 and SC-004** are set a little inside what the spike gave.
  The spike drew a finished walk; the app draws as the player goes, and FR-012 moves
  a group once, so the figures may differ slightly.

**Not known**

- Whether the band is tall enough to be worth looking at with the keyboard up, on the
  owner's phone. The sketch suggests about a third of the screen. Only SC-009 tells.
- Whether a pull to open the map out and a drag to move it can share the band without
  mistakes. The research found almost nothing on this.
- How the map reads when a player wanders, as against following a walkthrough.
  FR-043 is there to find out.
- What a screen reader's user needs of the map beyond the ways of the place they are
  in. This slice gives the compass's worth and no more.

**Out of scope**

The atlas of all sheets; travelling to a far place by touching it; what lies in a
place, and the room opened up; things left behind; the player's own notes; the
loose-ends and journey views; moving a place by hand; a map that survives starting
over; region names and an outdoor look (which need the constitution amended for
authored content); line weight for certainty and the eraser for change; games the
layer does not recognise (they get no map); Android.
