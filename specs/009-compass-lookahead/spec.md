# Feature Specification: Compass Look-Ahead

**Feature Branch**: `009-compass-lookahead`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "Compass look-ahead." Chosen by the project owner on
2026-11-13 as the next feature, from four put to them. Before the compass is shown, each
way the introspection interface cannot settle is tried on a throwaway copy of the game,
and the compass shows what the copy found.

## User Scenarios & Testing *(mandatory)*

Some ways out of a room are decided by the game's own code and not by the room's table
of exits. The introspection interface reports such a way as conditional, "may be
blocked", and the compass draws it with a question mark. The corpus has measured what
that mark is worth: of 198 ways reported conditional, 109 let the player through (55%).
The mark says little more than a coin would.

Played by hand on 2026-11-13, Zork I shows the cost in one room. In the living room the
way down is drawn with its question mark before the rug is moved, when the game refuses
it, and still after the trap door is opened, when the game allows it. And because the
room such a way leads to is not known, the way carries no warning of the dark cellar
below, though the game has just said the stairs descend into darkness.

The corpus already knows how to find out: it gives a throwaway copy of the game the
command a player would type, sees where the copy's player ends up, and throws the copy
away. This feature does the same for the player, before the compass is drawn.

Four words are used throughout:

- **Unsettled way**: a way the interface reports as conditional, or cannot answer for.
- **Look-ahead**: giving a throwaway copy of the game, as it stands at this prompt, the
  command that touching the way would type, and seeing what the copy did.
- **Finding**: what a look-ahead found: *walked*, to a named room; *refused*, the player
  stayed where they were and the game went on; or *no finding*.
- **Chance**: whatever the game draws at random. A copy can be given different draws
  from the game the player is playing.

### User Story 1 - An unsettled way is settled before it is shown (Priority: P1)

A player in Zork I's living room, the trap door open, looks at the compass. The way down
is drawn as an ordinary open way, and says where it leads. Before the trap door was
opened, the same way was not drawn as a way the player can take.

**Why this priority**: this is the feature. Everything else is what it must not get
wrong.

**Independent Test**: on built stories and on Zork I, bring the game to a place with a
way reported conditional, once where the game allows it and once where it refuses, and
ask the experience layer for its compass.

**Acceptance Scenarios**:

1. **Given** a way reported conditional that the game would let the player through,
   **When** the compass is asked for, **Then** the way is reported open, with the room
   the copy arrived in.
2. **Given** a way reported conditional that the game would refuse, **When** the compass
   is asked for, **Then** the way is reported as no way at all, as a direction that
   is not an exit is, and cannot be touched. Typing the direction still gives the
   game's own refusal.
3. **Given** a way the interface settles by itself (open, not an exit, a door open or
   closed), **When** the compass is asked for, **Then** it is reported exactly as
   before this feature, and no look-ahead is made for it.
4. **Given** any look-ahead, **When** it has been made, **Then** the game the player is
   playing is as it was: the same transcript, the same count of moves, the same answers
   to every introspection query, the same next draw of chance.
5. **Given** a story the interface does not recognise, or a stop where the game is not
   waiting for a command, **When** the compass is asked for, **Then** no look-ahead is
   made.

---

### User Story 2 - A way that depends on chance is not promised (Priority: P1)

A player stands where the game decides by chance whether a way may be taken. The compass
does not say the way is open because one copy happened to get through. It keeps its
question mark.

**Why this priority**: a compass that is confidently wrong is worse than one that says
it does not know. The owner accepts an honest unknown and wants one fixed only where it
would mislead or spoil.

**Independent Test**: on a built story with a way that is allowed on some draws and
refused on others, ask for the compass many times with different draws; the way is never
reported open or refused.

**Acceptance Scenarios**:

1. **Given** an unsettled way whose outcome differs between copies given different
   draws of chance, **When** the compass is asked for, **Then** the way is reported
   conditional, as today.
