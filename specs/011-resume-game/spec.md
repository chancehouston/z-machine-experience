# Feature Specification: Resume the Game

**Feature Branch**: `011-resume-game`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "Resume: the game comes back where the player left it."
Chosen by the project owner on 2026-11-13 as the feature to build before the map. A
review of the map's design the same day found that a map kept "one notebook per game"
is worth nothing if the game starts afresh each time the app opens. Decisions made by
the owner the same day: the app keeps the player's place by itself, and the game's own
SAVE and RESTORE commands stay as they are; and on opening the app with a game in
progress the player is carried straight on, with no question asked.

## User Scenarios & Testing *(mandatory)*

Today every launch of the app is a new game. Feature 004 said so on purpose
(its FR-005: "Each launch of the app MUST start a new game"), because nothing then
needed more. A phone is put down and picked up many times a day, and the system closes
apps it has not seen for a while. A player who has spent an evening reaching the
cellar, answers a call, and comes back to "West of House" has lost the evening.

This feature replaces FR-005 of feature 004. It adds nothing a player can see while
they play; what it changes is what they find when they come back.

Four words are used throughout:

- **Kept game**: what the device holds of a game in progress, so that it can be brought
  back.
- **Bring back**: to make the game, and everything the screen shows of it, what it was
  when it was kept.
- **Answered turn**: a turn whose reply has been shown to the player.
- **Start over**: to begin a new game on purpose, giving up the kept one.

### User Story 1 - The game is where I left it (Priority: P1)

A player plays for a while and leaves: they switch to another app, lock the phone, or
the app is closed, by them or by the system. Later they open the app. They are where
they were. The last reply is on the screen, the earlier turns are above it, the ways
out and what they carry are as they were, and they can type at once. Nothing asks
them whether to continue.

**Why this priority**: this is the feature.

**Independent Test**: play five commands, close the app completely, open it; the
screen shows the same five turns and the same ways out, and a sixth command is
answered as it would have been had the app stayed open.

**Acceptance Scenarios**:

1. **Given** a game with answered turns, **When** the app is closed and opened again,
   **Then** every answered turn is in the transcript, in order and in the same words,
   and the newest is in view.
2. **Given** the same, **When** the app has opened, **Then** the place, the score and
   moves where the game shows them, the ways out, what is carried and the note of
   darkness are what they were when the app closed.
3. **Given** the same, **When** the player sends a command, **Then** the reply is the
   reply the game would have given had the app never closed.
4. **Given** a game in progress, **When** the app opens, **Then** no question is put to
   the player and no choice is needed before they can type.
5. **Given** a device that has never played the story, **When** the app opens,
   **Then** a new game starts, as today.

---

### User Story 2 - Nothing I saw answered is lost (Priority: P1)

A player is in the middle of a game when the app dies without warning: the system
ends it, the phone runs out of charge, or it crashes. When they open it again they
have lost nothing they saw answered.

**Why this priority**: a place kept only when the app closes politely is lost exactly
when it matters. Phones do not close apps politely.

**Independent Test**: play five commands and end the app's process abruptly, without
letting it close; open it; all five turns are there.

**Acceptance Scenarios**:

1. **Given** an answered turn, **When** the app's process is ended abruptly at any
   moment after the next command could be typed, **Then** on opening that turn is
   there.
2. **Given** a command that was sent but whose reply had not been shown, **When** the
   process is ended, **Then** on opening the game is as it was before that command, or
   as it was after it with the reply shown: never part of each.
3. **Given** the app is ended while it is keeping a turn, **When** it is opened,
   **Then** the game brought back is the one kept before, whole.

---

### User Story 3 - The game brought back is the game I left (Priority: P1)

The game brought back is not a likeness of the one left. It is the same game: the
thief is where he was, the lamp has burned as long as it had, and where the game rolls
dice it rolls what it would have rolled. A player could not tell, from anything the
game does afterwards, that the app was ever closed.

**Why this priority**: the constitution's first principle is that the game behaves
exactly as its author wrote it. A game that is subtly different after a relaunch
breaks that in a way no player could find and no test of one launch would show.

