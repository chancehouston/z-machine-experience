# Feature Specification: Screen Reader Announcements

**Feature Branch**: `010-screen-reader-announcements`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "Screen reader announcements." Chosen by the project owner
on 2026-11-13 as the next feature, from four put to them. Decisions made by the owner
the same day: when a turn arrives the app speaks, unasked, the game's whole reply and
then the note of darkness when that note appears; and a new turn's announcement
replaces one still being spoken.

## User Scenarios & Testing *(mandatory)*

A text adventure is its text. A sighted player gives a command and the game's reply is
in front of them at once. A player using a screen reader gives the same command and
hears nothing: the app labels every control, and says what each is when the listener
moves to it, but it announces nothing of its own accord. The reply is on the screen,
further up than where the listener is, and they must go and find it, every turn.

This was seen on 2026-11-13, playing feature 008 by hand: the note "It is dark" appears
above the compass when the player walks into the dark, and a listener learns of it only
by moving to it. A sighted player cannot miss it.

Three words are used throughout:

- **Listener**: a player using the phone's screen reader.
- **Announcement**: what the app asks the screen reader to speak without the listener
  moving to anything.
- **Reply**: what the game printed in answer to one command, as the transcript shows
  it, without the prompt character the game ends it with.

### User Story 1 - The game's reply is spoken when it arrives (Priority: P1)

A listener types a command, or touches a way on the compass. When the game answers, the
screen reader speaks the whole reply. The listener has not moved, and where they were
on the screen is where they still are: they can give the next command at once.

**Why this priority**: this is the feature. Without it the game cannot be played by ear
at anything like the pace it can by eye.

**Independent Test**: with a screen reader running, start the game and give three
commands, one typed and two by touching ways; after each, what the app asked to be
spoken is that turn's reply.

**Acceptance Scenarios**:

1. **Given** a screen reader running and the game waiting for a command, **When** the
   listener sends a command by typing it, **Then** the app announces the reply, whole
   and in the game's own words.
2. **Given** the same, **When** the listener touches a way on the compass or beside it,
   **Then** the app announces the reply in the same way.
3. **Given** the app has just been launched, or a new game has just been started,
   **When** the opening text arrives, **Then** it is announced as any reply is.
4. **Given** a reply has been announced, **When** the listener looks for where they
   were, **Then** they are where they were: the announcement moved nothing.
5. **Given** a reply that ends with the game's prompt character, **When** it is
   announced, **Then** the prompt character is not spoken.
6. **Given** no screen reader running, **When** a turn arrives, **Then** nothing is
   asked to be spoken, and the app is as it was before this feature.

---

### User Story 2 - Darkness is said when it falls (Priority: P1)

A listener walks into a dark room. After the game's reply, the screen reader says "It
is dark", the same words a sighted player sees appear above the compass.

**Why this priority**: the note is a warning, and in these games the dark kills. It is
the case that showed the need.

**Independent Test**: on Zork I with its pack, walk from the kitchen up into the attic
without the lamp; the announcement is the reply followed by the note. Walk down again;
the announcement is the reply alone.

**Acceptance Scenarios**:

1. **Given** the player was not in the dark, **When** a turn leaves them in the dark,
   **Then** the announcement is the reply followed by the layer's note of darkness.
2. **Given** the player was in the dark, **When** a turn leaves them still in the dark,
   **Then** the announcement is the reply alone: the note is not said again.
3. **Given** the player was in the dark, **When** a turn leaves them in the light,
   **Then** the announcement is the reply alone: nothing is said of the note's going.
4. **Given** light is unknown, **When** a turn arrives, **Then** nothing is said of
   light.
5. **Given** the note has been announced, **When** the listener moves to the note on
   the screen, **Then** it is read as before this feature.

---

### User Story 3 - The newest reply is the one heard (Priority: P1)

A listener who knows the way sends three commands quickly. They do not hear three
replies in a queue, each finishing before the next begins. Each new reply cuts off the
one before, and what they hear to its end is where they are now.

