# Feature Specification: First Demo on a Phone

**Feature Branch**: `004-first-demo`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "Start the next feature: the first demo." Scope decisions
made by the project owner on 2026-11-11: iPhone only, with Android as a later feature;
done when it runs in the iOS Simulator, with a physical phone as a follow-up; the screen
has the game's text, a compass of exits and a text field, and nothing more; each launch
starts a new game, with nothing remembered between launches.

## User Scenarios & Testing *(mandatory)*

This is the first feature whose user is a player. It is also the first time the whole
stack runs together: the interpreter, the introspection interface, the layer that turns
a tap into a typed command, and a native phone screen. The constitution's third phase
asks for exactly this: the game boots, room text shows, and a tappable compass of exits
works end to end on a phone.

The game is Zork I, the only game in the repository whose license allows it to be
bundled in an app.

### User Story 1 - Open the app and read the game (Priority: P1)

A player opens the app on an iPhone and sees the opening of the game: its title text,
the first room's description, and a line at the top showing where they are and their
score.

**Why this priority**: Nothing else can be shown until the game runs on the phone. This
story alone proves the interpreter, the bindings and the screen work together.

**Independent Test**: Launch the app in the simulator. The game's opening text is on
screen, ending with the description of the first room, and the top of the screen names
that room.

**Acceptance Scenarios**:

1. **Given** the app is not running, **When** the player opens it, **Then** within two
   seconds the game's opening text is shown, ending with the first room's description.
2. **Given** the game is showing, **When** the player looks at the top of the screen,
   **Then** it shows the name of the room they are in, their score and their number of
   moves.
3. **Given** more text than fits on the screen, **When** new text arrives, **Then** the
   newest text is in view and the player can scroll back to read what came before.
4. **Given** the app has been closed, **When** the player opens it again, **Then** a new
   game starts from the beginning.

---

### User Story 2 - Walk by tapping the compass (Priority: P2)

The player sees a compass. The directions that lead somewhere from the current room are
lit; the others are not. They tap a lit direction, see the command that was sent on
their behalf, and read the game's reply. The compass changes to show the exits of the
new room.

**Why this priority**: This is the idea the product is built on, and the constitution's
acceptance test for this phase. It needs User Story 1.

**Independent Test**: From the opening, tap north, then north again, then the directions
the compass offers, for at least ten moves. Each tap shows the command sent, the game
moves the player, and the compass then shows the new room's exits.

**Acceptance Scenarios**:

1. **Given** a room with exits, **When** the player looks at the compass, **Then**
   exactly the directions that lead somewhere are lit.
2. **Given** a lit direction, **When** the player taps it, **Then** the command the tap
   stands for appears in the game's text as if typed, followed by the game's reply.
3. **Given** the player has moved, **When** the reply has arrived, **Then** the compass
   shows the exits of the room they are now in, and the top of the screen names it.
4. **Given** an exit whose destination the game stores as a fact, **When** the player
   taps it, **Then** they arrive in the room that exit leads to.
5. **Given** an exit that the game decides by its own rules, such as a door that may be
   closed, **When** the player looks at the compass, **Then** that direction is lit in a
   way that shows it is not certain; tapping it sends the direction and the game answers
   as it would to a typed command.
6. **Given** a direction that is not lit, **When** the player taps it, **Then** nothing
   is sent to the game.
7. **Given** exits that go up, down, in or out, or any other way that is not a compass
   point, **When** the room has one, **Then** it is offered beside the compass by name.

---

### User Story 3 - Type whenever tapping is not enough (Priority: P3)

The player can always type a command instead of tapping: to open the mailbox, to take
something, to do anything the compass does not offer.

**Why this priority**: The constitution requires that typing always remain available and
that anything reachable by touch be reachable by typing. The game cannot be played with
a compass alone.

**Independent Test**: Type `open mailbox`, then `take leaflet`, then `read leaflet`. Each
is shown as typed, and the game's replies are its usual ones.

**Acceptance Scenarios**:

1. **Given** the game is waiting for a command, **When** the player types one and sends
   it, **Then** it appears in the game's text and the game replies.
2. **Given** a typed command that moves the player, **When** the reply has arrived,
   **Then** the compass and the top of the screen show the new room, as after a tap.
