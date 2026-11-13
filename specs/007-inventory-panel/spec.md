# Feature Specification: Inventory Panel

**Feature Branch**: `007-inventory-panel`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "Start the inventory spec." The project owner wants less
typing: "one of the ways we would type less is if the user could just see their
inventory instead of having to ask for it." Decisions made by the owner on 2026-11-12:
the inventory is an always-visible strip above the compass; touching an item does
nothing in this first version; and game packs gain a fact, "this attribute means
see-through", so that the strip shows what the game's own inventory listing shows.

## User Scenarios & Testing *(mandatory)*

A player of a text adventure types `inventory` again and again. The introspection
interface can already say what the player carries without asking the game: over a
whole game of Zork I it agrees with the game's own listing wherever it answers at all.
This feature puts that answer on the screen, always in view.

One thing stands in the way. The interface lists what is inside a carried container
whether or not the player could see it. Measured on Zork I: with a brown sack carried
and closed, the game's own listing shows the sack and nothing in it; the interface
lists the two things inside. Shown on screen, that is a spoiler. The game also does the
opposite for a glass bottle: closed, it still lists the water inside, because the
bottle can be seen through.

So the screen must show what the game would show, no more. Two stored facts decide
that, and in games of the ZIL family both are numbered game by game:

- **Open.** Game packs already carry it for doors. Measured on Zork I, the same
  attribute is set on a mailbox, a sack and a bottle when each is opened.
- **See-through.** Not yet in a pack. In Zork I the bottle and a glass case carry an
  attribute the sack does not.

Three words are used throughout:

- **Strip**: the row on the screen that shows what the player carries.
- **Shown**: an item is shown when the game's own inventory listing would list it now.
- **Kept back**: an item the player carries inside something, which the game would not
  list now, and which the strip therefore does not show or count.

### User Story 1 - The interface says what of the inventory can be seen (Priority: P1)

Someone using the introspection interface asks what the player carries and is told, for
each container, whether it is open, closed or not known to be either, and for
everything inside one, whether it can be seen. With Zork I's pack in force, the closed
sack's contents are marked as not to be seen and the closed bottle's as seen.

**Why this priority**: Everything on the screen rests on this answer. Without it the
strip either spoils or shows less than the game does.

**Independent Test**: On Zork I with its pack, carry the sack closed and the bottle
closed, and ask for the inventory: the water is marked seen, the sack's contents are
not. Open the sack: they are. With no pack, every such mark says "unknown", as it would
have before, and nothing else in the answer has changed.

**Acceptance Scenarios**:

1. **Given** a carried container that is closed and cannot be seen through, **When**
   the inventory is asked for with the story's pack in force, **Then** the container
   is reported closed and what is in it is marked as not seen.
2. **Given** the same container opened, **When** the inventory is asked for, **Then**
   it is reported open and what is in it is marked as seen.
3. **Given** a carried container that is closed and can be seen through, **When** the
   inventory is asked for, **Then** it is reported closed and what is in it is marked
   as seen.
4. **Given** a container inside a container, **When** the inventory is asked for,
   **Then** something in the inner one is seen only if both can be seen into.
5. **Given** a story of the ZIL family with no pack, **When** the inventory is asked
   for, **Then** whether a container is open, and whether what is in it is seen, are
   both "unknown", and the rest of the answer is what it was before this feature.
6. **Given** a story of the Inform family, **When** the inventory is asked for,
   **Then** the same marks are given from the facts that family's library fixes, with
   no pack.
7. **Given** any game played from a script with the inventory asked at every prompt,
   **When** its transcript is compared with one made without asking, **Then** they are
   identical.

---

### User Story 2 - The marks are proven against the game's own listing (Priority: P2)

Someone who changes a pack, or the rules, runs the corpus. At every place a walkthrough
reaches, a throwaway copy of the game is asked for its inventory, and what it lists is
set against what the interface said could be seen. A "see-through" entry that the
game's listing contradicts fails; so does an "open" entry, now that it bears on
containers as well as doors.

**Why this priority**: The constitution admits a pack's fact only when it is proven.
The proof that exists is by walking through doors, which says nothing about a sack.

**Independent Test**: Give Zork I a pack that names the wrong attribute as see-through;
the corpus fails and names a place where the game's listing and the interface
disagree. Give it the right one; it passes.

**Acceptance Scenarios**:

1. **Given** a walkthrough and the story's pack, **When** the corpus runs, **Then** at
   every prompt where the interface answers, the items it marks as seen are exactly
   the items the game's own listing gives, and the run passes.
2. **Given** a pack naming an attribute that is not see-through, **When** the corpus
   runs, **Then** it fails, naming the entry and a place where the two lists differ, in
   numbers and with no text of the game in what is kept.
3. **Given** a "see-through" entry for a story where no closed, see-through container
   with something in it is ever carried or met, **When** the corpus runs, **Then** the
   entry is "not proven" and the run fails.
4. **Given** a place where the game's listing and the interface differ for a reason
   that is not the entry, **When** its cause is written in the ledger, **Then** the
   run passes, and the check shows the entry for a person to read.
5. **Given** a story whose file is absent, **When** the tests run, **Then** its
   inventory tests are reported as ignored, story absent.

---

### User Story 3 - The player sees what they carry, always (Priority: P3)

A player of Zork I in the app sees, between the game's text and the compass, a row of
what they are carrying. It changes as they take and drop things. A closed sack shows as
a closed sack; when they open it, what is inside appears with it. They never type
`inventory` to find out.

**Why this priority**: It is where the player gains. It depends on the two stories
before it for being right.

**Independent Test**: In the app, start a game: the strip says nothing is carried. Take
something: it appears. Take a closed container with things in it: the container
appears, marked closed, and its contents do not. Open it: they do.

**Acceptance Scenarios**:

1. **Given** a new game in which the player carries nothing, **When** the screen is
   shown, **Then** the strip says so in words.
2. **Given** the player takes an item by a typed command, **When** the game has
   answered, **Then** the item is in the strip, by the name the game prints for it,
   with no further action by the player.
3. **Given** the player carries a closed container with things in it, **When** the
   strip is shown, **Then** the container is shown as closed, and the things in it are
   neither shown nor counted.
4. **Given** the player opens that container, **When** the game has answered, **Then**
   the things in it are shown as being in it.
5. **Given** the player carries more than fits in the strip's width, **When** the
   strip is shown, **Then** it can be moved sideways to see the rest, and it is plain
   that there is more.
6. **Given** any item in the strip, **When** the player touches it, **Then** nothing is
   sent to the game.
7. **Given** a screen reader, **When** the strip is read, **Then** it says what is
   carried, what is in what, and which containers are closed.

---

### User Story 4 - Where the interface cannot say, the strip says so (Priority: P4)

At a few moments the interface cannot tell which object is the player, and so cannot
say what is carried. In a game with no pack it cannot tell whether a container is open.
In both cases the strip does not guess: it says the inventory is not known, or shows a
container without claiming anything about what is in it.

**Why this priority**: Honest unknowns are the project's rule, and they are rare: 6 of
332 prompts in a whole game of Zork I. The strip is useful without this story, but it
must not be wrong.

**Independent Test**: Drawn from a hand-made state in which the inventory is unknown,
the strip says so and suggests typing. Drawn from a state with a container whose
contents are "unknown", the container is shown and its contents are not.

**Acceptance Scenarios**:

1. **Given** the interface answers "unknown" for the inventory, **When** the strip is
   shown, **Then** it says the inventory is not known and that the player can type to
   ask, and shows no item.
2. **Given** a container whose state is "unknown", **When** the strip is shown,
   **Then** the container is shown with no mark of open or closed, and what is in it is
   not shown.
3. **Given** the game is not waiting for a command (it has ended, or asks a question),
   **When** the strip is shown, **Then** it is shown as the compass is at such times:
   present, and plainly not current.
4. **Given** a story of no family, **When** the strip is shown, **Then** it says the
   inventory is not known.

---

### Edge Cases

- **The player is inside something** (a boat, a bed). What the player carries is still
  the player's; the thing the player is in is not an item.
- **Something worn.** Where the family records it, the item is shown as worn. Where it
  does not, nothing is claimed.
- **A see-through container that is also open.** Its contents are seen; it is reported
  open.
- **A container with nothing in it.** Shown as a plain item. Whether an empty thing is
  a container at all is one more fact a game numbers for itself, and nothing rests on
  it here (amended in planning, research R5).
- **A carried thing that holds things but is not a container the player can open** (a
  creature, a surface). Whether its contents are seen follows the family's rule; where
  no rule or pack says, they are kept back.
