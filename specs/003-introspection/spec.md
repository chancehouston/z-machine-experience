# Feature Specification: Introspection Interface

**Feature Branch**: `003-introspection`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "Start the next feature: the introspection interface." Scope
decisions made by the project owner on 2026-11-11: both game families are covered,
Infocom/ZIL (tested on Zork I) and Inform 6 (tested on Advent); objects in scope are
worked out from the object tree and every such answer is labelled an approximation; an
exit decided by the game's own code is reported as a direction that exists with its
destination unknown; and the command-line host gains an option that prints the answers at
each prompt.

## User Scenarios & Testing *(mandatory)*

The "users" are the project's developers and the experience layer that will be built
next. That layer turns taps into typed commands, and to do so it must know where the
player is, which ways lead out, what is here and what is carried. This feature is the
read-only window through which it finds out.

The interface has two parts: a written specification that another interpreter could
implement, and this project's implementation of it.

### User Story 1 - See where the player is and what they carry (Priority: P1)

A developer plays a game with inspection turned on and, at each prompt, sees the name of
the player's location and the list of what the player is carrying.

**Why this priority**: Location and inventory are stored facts in both families, so they
can be answered exactly. They prove the whole path: recognising the family, finding the
player, reading the game's state without touching it.

**Independent Test**: Play the first moves of Zork I and of Advent with inspection on;
after each move the reported location and inventory equal what `look` and `inventory`
say in the game.

**Acceptance Scenarios**:

1. **Given** Zork I at its opening, **When** the developer reads the location, **Then**
   it is the room the game calls "West of House".
2. **Given** Advent after the player has entered the building and taken the lamp,
   **When** the developer reads the inventory, **Then** it lists the lamp and nothing
   else.
3. **Given** a carried container with something inside, **When** the developer reads the
   inventory, **Then** the contents are shown as inside the container, not as carried
   separately.
4. **Given** a story file whose family is not recognised, **When** the developer reads
   the location or inventory, **Then** the answer is "unknown" and the game plays as
   usual.

---

### User Story 2 - See the exits (Priority: P2)

At each prompt the developer sees which directions lead out of the current room and,
where the game stores it as a fact, which room each leads to.

**Why this priority**: The first demo is a tappable compass of exits. This story is the
data that compass shows.

**Independent Test**: At a set of rooms in each game, chosen to include plain exits,
exits decided by the game's code, and directions that only print a refusal, the reported
exits equal a list worked out by hand from the game's source.

**Acceptance Scenarios**:

1. **Given** a room with an exit stored as a plain destination, **When** the developer
   reads the exits, **Then** that direction is listed with the room it leads to.
2. **Given** a room with an exit the game decides by running its own code, such as a door
   that may be locked, **When** the developer reads the exits, **Then** the direction is
   listed, marked conditional, with no destination.
3. **Given** a direction for which the room only stores a message of refusal, **When**
   the developer reads the exits, **Then** that direction is not listed as an exit.
4. **Given** a room where the game works out all movement in code, **When** the developer
   reads the exits, **Then** the answer is "unknown", not an empty list.
5. **Given** a listed exit with a destination, **When** the player types that direction
   and the game moves them, **Then** the new location is the destination that was
   reported.

---

### User Story 3 - See what is within reach (Priority: P3)

At each prompt the developer sees the objects the player could plausibly act on: what is
in the room, what is in open or see-through containers and on surfaces, and what is
carried. The answer is labelled an approximation.

**Why this priority**: Tapping an object needs a list of objects. It comes after exits
because the answer cannot be exact: the games decide scope by running code.

**Independent Test**: At a set of positions in each game, the reported objects include
every object the game names in its room description and inventory, and exclude objects
that are in closed opaque containers or in other rooms.

**Acceptance Scenarios**:

1. **Given** a lit room with objects on the floor, **When** the developer reads the
   objects in scope, **Then** each is listed and the answer is labelled approximate.
2. **Given** a closed, opaque container in the room, in a family that records whether a
   container is open, **When** the developer reads the objects in scope, **Then** the
   container is listed and its contents are not. In a family that does not record it,
   the contents are listed and marked as of unknown visibility.
3. **Given** an open container or a surface in the room, **When** the developer reads the
   objects in scope, **Then** what is in or on it is listed.
