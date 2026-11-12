# Feature Specification: Game Packs

**Feature Branch**: `006-game-packs`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "Start the spec for game packs." Decisions made by the
project owner on 2026-11-12: the first version of the pack format states two kinds of
fact, which attribute means a door is open and which flags have a stored value that is
not to be trusted; the first packs are for Zork I and Mini-Zork; and a pack's answer
reaches the compass in the app. The rules a pack must keep are those of the
constitution's Principle IV as amended in version 2.0.0.

## User Scenarios & Testing *(mandatory)*

The introspection interface reads a game's stored facts by rules that hold for a whole
family of games. Corpus testing (feature 005) found two places where no rule of the
family can read a fact, because its meaning differs from game to game:

- **A door exit.** The room's table names the door and the room behind it. Whether the
  door is open is one attribute of the door, and each game numbers its attributes
  itself: opening a door sets attribute 11 in Zork I and attribute 10 in Mini-Zork. So
  today every door exit is reported "conditional", and the compass says "may be
  blocked".
- **A stale flag.** Some exits are open while a flag is set. In two rooms of Zork I the
  room sets that flag afresh when a command begins, so the stored value is one command
  out of date, and the exit is reported wrongly. These are in the corpus ledger as known
  wrong reports.

A **game pack** is a small published file of such facts about exactly one story file.
With it, a door exit is reported as open or closed, and an exit that hangs on a stale
flag is reported honestly as conditional. Without it, nothing changes.

Four words are used throughout:

- **Pack**: the file of facts about one story file.
- **Entry**: one fact in a pack, of a kind the introspection specification defines.
- **In force**: a pack is in force for a running game when its identity matches that
  game's story file exactly.
- **Marked**: an answer is marked when it would have been different without the pack.

### User Story 1 - A door exit is reported as open or closed (Priority: P1)

Someone using the introspection interface on Zork I, with its pack in force, asks for
the exits of a room that has a door. In place of "conditional" they are told the door's
state, open or closed, and the room behind it, and that the answer rests on the pack.

**Why this priority**: It is the whole idea in its smallest form: one entry, one game,
one kind of exit made definite. Along Zork I's walkthrough, door exits were 57 of the
109 trials reported conditional, and one attribute predicted all 57.

**Independent Test**: With Zork I and its pack, ask for the exits at a room with a door
before and after the door is opened; the answer changes from closed to open, names the
same room both times, and is marked. With the pack taken away, the answer is
"conditional", as it is today.

**Acceptance Scenarios**:

1. **Given** Zork I with its pack in force and a room whose exit is through an open
   door, **When** the exits are asked for, **Then** that exit is reported as an open
   door leading to the room the table names, and is marked.
2. **Given** the same room with the door closed, **When** the exits are asked for,
   **Then** that exit is still listed, reported as a closed door with the room behind
   it, and is marked.
3. **Given** Zork I with no pack, **When** the exits are asked for anywhere, **Then**
   every answer is what it was before this feature.
4. **Given** a pack whose identity differs from the running story in digest, release or
   serial number, **When** the exits are asked for, **Then** the pack is not in force
   and every answer is what it is with no pack.
5. **Given** any game played from a script with and without its pack, **When** the two
   transcripts are compared, **Then** they are identical.

---

### User Story 2 - An entry is proven or the build fails (Priority: P2)

Someone who writes or changes a pack runs the corpus. Every entry is tested against the
story file by trials: the pack's claim at each place is set against what a throwaway
copy of the game did. An entry that a trial contradicts, or that no trial tests, fails.

**Why this priority**: The constitution admits packs only because an entry can be proven
and need not be taken on trust. Without this story a pack is a guess with a file name.

**Independent Test**: Give Zork I a pack that names the wrong attribute as "open"; the
corpus run fails and names the entry and a case that contradicts it. Give it the right
one; the run passes.

**Acceptance Scenarios**:

