# Research: Introspection Interface

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Findings were made by reading the committed story files (Zork I; Advent as versions 5 and
8; czech as versions 3, 5 and 8; praxix) and the Advent and Zork I save files with a
throwaway script, and by reading the Inform 6.12 library source installed with the
compiler. Where a claim reaches beyond the files examined, it says so.

**Revised after independent review.** A second reviewer re-tested every claim here
against the files, at 330 prompts of the Zork I walkthrough and 171 of Advent's in each
build. Three rules were wrong as first written (R2, R7, and a supporting claim in R1) and
are corrected below; R3 to R6 held. The project owner then decided the four open
questions; R12 records the decisions.

The spec's two assumptions are answered in R1 to R4.

## R1. Recognising Inform 6 with the standard library

**Decision**: A story file is of the Inform 6 family when all of these hold in the file
as shipped:

1. Header bytes 0x3C to 0x3F are a digit, a full stop and two digits: the compiler
   version Inform writes there (`6.44` in Advent and czech, `6.31` in praxix).
2. Objects 1 to 4 are named `Class`, `Object`, `Routine` and `String`: the four
   metaclasses the compiler creates in every file.
3. There is exactly one object with each of the names `compass`, `(self object)`,
   `(darkness object)` and `(Inform Library)`.
4. Every child of `compass` provides property 21 (`door_dir`) with a value between 4 and
   63 that is not a property the compass child itself provides, and no two children
   share a value.
5. The attributes are numbered as the library declares them: `(self object)` has exactly
   attributes 0, 3, 16 and 23 (`animate`, `concealed`, `proper`, `transparent`), `compass`
   has exactly 3, and every child of `compass` has exactly 17 (`scenery`).
6. Exactly one global holds the number of `(Inform Library)`. That global is the
   library's `location` variable (R3).

**Rationale**: 1 and 2 identify the compiler; czech and praxix pass them. 3 to 6 identify
the library's world model; czech and praxix fail at 3, as they should: they are compiled
with Inform but have no library. Rule 5 was added in review: a game that declares an
attribute of its own before including the library would keep the property numbers and
shift every attribute, and without rule 5 would be recognised and then misread.

Confirmed on both builds of Advent: the compass children hold 7 to 18 in property 21,
each provides properties 1, 2, 3, 21, 35, 37, 41, 46 and 48 (`in` and `out` lack 1), and
the attributes are as rule 5 says.

**Limits**:

- Three of the four names are literal strings in the library's `parser.h`. `compass` is
  in `english.h`, the language file, so a translated game may name it otherwise and is
  then unrecognised. That is the safe outcome.
- Rules 4 and 5 fix property and attribute numbers by the order of declarations in the
  library's `linklpa.h` (name 1, two compiler-internal, then `before` 4, `after` 5,
  `life` 6, `n_to` 7 to `out_to` 18, `door_to` 19, `with_key` 20, `door_dir` 21, and on
  to `description` 35, `found_in` 39, `short_name` 46). Checked against library 6.12.8
  only. A library that orders them otherwise fails the rules and is unrecognised.
- `worn` (attribute 26) is on no object at any point of the Advent walkthrough. Its
  number rests on the declaration order alone.

## R2. Recognising Infocom/ZIL

**Decision**: A story file is of the ZIL family when all of these hold in the file as
shipped:

1. It is version 3 and fails rule 1 of R1 (bytes 0x3C to 0x3F are not an Inform version).
2. Dictionary entries are 7 bytes: 4 of text and 3 of data.
3. At least four dictionary words have bit 4 (0x10, "direction") set in their first data
   byte. For such a word the direction's property number is the second data byte when
   the low two bits of the first are 3, and the third data byte otherwise.
4. Leaving aside objects with no parent, every object providing any of those properties
   has the same parent, and that parent has no parent itself (ZIL's `ROOMS`). Call these
   objects rooms.
5. Every such property on every room is 1 to 5 bytes long, and every 1-byte value is the
   number of a room.

**Rationale**: Rules 2 and 3 are ZIL's dictionary layout. In Zork I the 25 direction
words give properties 19 to 31 (`land` is `13 13 00`, second byte; `out` is `18 fd 14`,
third byte). Rules 4 and 5 check that those properties really are exit tables before
anything is reported from them. In Zork I 110 rooms provide them, all children of object
39, with 277 exits of length 1, 37 of length 2, 7 of length 3, 25 of length 4 and 6 of
length 5.

**Corrected in review**: object 39, `ROOMS` itself, also provides one of the direction
properties (number 21, one byte, value 39) and has no parent. As first written, rules 4
and 5 rejected Zork I. Rule 4 now leaves parentless objects aside.

czech for version 3 has an empty dictionary and is not accepted.

**Limit**: Tested on one file. The rules describe conventions that Infocom's other
version 3 games are believed to share, but the repository has only Zork I, and the spec
promises nothing beyond it. Versions 4 and 5 of ZIL use a different dictionary flag
layout and are not attempted.

## R3. Finding the player and the location: Inform 6

**Decision**:

- **Player**: the object named `(self object)`.
- **Location**: the object reached by following the player's parents until one has no
  parent. If the player has no parent, the location is unknown.
- **Holder**: if the player's parent is not the location, that parent (a vehicle, a
  cage).