- **Darkness.** The game's inventory listing is the measure, in the dark as in the
  light. If a game lists differently in the dark, the proof finds it and the ledger or
  a later feature deals with it; the strip claims nothing about light.
- **Many items, long names.** The strip keeps one row. Names are not cut short in what
  a screen reader says.
- **Two carried items with the same printed name.** Both are shown.
- **The same turn changes what is carried and where the player is.** The strip and the
  compass both show the state after the turn.
- **A pack that is not in force.** The strip is what it is with no pack.
- **A new game.** The strip is that of the new game from its first prompt.
- **The keyboard is showing.** The strip gives way with the compass, by the rule the
  compass already follows.

## Requirements *(mandatory)*

### Functional Requirements

**What the interface reports**

- **FR-001**: The inventory answer MUST say, for each carried thing that holds other
  things, whether it is open, closed, or not known to be either.
- **FR-002**: The inventory answer MUST say, for each thing inside a carried thing,
  whether it is seen, not seen, or not known to be either. A thing is seen only if
  every container between it and the player is open or can be seen through.
- **FR-003**: With no pack in force, a story of the ZIL family MUST get "unknown" for
  both, and every value the answer had before this feature MUST be unchanged.
- **FR-004**: A story of the Inform family MUST get both from the facts its library
  fixes, by the rule the interface's scope query already states, with no pack.
- **FR-005**: Any part of the answer that would be different with no pack MUST be
  marked as resting on the pack, as an exit is.
- **FR-006**: Asking MUST remain a read of the story's state at the moment of asking.
  Nothing is run or tried, and a game's transcript MUST be identical with and without
  the asking.

**The pack**

- **FR-007**: The pack format MUST gain one kind of entry: the attribute that, in this
  story, is set on a thing that can be seen through. It applies to stories of the ZIL
  family.
- **FR-008**: The existing "open" entry MUST also decide whether a carried container
  is open. Its meaning for doors MUST NOT change.
- **FR-009**: The introspection specification MUST define the new entry, the new parts
  of the inventory answer and their marks, and what a reader of the earlier format
  does with a pack that holds the new entry. Its version MUST be raised by its own
  rule, and every new statement MUST have a test that cites it.
- **FR-010**: A pack MUST still hold only identity, format version and entries: no
  name, word or line of a game.

**Proof**

- **FR-011**: The corpus MUST be able to ask a throwaway copy of a game for its own
  inventory listing at a place, and set the items it lists against the items the
  interface marks as seen. The game being walked MUST be unchanged by the asking.
- **FR-012**: The command that asks a game for its inventory MUST be data about the
  story, kept with the corpus, not written in code.
- **FR-013**: A "see-through" entry is proven when, at every place of the story's walk
  where the interface answers, the two lists agree, and at least one place has a
  closed container whose contents are seen because of the entry. It is contradicted
  when the lists differ at a place with no cause in the ledger, and not proven when no
  such place exists. Anything but proven MUST fail.
- **FR-014**: The "open" entry's proof MUST now include the same comparison: a carried
  container the entry calls closed whose contents the game lists, or calls open whose
  contents the game does not list, contradicts it.
- **FR-015**: A difference between the two lists that has a cause other than an entry
  MAY be recorded in the ledger with that cause. The check MUST show such entries.
- **FR-016**: What the corpus keeps of these comparisons MUST be numbers and
  identifiers only. No line of a game's listing is kept.
- **FR-017**: A story whose file is absent MUST have its inventory tests reported as
  ignored, story absent. No test downloads a story.
- **FR-018**: Zork I's pack MUST gain a "see-through" entry, proven. Mini-Zork's pack
  gains one only if it can be proven where its file is present; otherwise it is left
  as it is and the strip keeps back what is in its containers.

**The strip**

- **FR-019**: The experience layer MUST give its host, at every stop, what to show in
  the strip: the items shown, in the game's own order, each with its printed name;
  for each, whether it is a container and whether it is open, closed or unmarked; and
  the items shown inside it. It MUST also give what a screen reader says.
- **FR-020**: The strip MUST show only items that are shown, in this specification's
  sense. An item that is not seen, or not known to be seen, MUST NOT be shown,
  counted, or hinted at.
- **FR-021**: The app MUST show the strip between the game's text and the compass at
  all times that the compass is shown, in one row, movable sideways when its items do
  not fit, with a plain sign that there is more.