1. **Given** a pack that says a door is open by an attribute that is in fact something
   else, **When** the corpus runs, **Then** it fails and names the entry, the room, the
   direction, what was reported and what the copy did.
2. **Given** a pack with the right attribute, **When** the corpus runs, **Then** every
   door exit reported open let the copy through to the room named, every door exit
   reported closed kept the copy where it was, and the run passes.
3. **Given** an "open" entry for a story in which the corpus reaches a door only in one
   state, **When** the corpus runs, **Then** the entry is reported as not proven and the
   run fails, since trials in one state cannot tell the attribute from many others.
4. **Given** an entry naming a flag as not to be trusted, **When** the corpus runs with
   and without the pack, **Then** without it at least one report that hangs on the flag
   is wrong, and with it none is; otherwise the entry is reported as unneeded and the
   run fails.
5. **Given** a pack for a story whose file is absent, **When** the tests run, **Then**
   the pack's tests are reported as ignored, story absent, and not as passed.

---

### User Story 3 - A stale flag no longer produces a wrong report (Priority: P3)

Someone using the introspection interface on Zork I, with its pack in force, asks for
the exits in one of the two rooms whose exit hangs on a flag the room sets afresh each
command. Today the answer follows the stored flag and can be wrong. With the pack the
exit is reported as conditional, and marked.

**Why this priority**: It turns known wrong answers into honest ones, and it shows that
the format carries more than one kind of fact. It changes fewer answers than User
Story 1.

**Independent Test**: With Zork I and its pack, the corpus ledger no longer needs any
entry whose cause is a stale flag, and the run passes with those entries removed.

**Acceptance Scenarios**:

1. **Given** Zork I with its pack in force and a room whose exit hangs on a flag the
   pack names, **When** the exits are asked for, **Then** that exit is reported as
   conditional whatever the flag holds, and is marked.
2. **Given** the same room and an exit that hangs on a flag the pack does not name,
   **When** the exits are asked for, **Then** the answer follows the stored flag, as it
   does today, and is not marked.
3. **Given** the corpus ledger with its stale-flag entries removed, **When** the corpus
   runs with the pack, **Then** it passes.

---

### User Story 4 - The format is published, and a second game uses it (Priority: P4)

Someone writing another interpreter reads the introspection specification and finds the
pack format there: how a pack names its story, which entries exist, what each one
changes in which answer, and what to do with a pack that is malformed or does not
match. They can read this project's packs with no other source. Mini-Zork has a pack
whose "open" attribute is a different number from Zork I's, read by the same rules.

**Why this priority**: Publishing is what keeps packs portable, and a second game with a
different number is the evidence that the fact is truly per game. It rests on the first
three stories.

**Independent Test**: Every statement in the specification's section on packs has a test
that cites it. The Mini-Zork pack passes User Story 2's proof wherever the story file is
present.

**Acceptance Scenarios**:

1. **Given** the introspection specification, **When** its section on packs is read,
   **Then** it defines the file's form, the identity rule, each kind of entry and its
   effect on each answer, the mark, and the treatment of a malformed pack, with a worked
   example that holds no text of any game.
2. **Given** a pack that is malformed in any way the specification names, **When** it is
   checked, **Then** it is refused as a whole, with the line at fault; and a running
   game given that pack behaves as if it had none.
3. **Given** Mini-Zork and its pack, **When** the corpus runs where the story file is
   present, **Then** its "open" entry is proven as in User Story 2.
4. **Given** the packs in the repository, **When** they are searched, **Then** they hold
   only the format's own words, numbers and a story's identity: no name, word or line
   of any game.

---

### User Story 5 - The compass shows a door as open or closed (Priority: P5)

A player of Zork I in the app comes to a room with a door. Where the compass said "may
be blocked", it now shows the way as open, like any other way out, or as closed. A
closed door is still a place on the compass: touching it types the direction, and the
game answers as it would have.

