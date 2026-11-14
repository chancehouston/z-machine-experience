# Feature Specification: The Inventory Card

**Feature Branch**: `014-inventory-card`

**Created**: 2026-11-14

**Status**: Draft

**Input**: User description: "Tapping inventory items should do something. Tapping should tell you about them without making things up. A glyph that represents the item is something we can make up. It would be nice to receive suggestions for use with inventory items, consistent with the canon of the game."

## Decisions the owner has made (2026-11-14)

1. A touch on a carried item opens a card: a glyph, the game's own description of
   the item, and buttons for things to do with it.
2. The description is fetched by a **peek**: the game's command for examining is run
   on a copy of the game, and the real game does not move. "The user could always
   have done that, and this is just a way for them to do it."
3. Actions are for one item only. No pairing of items with other things is
   suggested: those are the puzzles. Suggestions are not tested on a copy: a
   suggestion that lit up only when it would work would be a hint.
4. **Two stages**, each its own pull request and phone trial. Stage 1: the card, the
   description, and Drop. Stage 2: actions read from the item's own flags (open,
   close, turn on, turn off, read, eat, wear), with the Zork I pack extended.
5. **Glyph**: in stage 1 every item is the same plain token, drawn as the map is,
   showing open or closed where the layer already knows it. Glyphs by kind come
   with stage 2. No art is authored per game.
6. **The card is shown over the map's band.** The keyboard, the transcript and the
   command field are not moved.

## Decisions for the later stages (the owner, 2026-11-14, after stage 1's trial)

7. **Stage 2's actions**: open and close; turn on and turn off; read; eat, drink
   and wear. Each only where the item's flags say it applies and in the state it
   is in.
8. **Their order** after the close control: "Safest first, Drop and Eat last".
   Open, close, turn on, turn off and read are nearest the close control; Drop,
   eat and drink are farthest from it.
9. **Infocom stories**: the flags are found, proven and written into the pack for
   Zork I only.
10. **Tokens**: five kinds and plain: a lamp for a light source, a box with a lid
    for a container, a page for something readable, a mark for food and drink, a
    mark for clothing. The token is on the item's chip in the strip as well as on
    its card.
11. **A third stage**: the card also shows the game's own text for a thing as it
    was first seen lying where it started, marked as what the game said then, and
    only for a thing the player did see there. Chosen by the owner against the
    recommendation to show only what the game says now. Its own stage, after the
    buttons.

## What was measured before this was written (2026-11-14)

On the 18 story files the repository holds: in every recognised story, exactly one
property of each object holds the dictionary words that name it (always the same
property in Inform stories; a different number in each Infocom story, found from
the file alone). Every recognised story's dictionary has words for examining,
dropping, taking, opening and reading. The Zork stories have no short word for
examining. Flags that say what an item is are numbered alike in every Inform story
and differently in each Infocom story.

## Strongest objection

**A peek shows the player text the real game never printed.** It is not in the
transcript and not in the kept game, and a screen reader will speak it. If a game
changes something when an item is examined (a few do: examining can reveal a thing,
start a timer or score a point), the copy does it and the real game does not, and
the card shows a reply the player has not earned in the game they are playing. The
owner accepted the peek knowing a look is always available; this is the case where
it is not quite the same thing. The remedy for a game that minds is the typed
command. (The first build said on every card, in the layer's words, that it was a
look ahead. The owner found the words unclear on their phone and ruled on
2026-11-14 that a card says something of the layer's only when it could not get a
description.)

**Second: the layer chooses a word for the item.** An item may be named by several
words, and two carried items may share one. The layer picks by a fixed rule and
cannot know the game will take it as meant. Where the game asks which is meant, the
copy stops at the question and the card says the description could not be had.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - I tap an item and read what it is (Priority: P1, stage 1)

The player taps the brass lantern in the strip of what they carry. A card appears
over the map: a token, the item's name, and the game's own description of it. The
count of moves has not changed and nothing has been added to the transcript.

**Why this priority**: it is what was asked for first: to be told about an item
without anything made up.

**Independent Test**: on Zork I, take the leaflet, tap it, and read on the card the
text the game gives for examining it; the moves and the transcript are as they were.

**Acceptance Scenarios**:

1. **Given** the game waits for a command and the player carries an item, **When**
   they tap it, **Then** a card shows the item's name as the game prints it and the
   game's reply to examining it, and the game itself has not moved: the same turn,
   the same count of moves, the same transcript.
