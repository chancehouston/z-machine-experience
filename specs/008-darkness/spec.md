# Feature Specification: Darkness

**Feature Branch**: `008-darkness`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "Start the spec for darkness." Decisions made by the
project owner on 2026-11-12: when the player is in the dark the app shows a warning and
the compass stays as it is; and the compass also marks a way that leads into darkness,
before the player walks in.

## User Scenarios & Testing *(mandatory)*

In these games darkness is dangerous: a player who walks about in the dark is killed.
The game knows whether it is dark by running its own code. The introspection interface
does not: for games of the ZIL family it answers "unknown" for light, because the fact
it would read is numbered game by game. The corpus shows the cost. Of the wrong exit
reports its ledger holds, 87 have darkness as their cause, 39 of them in Zork I: in the
dark the game does not go by the room's table of exits.

Measured on Zork I on a short walk into two dark rooms, with and without a lamp: one
attribute is set on every room that is lit of itself, and on the lamp while it is
switched on. The game was dark exactly when neither the room, nor anything the player
carried or had put down in it, had that attribute. That one number, in a game pack,
lets the interface say whether it is dark, and whether a way leads into the dark.

Three words are used throughout:

- **Lit**: the game would describe the room to the player. **Dark**: it would not.
- **A light**: a thing that has the pack's attribute now and that the player could see:
  in the room or carried, and not shut inside something that cannot be seen through.
- **Dark ahead**: a way whose room would be dark for the player on arriving as they
  are now.

### User Story 1 - The interface says whether it is dark (Priority: P1)

Someone using the introspection interface on Zork I, with its pack in force, asks where
the player is and is told that the room is lit, or dark, in place of "unknown", and
that the answer rests on the pack.

**Why this priority**: Every other part of the feature reads this answer.

**Independent Test**: With Zork I and its pack, walk into a dark room with the lamp
off: dark. Switch it on: lit. Put it down, still on: lit. Switch it off: dark. In a
room that is lit of itself: lit throughout. With no pack: "unknown", as before.

**Acceptance Scenarios**:

1. **Given** a room that is lit of itself, **When** light is asked for with the pack in
   force, **Then** the answer is lit, and marked.
2. **Given** a room that is not lit of itself and no light, **When** light is asked for,
   **Then** the answer is dark, and marked.
3. **Given** the same room and a light the player carries or that lies in the room,
   **When** light is asked for, **Then** the answer is lit.
4. **Given** a light shut inside something closed that cannot be seen through, **When**
   light is asked for, **Then** it does not light the room; inside something open, or
   something that can be seen through, it does.
5. **Given** a pack that says which attribute means lit but not which means open,
   **When** a thing with the attribute is inside another thing, **Then** whether it
   lights the room is unknown, and so is the answer unless something else settles it.
6. **Given** no pack, or a story of the Inform family, **When** light is asked for,
   **Then** the answer is what it was before this feature.
7. **Given** any game played from a script with light asked at every prompt, **When**
   the transcript is compared with one made without asking, **Then** they are
   identical.

---

### User Story 2 - The answer is proven against the game (Priority: P2)

Someone who changes a pack runs the corpus. At every place a throwaway copy of the game
is asked to describe where the player is, and whether it does so is set against what
the interface said about light. A pack that names the wrong attribute fails. And a
wrong exit report made in the dark is no longer counted against the interface, since
the interface now says that it is dark.

**Why this priority**: The constitution admits a pack's fact only when it is proven,
and this is where 39 of Zork I's ledger entries go away.

**Independent Test**: Give Zork I a pack that names the wrong attribute as lit: the
corpus fails and names a place where the game and the interface disagree. Give it the
right one: it passes, with no ledger entry whose cause is darkness.

**Acceptance Scenarios**:

1. **Given** a walk that reaches rooms both lit and dark, **When** the corpus runs with
   the right pack, **Then** at every place the interface's answer agrees with whether
   the game described the room, and the entry is proven.
2. **Given** a pack naming an attribute that does not mean lit, **When** the corpus
   runs, **Then** it fails, naming the entry and a place where the two differ, in
   numbers.