4. **Given** the player in darkness, **When** the developer reads the objects in scope,
   **Then** only what the player carries is listed. Where the family does not record
   whether a room is lit, the room's contents are listed and the answer says light is
   unknown.
5. **Given** any answer to this query, **When** it is delivered, **Then** it says it is
   an approximation, so that no caller can mistake it for the game's own judgement.

---

### User Story 4 - Read the object tree and the dictionary (Priority: P4)

A developer, or a layer built on the interface, reads the game's whole object tree with
each object's name, and the list of words the game's parser knows.

**Why this priority**: These are properties of the Z-machine itself, not of a family, so
they work for every story file. The other queries are built on them, but a host needs
them directly only for debugging and for checking that a command it builds uses words the
game understands.

**Independent Test**: For each committed story file, the tree read through
the interface has the same objects, names and parent-child links as one built from the
story file by an independent check, and every dictionary word is one the game's parser
recognises.

**Acceptance Scenarios**:

1. **Given** any supported story file, **When** the developer reads the object tree,
   **Then** every object appears once with its number, name, parent, and children in
   order.
2. **Given** any supported story file, **When** the developer reads the dictionary,
   **Then** every word the parser knows is listed, as the player would type it.
3. **Given** a family that marks words as nouns, verbs or directions, **When** the
   developer reads the dictionary, **Then** each word carries those marks; for other
   story files the marks are "unknown".
4. **Given** a story file of an unrecognised family, **When** the developer reads the
   tree or dictionary, **Then** both are answered in full.

---

### User Story 5 - Inspection changes nothing (Priority: P5)

A developer plays a full game with inspection on, asking every query at every prompt, and
the game behaves exactly as it does with inspection off.

**Why this priority**: This is the constitution's guarantee (Principle IV). It is listed
last because it is checked last, not because it matters least: no other story is
acceptable without it.

**Independent Test**: The Zork I and Advent walkthroughs, run with every query asked at
every prompt, produce transcripts and final save files identical to those produced with
inspection off.

**Acceptance Scenarios**:

1. **Given** each walkthrough, **When** it is run with every query asked at every prompt,
   **Then** its transcript is byte for byte the transcript without inspection.
2. **Given** a game at any prompt, **When** every query is asked and the game is then
   saved, **Then** the save file is byte for byte the one written without the queries.
3. **Given** a damaged story file that the interpreter accepts, **When** queries are
   asked of it, **Then** each answers or says "unknown", and none stops the interpreter
   or the host.

---

### User Story 6 - Another interpreter could implement it (Priority: P6)

A developer of a different interpreter reads the published specification and can tell,
without reading this project's code, what each query returns, when it may say "unknown",
and how to recognise each family.

**Why this priority**: The constitution makes the specification the contract and this
project's code one implementation of it. It is last because the document can only be
finished once the queries above have been shown to work.

**Independent Test**: A reviewer who has not seen the implementation reads the
specification and, for a set of positions in the two games, predicts each query's answer
from the document and the story file alone; the predictions match the implementation.

**Acceptance Scenarios**:

1. **Given** the specification, **When** a reader looks up any query, **Then** they find
   what it returns, each way it can be "unknown", and whether it is exact or approximate.
2. **Given** the specification, **When** a reader looks up a family, **Then** they find
   how a story file is recognised as belonging to it and how each family-dependent query
   is answered for it, in terms of the story file's contents.
3. **Given** every query the implementation offers, **When** the specification is
   checked against it, **Then** each is described and none is described that is not
   offered.

---

### Edge Cases

- A story file of neither family, such as czech or praxix: family-dependent queries
  answer "unknown"; the tree and dictionary are answered.
- A story file that resembles both families, or neither clearly: it is treated as
  unrecognised. The interface does not pick the likelier one.
- The player is inside something that is not a room: a vehicle, a container, a cage. The
  location query reports the room, and says the player is in or on the other object.
- The game changes which object is the player (Zork I does not; some games do).
- A room with no name, or an object whose name is empty.
- The game is not at a prompt: it is waiting for a single key, asking a yes-or-no
  question, showing a menu, or has ended. Queries are answered from the state as it
  stands; no query fails because of when it is asked.
- Darkness: whether the room is lit decides what is in scope, and light is itself decided
  by the game's code in both families.
- An exit that leads back to the same room, and two directions that lead to the same
  room.
