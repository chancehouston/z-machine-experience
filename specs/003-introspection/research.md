# Research: Introspection Interface

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Findings were made by reading the four committed story files and the Advent and Zork I
save files with a throwaway script, and by reading the Inform 6.12 library source
installed with the compiler. Nothing here was taken from memory alone; where a claim
reaches beyond the files examined, it says so.

The spec's two assumptions are answered in R1 to R4. One of them holds less firmly than
hoped for the ZIL family (R4).

## R1. Recognising Inform 6 with the standard library

**Decision**: A story file is of the Inform 6 family when all of these hold:

1. Header bytes 0x3C to 0x3F are a digit, a full stop and two digits: the compiler
   version Inform writes there (`6.44` in Advent and czech, `6.31` in praxix).
2. Objects 1 to 4 are named `Class`, `Object`, `Routine` and `String`: the four
   metaclasses the compiler creates in every file.
3. There are objects named `compass`, `(self object)`, `(darkness object)` and
   `(Inform Library)`. These names are literal strings in the library's `parser.h`, not
   in its language file, so a translated game has them too.
4. Every child of `compass` provides property 21 (`door_dir`) with a value between 4 and
   63 that is not a property the compass child itself provides, and no two children
   share a value.

**Rationale**: 1 and 2 identify the compiler; czech and praxix pass them. 3 and 4
identify the library's world model; czech and praxix fail them, as they should: they are
compiled with Inform but have no library. All four rules are about the family, none about
a game.

**Alternatives considered**: The banner text ("Inform v6.44 Library v6.12.8") is printed
by running code and is not stored in one place. A table of known serial numbers is
per-game data, which the constitution forbids.

**Limit**: Rule 4 fixes `door_dir` at property 21. That follows from the order of
declarations in the library's `linklpa.h` (name 1, two compiler-internal, then `before`
4, `after` 5, `life` 6, `n_to` 7 to `out_to` 18, `door_to` 19, `with_key` 20, `door_dir`
21). Checked against library 6.12.8 only. A library version that orders them otherwise
fails rule 4 and is unrecognised, which is the safe outcome.

## R2. Recognising Infocom/ZIL

**Decision**: A story file is of the ZIL family when all of these hold:

1. It is version 3 and fails rule 1 of R1 (bytes 0x3C to 0x3F are not an Inform version).
2. Dictionary entries are 7 bytes: 4 of text and 3 of data.
3. At least four dictionary words have bit 4 (0x10, "direction") set in their first data
   byte. For such a word the direction's property number is the second data byte when
   the low two bits of the first are 3, and the third data byte otherwise.
4. Every object providing any of those properties has the same parent, itself an object
   with no parent (ZIL's `ROOMS`). Call these objects rooms.
5. Every such property on every room is 1 to 5 bytes long, and every 1-byte value is the
   number of a room.

**Rationale**: Rules 2 and 3 are ZIL's dictionary layout: the first data byte holds part
of speech bits, and its low two bits say which value comes first. In Zork I the 25
direction words give properties 19 to 31. Rules 4 and 5 check that those properties
really are exit tables before anything is reported from them: in Zork I, 110 objects
provide them, all children of object 39, with lengths 1, 2, 3, 4 and 5 only.

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
- **Darkness**: the library's `location` variable is the one global whose value in the
  story file as shipped is the number of `(Inform Library)`. The room is dark exactly
  when that global now holds the number of `(darkness object)`.
- **Consistency check**: if that global holds neither the location found from the tree
  nor the darkness object, every player-dependent query answers "unknown".

**Rationale**: In Advent's saves the player's parent is the room at all three points,
and the global at index 1 holds the same room. The library keeps the tree and the
variable in step, and substitutes the darkness object in the variable when the room is
unlit; the tree still holds the real room, so both facts are stored and neither needs
code to be run.

**Limit, stated plainly**: the library lets a game make another object the player
(`ChangePlayer`). The library's `player` variable is set by code at start-up, so it
cannot be found from the file as shipped the way `location` can. If a game changes the
player and the old and new players are in the same room, the consistency check passes
and the inventory reported is the old player's. Advent never does this. The interface
specification must say so; a rule to find `player` is left as an open question (R12).

## R4. Finding the player and the location: ZIL

**Decision**:

- **Location**: the object whose number is in global 0, provided it is a room (R2). The
  Standard requires version 3 games to keep the location there for the status line
  (§8.2.2), so this is exact.
- **Player**: among the objects that had no parent and no children in the story file as
  shipped, those now inside the location (directly, or inside something that is) and not
  inside another such object. If exactly one of them has its number in some global other
  than global 0, it is the player. If none or more than one, the player is unknown.

**Rationale**: ZIL games create the player object unplaced and move it into the starting
room at start-up, and keep its number in the `WINNER` variable, whose index differs from
game to game. In Zork I eight objects start with no parent and no children; at all five
save points exactly one, the player, is in the location.

**This is the weakest finding.** The rule can be ambiguous: Zork I has other objects that
start unplaced (the diamond the machine makes, the broken lantern), and one of them lying
in the room while a parser variable holds its number would tie with the player. The rule
then answers "unknown"; it does not pick. It cannot, as far as I can see, answer wrongly
unless the true player is not in the location at all. SC-002 and SC-005 measure it at
every prompt of the walkthrough.

**If the measurement shows frequent "unknown"**: stop for the project owner. The options
then are to read the start-up code for the instruction that sets `WINNER` (reading code
is not running it, but it is fragile), or to accept a per-family table of variable
positions by game release, which the constitution would need amending to allow.

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

**Rationale**: These are the five forms of the ZIL exit table, and the lengths found in
Zork I (278, 37, 7, 25 and 6 of them). A CEXIT's condition is a variable's value, which
is a stored fact: the game's own rule for it is "go if the variable is set, else print
the message", with no code run. Evaluating it is within the project owner's decision,
which was about exits decided by code.

A DEXIT depends on whether the door is open, and which attribute means "open" differs
from game to game in ZIL (R9), so it is conditional.

**Alternatives considered**: Reporting a CEXIT as conditional is safe but throws away an
exact answer. Reporting the destination of a DEXIT or CEXIT "if it opens" was not what
the project owner chose.

## R6. Exits: Inform 6

**Decision**: The direction properties are the `door_dir` values of the compass's
children, and the words for each are that child's `name` property (dictionary
addresses). For each direction property the location provides, by its value:

| Value | Reported as |
| --- | --- |
| 0 | not an exit |
| the number of an object with no parent | fixed exit to that room |
| the number of an object with the `door` attribute (5) | conditional exit |
| the number of any other object | conditional exit |
| a routine (R7) | conditional exit |
| a string (R7) | not an exit: the string is the refusal |
| not determinable (R7) | that direction is unknown |

A location that provides no direction property but is not a plain room (it has a parent)
cannot occur, by R3. A property longer than two bytes makes that direction unknown.

**Rationale**: This is how the library's `GoSub` reads the property. Advent's 458
direction properties are 419 objects, 26 strings and 13 routines.

The library also consults `cant_go` and may be overridden by `before` rules that react to
`Go`. Those are code. An exit so overridden is still reported from the property; the
specification says exits are what the room's table says, and the game has the last word.

## R7. Telling a routine from a string in an Inform file

**Problem**: A property value that is not an object is the packed address of a routine or
of a string, and the file's header does not say where one region ends and the other
begins. The compiler's own test compares against a constant compiled into its code.

**Decision**: Inform places all routines before all strings. So:

- Let S be the lowest packed address, among all two-byte values in all properties of all
  objects that point into high memory, whose target's first byte is above 15. A routine
  starts with its count of locals, 0 to 15, so such a target cannot be a routine. Every
  value at or above S is a string.
- Let R be the highest packed address among values of the properties `before`, `after`
  and `life` (4, 5, 6), which the library only ever calls. Every value at or below R that
  is not an object is a routine.
- A value strictly between R and S is not determinable.

**Evidence**: In both builds of Advent, 555 such values; none falls between R and S, and
no value at or below R has a first byte above 15 (which would contradict the layout).
The lowest string found is the first string in the file.

**Limit**: A string that begins with a space or an abbreviation has a first byte of 15 or
less and does not help find S; a file whose earliest strings are all such, and are used
as refusals, would have some directions unknown. That is the honest answer for them.

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
  is carried is listed. `add_to_scope` and `found_in` (objects present in several rooms,
  such as Advent's stream) are properties that may hold routines; objects named by a
  plain list in `found_in` that includes the location are added, and the rest is part of
  why the answer is approximate.
- **ZIL**: attribute numbers are assigned by the compiler game by game, so "open",
  "light" and "invisible" cannot be told from the file. Everything in the tree under the
  location is listed, and each object inside another is marked "visibility unknown". The
  answer's light is "unknown".

**This changes the spec.** FR-023 as first written said room contents are "unknown"
where light is not settled. For ZIL that would make scope empty in every room of the
game the first demo uses. FR-023 and User Story 3 scenario 4 are revised: the contents
are listed and the answer carries "light unknown". The project owner should confirm.

## R10. Dictionary parts of speech

- **ZIL**: first data byte: 0x80 noun, 0x40 verb, 0x20 adjective, 0x10 direction, 0x08
  preposition, 0x04 buzz word.
- **Inform**: first data byte (`#dict_par1`): 0x80 noun, 0x08 preposition, 0x04 plural,
  0x02 meta, 0x01 verb. A word is a direction if it is in a compass child's `name`.
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

## R12. Open questions for the project owner

1. **FR-023 revised** as in R9. Confirm.
2. **Inform's `player` variable** (R3): accept the stated limit for now, or look for a
   rule before the interface specification reaches version 1.
3. **ZIL's player** (R4): the rule may answer "unknown" at some prompts. The measurement
   in the walkthrough decides whether that is acceptable.
4. **Expected exits for Zork I** (FR-039) must be worked out from the game's ZIL source,
   which is MIT-licensed but not in this repository. The task will fetch it to read, and
   record where from; it need not be committed.

## R13. Form of the answers and of the specification

**Decision**: The specification, `docs/introspection.md`, describes each answer as a
record of named fields with plain types (number, text, list, one of a set of words), and
gives each a JSON rendering, which is what `zm` prints. It carries a version starting at
0.1.0: the minor part changes when a query or field is added, the major part when one is
removed or changes meaning.

A test reads the query names from the document's headings and from the implementation
and fails if they differ (FR-030).

**Rationale**: JSON is language-independent, already familiar to the Swift and Kotlin
shells that come later, and regular enough for tests to read (FR-035). The CLI writes it
by hand; no dependency is added.