- **Darkness**: the room is dark exactly when the `location` variable (R1 rule 6) holds
  the number of `(darkness object)`.
- **Consistency checks**: every player-dependent query answers "unknown" if the
  `location` variable holds neither the location found from the tree nor the darkness
  object, or if no global at all holds the player's number.

**Evidence**: At 173 states in each build, and four dark ones (lamp off in the Hall of
the Mountain King, and after walking on in the dark), the top of the player's parent
chain was the room every time; the variable held the same room when lit and the darkness
object when dark, while the tree still held the true room.

**Display names**: an Inform object's stored name is not always what the game prints.
Advent's maze rooms are stored as `(Alike_Maze_1)` and print as "Maze"; compass
directions are stored as `(n_obj)`. The printed name is the `short_name` property (46)
when it holds a string (R7); when it holds a routine the printed name is unknown.

**Known gap, accepted by the project owner**: the library lets a game make another object
the player (`ChangePlayer`). The library's `player` variable has no value in the file as
shipped (confirmed in `parser.h`), so it cannot be located the way `location` can. The
second consistency check catches a change of player in most cases, since the variable
that pointed at the original player then points elsewhere. It does not catch one where
some other variable happens to hold the original player's number and both are in one
room; the inventory reported is then the original player's. Advent never changes the
player. This is recorded as a deviation in the plan and as a stated limit in the
published specification.

## R4. Finding the player and the location: ZIL

**Decision**:

- **Location**: the object whose number is in global 0, provided it is a room (R2). The
  Standard requires version 3 games to keep the location there for the status line
  (§8.2.2), so this is exact.
- **Player**: among the objects that had no parent and no children in the story file as
  shipped, those now inside the location (directly, or inside something that is) and not
  inside another such object. If exactly one of them has its number in some global other
  than global 0, it is the player. If none, the player is unknown (`no-player`); if more
  than one, unknown (`ambiguous-player`). A tie is never broken.

**Rationale**: ZIL games create the player object unplaced and move it into the starting
room at start-up, and keep its number in the `WINNER` variable, whose index differs from
game to game.

**Evidence**: Implemented as written and run at 330 prompts of the Zork I walkthrough
and the five reference saves: 329 correct, 6 unknown by tie, none unknown by absence,
none wrong. The ties are with other objects that start unplaced and are later made: the
red hot bell at the Entrance to Hades (2 prompts), the huge diamond (3) and the diamond
with the bauble (1). Global 0 matched the tree at all 335.

**Decision of the project owner**: accept the ties. Location is unaffected; at about 2%
of prompts the inventory, the holder and the carried part of scope are unknown. The
tests require no wrong answer and at most 3% of prompts unknown.

**Not tested**: play off the walkthrough's path (death, other seeds).

## R5. Exits: ZIL

**Decision**: For each direction property a room provides, by its length:

| Length | ZIL's name | Contents | Reported as |
| --- | --- | --- | --- |
| 1 | UEXIT | room | fixed exit to that room |
| 2 | NEXIT | address of a message | not an exit |
| 3 | FEXIT | routine, zero byte | conditional exit |
| 4 | CEXIT | room, variable number, address of a message | fixed exit to the room if the variable is non-zero; otherwise not an exit |
| 5 | DEXIT | room, door object, address of a message, zero byte | conditional exit |

The words for an exit are the dictionary words whose direction property is that one.

**The CEXIT variable** is a Z-machine variable number: 16 is the first global, so the
global's index is the byte minus 16. A byte below 16 would name the stack or a local,
which mean nothing outside a routine; such an exit is conditional.

**Evidence**: All five forms were checked on real rooms. For CEXIT, the Troll Room's
exits name variable 37, and global 21 goes from 0 to 1 at "kill troll with sword"; the
same holds for the rope, the rainbow, the reservoir, Hades, the cyclops and the barrow.
All 12 distinct variables are 0 in the file as shipped, and at all 14 CEXITs the
walkthrough passes through the flag was 1. An attempt to catch a stale flag (the Altar
while carrying the coffin) found the flag 0 and the game refusing.