3. **Given** a walk that never reaches a dark place, or never a lit one, **When** the
   corpus runs, **Then** the entry is "not proven" and the run fails.
4. **Given** a place the interface reports as dark, **When** an exit report there is
   not borne out by a trial, **Then** it is counted as made in the dark, and is neither
   wrong nor in need of a ledger entry.
5. **Given** Zork I's ledger entries whose cause is darkness, **When** its pack states
   the attribute and the corpus runs, **Then** those entries match nothing, are
   removed, and the run passes.
6. **Given** a place where the game and the interface differ about light for a reason
   that is not the entry, **When** the cause is written in the ledger, **Then** the run
   passes and the check shows the entry.
7. **Given** a story whose file is absent, **When** the tests run, **Then** its light
   tests are reported as ignored, story absent.

---

### User Story 3 - A way into darkness is marked before the player takes it (Priority: P3)

Someone using the interface asks for the exits of a room and is told, for each way
whose room is known, whether that room would be dark for the player as they are now:
carrying what they carry, lit or unlit. Nothing is tried to find out.

**Why this priority**: It is the warning that saves the player, and it needs User
Story 1's rule applied to a room the player is not in.

**Independent Test**: In Zork I with its pack, stand in the kitchen with no lamp: the
way up is marked as dark ahead. Carry the lamp switched on: it is not. Carry it
switched off: it is.

**Acceptance Scenarios**:

1. **Given** a way to a room that is not lit of itself and holds no light, and a player
   who carries no light, **When** the exits are asked for, **Then** that way is marked
   dark ahead.
2. **Given** the same way and a player who carries a light, **When** the exits are
   asked for, **Then** it is not marked.
3. **Given** a way to a room that is lit of itself, or in which a light lies, **When**
   the exits are asked for, **Then** it is not marked.
4. **Given** a way whose room cannot be read (the game decides it in its own code),
   **When** the exits are asked for, **Then** whether it is dark ahead is unknown.
5. **Given** a trial that walks a marked way, **When** the copy arrives, **Then** the
   game does not describe the room; and where the way was not marked, it does. A
   difference fails like any other.

---

### User Story 4 - The player is warned (Priority: P4)

A player of Zork I in the app walks into a dark room. Beside the compass the app says,
in words, that it is dark. The compass still shows the room's ways as the room's table
gives them. In a lit room next to a dark one, the way into the dark is drawn so that
it can be told from the others, and read aloud as leading into darkness. When the
player switches the lamp on, the warning goes and the marks go.

**Why this priority**: It is where the player gains. It depends on the three stories
before it.

**Independent Test**: In the app, go to the kitchen: the way up is marked. Go up: the
note says it is dark. Come down, fetch the lamp, switch it on: no way is marked. Go up:
no note.

**Acceptance Scenarios**:

1. **Given** the player is in the dark, **When** the screen is shown, **Then** a note
   beside the compass says so, and a screen reader reads it.
2. **Given** the player is in the dark, **When** the compass is shown, **Then** it is
   the room's own, drawn as it would be in the light.
3. **Given** a way that is dark ahead, **When** the compass is shown, **Then** that way
   differs from an ordinary open way to the eye, by more than colour, and is read
   aloud as leading into darkness. Touching it types the direction, as for any way.
4. **Given** the player switches on a light they carry, **When** the game has
   answered, **Then** no way is marked dark ahead, and if they were in the dark the
   note is gone.
5. **Given** a game with no pack, **When** the screen is shown, **Then** there is no
   note and no mark: the app is as it was.

---

### Edge Cases

- **A light whose fuel runs out.** The game clears the attribute by its own clock; the
  answer follows the stored attribute at the moment it is asked.
- **A light carried by someone else in the room.** It is under the room in the object
  tree, so it counts, if it could be seen.
- **A room lit by the game's own code and not by the attribute** (a flag that lights
  everything, daylight through an opening). The interface would say dark and the game
  would describe the room: a difference, with a cause, for the ledger. If a game has
  many, the fact does not fit it and its pack does not get the entry.