- **FR-022**: When the player carries nothing, the strip MUST say so in words. When
  the inventory is not known, it MUST say that, and that the player can type to ask.
- **FR-023**: Touching an item MUST send nothing to the game and change nothing.
- **FR-024**: The strip MUST be current after every turn without any action by the
  player, and MUST be that of the new game after a new game starts.
- **FR-025**: When the game is not waiting for a command, the strip MUST be shown as
  not current, as the compass is.
- **FR-026**: Closed containers, and items inside containers, MUST be told apart from
  plain items by more than colour, and by what a screen reader says.
- **FR-027**: The app MUST compose no text of its own for an item: every name is the
  layer's, which is the game's.

**Layering and limits**

- **FR-028**: The core MUST NOT change. No code may be written for one game.
- **FR-029**: With the strip shown, a command and the snapshot that follows it MUST
  stay within the limit feature 004 set, and the app's launch within its limit.
- **FR-030**: Every test of the earlier features MUST pass; a test that fixes the
  written form of the inventory answer or of a snapshot changes only where this
  feature changes that form, and each such change MUST be listed.

### Key Entities

- **Carried item**: a thing the player carries, with its printed name and, where the
  family records it, whether it is worn.
- **Container state**: for a carried item that holds things: open, closed, or unknown.
- **Seen mark**: for a thing inside a carried item: seen, not seen, or unknown.
- **See-through entry**: a pack's fact naming the attribute set on things that can be
  seen through.
- **Game's listing**: what a game prints when asked for its inventory; used only by
  the corpus, on copies, as the measure.
- **Strip**: what the experience layer tells the host to show of the inventory at a
  stop, and what a screen reader says for it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Over a whole game of Zork I with its pack, at every prompt where the
  interface answers, the items it marks as seen are exactly the items the game's own
  listing gives, with no entry in the ledger whose cause is an entry of the pack.
- **SC-002**: At no prompt of that game does the strip show an item the game's own
  listing does not.
- **SC-003**: A pack naming any attribute but the right one as see-through fails the
  corpus on Zork I. This is shown for every other attribute number.
- **SC-004**: With no pack, the inventory answer for every story of the corpus has
  every value it had before this feature.
- **SC-005**: Every walkthrough the project holds gives the same transcript, byte for
  byte, with the inventory asked and compared at every prompt.
- **SC-006**: In the app, on Zork I, a player can take a closed container, see it
  marked closed with nothing shown in it, open it, and see what is in it, without
  typing `inventory`.
- **SC-007**: In a whole game of Zork I played in the app's layer, the number of times
  a player would need to type `inventory` to learn what they carry is at most the
  number of prompts where the interface cannot say (6 of 332 before this feature).
- **SC-008**: A person using a screen reader can learn from the strip everything a
  sighted person can.
- **SC-009**: No pack and no kept file of the corpus holds a word of any committed
  story.
- **SC-010**: A command and its snapshot, and the app's launch, stay within the limits
  the earlier features set.
- **SC-011**: Every earlier test passes; those whose form changed are listed.

## Assumptions

- **"Open" is one attribute for doors and containers.** Measured on Zork I for a
  mailbox, a sack, a bottle and a window. The proof in this feature checks it on
  containers; if a game uses two attributes, the proof fails and the vocabulary is
  revisited.
- **The game's own listing is the measure of "shown".** Where a game's listing is odd
  (it leaves out something carried, or adds something), the strip follows the rule and
  the difference goes in the ledger with its cause.
- **The strip follows the compass's rules for when it is shown.** It gives way to the
  keyboard and is shown as not current at the same times.
- **An item's name is the game's printed name for the object**, without the article
  the game's listing puts before it.
- **How a container and its contents are drawn in one row** is settled in the plan,
  with pictures for the owner, as the compass's closed way was.
- **A fact may need a short walk of its own to be proven.** Zork I's walkthrough never
  carries a closed, see-through container with something in it, so its "see-through"
  entry is proven on a few commands recorded for the purpose (found in planning,
  research R2).
- **Mini-Zork and the Inform stories get no new pack work unless it is cheap.** Inform
  stories need none. Mini-Zork's is optional (FR-018).
- **Out of scope**: touching an item to act on it, or to put its name in the field;
  menus of verbs; things in the room that are not carried; a fact for "lit" and
  anything about darkness; counts, weights and scores of items; the map; a pack for
  any further story; changes to the compass.
