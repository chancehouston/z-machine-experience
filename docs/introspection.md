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

How a story file is recognised as belonging to a family, from its contents alone.

### 3.1 Inform 6

Story files compiled with Inform 6 and its standard library.

### 3.2 ZIL

Story files compiled with Infocom's ZIL tools.

### 3.3 Unrecognised

Every other story file.

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