**Independent Test**: play a walkthrough twice with the same chance, once without
stopping and once closing and bringing back at many points; every reply and every
snapshot of the two is the same.

**Acceptance Scenarios**:

1. **Given** two sessions of one story with the same chance and the same commands,
   one never closed and one brought back after any turn, **When** both go on with the
   same commands, **Then** every later reply is the same in both.
2. **Given** the same, **Then** every later snapshot (the ways out with their states,
   what is carried, the light, the status) is the same in both.
3. **Given** a game brought back more than once, **Then** the same holds.

---

### User Story 4 - I can start over when I choose (Priority: P1)

Until now a player who wanted a fresh game closed the app and opened it. That no
longer works: the game comes back. So a way to start over must be within reach while a
game is in progress, and because it now gives up something that would otherwise be
kept, it asks first.

**Why this priority**: without it this feature traps the player in their first game.

**Independent Test**: in the middle of a game, choose to start over and confirm; the
opening text is shown and the transcript holds nothing else; close and open the app;
it is the new game that comes back.

**Acceptance Scenarios**:

1. **Given** a game in progress, **When** the player looks for a way to start over,
   **Then** there is one they can reach without ending the game first.
2. **Given** the player chooses it, **When** they are asked to confirm and decline,
   **Then** nothing has changed.
3. **Given** they confirm, **Then** a new game starts, the earlier one is no longer
   kept, and it cannot be brought back.
4. **Given** a game that has ended, **When** the player chooses New game as today,
   **Then** it starts without asking, as today: nothing of worth is given up.

---

### User Story 5 - A kept game that cannot be brought back (Priority: P2)

Rarely, what was kept cannot be used: an update to the app carried a different story
file, or what was kept is damaged. The player is given a new game and told once, in
plain words, that the earlier one could not be brought back. The app does not crash,
and it never shows a game that is quietly not the one they left.

**Why this priority**: it should be rare, but the two ways it can go wrong (a crash at
every launch, or a wrong game shown as the right one) are the worst outcomes this
feature can have.

**Independent Test**: keep a game; replace what was kept with damaged data, and
separately with a game kept for a different story file; open the app each time; a new
game starts and the message is shown once.

**Acceptance Scenarios**:

1. **Given** what was kept is damaged or cut short, **When** the app opens, **Then** a
   new game starts and the player is told the earlier game could not be brought back.
2. **Given** what was kept belongs to a story file that is not the one the app now
   holds, **Then** the same.
3. **Given** what was kept was written in a form this version of the app does not
   read, **Then** the same.
4. **Given** the message has been shown, **When** the app is opened again, **Then** it
   is not shown again, and the new game is brought back like any other.
5. **Given** any of these, **Then** the app does not crash and does not hang.

---

### Edge Cases

- **The game had ended when the app closed.** It comes back ended: the transcript, the
  words for an ended game, and New game. It does not start a new game by itself.
- **The game had stopped with a fault.** It comes back as it was after the last
  answered turn before the fault, ready for a command. The fault is not kept.
- **The player typed the game's own RESTART and the game restarted.** What is kept
  follows the game: it is the restarted game that comes back.
- **The player typed QUIT and the game ended.** As for any ended game.
- **The game was waiting for a key, or for the answer to a question.** It comes back
  waiting for the same thing.
- **The app is only sent to the background and returns without being closed.** Nothing
  is brought back, since nothing was lost; the screen is as it was.
- **A command was typed but not sent.** The text in the command field is not kept.
- **The game's own SAVE and RESTORE.** They are answered as today: the game is told
  they failed.
- **A story's game pack changed with an update, and the story file did not.** The game
  is brought back. The ways out and the light are worked out with the pack the app now
  holds.
- **The device is full and the game cannot be kept.** Play goes on. The player is told
  once that their place is not being kept, and keeping is tried again at each turn.
- **A very long game.** It is brought back within the same limit as a short one
  (FR-020).
- **Two stories.** Each has its own kept game; bringing one back does not disturb the
  other. (The app offers one story today; this is so that a second needs no change.)
- **A screen reader is running when the app opens on a kept game.** The newest turn is
  announced once, as the opening text is at a first launch, so the listener knows
  where they are. No earlier turn is announced.
