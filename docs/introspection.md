# Z-Machine Introspection Interface

**Version**: 0.1.0 | **Status**: draft

A specification for reading the state of a running Z-machine game without changing it.
It is written so that any interpreter could implement it. Section numbers are stable:
tests and other documents cite them as "I§" followed by the number.

This is a skeleton. Each section states what it will define; the sections are filled in
as the reference implementation is built, and none is renumbered.

## 1. Rules for every query

The rules below hold for all six queries.

### 1.1 Reads only

No query writes to the game's memory, stack, random state, screen or output, and none
runs the game's code.

### 1.2 Any time

A query may be asked whenever the interpreter has stopped for its host.

### 1.3 Unknown is an answer

"Unknown" is distinct from an empty answer.

### 1.4 No guessing

Where the story file's contents do not settle an answer, it is unknown. A tie is unknown.

### 1.5 Exactness is declared

Every answer says whether it is exact or approximate.

### 1.6 Never fails

A damaged table yields "unknown" for what could not be read. No query fails.

### 1.7 Deterministic

The same story file in the same state gives the same answer.

## 2. Answers

The form of an answer, the reasons an answer or a part of one can be unknown, exactness,
and the JSON rendering.

## 3. Families

Where the player is, what an exit is, and what is in reach are not ideas the Z-machine
has. They are conventions of the compiler and library a game was made with. A **family**
is a set of story files made with the same ones, which therefore keep these things the
same way.

The family is a fact about a story file, not a query. It is one of `inform6`, `zil` or
`unrecognised`, it is decided once, and it accompanies every answer.

- **Recognition reads the story file as shipped**: the bytes of the file before the game
  has run. It therefore never changes during play, across a restart, a restore or an
  undo.
- **A file belongs to a family only if it satisfies every one of that family's rules.**
  The rules are not weighed or scored. A file that satisfies neither set in full is
  `unrecognised`.
- **The rules name no game.** They do not consult a file's name, serial number, release
  or checksum.

The rules are deliberately stricter than is needed to tell the families apart: each also
checks a convention that a later query relies on, so that a file which keeps the
convention differently is `unrecognised` and not misread.

### 3.1 Inform 6

Story files compiled with Inform 6 and its standard library. A file is `inform6` when
all of these hold:

1. **The compiler's stamp.** Header bytes 0x3C to 0x3F are an ASCII digit, a full stop,
   and two ASCII digits (for example `6.44`): the compiler version Inform writes there.
2. **The metaclasses.** Objects 1, 2, 3 and 4 have the stored names `Class`, `Object`,
   `Routine` and `String`.
3. **The library's objects.** Exactly one object has each of the stored names `compass`,
   `(self object)`, `(darkness object)` and `(Inform Library)`.