2. **Given** the card is open, **When** the player taps its close control, or sends
   any command, **Then** the card goes and the map is back.
3. **Given** an item shown inside an open container, **When** it is tapped, **Then**
   it has a card as any item has.
4. **Given** the same item tapped twice at the same turn, **Then** the same text is
   shown.
5. **Given** a game left with the card open, **When** it is brought back, **Then**
   the game is as it was and no card is open: the peek was never part of the game.

---

### User Story 2 - I drop an item from its card (Priority: P1, stage 1)

The card has a Drop button. The player presses it. The command appears in the
transcript as a tapped command, the game answers, and the card closes.

**Why this priority**: it is the one action every carried item has, and it proves
the path that stage 2's actions will use.

**Independent Test**: on Zork I, take the leaflet, tap it, press Drop; the
transcript shows the command and the game's reply, and the strip no longer has it.

**Acceptance Scenarios**:

1. **Given** the card is open, **When** Drop is pressed, **Then** the layer's
   command for dropping that item is sent as a tapped line, shown in the transcript
   and kept with the game as a tap, and the card closes.
2. **Given** a story whose dictionary has no word for dropping, **Then** the card
   has no Drop.
3. **Given** an item inside a container, **Then** its card has no Drop: the game
   would have to take it out first, and the card does not guess how.

---

### User Story 3 - The card is honest when it cannot say (Priority: P1, stage 1)

**Why this priority**: a wrong or empty description presented as the game's would
break the rule the feature exists to keep.

**Independent Test**: with a story built for the purpose, an item whose examining
ends in a question, one that ends the game, and one with no word that names it
each give a card with the layer's words and no description.

**Acceptance Scenarios**:

1. **Given** an item the layer has no word for, **Then** it cannot be tapped.
2. **Given** a story with no word for examining, **Then** no item can be tapped.
3. **Given** a peek whose copy does not come back to the command prompt (a
   question, a key, the end of the game, a fault, or the limit on what a copy may
   run), **Then** the card shows the layer's words that the description could not
   be had, and nothing of the copy's text.
4. **Given** the game is not waiting for a command, or a command is being worked
   out, **Then** no item can be tapped.
5. **Given** what is carried is not known, **Then** the strip is as it is today and
   nothing can be tapped.