**Decision of the project owner**: evaluate the flag. A CEXIT's condition is a stored
value, not code. A DEXIT depends on whether the door is open, and which attribute means
"open" differs from game to game in ZIL (R9), so it stays conditional.

## R6. Exits: Inform 6

**Decision**: The direction properties are the `door_dir` values of the compass's
children. For each direction property the location provides, by its value, testing in
this order:

| Value | Reported as |
| --- | --- |
| The property is not two bytes long | that direction is unknown |
| 0 | not an exit |
| the number of an object with the `door` attribute (5) | conditional exit |
| the number of an object with no parent that provides at least one direction property or has the `light` attribute | fixed exit to that room |
| the number of any other object | conditional exit |
| a routine (R7) | conditional exit |
| a string (R7) | not an exit: the string is the refusal |
| not determinable (R7) | that direction is unknown |

**Words**: a compass child's `name` property (1) lists dictionary addresses; those are
the direction's words. The children for `in` and `out` (direction properties 17 and 18)
have no `name`: the library takes those two as verbs. For them the words are `in` and
`out` when the dictionary has those words marked as verbs, and otherwise the direction is
reported with no words.

**Evidence**: Advent's 458 direction properties, all two bytes, are 419 objects, 26
strings and 13 routines. Of the 419, 417 are parentless and every one of the 125 distinct
targets provides direction properties of its own; the other 2 are the two steel grates,
which have `door`. The door test comes first because a door that is in several rooms has
no parent either.

The library also consults `cant_go` and may be overridden by `before` rules that react to
`Go`. Those are code. An exit so overridden is still reported from the property; the
specification says exits are what the room's table says, and the game has the last word.

## R7. Telling a routine from a string in an Inform file

**Problem**: A property value that is not an object is the packed address of a routine or
of a string, and the file's header does not say where one region ends and the other
begins. The compiler's own test compares against a constant compiled into its code.

**Decision**: Inform places all routines before all strings (confirmed by the compiler's
own memory map for Advent). Consider the two-byte values in the properties of all objects
in the file as shipped, **leaving out properties 1, 2 and 3**, that are above the last
object's number and whose unpacked address lies in high memory. Then:

- Let S be the lowest such value whose target's first byte is above 15. A routine starts
  with its count of locals, 0 to 15, so such a target cannot be a routine. Every value at
  or above S is a string.
- Let R be the highest such value among properties `before`, `after` and `life` (4, 5,
  6), which the library only ever calls. Every value at or below R that is not an object
  is a routine.
- A value strictly between R and S is not determinable.
- If S is not above R, the layout is not as expected and every non-object value is not
  determinable.

**Corrected in review**: the first wording took values from all properties. Property 1
holds dictionary addresses and property 3 the address of a table; multiplied as packed
addresses they land in code, which on the version 5 build put S below R and classed all
13 routine exits as strings. The version 8 build escaped only because its multiplier is
larger. Properties 1, 2 and 3 are the compiler's own and never hold a routine or string.

**Evidence**: With that exclusion, 549 values in each build. Version 5: S = 0x687f, R =
0x61dd. Version 8: S = 0x34dc, R = 0x3172. Nothing between, and no value at or below R
has a first byte above 15.

**Limits**: S is not the start of the strings area: in Advent six strings precede it,
none of them used as a property value. A string that begins with a space or an
abbreviation has a first byte of 15 or less and does not help find S. A file whose
earliest strings are such, and are used as refusals, would have some directions unknown.
That is the honest answer for them.

## R8. Inventory and the object tree

**Decision**: Inventory is the player's children, each with its own children, in the
tree's order. For Inform, a carried object with the `worn` attribute (26) is marked worn.
The object tree and dictionary word list come straight from the Z-machine's tables
(Standard §12, §13) for any story file.

Damaged trees: a walk stops, and that branch is unknown, when an object number is 0 or
past the last object, or when more objects have been visited than exist (a loop).

**The last object** is found as the Standard's tools find it: the object table ends
where the first property table begins.

## R9. Scope, and what each family records

**Decision**: Scope is every object in the location and every object carried, walked
through the tree, each tagged with how it is reached: in the room, inside or on another
object (named), or carried. Then, by family:

| | Inform 6 | ZIL |
| --- | --- | --- |
| Is the room lit? | Known (R3) | Unknown |
| Is a container open or see-through? | Known: `container` 4, `open` 14, `transparent` 23, `supporter` 20 | Unknown |
| Scenery or concealed? | Known: `scenery` 17, `concealed` 3 | Unknown |