- **The player is inside something** (a boat, a basket). Light is that of the room the
  player is in, as the interface already reports it.
- **A way that leads back to the room the player is in.** Dark ahead is whether that
  room would be dark, which is whether it is dark now.
- **A way through a closed door.** It has a room behind it; it is marked or not like
  any way with a known room.
- **The inventory strip in the dark.** Unchanged: the game lists what is carried in the
  dark, and the strip follows the game's listing, which the corpus already compares.
- **A place where the interface cannot tell which object is the player.** What the
  player carries is unknown; if the room is lit of itself the answer is lit, and
  otherwise unknown.
- **"Dark ahead" on a way the game then refuses.** The mark says what the room would
  be, not that the player will get there.
- **A pack that is not in force.** Everything is as with no pack.

## Requirements *(mandatory)*

### Functional Requirements

**What the interface reports**

- **FR-001**: The pack format MUST gain one kind of entry: the attribute that, in this
  story, is set on a room that is lit of itself and on a thing that gives light. It
  applies to stories of the ZIL family.
- **FR-002**: With that entry in force, the interface MUST report the player's room as
  lit if the room has the attribute or a light is present, and dark otherwise, wherever
  it reports light today (in `location` and in `scope`), and the two MUST agree.
- **FR-003**: A thing with the attribute MUST count as a light only if it is in the
  room or carried, and every container between it and the room or the player is open
  or can be seen through. Where the pack does not say which attribute means open, and
  the thing is inside another, whether it counts is unknown.
- **FR-004**: The exits answer MUST say, for each way whose room is known, whether that
  room would be dark for the player now: yes, no or unknown. It is yes when the room
  has not the attribute, no light is in it, and the player carries no light.
- **FR-005**: Every part of an answer that would be different with no pack MUST be
  marked as resting on the pack.
- **FR-006**: With no pack in force, and for every story of the Inform family, every
  value of every answer MUST be what it was before this feature.
- **FR-007**: Asking MUST remain a read of the story's state at the moment of asking.
  Nothing is run or tried, and a game's transcript MUST be identical with and without
  the asking.
- **FR-008**: The introspection specification MUST define the entry, the rule, the new
  part of the exits answer and its mark; its version MUST be raised by its own rule;
  and every new statement MUST have a test that cites it.

**Proof**

- **FR-009**: The corpus MUST be able to ask a throwaway copy of a game to describe
  where the player is, and decide from the reply whether the game described the room,
  without keeping any of the reply. The game being walked MUST be unchanged.
- **FR-010**: The command that asks a game to describe the room MUST be data about the
  story, kept with the corpus, not written in code.
- **FR-011**: The entry is proven when, at every place of the story's walk where the
  interface answers lit or dark, the game agrees, with at least one place dark and one
  lit by a light and not by the room. It is contradicted by a difference with no cause
  in the ledger, and not proven without both kinds of place. Anything but proven MUST
  fail.
- **FR-012**: "Dark ahead" MUST be checked by the trials the corpus already makes: when
  a trial walks a way and arrives in the room the interface named, whether the game
  describes that room to the copy is set against whether the way was marked.
- **FR-013**: An exit report that a trial does not bear out, at a place the interface
  reports as dark, MUST be counted as made in the dark: not wrong, and needing no
  ledger entry. The count MUST be in the story's report.
- **FR-014**: A difference about light that has a cause other than the entry MAY be
  recorded in the ledger with that cause, and the check MUST show such entries.
- **FR-015**: What the corpus keeps MUST be numbers and identifiers only.
- **FR-016**: A story whose file is absent MUST have its light tests reported as
  ignored, story absent. No test downloads a story.
- **FR-017**: Zork I's pack MUST gain the entry, proven, and its ledger entries whose
  cause is darkness MUST be removed. Mini-Zork's pack gains it only if it can be
  proven, with one attribute alone left standing, where its file is present.

**The app**

- **FR-018**: The experience layer MUST give its host, at every stop, whether the
  player is in the dark, with the words to show and to speak; and for each place on the
  compass, whether it is dark ahead.
- **FR-019**: When the player is in the dark the app MUST show the layer's note beside
  the compass, and the compass MUST be drawn as it would be in the light.