6. **Given** a card with a description, **Then** it holds no words of the layer's
   about the description. (Changed 2026-11-14 by the owner's ruling: "Only when
   it matters".)

---

### User Story 4 - A listener can use the card (Priority: P2, stage 1)

**Acceptance Scenarios**:

1. **Given** a screen reader, **Then** each item that can be tapped is a button
   whose label is what the strip speaks for it today.
2. **Given** the card opens, **Then** its description is spoken once, as a reply
   is, and its controls can be reached.

---

### User Story 5 - The card suggests what the item is for (Priority: P2, stage 2)

The lantern's card offers Turn on; lit, it offers Turn off. A closed sack offers
Open. A leaflet offers Read. Each is sent as a tapped line.

**Why this priority**: it is the "suggestions for use" that was asked for, and it
waits on reading flags, which the Zork stories need pack work for.

**Independent Test**: on Zork I with its pack, the lantern, the sack and the
leaflet each offer the action their flags say, and no other.

**Acceptance Scenarios**:

1. **Given** an item whose flags say it can be opened, lit, read, eaten or worn,
   **Then** the card offers that action, in the state the item is in now.
2. **Given** an item whose flags the layer cannot read, **Then** its card has the
   description and Drop only.
3. **Given** any suggested action, **Then** it names this one item and nothing
   else, and has not been tried on a copy.
4. **Given** items of different kinds, **Then** their tokens differ by kind.

### Edge Cases

- The item's name on the card is the game's printed name; the word sent is one of
  the item's dictionary words. They may differ ("brass lantern", `lamp`).
- Two carried items share the word chosen: the peek ends at the game's question and
  the card says so (US3 scenario 3).
- The description is long: the card scrolls within the band's height.
- The player is in the dark: the copy says what the game says; the card shows it.
- The peek runs while a look-ahead for the ways has spent the prompt's allowance:
  the peek has its own allowance.
- An item leaves the inventory while its card is open (a command typed with the
  card open): the card closes at the new turn.
- Largest text size: the card's text scales; its buttons are at least 44 points.

## Requirements *(mandatory)*

### Functional Requirements

**The item's word and the verbs (stage 1)**

- **FR-001**: The layer MUST find, for each carried item, the dictionary words that
  name it, from the story file alone. An item with none cannot be tapped.
- **FR-002**: The word sent for an item MUST be chosen by a fixed rule from those
  words, the same every time for the same item.
- **FR-003**: The words for examining and for dropping MUST be words of the story's
  dictionary, the long form where the story has it. Without a word for examining no
  item can be tapped; without a word for dropping no card has Drop.

**The peek (stage 1)**

- **FR-004**: A peek MUST run on a copy of the game and MUST leave the game itself
  exactly as it was: its memory, its chance, its turn, its transcript, its kept
  game and its map.
- **FR-005**: A peek MUST give the text the copy printed in reply only when the copy
  came back to the command prompt. Otherwise it gives no text and says why in the
  layer's words.
- **FR-006**: A peek MUST be limited in what its copy may run, as look-ahead is, and
  MUST answer within the time a command is allowed.
- **FR-007**: A peek is asked for by the player's touch and by nothing else: no item
  is examined ahead of being tapped.
- **FR-008**: The same item peeked twice at the same turn MUST give the same text.

**The card (stage 1)**

- **FR-009**: The snapshot MUST say which shown items can be tapped now. Asking for
  an item's card MUST be checked against the turn it was offered at.
- **FR-010**: The card MUST hold: the item's printed name, its token's state, the
  description or the layer's words in its place, and the actions on offer, each
  with its label and spoken words.
- **FR-011**: Pressing an action MUST send that action's command as a tapped line,
  visible in the transcript and kept as a tap, and close the card.
- **FR-012**: The app MUST compose no text and hold no command word for any of
  this. Every word on the card is the game's or the layer's.
- **FR-013**: The card is shown over the map's band and does not move the keyboard,
  the transcript or the command field. Closing is what any touch does that is not
  on an action: on the close control, on the card, on the band beside it, on the
  item again; and it closes at any new turn. An action is small and must be aimed
  at. (Settled at the owner's phone trial, 2026-11-14.)
- **FR-014**: The description on the card MUST be spoken once to a screen reader
  when the card opens, by the means replies are spoken.
- **FR-015**: A closed carried container's contents are not shown or told, as now.

**Suggested actions and kinds (stage 2)**

- **FR-016**: The layer MUST offer an action for an item only where the item's own
  flags say it applies, and in the state the item is in. Where a story's flags
  cannot be read, none is offered beyond Drop.
- **FR-017**: A game pack MAY name the flags of a story that numbers its own. These
  are facts of the story file, as the pack's other entries are.
- **FR-018**: No action names a second thing. No action is tried on a copy.
- **FR-019**: An item's token MUST show its kind where the flags give one.

**Limits**

- **FR-020**: The limits of earlier features hold. The snapshot at each stop costs
  no more than it does now by more than a twentieth.

### Key Entities

- **Item**: something carried and shown, with its printed name, its state, whether
  it can be tapped now, and its place among what is shown.
- **Card**: what the layer gives for one item at one turn: name, state, description
  or the words in its place, the actions.
- **Action**: a label, spoken words, and the command it sends.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On Zork I and on Adventure, every item carried along the walkthrough
  that has a word gives a card, and the game's state after each peek is identical
  to its state before, at every one of them.
- **SC-002**: For those same items, the card's description equals the reply the game
  gives when the same command is typed in a copy made at that turn.
- **SC-003**: Across the corpus, the share of carried items that can be tapped, and
  the share of peeks that give a description, are recorded for each story. No peek
  faults.
- **SC-004**: A peek and its card are made within 10 ms on the walkthroughs.
- **SC-005**: Every test that passed before this feature passes after it.
- **SC-006** (stage 1): the owner, on their phone, reads an item's description from
  its card and drops an item from it, and says whether it is what they wanted.
- **SC-007** (stage 2): on Zork I, the lantern, a container and something readable
  each offer the action their flags give; the owner tries them on their phone.

## Assumptions

- The words looked for are English: `examine` then `x`; `drop`. A story in another
  language gets no card.
- The item's word is the word of its printed name that the game knows it by, or
  failing that the first of its dictionary words (research R2; changed in building).
  Adjectives are not added in stage 1.
- A copy that prints a question is treated as having no description, even where a
  description came before the question.
- Things in the room, not carried, are out of scope; so are things worn, unless the
  strip already shows them.
- This feature is built on feature 013 and merged after it.

## Out of Scope

Authored glyphs; pairing items; dragging items onto the map or each other; things
in the room; taking; a history of cards; peeking at rooms.