4. **The compass.** The object named `compass` has at least one child. Every child
   provides property 21 (the library's `door_dir`) with exactly two bytes of data, whose
   value is from 4 to 63, is not the number of a property that child itself provides,
   and is not the value of any other child's. Each value is a **direction**: the number
   of the property in which a room gives its exit that way.
5. **Attribute numbering.** `(self object)` has exactly attributes 0, 3, 16 and 23;
   `compass` has exactly attribute 3; every child of `compass` has exactly attribute 17.
6. **The location variable.** Exactly one of the 240 global variables holds the number
   of `(Inform Library)`. That variable is the library's `location`.

Rules 4 and 5 fix the numbers the library gives its properties and attributes by the
order in which it declares them. Those this specification uses:

| Attribute | Number | | Property | Number |
| --- | --- | --- | --- | --- |
| `animate` | 0 | | `name` | 1 |
| `concealed` | 3 | | `before`, `after`, `life` | 4, 5, 6 |
| `container` | 4 | | `n_to` … `out_to` | 7 … 18 |
| `door` | 5 | | `door_dir` | 21 |
| `light` | 9 | | `found_in` | 39 |
| `open` | 14 | | `short_name` | 46 |
| `scenery` | 17 | | | |
| `supporter` | 20 | | | |
| `transparent` | 23 | | | |
| `worn` | 26 | | | |

**The words for a direction** are the dictionary words whose entry addresses are listed,
two bytes each, in the `name` property (1) of the compass child for that direction. The
library takes "in" and "out" as verbs, and their compass children have no `name`: for a
child without `name` whose direction is 17 or 18, the word is `in` or `out`
respectively, provided the dictionary holds that word with the verb bit (0x01) set in
its first data byte; otherwise the direction has no words.

**Printed names.** An object's printed name is the text at its `short_name` property
(46) when that property is two bytes and holds a string (below); unknown
(`not-determinable`) when it holds anything else that is not an object number 0; and
the stored name when the object does not provide the property.

**Telling a routine from a string.** A two-byte property value that is not an object
number is the packed address of a routine or of a string, and nothing in the header says
which. Inform places every routine before every string. Take every two-byte word in the
data of every even-length property, other than properties 1, 2 and 3, of every object
in the file as shipped; keep those greater than the last object's number whose unpacked
address lies at or above the high-memory base (header word 0x04) and inside the file.
Then:

- **S** is the lowest such value whose target's first byte is greater than 15. A routine
  begins with its count of local variables, 0 to 15, so that target is not a routine.
  Any value at or above S is a **string**.
- **R** is the highest such value found in property 4, 5 or 6, which the library only
  ever calls. Any value at or below R that is not an object number is a **routine**.
- A value strictly between R and S, and every non-object value if S or R cannot be found
  or S is not greater than R, is **not determinable**.

### 3.2 ZIL

Story files compiled with Infocom's ZIL tools. A file is `zil` when all of these hold:

1. **Version 3, not Inform.** The file is version 3 and fails rule 1 of section 3.1.
2. **Dictionary entries are 7 bytes**: 4 of encoded text and 3 of data.
3. **Direction words.** At least four dictionary entries have bit 0x10 set in their
   first data byte. For such an entry the **direction** is the second data byte when the
   low two bits of the first are both set, and the third data byte otherwise. A
   direction is the number of the property in which a room gives its exit that way.
4. **One parent for all rooms.** Among objects that have a parent, every one providing a
   direction property has the same parent, and that parent has no parent. At least one
   such object exists. The children of that parent are the **rooms**.
5. **The shape of exits.** Every direction property of every object counted in rule 4 is
   1 to 5 bytes long, and each that is 1 byte holds the number of a room.

Objects with no parent are left out of rule 4 because the object that holds the rooms
may itself carry a direction property.

**The words for a direction** are all the dictionary words whose entries give that
direction by rule 3.

**Printed names** are the stored names.

The parts of speech in the first data byte of a dictionary entry are: 0x80 noun, 0x40
verb, 0x20 adjective, 0x10 direction, 0x08 preposition, 0x04 buzz word.

### 3.3 Unrecognised

Every other story file, including one compiled with Inform but without its standard
library. For these the object tree and the dictionary's words are answered as for any
file, and `location`, `inventory`, `exits` and `scope` answer `unrecognised-family`.

## 4. `object_tree`

Every object of the game, by number. **Exact.** Answered for every story file, whatever
its family.

The answer is a list with one entry for each object, in order of object number, from 1
to the last object. The Z-machine's object table has no count (Standard §12.3): the last
object is the one whose entry ends where the lowest-addressed property table begins.

Each entry has:

| Field | Value |
| --- | --- |
| `number` | The object's number, from 1 |
| `name` | Its short name as the story file stores it (Standard §12.4), decoded to text. Empty if it has none |
| `printed_name` | What the game prints for the object. The stored name, unless the family says otherwise (section 3); unknown where the family computes it by code |
| `parent` | The number of its parent, or none |
| `children` | The numbers of its children, in the tree's order: its child, then that object's sibling, and so on |
| `attributes` | The numbers of the attributes that are set, ascending. Attribute 0 is the top bit of the first byte |
| `properties` | The numbers of the properties it provides, in the order of its property table |

What a number means, for an attribute or a property, is a matter for the family and is
not part of this answer.

**When a part is `damaged`:**

- `parent` is `damaged` if the stored number is beyond the last object.
- `children` is `damaged` if following the child and sibling links reaches a number that
  is 0 where it should not be or beyond the last object, visits more objects than exist,
  or reaches an object whose own `parent` is some other object. The two stored facts
  then disagree and neither is preferred.
- The other objects' entries are unaffected. The walk for one object never takes more
  steps than there are objects.

A parent that is its own ancestor is reported as stored: each object's `parent` is one
stored number, and nothing follows a chain of parents here.

## 5. `dictionary`

Every word of the game's main dictionary, in the dictionary's own order. **Exact.** The
list of words is answered for every story file; the parts of speech need a family.

Only the dictionary the header names (Standard §11.1, the word at 0x08) is read. A game
may keep others and use them by instruction; nothing in the header leads to them and
they are not reported.

Each entry has:

| Field | Value |
| --- | --- |
| `word` | The word as stored, decoded to text. A story file keeps only the first 6 characters of a word in version 3, and the first 9 in versions 5 and 8 (Standard §13.3, §13.4) |
| `parts` | The parts of speech the family records for it (section 3), a list that may be empty; `unrecognised-family` for a story file of no recognised family |
| `direction` | For a word that names a direction, the direction's number, as the exits query uses it (section 8); otherwise none |

A dictionary marked unsorted (a negative count, Standard §13.2) is listed in table
order like any other.

**The whole answer is `damaged`** if the dictionary's header cannot be read, if its
entry length is shorter than the encoded text an entry must hold, or if the table does
not lie wholly inside the story file.

## 6. `location`

The room the player is in.

## 7. `inventory`

What the player carries.

## 8. `exits`

The directions that lead out of the player's location.

## 9. `scope`

The objects the player could plausibly act on. Always approximate.

## 10. Known limits

Where an implementation following this specification can be wrong or unable to answer.

## 11. Versioning

What kind of change moves which part of the version number.