2. **Given** an unsettled way whose outcome is the same on every copy tried, **When**
   the compass is asked for, **Then** the way is reported by that finding.
3. **Given** a way settled by look-ahead, **When** a screen reader speaks it, **Then**
   it says what it says of any open way, or any refused one: where it leads, and no
   more certainty than the layer has.

---

### User Story 3 - A copy that comes to harm tells the player nothing they should not know (Priority: P1)

A player stands before a way that would kill them, or end the game, or leave the game
asking a question. The copy that tries it comes to that end. The player's game does not,
and the compass does not announce what lies there beyond what the owner has chosen.

**Why this priority**: look-ahead walks into every trap in the game before the player
does. What it then says is a matter of spoiling, and of safety for the player's own
game.

**Independent Test**: on built stories with a way that ends the game, a way that leaves
the game asking yes or no, and a way whose command never returns to a prompt, ask for
the compass; the player's game goes on untouched and each way is reported as specified.

**Acceptance Scenarios**:

1. **Given** an unsettled way on which the copy's game ends, **When** the compass is
   asked for, **Then** there is no finding: the way is reported conditional, as
   today, and nothing more is said of it.
2. **Given** an unsettled way after which the copy is not waiting for a command (it
   asks a question, waits for a key, or runs past the limit of FR-016), **When** the
   compass is asked for, **Then** there is no finding and the way is reported
   conditional, as today.
3. **Given** an unsettled way on which the copy walked, but to a room the interface
   cannot name, **When** the compass is asked for, **Then** there is no finding.

---

### User Story 4 - A way found open says whether it is dark ahead (Priority: P2)

A player in Zork I's living room without the lamp, the trap door open, sees the way down
drawn open and carrying the mark of darkness that feature 008 gave to ways into dark
rooms. With the lamp on, the mark is gone.

**Why this priority**: it is the case that showed the need, and it costs little once the
room is known. It rests on feature 008 and on a game pack, so it follows the stories
that need neither.

**Independent Test**: on Zork I with its pack, at the living room with the trap door
open, ask for the compass with the lamp off and with it on.

**Acceptance Scenarios**:

1. **Given** a way found walked to a room, and a pack that says what is lit, **When**
   the compass is asked for, **Then** the way says whether it is dark ahead by feature
   008's rule for that room and what the player carries now.
2. **Given** no pack, or a room whose light is unknown, **When** the compass is asked
   for, **Then** the way is reported open and nothing is said of light.

---

### User Story 5 - The corpus says how often look-ahead is right (Priority: P2)

Someone working on the project runs the corpus and reads, for each story, how many
unsettled ways look-ahead settled, how many it left, and how many findings a second,
independent trial did not bear out. Each of those has its cause in the ledger, or the
build fails.

**Why this priority**: the project's rule since feature 005 is that what the interface
says of a way is measured against what the game does. A finding checked only against
itself proves nothing.

**Independent Test**: run the corpus on a built story with one way that depends on
chance and is missed by look-ahead's own check; the run fails until the ledger names it.

**Acceptance Scenarios**:

1. **Given** a story of the corpus, **When** it is run, **Then** at every place, each
   finding is set against a trial made apart from it with other draws of chance, and
   the story's report counts findings borne out, findings not borne out, and ways left
   conditional.
2. **Given** a finding that the trial does not bear out, **When** the ledger holds no
   cause for it, **Then** the corpus fails.
3. **Given** the figures before this feature, **When** the corpus is run without
   look-ahead, **Then** every figure is what it was.

---

### Edge Cases

- **A way the copy walks that then cannot be walked back.** The trap door shuts behind
  the player. The compass says the way is open and where it leads; it promises nothing
  about the way back.
- **A way the copy walks that moves the player somewhere else again at once**, by a
  timed event or a vehicle. The finding names the room the copy's player is in when the
  copy next waits for a command.
- **A refusal that changes the game.** A refused walk may still use a turn, burn the
  lamp, or wake something. That happens to the copy only.