- A direction the game understands that is not a compass direction: in, out, up, down,
  and words a single game adds.
- A door: in Inform an exit often leads to a door object, not a room.
- An object tree that is damaged: a loop of parents, a child whose parent disagrees, an
  object number past the end of the table.
- A dictionary whose entries are not in order, or a game with more than one dictionary.
- Objects that are scenery, or that the game conceals: they are in the tree and may be in
  reach, but a player would not expect them in a list.
- Very many objects in one place.
- Queries asked before the game has run at all, when the player may not yet be placed.

## Requirements *(mandatory)*

### Functional Requirements

**The interface as a whole**

- **FR-001**: The interface MUST offer six queries: the object tree, the player's
  location, the player's inventory, the exits from the player's location, the objects in
  scope, and the dictionary. The family a story file was recognised as (FR-013) is a
  fact about the file that goes with every answer, not a seventh query.
- **FR-002**: No query may alter the game's memory, call stack, random-number state,
  screen, output or any other state of the interpreter. A game's behaviour MUST be
  identical whether or not queries are asked, and whichever queries are asked.
- **FR-003**: The interface MUST NOT run any of the game's code to answer a query.
- **FR-004**: Every query MUST be able to answer "unknown", in whole or, where its answer
  has parts, for a part. "Unknown" MUST be distinguishable from an empty answer: "no
  exits" and "exits unknown" are different answers.
- **FR-005**: The interface MUST NOT guess. Where the story file's contents do not settle
  an answer, the answer is "unknown". The one sanctioned approximation is the scope
  query, under FR-022.
- **FR-006**: Every answer MUST say whether it is exact or approximate.
- **FR-007**: A query MUST be answerable whenever the interpreter has stopped for the
  host: at a request for a line or a key, at a request to save or restore, after the game
  has ended, and after a fault.
- **FR-008**: No query may stop the interpreter or the host, whatever the story file
  contains. A damaged structure yields "unknown" for what cannot be read.
- **FR-009**: The same story file in the same state MUST give the same answers on every
  run and every platform.

**Families**

- **FR-010**: The interface MUST recognise two families of story file: those compiled
  with Infocom's ZIL tools, and those compiled with Inform 6 and its standard library.
- **FR-011**: Recognition MUST be decided from the story file's contents alone, by rules
  the specification states. It MUST NOT depend on the file's name, on a list of known
  games, or on anything supplied per game.
- **FR-012**: A story file that does not clearly belong to one family MUST be reported as
  unrecognised.
- **FR-013**: The interface MUST report which family a story file was recognised as, or
  that it was not recognised.
- **FR-014**: There MUST be no code and no data specific to a single game. Each family's
  rules apply to every story file of that family.
- **FR-015**: The object tree and the dictionary's list of words MUST be answered for
  every story file the interpreter runs, recognised or not.

**The queries**

- **FR-016**: The object tree query MUST give every object's number, its name as the game
  prints it, its parent, and its children in the game's order. It MUST also give each
  object's attributes and the numbers of the properties it provides.
- **FR-017**: The location query MUST give the room the player is in, as an object of the
  tree. Where the player is inside or on another object within that room, it MUST give
  that object as well.
- **FR-018**: The inventory query MUST give the objects the player carries, with what
  each contains, as objects of the tree.
- **FR-019**: The exits query MUST give, for the player's location, each direction that
  leads somewhere. Each exit MUST carry the word or words a player types to go that way.
- **FR-020**: An exit whose destination the story file stores as a fact MUST carry that
  destination. That includes an exit the game opens or closes by a stored value alone: it
  is an exit with its destination while the value says open, and not an exit while it
  says closed. An exit that the game decides by running code MUST be marked conditional
  and carry no destination. A direction for which the game stores only a refusal MUST NOT
  be listed.
- **FR-021**: Where the game works out a room's movement in a way the interface cannot
  read, the exits query MUST answer "unknown" for that room.
- **FR-022**: The scope query MUST give the objects in the player's location, the
  contents of containers there that are open or see-through, the objects on surfaces
  there, and what the player carries, by the same rules applied to what is carried. Where
  the family does not record whether a container is open, FR-023 applies. Every answer
  to this query MUST be labelled approximate.