3. **Given** the keyboard is showing, **When** the player is reading or tapping the
   compass, **Then** the keyboard does not hide the newest text or the compass.
4. **Given** the game asks a question that is not a command, such as whether to quit,
   **When** it is waiting, **Then** the compass offers nothing and the player answers by
   typing.

---

### User Story 4 - The app never guesses and never gets stuck (Priority: P4)

When the app cannot know the exits, it says so by offering none, and the player types.
When the game ends, the player can start again. Nothing the game does leaves the app
unusable.

**Why this priority**: The constitution forbids the interface from guessing, and a demo
that can be driven into a dead end by playing the game is not a demo.

**Independent Test**: Play until the game ends (quit, or die until the game stops), and
start again from the app. Play to a place where the exits are not known and confirm the
compass offers none while typing still works.

**Acceptance Scenarios**:

1. **Given** the exits of the current room cannot be determined, **When** the player
   looks at the compass, **Then** no direction is lit, the compass shows that the exits
   are unknown and not that there are none, and typing works.
2. **Given** the game has ended, **When** the player looks at the screen, **Then** it
   says the game is over and offers to start a new one; tapping that starts from the
   beginning.
3. **Given** the player types `save` or `restore`, **When** the game asks the app to do
   it, **Then** the game is told it could not be done and says so; play continues.
4. **Given** the player types `script`, **When** the game asks the app for a transcript,
   **Then** it is declined, none is kept, and play continues.
5. **Given** the game's own file is damaged or missing from the app, **When** the app
   starts, **Then** it shows a message saying the game could not be started, and does
   not crash.

---

### User Story 5 - What a tap does is exactly what typing does (Priority: P5)

A developer replays a whole game twice: once by typing every command, and once by
sending every movement the compass offers as a tap and typing the rest. The game's text
is the same both ways.

**Why this priority**: This is the constitution's guarantee that the engine only ever
receives typed commands, measured on a full game. It is last because it is checked last,
not because it matters least.

**Independent Test**: The Zork I walkthrough, with every command that the compass offered
at that moment sent as a tap and every other command typed, produces the committed
reference transcript.

**Acceptance Scenarios**:

1. **Given** the walkthrough, **When** it is replayed with taps wherever the compass
   offered the move, **Then** the game's transcript is byte for byte the reference.
2. **Given** any tap, **When** it is sent, **Then** what reaches the game is a line of
   text a player could have typed, and that line is what the player is shown.
3. **Given** the same taps and typed commands with the same random seed, **When** the
   game is replayed, **Then** the text is the same every time.

---

### Edge Cases

- A room with no exits at all, and a room where every exit is conditional.
- The same destination in several directions, and a direction that leads back to the
  same room.
- The dark: the game says it is pitch black. The exits of the room are still what the
  room's table says.
- The player inside the boat on the river: the room changes without a command to move.
- A direction the game has but the compass has no place for (Zork I has `land`).
- A very long reply, and many replies: the text must stay scrollable and the app
  responsive after a whole game.
- A command that makes the game run for a long time without asking for input.
- The player taps twice quickly, or taps while the game is still replying.
- The app goes to the background and returns: the game is as it was. If the system ends
  the app while it is in the background, the next launch is a new game.
- The phone is rotated, the text size is set very large, or the screen is small.
- The game restarts itself (the player types `restart`).
- The game faults: an illegal operation by the story file.
- The player types nothing and sends an empty line.
- Text the game prints in a fixed layout: Zork I's opening banner and its few tables.

## Requirements *(mandatory)*

### Functional Requirements

**The game on the phone**

- **FR-001**: The app MUST run Zork I, from the story file bundled inside it, with no
  network connection and no other file needed.
- **FR-002**: The app MUST show the game's text in the order the game prints it, with the
  player's commands shown where they were given.
- **FR-003**: The app MUST show the game's status line: the room's name and the score and
  moves, or the time for a game that keeps time, kept current as the game changes it.
- **FR-004**: The newest text MUST be in view after each reply, and all earlier text of
  the session MUST be reachable by scrolling.
- **FR-005**: Each launch of the app MUST start a new game. Nothing is kept between
  launches.
- **FR-006**: While the app stays in memory, leaving it and returning MUST find the game
  as it was.

**The compass**

- **FR-007**: The app MUST show a compass with the eight compass points, and places for
  up, down, in and out.