**Why this priority**: It is where a player sees what a pack buys. It depends on all the
stories before it, and the interface is useful without it.

**Independent Test**: In the app, walk to a room with a closed door: the compass shows
that way as closed, and reads it aloud as closed. Open the door by typing: the compass
shows the way as open. Touch it: the player goes through.

**Acceptance Scenarios**:

1. **Given** Zork I in the app at a room with a closed door, **When** the compass is
   shown, **Then** that direction is shown as closed, in a way that differs to the eye
   and to a screen reader from an open way, from "may be blocked" and from no way.
2. **Given** the door is opened by a typed command, **When** the compass is next shown,
   **Then** that direction is shown as an open way.
3. **Given** a closed door on the compass, **When** the player touches it, **Then** the
   direction is typed as a command, as for any other direction, and nothing else is
   done.
4. **Given** an exit decided by the game's own code, **When** the compass is shown,
   **Then** it still says "may be blocked": a pack does not change it.

---

### Edge Cases

- **An open door the game still refuses.** The game has the last word: something else
  may block the way, or the player may be in the dark. A door reported open that did
  not let a trial through is a wrong report like any other, and goes in the ledger with
  its cause, provided the cause is not the entry. If the attribute itself was not what
  the pack says, no ledger entry excuses it.
- **A door exit whose door or room is not an object's number.** It is reported unknown,
  as a fixed exit with a bad room is today.
- **Two packs that name the same story.** Refused when the packs are checked.
- **A pack for a story the corpus does not hold.** Refused when the packs are checked:
  an entry with no test may not exist.