- **A look-ahead that prints.** Whatever the copy prints is discarded. None of it
  reaches the transcript, the status line, or the player.
- **A look-ahead that tries to save, restore or restart.** The copy is given none of
  them; such a trial has no finding.
- **Many unsettled ways at one place.** Each is tried on its own copy, from the same
  state. If the limit of FR-016 is reached, the ways not yet tried stay conditional.
- **A way that is not on the compass.** Ways shown beside it, such as "land" or
  "cross", are tried by the same rule.
- **A way settled by a pack.** A door the pack reports open or closed is settled, and
  is not tried.
- **The player in the dark.** The game does not go by its exits in the dark (feature
  008). Look-ahead is made as anywhere else; a fatal walk gives no finding.
- **A second look at the same prompt.** Asking for the compass twice at one prompt
  gives the same answer both times.

## Requirements *(mandatory)*

### Functional Requirements

**Which ways, and when**

- **FR-001**: Look-ahead MUST be made only for an unsettled way: one the introspection
  interface reports as conditional, or cannot answer for, in a story it recognises.
- **FR-002**: Look-ahead MUST be made only while the game waits for a command.
- **FR-003**: A way the interface settles by itself or by a game pack MUST be reported
  as before this feature.
- **FR-004**: Look-ahead MUST work without a game pack. Only what is said of light
  (FR-012) needs one.

**The trial**

- **FR-005**: A look-ahead MUST give its copy exactly the text that touching the way
  would type, and nothing else (constitution Principle III).
- **FR-006**: A look-ahead MUST leave the player's game unchanged in every respect a
  player or the interface could observe, including its next draw of chance
  (constitution Principle IV). Every walkthrough the project holds MUST give the same
  transcript, byte for byte, with look-ahead made at every prompt.
- **FR-007**: A finding is *walked* when the copy is again waiting for a command and
  its player is in a different room that the interface can name; *refused* when the
  copy is again waiting for a command and its player is in the same room; and *no
  finding* otherwise.
- **FR-008**: Each unsettled way MUST be tried on more than one copy, with different
  draws of chance. If the findings differ, or any is *no finding*, the way has no
  finding.
- **FR-009**: Nothing a copy prints, and no state of a copy, may reach the player or be
  kept after the finding is made.

**What is reported**

- **FR-010**: A way found *walked* MUST be reported open, with the room reached.
- **FR-011**: A way found *refused* MUST be reported as a direction that is not an exit
  is: it sends nothing when touched, and is not told apart from a direction with no
  way. Typing the direction MUST still reach the game.
- **FR-012**: A way found *walked* MUST say whether it is dark ahead, by feature 008's
  rule, where a pack makes that known, and nothing of light otherwise.
- **FR-013**: A way with no finding MUST be reported conditional, exactly as before this
  feature. A copy whose game ended gives no finding, and the way is not marked as
  dangerous.
- **FR-014**: The experience layer MUST give its host the words to show and to speak
  for every way; the app MUST compose no text of its own, and MUST draw a way settled
  by look-ahead from the same reported state as any other way.
- **FR-015**: What a screen reader says of a way settled by look-ahead MUST carry
  everything a sighted player is shown of it.

**Limits**

- **FR-016**: All look-aheads at one prompt, together, MUST finish within a limit the
  plan sets from measurement, small enough that the limits earlier features set for a
  command and its snapshot still hold with look-ahead on. A way not tried within the
  limit has no finding. A single copy that does not return to a prompt within a fixed
  count of steps has no finding.
- **FR-017**: The limits the earlier features set for a prompt's queries and for launch
  MUST still hold.

**Proof**

- **FR-018**: At every place of every story of the corpus, each finding MUST be set
  against a trial made apart from the look-ahead, with draws of chance the look-ahead
  did not use. A finding the trial does not bear out fails the corpus until its cause
  is in the ledger.
- **FR-019**: Each story's report MUST count, of its unsettled ways: found walked, found
  refused, left conditional, and findings not borne out. The summary MUST total them by
  family and for all.