- **FR-023**: The scope query MUST take account of darkness where the story file's
  contents settle whether the location is lit. Where they do not, it MUST list the
  room's contents and say that light is unknown. Likewise, where the family does not
  record whether a container is open, its contents MUST be listed and marked as of
  unknown visibility. (Revised in planning with the project owner's agreement: research
  R9. The first wording made scope empty for the whole ZIL family.)
- **FR-024**: The scope query MUST mark objects the game treats as scenery or concealed,
  where the family records that as a fact, so that a caller can leave them out of a list.
- **FR-025**: The dictionary query MUST give every word in the game's main dictionary, in
  the form a player types. For a recognised family it MUST also say, where the family
  records it, whether each word is a verb, a noun, a direction or a preposition.

**The specification**

- **FR-026**: The interface MUST be defined by a document in this repository, written so
  that the author of another interpreter could implement it without reading this
  project's code.
- **FR-027**: The document MUST state, for each query: what it returns, every
  circumstance in which it or a part of it is "unknown", and whether it is exact or
  approximate.
- **FR-028**: The document MUST state, for each family: how a story file is recognised,
  and how each family-dependent query is answered, in terms of the story file's contents.
- **FR-029**: The document MUST describe the answers in a form independent of any
  programming language.
- **FR-030**: The document and the implementation MUST agree: a check MUST fail if a
  query exists in one and not the other.
- **FR-031**: The document MUST carry a version, and state what kind of change requires
  the version to change.

**The command-line host**

- **FR-032**: The command-line host MUST offer an option that, each time the game waits
  for input, prints the family, the location, the exits, the inventory and the objects in
  scope.
- **FR-033**: What that option prints MUST go where it cannot be mistaken for the game's
  text, and MUST NOT appear in a transcript.
- **FR-034**: The command-line host MUST offer a way to print the whole object tree and
  the dictionary of a story file.
- **FR-035**: What the host prints MUST be regular enough for a test to read.

**Evidence**

- **FR-036**: The transcripts and final saves of the Zork I and Advent walkthroughs MUST
  be identical with and without every query asked at every prompt.
- **FR-037**: At every prompt of both walkthroughs, the location reported MUST agree with
  the game, by a check that does not read the same stored value the query reads: whenever
  the game prints a room's name as a heading, it is the reported location's name, and
  whenever the reported location changes, the game printed its name or said it was dark.
- **FR-038**: At every prompt of both walkthroughs, after each move made through a listed
  exit that carried a destination, the new location MUST be that destination.
- **FR-039**: Expected exits, worked out by hand from each game's source, MUST be
  recorded for a set of rooms in each game, and the reported exits MUST equal them.
- **FR-040**: Each query MUST have tests that cite the section of the introspection
  specification they verify (constitution Principle II).
- **FR-041**: Every test from the earlier features MUST continue to pass unchanged.

### Key Entities

- **Family**: a group of story files produced by the same compiler and library, which
  therefore keep the player, rooms, exits and object properties in the same way. One of
  Infocom/ZIL, Inform 6, or unrecognised.
- **Adapter**: the rules for one family: how to recognise it and how to answer each
  family-dependent query from a story file's contents.
- **Object**: one node of the game's object tree, with a number, a name, a parent,
  children, attributes and properties.
- **Location**: the room the player is in, and the object within it that holds the
  player, if any.
- **Exit**: a direction out of a room, the words that name it, whether it is fixed or
  conditional, and for a fixed exit the destination.
- **Scope**: the objects the player could plausibly act on, each with how it comes to be
  in reach (in the room, in or on something, carried) and whether the game treats it as
  scenery or concealed. Always approximate.
- **Dictionary word**: a word the game's parser knows, with the parts of speech the
  family records for it.
- **Answer**: what a query returns: a value, or "unknown", with a mark saying whether it
  is exact or approximate.
- **Introspection specification**: the document that defines all of the above, with a
  version.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Both walkthroughs, with every query asked at every prompt, give transcripts
  with zero bytes of difference from runs without queries; and so do the save files
  written at each of the committed save points (five in Zork I, three in each build of
  Advent).
- **SC-002**: At every prompt of both walkthroughs (over 500 prompts in all), the
  location is reported, never unknown, and agrees with the game by the check of FR-037
  with zero disagreements.