- **FR-020**: A way that is dark ahead MUST be told apart from an ordinary open way by
  more than colour, and by what a screen reader says. Touching it MUST type the
  direction and do nothing else.
- **FR-021**: Where light is unknown, the app MUST show no note and no mark.
- **FR-022**: The app MUST compose no text of its own for any of this.

**Layering and limits**

- **FR-023**: The core MUST NOT change. No code may be written for one game.
- **FR-024**: The limits the earlier features set for a prompt's queries, for a command
  and its snapshot, and for launch MUST still hold.
- **FR-025**: Every test of the earlier features MUST pass; a test that fixes a written
  form changes only where this feature changes that form, and each such change MUST be
  listed.

### Key Entities

- **Lit entry**: a pack's fact naming the attribute set on lit rooms and on things that
  give light.
- **Light**: a thing with that attribute that the player could see.
- **Light of a place**: lit, dark or unknown.
- **Dark ahead**: for a way with a known room: yes, no or unknown.
- **Room description check**: what the corpus makes of a copy's reply when asked to
  describe the room: described, or not.
- **Made in the dark**: the verdict on an exit report not borne out at a place the
  interface reports as dark.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On Zork I with its pack, at every place of its walk where the interface
  says lit or dark, the game agrees, and the interface says "unknown" for light at no
  place where it can tell which object is the player.
- **SC-002**: The corpus ledger holds no entry for Zork I whose cause is darkness (39
  before this feature).
- **SC-003**: A pack naming any attribute but the right one as lit fails the corpus on
  Zork I. This is shown for every other attribute number.
- **SC-004**: On Zork I, every trial that walks a way marked dark ahead arrives in a
  room the game does not describe, and every trial that walks an unmarked way to a
  known room arrives in one it does, or the difference has a cause in the ledger.
- **SC-005**: With no pack, every value of every answer for every story of the corpus
  is what it was before this feature.
- **SC-006**: Every walkthrough the project holds gives the same transcript, byte for
  byte, with light asked and compared at every prompt.
- **SC-007**: In the app, on Zork I, a player sees the way into a dark room marked
  before taking it, sees the note on arriving, and sees both go when they switch on a
  light.
- **SC-008**: A person using a screen reader learns from the app everything about
  light that a sighted person does.
- **SC-009**: No pack and no kept file of the corpus holds a word of any committed
  story.
- **SC-010**: The limits of the earlier features hold, and every earlier test passes;
  those whose form changed are listed.

## Assumptions

- **One attribute serves for lit rooms and for lights.** Measured on Zork I on a walk
  of 24 commands through three dark rooms and five lit ones, with a lamp carried, put
  down, on and off. If a game uses two, the proof fails and the vocabulary is
  revisited.
- **The game described the room if its reply names the room.** The corpus looks in the
  reply for the room's own printed name, as it looks for a carried thing's name in an
  inventory listing, and keeps nothing. A game that names its dark rooms when it
  refuses to describe them would defeat this measure.
- **A proof needs a walk into the dark.** Zork I's walkthrough is never in the dark at
  a prompt (measured: 0 of 330), so the proof rests on asides, as "see-through" does.
  The corpus's crawl also wanders into dark rooms, which is where its ledger entries
  came from.
- **The game also keeps its own flag for "lit".** One variable of Zork I was not zero
  exactly when the game was lit, over the same 24 commands. This feature reads the
  attribute and not the flag: the flag says only what is so now, and cannot say what
  lies ahead; and a stored flag has already been found one command out of date
  elsewhere in the same game. The plan may use it as a second check.
- **Marking a way as dark ahead tells the player something the game would only reveal
  by walking in.** The owner chose this on 2026-11-12.
- **The words of the note and of the mark are settled in the plan**, with pictures for
  the owner, as the compass's closed way and the strip were.
- **Out of scope**: how long a light will last; which things could give light but are
  switched off; what kills a player in the dark, and any promise about it; the Inform
  family, which already answers; packs for stories that have none; trying a way on a
  copy to see what happens (look-ahead); the map.