- **A pack naming an attribute or a flag the story cannot have** (an attribute number
  beyond the version's range, a flag number that is not a global variable). Refused as
  malformed.
- **A pack with an entry of a kind the reader does not know.** The whole pack is
  refused; a reader never applies half a pack.
- **A pack for a family whose rules have no use for an entry** (an "open" entry for a
  game of the Inform family, which reads doors by its own rule). Refused when checked.
- **A flag named as not to be trusted that also happens to hold the right value.** The
  exit is conditional regardless; the pack says the value cannot be relied on, not that
  it is wrong.
- **The game is saved, restored or restarted.** A pack stays in force: it belongs to
  the story file, not to a session.
- **A story file that matches in digest but is given a different release or serial.**
  Cannot happen for an unaltered file, since both are inside it; the rule of three is
  kept because the constitution states it.

## Requirements *(mandatory)*

### Functional Requirements

**The pack and its identity**

- **FR-001**: A pack MUST name exactly one story file by SHA-256 digest, release number
  and serial number, and MUST be in force only when all three match the running story.
- **FR-002**: A pack MUST state the version of the pack format it is written in. A
  reader MUST refuse a pack of a version it does not know.
- **FR-003**: A pack MUST hold only its identity, its format version and entries. It
  MUST NOT hold conditions, scripts, or any name, word or line of a game.
- **FR-004**: A pack MUST be plain text that a person can read and write by hand, and
  that changes legibly in a review.
- **FR-005**: A pack that is malformed in any respect MUST be refused as a whole. A
  check of the packs MUST name the file and line at fault. A running game given a
  refused pack MUST behave as it does with none: no query may fail because of a pack.
- **FR-006**: The interface MUST let its user ask whether a pack is in force for the
  running game.

**The first vocabulary**

- **FR-007**: The format MUST define an entry stating which attribute, in this story,
  is set on a door while it is open. It applies to stories of the ZIL family.
- **FR-008**: The format MUST define an entry stating that a named flag's stored value
  is not to be trusted. A pack may hold several. It applies to stories of the ZIL
  family.
- **FR-009**: The format MUST define no other kind of entry in this feature. A new kind
  is added only by changing the introspection specification.

**What a pack changes in an answer**

- **FR-010**: With an "open" entry in force, an exit through a door MUST be reported
  with the door's state, open or closed, the door's object, and the room the table
  names behind it. A closed door MUST remain listed as an exit and MUST be told apart
  from a direction that is not an exit.
- **FR-011**: With a flag named as not to be trusted, every exit that hangs on that flag
  MUST be reported as conditional, whatever the flag holds.
- **FR-012**: Every answer, or part of one, that would have been different with no pack
  MUST be marked as resting on the pack. No other answer may be marked.
- **FR-013**: With no pack in force, every answer of every query MUST be exactly what
  it was before this feature, including "unknown" and "conditional".
- **FR-014**: A pack MUST NOT change what a game does. A game played from the same
  script with and without its pack MUST give identical transcripts.
- **FR-015**: Reading a pack's fact MUST remain a read of the story's stored state at
  the moment of the query; nothing may be run or tried to answer.

**Proof**

- **FR-016**: Every entry in every pack in the repository MUST have a test against the
  story file the pack names, made by corpus trials.
- **FR-017**: An "open" entry is proven when, over the corpus's trials on its story,
  every door exit reported open let the copy through to the room named and every door
  exit reported closed kept the copy where it was, with at least one trial in each
  state. Fewer than one in each state is "not proven" and MUST fail.
- **FR-018**: A flag entry is proven when at least one report that hangs on the flag is
  wrong without the pack, and none is wrong with it. A flag entry that changes no
  verdict MUST fail as unneeded.
- **FR-019**: A wrong report on a marked answer MAY be recorded in the ledger only with
  a cause that is not the entry itself. The ledger's check MUST show which entries
  excuse a marked answer, so that the owner can see them.
- **FR-020**: A pack whose story file is absent MUST have its tests reported as
  ignored, story absent. No test and no CI job may download a story.
- **FR-021**: The corpus MUST be able to run with packs set aside, and that run MUST
  give the reports of feature 005 unchanged apart from any change this feature makes
  to their form.
- **FR-022**: A story's corpus report MUST say whether a pack was in force and how many
  trials were on marked answers, with their verdicts.
- **FR-023**: A check MUST refuse two packs for one story, a pack for a story the
  corpus manifest does not list, and an entry its story's family has no use for.

**Publication**

- **FR-024**: The introspection specification MUST define the pack format completely:
  the file's form, the identity rule, each kind of entry, its effect on each answer,
  the mark, and the treatment of a malformed or unmatched pack, such that another
  interpreter could read this project's packs from the specification alone.
- **FR-025**: The specification's version MUST be raised by its own versioning rule,
  and the specification MUST say which of its statements changed.
- **FR-026**: Every statement the specification makes about packs MUST have a test
  that cites it.

**The first packs**

- **FR-027**: The repository MUST hold a pack for Zork I with an "open" entry and an
  entry for each flag behind a stale-flag case in the corpus ledger.
- **FR-028**: The repository MUST hold a pack for Mini-Zork with an "open" entry. The
  story file stays fetched and uncommitted; the pack, which holds none of its text, is
  committed.
- **FR-029**: The ledger entries whose cause is a stale flag MUST be removed once the
  Zork I pack makes their reports conditional.

**The compass**

- **FR-030**: The experience layer's compass MUST show an exit through an open door as
  an open way, and one through a closed door as closed: distinct, to the eye and to a
  screen reader, from an open way, from "may be blocked" and from no way.
- **FR-031**: Touching a closed door on the compass MUST type that direction as a
  command, like any other direction, and MUST do nothing else.
- **FR-032**: The app MUST carry the pack for each story it carries, and MUST apply a
  pack only by the identity rule.
- **FR-033**: With no pack in force the compass MUST be exactly what it was before
  this feature.

**Layering**

- **FR-034**: The core MUST NOT read, hold or know of packs. A pack is given to the
  introspection interface by its host; the interface itself does no reading of files.
- **FR-035**: No code may be written for one game. Everything that differs between
  games MUST be in a pack.

### Key Entities

- **Pack**: a file of facts about one story file. Has an identity, a format version and
  entries.
- **Identity**: a story file's SHA-256 digest, release number and serial number.
- **Entry**: one fact, of a kind from the vocabulary. In this feature: the attribute
  that means open; a flag not to be trusted.
- **Vocabulary**: the kinds of entry the introspection specification defines, each with
  the answers it changes and how.
- **Door exit**: an exit whose table names a door and the room behind it. With an
  "open" entry it has a state: open or closed.
- **Mark**: what an answer carries when it rests on a pack.
- **Proof**: the outcome of testing an entry by trials: proven, contradicted, not
  proven, or unneeded. Only "proven" passes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On Zork I with its pack, every trial of a door exit is on an answer of
  "open" or "closed", none on "conditional". The trials reported conditional fall from
  113 to 56 or fewer.
- **SC-002**: On Zork I with its pack, no door exit reported open or closed is wrong
  without a ledger entry, and no such ledger entry names the entry as its cause.
- **SC-003**: The corpus ledger holds no entry whose cause is a stale flag.
- **SC-004**: A pack naming any attribute but the right one as "open" fails the corpus
  run on Zork I. This is shown for every other attribute number.
- **SC-005**: With packs set aside, every story's verdict counts equal those of feature
  005's reports.
- **SC-006**: Every walkthrough the project holds gives the same transcript, byte for
  byte, with and without packs.
- **SC-007**: The Zork I and Mini-Zork packs name different attributes as "open", are
  read by the same rules, and each is proven where its story is present.
- **SC-008**: No pack in the repository holds a word that is in any committed story's
  dictionary or object names, other than the format's own words.
- **SC-009**: A person who has read only the introspection specification can write a
  pack for a story by hand that the check accepts.
- **SC-010**: In the app, on Zork I, a player can see a door's state on the compass
  change from closed to open after opening it, and the screen reader says which it is.
- **SC-011**: Every query still answers within the time limit feature 003 set, with a
  pack in force.
- **SC-012**: Every test of the earlier features passes; those that fix the form of an
  answer or a report are changed only where this feature changes that form, and each
  such change is listed.

## Assumptions

- **The amendment is in force.** This feature rests on constitution version 2.0.0,
  which at the time of writing is proposed in a pull request and not yet merged. The
  feature's branch is cut from that proposal. If the amendment changes before it is
  merged, this specification is revisited.
- **A door reported open may still be refused by the game.** The introspection
  specification already says the game has the last word. "Open" is the door's stored
  state, not a promise of passage.
- **Mini-Zork's proof needs a door in both states.** The crawl reaches Mini-Zork's
  doors only while they are closed. Proving its entry therefore needs a few commands
  recorded for that story that open one. Those commands are the project's own words,
  not the game's text. How they are recorded is for the plan.
- **Mini-Zork's pack is proven only where its file is present.** That is the owner's
  machine, not CI. The constitution's rule that every entry has a test is read as
  satisfied by a test that is reported as ignored where the story is absent; the owner
  chose this on 2026-11-12 by choosing Mini-Zork.
- **Mini-Zork may have no stale flag.** None has been looked for. Its pack starts with
  the one entry.
- **The Inform family needs no "open" entry.** Its library gives doors and "open" fixed
  attribute numbers, so a rule of the family could read them. Doing so is a change to
  the family's rules and is not part of this feature.
- **The mark is for users of the interface.** The app need not show a player that an
  answer rests on a pack.
- **Out of scope**: an attribute meaning "lit" and any query about light; look-ahead in
  the compass; packs for Zork II, Zork III or any other story; a tool that proposes
  entries for a new game; an action on the compass that opens a door; exits decided by
  a routine, which stay "may be blocked"; authored presentation content (art, map
  layouts, icons), which the constitution leaves to a separate amendment; a graphical
  UI.