**Why this priority**: the owner's choice. A backlog of speech puts the listener
several turns behind the game, which is worse than missing the middle of a reply that
is still in the transcript.

**Independent Test**: send a second command while the first reply would still be being
spoken; the app asks for the second reply in a way that replaces the first.

**Acceptance Scenarios**:

1. **Given** an announcement is being spoken, **When** a new turn arrives, **Then** the
   new announcement replaces it, and the earlier one is not finished.
2. **Given** an announcement was cut off, **When** the listener goes to the transcript,
   **Then** the earlier reply is there in full, as before this feature.
3. **Given** an announcement is being spoken, **When** the listener moves to a control,
   **Then** the screen reader behaves as it does for any announcement: that is the
   system's to decide, and the app does not fight it.

---

### User Story 4 - A question, an ending and a failure are heard (Priority: P2)

A listener is asked a question by the game, or the game ends, or it stops with a fault.
In each case they hear what a sighted player would read.

**Why this priority**: these are rarer than a command's reply, and each already has its
words on the screen; what is missing is only that they are not spoken.

**Independent Test**: on stories built for the purpose: one that asks a question, one
that ends, one that faults; each announcement is what the screen shows.

**Acceptance Scenarios**:

1. **Given** a turn that leaves the game asking a question, or waiting for a key,
   **When** it arrives, **Then** the reply, which holds the question, is announced.
2. **Given** a turn that ends the game, **When** it arrives, **Then** the reply is
   announced, followed by the words the screen shows for a game that has ended.
3. **Given** the game stops with a fault, **When** that is shown, **Then** the message
   the screen shows is announced.

---

### Edge Cases

- **A reply with nothing in it but the prompt.** Nothing is announced for it.
- **A very long reply.** It is announced whole. The listener can stop it as they stop
  any speech, and the next turn replaces it.
- **A screen reader turned on in the middle of a game.** Nothing is announced until the
  next turn arrives; the turns already shown are in the transcript.
- **A screen reader turned off in the middle of a game.** No further announcement is
  asked for.
- **The About sheet is open when a turn's announcement would be made.** No turn can
  arrive then, since nothing can be sent from the sheet.
- **A command the layer refuses** (it was made against an earlier turn). No turn
  arrives, and nothing is announced.
- **Several lines typed ahead.** They are sent one at a time, as today; each reply is
  announced as it arrives and replaces the one before.
- **The player's own command.** It is not announced: a typed command was the
  listener's own, and a touched way was spoken by its label when they moved to it.
- **A reply holding characters a screen reader reads oddly.** The game's text is
  passed as it is; nothing is rewritten for speech.
- **Other ways the phone speaks or shows text for a listener**, such as a braille
  display. They receive the same announcement, as the system provides.

## Requirements *(mandatory)*

### Functional Requirements

**What is announced**

- **FR-001**: When a turn arrives and a screen reader is running, the app MUST ask the
  screen reader to speak that turn's reply, whole, in the game's own words and in the
  order the game printed them.
- **FR-002**: The opening text, at launch and after a new game, MUST be announced as a
  reply is.
- **FR-003**: The prompt character that ends a turn's text MUST NOT be part of what is
  announced. A turn with no other text is not announced.
- **FR-004**: When a turn leaves the player in the dark and the turn before did not,
  the layer's note of darkness MUST be announced after the reply. It MUST NOT be
  announced on a later turn while the player stays in the dark, nor when the dark
  ends, nor where light is unknown.
- **FR-005**: When a turn ends the game, the words the screen shows for an ended game
  MUST be announced after the reply. When the game stops with a fault, the message
  shown MUST be announced.
- **FR-006**: The ways out, what is carried, the place, the score and the count of
  moves MUST NOT be announced. They keep the labels they have.
- **FR-007**: The player's command MUST NOT be announced.

**How it is announced**

- **FR-008**: An announcement MUST replace an earlier announcement of this app that is
  still being spoken.