- **FR-020**: With look-ahead set aside, every figure of the corpus MUST be what it was
  before this feature.
- **FR-021**: What the corpus keeps MUST be numbers and identifiers only.

**Layering**

- **FR-022**: The answers of the introspection interface MUST NOT change: look-ahead is
  made above it. `docs/introspection.md` MUST say that a conditional answer may be
  settled by a trial on a copy, and what such a trial must leave untouched, so that
  another interpreter could do the same.
- **FR-023**: Look-ahead MUST live in the shared layer both phone platforms will use;
  the app holds presentation only (constitution Principle V). No code may be written
  for one game.
- **FR-024**: Every test of the earlier features MUST pass; a test that fixes a written
  form changes only where this feature changes that form, and each such change MUST be
  listed.

### Key Entities

- **Unsettled way**: a way reported conditional, or unanswered, in a recognised story.
- **Look-ahead**: one trial of one way on one throwaway copy.
- **Finding**: walked (with a room), refused, or no finding.
- **Agreement**: the rule that a way has a finding only when every copy tried gave the
  same one.
- **Borne out**: the corpus's verdict on a finding that an independent trial repeats.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In Zork I's living room, the way down is shown as a way the player can
  take only once the trap door is open, and then says it leads to the cellar.
- **SC-002**: Across the corpus, of the ways reported conditional (198 before this
  feature), at least nine in ten are settled by look-ahead, and the number left
  conditional is reported for each story.
- **SC-003**: Across the corpus, every finding is borne out by an independent trial, or
  its cause is in the ledger. No finding is wrong without a written cause.
- **SC-004**: Every walkthrough the project holds gives the same transcript, byte for
  byte, with look-ahead made at every prompt.
- **SC-005**: A player notices no delay: the time from a command to a drawn compass
  stays within the limit feature 004 set, on the slowest place of the corpus.
- **SC-006**: On Zork I with its pack and no lamp, the open trap door's way down carries
  the mark of darkness; with the lamp on it does not.
- **SC-007**: A person using a screen reader learns of each way everything a sighted
  person does.
- **SC-008**: With look-ahead set aside, every figure of the corpus and every answer of
  the interface is what it was before this feature.
- **SC-009**: No kept file of the corpus holds a word of any committed story.

## Assumptions

- **Only unsettled ways are tried.** The interface is right about 96.6% of the ways it
  reports as leading to a room and 98.1% of those it reports as no exit; every
  exception is the game's code overriding its table, and is in the ledger. Trying every
  direction would catch some of those, and would also show the player hidden ways the
  game means them to find. That is left for the owner to ask for.
- **The copy's answer can differ from the game's only through chance.** A copy is made
  at the prompt and given the command the player would give, so every timer and counter
  stands as it will for the player. What the copy cannot share is the draw of chance.
  FR-008 guards against that; FR-018 measures what it misses.
- **A refused way is hidden, by the owner's choice of 2026-11-13.** It is drawn as no
  way at all, so a way the game hides stays hidden until the player opens it. The cost
  is accepted: a guarded way looks like a wall until it is typed.
- **A fatal way keeps its question mark, by the owner's choice of 2026-11-13.** The
  compass neither warns of it nor invites the player into it.
- **A way settled by look-ahead looks like any other way.** The player is not shown how
  the compass came to know. The corpus, not a mark on the screen, answers for its
  truth.
- **Naming the room a way leads to is not new.** The compass already says where a
  settled way leads before the player has walked it.
- **The principles need no amendment.** A trial on a copy writes nothing to the game
  being played, and the copy is driven only by typed text. The plan's constitution
  check confirms or refutes this before any code is written.
- **The corpus's own trial is the model.** It has made 11,310 such trials. Where it
  lives, and whether the corpus and the layer share it, is for the plan.
- **Out of scope**: trying ways the interface settles; a mark for one-way passages; any
  promise about what follows an arrival; the self-revealing map; the scene snapshot
  and graphical interface; Android; hooks inside the core.
