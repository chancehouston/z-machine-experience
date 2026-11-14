# Feature Specification: Touch the Room to Look

**Feature Branch**: `013-room-look`

**Created**: 2026-11-14

**Status**: Draft

**Input**: User description: "Room tap for the full description (feature 013). Asked for by the project owner on 2026-11-14 at the phone trial of the map's stage 3: 'an easy way to get a full description', by touching the room on the map."

## Decisions the owner has made (2026-11-14)

1. A touch on the place the player is standing in sends the game its own command for
   looking, as a tapped line shown in the transcript like a tapped way. It spends the
   turn the game charges for it ("Send look, spend the turn"). It is not a peek on a
   copy of the game.
2. A touch on any other place of the map does nothing, for now ("Nothing, for now").

## Strongest objection

**The game decides what "look" costs and says, and the map cannot promise more.** In
Zork I a look is a move: the lamp burns and the thief walks. The owner knows this and
chose it. In the dark the game answers a look with its words for darkness, not with a
description. And a few stories may not have the word at all; there the place cannot
be touched, and the player sees no reason why. The feature is honest about all three
by doing nothing clever: it sends one visible command and shows the game's reply.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - I touch my room and the game describes it (Priority: P1)

The player has walked through a room several times and the game now names it in one
line. They touch the room's box on the map. The command for looking appears in the
transcript as a tapped command, and the game's full description follows it.

**Why this priority**: it is the whole of what was asked for.

**Independent Test**: start Zork I, walk north and back south, touch the place the
player is in, and read the transcript: the look command, then the full description of
West of House.

**Acceptance Scenarios**:

1. **Given** the game is waiting for a command and the player's place is known,
   **When** they touch that place on the map, **Then** the game's own command for
   looking is sent, it is shown in the transcript as a tapped command, and the
   game's reply follows as for any command.
2. **Given** the same, **When** the reply arrives, **Then** the count of moves is
   whatever the game made it: the map adds and hides nothing.
3. **Given** the map is opened out to the whole sheet, **When** the player touches
   their place, **Then** the same command is sent and the sheet stays open.
4. **Given** the player is in the dark, **When** they touch their place, **Then** the
   command is sent and the game's reply about the dark is shown.
5. **Given** a game that was left after such a touch, **When** it is brought back,
   **Then** it is the same game: the touch was kept as the tap it was.

---

### User Story 2 - It cannot be touched when it would do nothing (Priority: P1)

The player's place is offered for touching only when a touch would send the command.

**Why this priority**: a control that does nothing, or sends a word the game does not
know, breaks the rule that the map guesses nothing.

**Independent Test**: at a question ("Do you wish to restart?"), after the game has
ended, and in a story with no word for looking, the player's place is not offered.

**Acceptance Scenarios**:

1. **Given** the game is asking a question or waiting for a key, or has ended,
   **Then** the player's place cannot be touched.
2. **Given** a command is being worked out, **Then** a touch on the place sends
   nothing.
3. **Given** the layer cannot say where the player is, **Then** there is no place of
   theirs to touch.
4. **Given** a story whose dictionary has no word for looking, **Then** the place
   cannot be touched, in any turn.
5. **Given** any other place on the map, **When** it is touched, **Then** nothing is
   sent.

---

### User Story 3 - The ways, the lamp and the drag still work (Priority: P1)

The place the player is in sits in the middle of the ways' controls and under the
lamp. Making it touchable must not take touches from them, or from the drag that
moves the opened sheet and pulls the band down.

**Why this priority**: the ways are how the game is played by touch; losing one to a
new control would be a fault in what already works.

**Independent Test**: every existing test of the ways, of opening out, and of
dragging passes unchanged.

**Acceptance Scenarios**:

1. **Given** a place with ways on every side, **When** each way's control is
   touched, **Then** that way is walked and no look is sent.
2. **Given** the opened sheet, **When** a drag begins on the player's place, **Then**
   the sheet moves and no look is sent.
3. **Given** the band, **When** it is pulled down from the player's place, **Then**
   the sheet opens and no look is sent.

---

### User Story 4 - A listener knows the room can be touched (Priority: P2)

With a screen reader, the player's place is a control. Its name is spoken as before;
what a touch on it does is said in the layer's words.