- **FR-008**: After every reply, the compass MUST show the exits the introspection
  interface reports for the player's location: a direction is lit exactly when an exit
  is reported for it.
- **FR-009**: An exit reported as conditional MUST be distinguishable from one reported
  with a destination, by more than colour alone.
- **FR-010**: Tapping a lit direction MUST send the game one line of text that names that
  direction, exactly as if the player had typed it.
- **FR-011**: The line a tap sends MUST be shown to the player in the game's text, in the
  same way a typed command is.
- **FR-012**: Tapping a direction that is not lit MUST send nothing.
- **FR-013**: A reported exit in a direction the compass has no place for MUST be offered
  as a labelled control beside it.
- **FR-014**: When the exits are unknown, in whole or for a direction, the app MUST NOT
  light a direction it does not know to be an exit, and MUST show that exits are unknown
  in a way that differs from showing there are none.
- **FR-015**: When the game is waiting for something other than a command, the compass
  MUST offer nothing, and MUST NOT say that there are no exits. A game does not mark its
  questions; the rule by which the app tells one from a command prompt is the project
  owner's choice (research R6), and where that rule is unsure the compass offers
  nothing.
- **FR-016**: The compass MUST NOT offer anything while a reply is being worked out; a
  second tap in that time MUST NOT send a second command. A tap made on a compass that
  has since been replaced MUST be refused, even if the new compass offers the same
  direction.

**Typing**

- **FR-017**: The player MUST be able to type and send a line of text whenever the game
  is waiting for one.
- **FR-018**: Everything a tap can do MUST be possible by typing the line the tap would
  send.
- **FR-019**: The keyboard MUST NOT cover the newest text or the field being typed in.

**Honesty and resilience**

- **FR-020**: The app MUST NOT change the game's state except by giving it a line of
  text a player could have typed, or by starting a new game.
- **FR-021**: The app MUST NOT show the player anything about the game's state that the
  introspection interface reports as unknown, as if it were known.
- **FR-022**: When the game ends, the app MUST say so and offer to start a new game.
- **FR-023**: When the game asks to save, to restore or to keep a transcript, the app
  MUST decline. Whatever the game then prints is shown, and play continues. (Zork I
  reports a declined save or restore as a failure; it announces a transcript whether or
  not one is kept.)
- **FR-024**: A fault in the story file, or a story file that cannot be loaded, MUST
  produce a message to the player and MUST NOT crash the app.
- **FR-025**: A command that makes the game run for a long time MUST NOT freeze the
  screen; scrolling MUST keep working.

**How it is built**

- **FR-026**: Deciding which directions are lit, what line a tap sends, and what the
  game is waiting for MUST be done in the shared layer that both phone platforms will
  use, not in the iPhone app's own code. The iPhone app presents what that layer gives
  it and passes taps and typed lines back.
- **FR-027**: That shared layer MUST be usable and testable without a phone or a
  simulator.
- **FR-028**: The app's name, icon and any text of its own MUST NOT use the name "Zork".
  The game's own text is shown as the game prints it.
- **FR-029**: The app MUST include the story file's license notice where a player can
  read it.
- **FR-030**: The interface MUST work with the system's larger text sizes, and every
  control on the compass MUST have a spoken label saying its direction and whether it
  leads somewhere.

**Evidence**

- **FR-031**: The Zork I walkthrough, replayed through the shared layer with a tap sent
  wherever the compass offered the move and typing elsewhere, MUST produce the committed
  reference transcript.
- **FR-032**: Automated checks MUST launch the app in the simulator, read the opening
  text, tap a direction, and confirm the command shown, the reply and the changed
  compass.
- **FR-033**: Every test from the earlier features MUST continue to pass unchanged.

### Key Entities

- **Session**: one game being played, from its start until it ends or the app is
  closed. It holds the game and what has been shown.
- **Turn**: what the game printed since the last command, the command that led to it,
  and whether that command was typed or tapped.
- **Status**: the room's name and the score and moves, as the game's status line gives
  them.
- **Compass**: for each direction it has a place for, one of: leads somewhere for
  certain, may lead somewhere, does not lead anywhere, or not known. Plus any other
  directions the room offers, by name.