- **The app was removed and installed again.** Nothing is kept across that; a new game
  starts, with no message.

## Requirements *(mandatory)*

### Functional Requirements

**What comes back**

- **FR-001**: When the app opens and holds a kept game for its story, it MUST bring
  that game back and MUST NOT start a new one. This replaces FR-005 of feature 004.
- **FR-002**: A game brought back MUST show every answered turn of that game, in
  order, each with the command and the reply as they were first shown.
- **FR-003**: A game brought back MUST show the status, the ways out with their
  states, what is carried and the note of darkness as they were after the newest
  answered turn.
- **FR-004**: A game brought back MUST be waiting for what it was waiting for: a
  command, a key, an answer, or nothing because it had ended.
- **FR-005**: No question, sheet or choice MAY stand between opening the app and
  typing a command in a game brought back.
- **FR-006**: With nothing kept for the story, the app MUST start a new game, as today.

**The same game**

- **FR-007**: A game brought back MUST answer every later command as the game would
  have had it never been closed. This includes everything the game decides by chance.
- **FR-008**: Everything the layer reports after a game is brought back, including
  what it settles by trying a way on a copy (feature 009), MUST be what it would have
  reported had the game never been closed.
- **FR-009**: Bringing a game back MUST send the game nothing the player did not type
  or tap in that game (constitution Principle III). Nothing is added to the transcript
  by bringing it back.

**When it is kept**

- **FR-010**: Each answered turn MUST be kept before the next command is accepted.
- **FR-011**: What is kept MUST always be one whole game. If the app ends while
  keeping, what was kept before MUST still be whole and MUST be what is brought back.
- **FR-012**: If the app ends for any reason, at most one turn MAY be lost, and only a
  turn whose reply had not yet been followed by the chance to type another command.
- **FR-013**: If a turn cannot be kept, play MUST go on. The player MUST be told once,
  in the layer's words, that their place is not being kept; keeping MUST be tried
  again at each later turn.

**Starting over**

- **FR-014**: While a game is in progress there MUST be a way to start over that does
  not need the game to end first. It MUST ask the player to confirm, and say that the
  game in progress will be given up.
- **FR-015**: When starting over is confirmed, the kept game MUST be given up before
  the new one is shown, and MUST NOT be brought back afterwards.
- **FR-016**: From a game that has ended, New game MUST start without asking, as
  today.

**When it cannot come back**

- **FR-017**: What is kept MUST record which story file it belongs to, exactly, and in
  what form it was written. A kept game MUST NOT be brought back for any other story
  file, or by a version of the app that does not read its form.
- **FR-018**: When a kept game cannot be brought back, for those reasons or because it
  is damaged, the app MUST start a new game and tell the player once, in the layer's
  words, that the earlier game could not be brought back. It MUST NOT crash, hang, or
  show a game that differs from the one kept.
- **FR-019**: A change to a story's game pack alone MUST NOT prevent a game from being
  brought back.

**Limits**