- **SC-003**: Every move in both walkthroughs made through an exit reported with a
  destination arrives at that destination: zero mismatches. Advent's walkthrough makes
  only 67 such moves, so each game is also toured beyond its walkthrough by following
  the reported exits, for at least 100 further moves; there, the only moves that may end
  elsewhere are those the game's own code redirects on arrival to a room of the same
  printed name, each of which the specification must name. (Revised in implementation:
  the first wording asked for 100 moves in each walkthrough, which Advent's does not
  have, and did not allow for the forest.)
- **SC-004**: For at least 25 rooms in each game, chosen to include fixed exits,
  conditional exits, refusals and exits a stored value opens, the reported exits equal
  the list worked out by hand from the game's source. Each expectation names the point in
  the walkthrough at which it holds.
- **SC-005**: At every prompt of both walkthroughs where the game is waiting for a
  command, every object the game lists in its reply to `inventory` is in the reported
  inventory, and the reverse, with zero disagreements. The inventory may be "unknown" at
  no more than 3% of a game's prompts, and at none of Advent's.
- **SC-006**: At no fewer than 20 positions in each game, every object the game's room
  description names is in the reported scope, and no reported object is in another room.
  In Advent, no reported object is inside a closed opaque container. In Zork I, every
  reported object that is inside another is marked as of unknown visibility.
- **SC-007**: For each committed story file (Zork I; Advent in versions 5 and 8; czech in
  versions 3, 5 and 8; praxix), the object tree and dictionary read through the interface
  equal those produced by an independent check, object for object and word for word.
- **SC-008**: czech, in all three versions, and praxix are reported as unrecognised, and
  every family-dependent query answers "unknown" for them.
- **SC-009**: Thousands of damaged copies of the committed story files are queried in
  full; none stops the interpreter or the host.
- **SC-010**: Asking all six queries at a prompt takes less time than a person would
  notice: under 5 milliseconds on a developer's machine for the largest committed game.
- **SC-011**: Every query the implementation offers is described in the specification
  and the reverse, confirmed by an automated check.
- **SC-012**: A reviewer who has not seen the implementation predicts, from the
  specification and the story files alone, the answers to all six queries at five
  positions in each game, with every prediction matching.

## Assumptions

- **Families can be told apart from a story file's contents.** Inform 6 story files carry
  the compiler's version in their header and lay out the library's objects in a
  recognisable way; Infocom's files have their own regularities. Research must confirm
  rules that recognise every committed story file correctly and admit no game-specific
  knowledge. If no such rule can be found for a family, this assumption fails and the
  project owner decides how to proceed.
- **The player and the location can be found without running code.** Both families keep
  them in known places. Research must confirm this for each family as a rule, not for
  each game.
- **"Inform 6" means the standard library's world model.** A game compiled with Inform 6
  but with a different library is unrecognised.
- **Infocom's version 3 files are the ZIL family for this feature.** Infocom's later
  versions follow similar conventions and may be recognised if the same rules hold, but
  only Zork I is in the repository to test against, so nothing is promised for them.
- **Inform 7 is out of scope.** Its story files are compiled through Inform 6 but with a
  different library. They are expected to be unrecognised.
- **Scope is an approximation by decision.** The games decide scope and light by running
  code, which FR-003 forbids. The project owner chose a tree-based approximation, labelled
  as such, over leaving the query out.
- **Conditional exits are reported without a destination by decision.** The project owner
  chose this over omitting them or declaring the whole room unknown. An exit governed by
  a stored value alone is not conditional in this sense and is evaluated (FR-020).
- **A change of player in an Inform game is not always detected.** The project owner
  accepted this as a recorded deviation: research R3.
- **In Zork I the inventory is "unknown" at a few prompts**, where the family's rule for
  finding the player ties. The project owner accepted this: research R4.
- **The object tree's attributes and properties are given by number.** What a number
  means is a family matter: where a family fixes the meaning (Inform's `light`,
  `container`, `open`), the adapter uses it; the interface does not name every attribute
  of every game.
- **Out of scope**: translating taps into commands, deciding which objects to show a
  player, any user interface, the C bindings and the phone shells. Those are the
  experience layer and the first demo. Writing to the game in any way, including
  "safe" writes, is out of scope permanently (constitution Principle IV).
- **The specification lives in this repository** as Markdown until it stabilises, as the
  constitution says.
- **Performance** matters only so far as SC-010: the queries are asked once per prompt,
  not continuously.