- **FR-009**: An announcement MUST NOT move the screen reader from where the listener
  has put it, and MUST NOT take the focus from the command field.
- **FR-010**: With no screen reader running, the app MUST ask for nothing to be spoken.
- **FR-011**: Nothing about what a screen reader says when the listener moves to a
  control, a turn of the transcript or the note MAY change.

**Whose words they are**

- **FR-012**: The app MUST compose no text for an announcement. Each announcement is
  given whole by the shared layer, made of the game's text and the layer's own words,
  so that a second platform would speak the same.
- **FR-013**: Nothing is sent to the game, and no transcript changes, because of an
  announcement (constitution Principle III).
- **FR-014**: The game's text in an announcement MUST be exactly the text shown in the
  transcript for that turn, save the prompt character.

**Proof**

- **FR-015**: There MUST be tests of what the layer gives to be spoken, on stories
  built for the purpose and on Zork I: for a plain turn, the opening, a turn into the
  dark, a turn that stays in it, a turn out of it, a question, an ending and a fault.
- **FR-016**: There MUST be a test in the app that what it asks to be spoken at a turn
  is what the layer gave, and that a second turn's request replaces the first. A test
  cannot hear; this is the nearest it can come.
- **FR-017**: The feature is not done until the owner has listened to it on a phone
  with the screen reader running, through the walk of User Story 2, and said what they
  heard. The same sitting closes the listening that features 008 and 009 left open.

**Limits**

- **FR-018**: The limits the earlier features set for a command and its snapshot, for a
  prompt's queries and for launch MUST still hold.
- **FR-019**: Every test of the earlier features MUST pass; a test that fixes a written
  form changes only where this feature changes that form, and each such change MUST be
  listed.

### Key Entities

- **Reply**: one turn's text, without the prompt character that ends it.
- **Announcement**: what is to be spoken for one turn: the reply, then at most one of
  the layer's notes (darkness, on the turn it falls; the game's ending; a fault's
  message), or nothing.
- **Fell dark**: a turn that leaves the player in the dark when the turn before did
  not. The first turn of a game counts if it is dark.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A listener can play from the opening of Zork I into the kitchen, up into
  the attic and down again without once moving the screen reader to the transcript,
  hearing every reply.
- **SC-002**: On that walk the listener hears "It is dark" once, on arriving in the
  attic, and not again while they stay there.
- **SC-003**: When three commands are sent faster than their replies can be spoken, the
  reply heard to its end is the third.
- **SC-004**: After any number of announcements the listener's place on the screen is
  where they left it, and typing still goes to the command field.
- **SC-005**: With no screen reader running, no announcement is asked for across a
  whole walkthrough, and every earlier test passes unchanged in its values.
- **SC-006**: What is announced for each of the 331 turns of Zork I's walkthrough is
  that turn's text as the transcript holds it, without the prompt character.
- **SC-007**: The owner, listening on a phone, confirms the walk of SC-001 and SC-002,
  and records what was heard.

## Assumptions

- **The note of darkness is said once, when it falls.** The owner's words were "when
  that note appears". Said on every turn in the dark it would be heard more than the
  game.
- **Nothing is said when the dark ends.** The game's own reply describes the room, and
  that is how a listener knows.
- **The player's command is not spoken.** A way's label was heard when the listener
  moved to it; the reply follows.
- **The reply is spoken whole.** No shortening, and no setting for it; the owner put
  both out of scope.
- **The layer gives the announcement.** The earlier features hold the app to composing
  no text, and Android is to speak the same. How the layer gives it is for the plan.
- **The words for an ended game are the layer's.** The screen shows them today from
  the app's own source; the plan decides whether that moves to the layer with this
  feature, since FR-012 asks for it.
- **The simulator cannot be listened to.** The app's tests check what it asks the
  system to speak. Hearing it is the owner's, on a device.
- **Out of scope**: Android; haptics and sounds; a setting to turn announcements off or
  choose their length; speaking the ways out or what is carried; changes to the labels
  the controls already have; the map; the graphical interface.