- **FR-020**: Opening the app on a kept game MUST show it within the limit feature 004
  set for launch (the opening shown in under two seconds), for a game of 331 turns
  (Zork I's walkthrough) and for a game of 2,000 turns. Both MUST be measured and the
  figures recorded.
- **FR-021**: The limits the earlier features set for a command and its snapshot MUST
  still hold with keeping added. What keeping a turn costs MUST be measured and
  recorded, for the first turn and for the 2,000th.
- **FR-022**: What is kept for one game of 2,000 turns MUST be measured and recorded
  in bytes. It MUST NOT grow faster than in proportion to the number of turns.

**Whose it is, and where the work lies**

- **FR-023**: What is kept is the player's own session. It MUST stay on the device,
  MUST NOT be sent anywhere, and MUST NOT be committed to the repository or bundled
  with the app. No story file's content beyond that session is kept (Principle VI).
- **FR-024**: What is kept, and how a game is brought back from it, MUST be decided by
  the shared layer. The app's part is to hold the bytes it is given and hand them
  back, so that a second platform resumes the same way (Principle V).
- **FR-025**: The commands the player has given in the kept game, in order, MUST be
  recoverable from what is kept. (The map is to be rebuilt from them. This feature
  builds nothing of the map.)
- **FR-026**: The messages of FR-013 and FR-018 and the words of the confirmation in
  FR-014 MUST come from the layer; the app composes no text.

**Screen readers**

- **FR-027**: When a game is brought back and a screen reader is running, the newest
  turn's announcement (feature 010) MUST be asked for once. No earlier turn's
  announcement is asked for. With no screen reader running nothing is asked.

**Proof**

- **FR-028**: There MUST be tests, on stories built for the purpose and on Zork I,
  that a session kept and brought back after every turn of a walkthrough gives, turn
  for turn, the same replies and the same snapshots as a session never closed. At
  least one built story MUST decide something by chance.
- **FR-029**: There MUST be tests that each kind of unusable kept game (damaged, cut
  short at every length, another story's, an unread form) brings a new game and the
  message, and never a fault.
- **FR-030**: There MUST be tests in the app that after it is ended abruptly and
  opened again the earlier turns are shown and a command is accepted; that starting
  over asks, and on confirming is not undone by a relaunch; and that an ended game
  comes back ended.
- **FR-031**: Every test of the earlier features MUST pass. The app's tests MUST be
  able to launch with nothing kept, so that each starts from a new game as before.
  Each test that fixed "a launch is a new game" changes, and each change MUST be
  listed.

### Key Entities

- **Kept game**: one per story file on a device. It identifies the story file exactly
  and the form it is written in, and holds enough to bring the game back whole,
  including the player's commands in order.
- **Answered turn**: a command and the reply shown for it. The unit of keeping.
- **Story identity**: what tells one story file from every other, byte for byte.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player who closes the app in the middle of a game and opens it again
  is typing their next command within two seconds, without answering any question.
- **SC-002**: Across the whole of Zork I's walkthrough, closed and brought back after
  every one of its 331 commands, not one reply and not one snapshot differs from the
  same walkthrough played without stopping.
- **SC-003**: When the app's process is ended abruptly at 100 moments chosen across a
  session, every answered turn is there on opening, every time.
- **SC-004**: A game of 2,000 turns is brought back within the same two seconds.
- **SC-005**: With keeping added, every command of the walkthrough is still answered
  within the limit the earlier features set.
- **SC-006**: Of 1,000 kept games damaged in different ways, none crashes the app,
  none hangs it, and none is shown as a game: each brings a new game and the message.
- **SC-007**: After starting over is confirmed, the earlier game does not come back on
  any later opening.
- **SC-008**: The owner plays on their phone for a few turns, closes the app from the
  app switcher, opens it, and finds the game where they left it; and does the same
  after restarting the phone.

## Assumptions

- **Two seconds, at any length.** The limit feature 004 set for launch is kept for a
  game brought back, and FR-020 holds it at 2,000 turns. No game in the corpus has a
  walkthrough near that long; the figure is there so that a long game is not slower to
  open than a short one. How that is met is for the plan.
- **A fault is not kept.** A game that stopped with a fault comes back as it was
  before the command that caused it. The alternative, bringing back the fault, would
  show the player a dead game at every launch.
- **An ended game is kept.** The player may want to read how it ended. It costs one
  tap on New game to go on.
- **Starting over asks; New game after an ending does not.** The first gives up a game
  that could be gone back to; the second does not.
- **Where the way to start over is put on the screen is for the plan**, and is the
  owner's to approve there. This specification asks only that it can be reached during
  play and that it asks first.
- **The text in the command field is not kept.** It was not sent.
- **The newest turn is announced on reopening.** A listener opening the app needs to
  know where they are, as a sighted player sees it at once. The owner has not been
  asked; it follows from feature 010's rule for the opening text.
- **Removing the app removes the kept game.** Nothing is kept anywhere but on the
  device, in the app's own space.
- **One game per story.** The owner's roadmap has "one notebook per game". Several
  games of one story in progress at once is out of scope.
- **Out of scope**: the game's own SAVE and RESTORE, and any saved positions; undo
  across launches; more than one game in progress for a story; carrying a game from
  one device to another; the map and its notebook; Android.