- **Command**: a line of text given to the game. A tap produces one; so does typing.
- **Waiting state**: what the game wants next: a command, some other line of text, or
  nothing because it has ended.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From the start of the app's process in the simulator to the game's opening
  text and the first room's exits being shown takes under two seconds, as the app itself
  measures it.
- **SC-002**: A player can walk from the opening to at least ten different rooms using
  only taps, and at every step the compass shows exactly the exits the introspection
  interface reports.
- **SC-003**: After a tap or a typed command, the shared layer has the reply and the
  updated compass ready in under a hundredth of a second for every command of the
  walkthrough, and in the simulator the reply is on screen within one second of the tap.
  (Reworded in planning: a tenth of a second "on screen" cannot be measured reliably in
  a simulator.)
- **SC-004**: The walkthrough replayed with taps wherever the compass offered the move
  gives a transcript with zero bytes of difference from the reference; at least 150 of
  its commands are sent as taps.
- **SC-005**: At no command prompt of that replay is a direction lit that the
  introspection interface, asked separately of an identical game, did not report as an
  exit, and at none is a reported exit left unlit: zero disagreements over more than 300
  prompts. At the walkthrough positions that have hand-made exit lists, the compass
  agrees with those too.
- **SC-006**: Every command a tap sends is shown to the player: zero taps whose command
  is not in the visible text.
- **SC-007**: After a complete game played in one session, the shared layer still
  answers a command, and hands over what is new, in under a hundredth of a second; and
  in the simulator, after at least thirty commands, the first text of the session can
  still be scrolled to.
- **SC-008**: Ending the game and starting a new one, ten times in a row, leaves the app
  showing the opening text each time.
- **SC-009**: A damaged story file, a fault during play, and `save`, `restore` and
  `script` each leave the app running and able to start or continue a game.
- **SC-010**: With the system's text size at its largest accessibility setting, the
  compass, the text and the text field are all usable without any control overlapping
  another.
- **SC-011**: None of the app's own code decides what an exit is or composes a command.
  A check of the iPhone app's source finds that the only text it ever sends the game is
  what the player typed in the field; that it names the twelve compass places only to
  position them; and that every label and state it shows comes from the shared layer.

## Assumptions

- **The game is Zork I.** It is the only committed game whose license allows bundling.
  Advent's does not, so the demo does not include it, though the layers beneath work
  for it. Choosing among several games is out of scope.
- **The player is English-speaking and has an iPhone running a current system.** The
  oldest system supported is decided in planning.
- **"On a phone" means the iOS Simulator for this feature**, by the project owner's
  decision. Nothing may be done that would prevent running on a physical iPhone, which
  needs only the owner's signing identity.
- **The compass shows exits only.** Tappable objects and an inventory list are later
  features, by the project owner's decision. The scope and inventory queries are not
  used by this demo.
- **Nothing is remembered between launches**, by the project owner's decision. The
  game's `save` and `restore` report failure.
- **Conditional exits are lit and can be tapped.** The game may refuse them; that is the
  game's answer, shown as it gives it. This follows from the decision in the
  introspection feature to report them as directions that exist.
- **A lit exit is what the room's table says.** The game's own code can still send the
  player elsewhere (introspection specification, section 8.3). The compass is redrawn
  from where the player actually is after each reply.
- **The word a tap sends is one the game's dictionary has for that direction.** Which of
  a direction's words is used, and how it is shown, is decided in planning; Zork I
  stores words cut to six letters, and the player should not be shown `northe`.
- **The game's text is shown in a readable proportional font**, with layout by the
  phone. Zork I has almost nothing that depends on fixed-width columns.
- **Random events use a fresh seed each launch and each "New game".** Tests use a fixed
  seed. If the player types `restart`, the game restarts itself by its own means.
- **The compass misses one prompt in Zork I.** In the Loud Room the game reads input
  from a different place in its code, and by the owner's chosen rule the compass offers
  nothing there; the player types.
- **Out of scope**: Android; a physical device; the App Store and anything about
  distribution; saving, restoring and transcripts; inventory and tappable objects; other
  games and a way to choose one; version 5 and 8 screen features in the phone interface
  (styles, the upper window, single-key input), since Zork I is version 3; sound;
  settings; an iPad layout.
- **The "experience layer" of the constitution begins here**, as the shared layer of
  FR-026, with only what this demo needs.