**Why this priority**: the feature must not be for sighted players only; but the
typed command is always there for a listener.

**Independent Test**: the element for the player's place is a button when it can be
touched and is not one when it cannot; the words that say what it does are the
layer's.

**Acceptance Scenarios**:

1. **Given** the place can be touched, **Then** it is a button whose label is the
   place's name, and the words for what it does are given by the layer.
2. **Given** it cannot be touched, **Then** it is not a button, and its label is the
   place's name as before.

### Edge Cases

- The story has a short and a long word for looking: the layer sends the longer,
  so that the transcript reads as a player would have typed it in full.
- The story's word is cut short by its dictionary (six letters in the oldest
  stories): the word is found as the story keeps it.
- The game rebinds or refuses the word in some scene: the game's reply says so; the
  map does not know and does not need to.
- Two quick touches: the second is refused while the first is being worked out, as
  for a way.
- The place has been seen only in the dark and has no name: it can still be touched.
- Largest text size: the place's box does not grow; its touch area is at least 44 points tall.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The map MUST say, at every stop, whether the place the player is in can
  be touched to look, and give the command that a touch sends.
- **FR-002**: The command MUST be a word of the story's own dictionary. The layer
  looks for the words for looking in a fixed order, longest first, and uses the
  first the dictionary has. Where it has none, the place cannot be touched.
- **FR-003**: The place can be touched only while the game waits for a command and
  the player's place is known.
- **FR-004**: A touch MUST be sent to the layer as its own entry, checked against the
  turn it was offered at as a touch on a way is, and refused when it is not offered.
- **FR-005**: The layer MUST send the command as a tapped line: shown in the
  transcript as tapped commands are, and kept with the game as a tap.
- **FR-006**: A game brought back after such touches MUST be the same game, with the
  same map.
- **FR-007**: The map itself MUST be unchanged by a look: no place, line or mark is
  added by it, since the player has not moved.
- **FR-008**: The layer MUST give the words a screen reader uses to say what a touch
  on the player's place does. The place's spoken name is unchanged.
- **FR-009**: The app MUST hold no command word and compose no text for this. It
  draws the place as a control when the map says it can be touched, and sends the
  touch.
- **FR-010**: The control MUST be the place's box in the band and on the opened
  sheet, with a touch area at least 44 points tall and as wide as the box allows without
  overlapping any way's control. The lamp is part of the place.
- **FR-011**: A drag that begins on the place MUST move the opened sheet, or pull the
  band down, and MUST NOT send a look.
- **FR-012**: A touch on any place other than the player's MUST send nothing.
- **FR-013**: The limits of earlier features hold: a command and its snapshot within
  10 ms; a kept game of 2,000 turns brought back within the layer's second.
- **FR-014**: The snapshot's other keys, and the map's other parts, are unchanged.

### Key Entities

- **The look**: part of the map as a snapshot gives it. Present when the player's
  place can be touched now: the command a touch sends, and the words to speak.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On Zork I and on Adventure, from a place walked into twice, one touch
  on the player's place brings the game's full description of it into the
  transcript.
- **SC-002**: Across every recognised story of the corpus, the layer either finds a
  word for looking in the story's dictionary or offers no touch; it never sends a
  word the dictionary lacks. The count of stories with and without is recorded.
- **SC-003**: A kept game holding touches on the place comes back identical, in 100
  of 100 trials of one fixed game.
- **SC-004**: Every test of the ways, of opening out and of dragging that passed
  before this feature passes after it, unchanged.
- **SC-005**: The owner, on their phone, gets a full description by touching the
  room, and says in their own words whether it is the easy way they asked for.

## Assumptions

- The words for looking are "look" and "l", as in every story the project has met.
  A story in another language, or with another word, gets no touch; that is honest
  and is not solved here.
- The game's own setting for long and short descriptions is left alone. A look
  gives the full description whatever that setting is.
- A look that the game answers with a question (none is known) is handled as any
  command is.
- This feature is built on the map's stage 3 (pull request 21) and is merged after
  it.

## Out of Scope

Touching other places; walking to a place; peeking without spending a turn; a card
over the map; inventory items; the game's verbose and brief settings.