- **Inform**: contents of a closed, opaque container are left out. In darkness only what
  is carried is listed. An object whose `found_in` property (39) is a plain list of
  objects that includes the location is added, as in the room (Advent's stream and
  forest are such). `found_in` holding a routine, and `add_to_scope`, are code, and are
  part of why the answer is approximate.
- **ZIL**: attribute numbers are assigned by the compiler game by game, so "open",
  "light" and "invisible" cannot be told from the file. Everything in the tree under the
  location is listed, and each object inside another is marked "visibility unknown". The
  answer's light is "unknown".

**This changed the spec, with the project owner's agreement.** FR-022, FR-023, SC-006
and User Story 3's scenarios assumed every family records light and open containers.
They now apply the closed-container and darkness rules where the family records them,
and otherwise require the contents to be listed and marked.

## R10. Dictionary parts of speech

- **ZIL**: first data byte: 0x80 noun, 0x40 verb, 0x20 adjective, 0x10 direction, 0x08
  preposition, 0x04 buzz word. Confirmed with examples of all 12 values the byte takes in
  Zork I.
- **Inform**: first data byte (`#dict_par1`): 0x80 noun, 0x04 plural, 0x02 meta, 0x01
  verb. Bit 0x08 marks a word used in a grammar line, which is usually a preposition but
  not always (Advent's `fee`, `fie`, `foe`, `foo`); it is reported as `grammar-word`, not
  as a preposition. Bit 0x40 accompanies every verb in Advent and is not reported. A word
  is a direction if it is in a compass child's `name`. No Advent word is marked plural,
  so that bit rests on the library's documentation.
- **Unrecognised**: unknown.

## R11. Where the code goes, and how "read-only" is guaranteed

**Decision**: A new crate, `zmachine-introspect`, depending only on the core. The core
gains one read-only module giving byte-level access to current memory, the story file as
shipped, and the header. Everything the queries need is computed in the new crate from
those bytes.

Every query takes a shared reference to the machine. The core has no interior
mutability, so the compiler itself guarantees no query alters it; a test asserts the core
contains no `Cell`, `RefCell`, `static mut` or `UnsafeCell`, so the guarantee cannot
quietly lapse. SC-001 is then confirmation, not the only defence.

**Rationale**: The constitution's layering puts the introspection interface between the
core and the experience layer. Object and text decoding exist inside the core, but
exposing them would freeze internal types into the public interface; the new crate
decodes for itself from bytes, and the two are checked against each other (SC-007 uses
the core's own decoder as the independent check).

**Alternatives considered**: Putting the queries in the core mixes family conventions
into the interpreter. Giving the new crate its own copy of memory costs a copy per
prompt for no gain.

## R12. Decisions by the project owner (2026-11-11)

1. **Scope for ZIL**: list everything under the room, mark what is inside another object
   as of unknown visibility, and say light is unknown. The requirements were revised to
   match (R9).
2. **ZIL's player**: accept the ties as "unknown" (R4).
3. **Inform's change of player**: add the guard, keep the answers exact, record the gap
   as a deviation (R3).
4. **Flag exits in ZIL**: evaluate the flag (R5).

Still open, for the task list:

- **Expected exits for Zork I** (FR-039) are worked out from the game's ZIL source, which
  is MIT-licensed but not in this repository. `stories/zork1/SOURCE.md` names the
  repository it came from; the task reads that revision and records it in the
  expectations' README. The source need not be committed.

## R13. Form of the answers and of the specification

**Decision**: The specification, `docs/introspection.md`, describes each answer as a
record of named fields with plain types (number, text, list, one of a set of words), and
gives each a JSON rendering, which is what `zm` prints. It carries a version starting at
0.1.0: the minor part changes when a query or field is added, the major part when one is
removed or changes meaning.

A test reads the query names from the document's headings and from the implementation
and fails if they differ (FR-030).

**Unknown in JSON**: wherever a value could be unknown, it is either the value or the
object `{"unknown": "<reason>"}`. That holds for a whole answer and for any field. Fields
the data model calls three-valued (`lit`, `worn`, `scenery`, `concealed`, `visibility`)
are the strings the data model lists. An answer is `{"exactness": "...", "value": ...}`.

**The document's skeleton comes first.** Tests must cite its sections (FR-040), so its
headings and section numbers are written before any query is, and filled in as each is
built.

**Rationale**: JSON is language-independent, already familiar to the Swift and Kotlin
shells that come later, and regular enough for tests to read (FR-035). The CLI writes it
by hand; no dependency is added.
