# Z-Machine Introspection Interface

**Version**: 0.4.0 | **Status**: draft

A specification for reading the state of a running Z-machine game without changing it:
where the player is, which ways lead out, what is here and what is carried. It is
written so that any interpreter could implement it; one implementation accompanies it.
Section numbers are stable, and are cited elsewhere as "I§" followed by the number.

"Standard" means the Z-Machine Standards Document, version 1.1, and "§" one of its
sections.

## 1. Rules for every query

There are six queries: `object_tree`, `dictionary`, `location`, `inventory`, `exits` and
`scope`. These rules hold for all of them.

### 1.1 Reads only

No query writes to the game's memory, stack, random-number state, screen or output, and
none runs any of the game's code. A game plays exactly the same whether queries are
asked or not, and whichever are asked: the same text, the same transcript, the same
save files.

### 1.2 Any time

A query may be asked whenever the interpreter has stopped for its host: when the game
wants a line or a key, when it asks to save or restore, before it has run at all, after
it has ended, and after it has faulted. A query answers from the state as it stands. It
does not fail because of when it is asked; before a game's start-up code has run, the
honest answer to some queries is that the player is not yet anywhere.

### 1.3 Unknown is an answer

Every query, and many parts of an answer, can be unknown, with a reason (section 2).
Unknown is not the same as empty. "No exits" is a list of none; "exits unknown" is a
reason. A caller must be able to tell them apart, and a rendering must keep them apart.

### 1.4 No guessing

An answer is given only where the story file's contents settle it by the rules of this
specification. Where they do not, the answer is unknown. Where two candidates fit a
rule, the answer is unknown: a tie is never broken by a preference. The one sanctioned
approximation is `scope`, which says that it is one.

### 1.5 Exactness is declared

Every answer says whether it is `exact`, the fact, or `approximate`, the best the
stored facts allow. Only `scope` is ever `approximate`, and it always is. An unknown
answer still says which it would have been.

### 1.6 Never fails

A story file is data from outside. Whatever it contains, a query returns an answer: if
a table cannot be read, because a number in it is out of range, a chain in it loops, or
it runs off the end of the file, the part that depended on it is unknown with the
reason `damaged`. No query reads outside the story file, and the work a query does is
bounded by the size of the file and the number of its objects.

### 1.7 Deterministic

The same story file in the same state gives the same answers, on every run and every
machine. An implementation keeps nothing from one query that changes the answer to the
next: after a restore, a restart or an undo, the answers are those of the state arrived
at.

## 2. Answers

### 2.1 The form of an answer

An answer has two parts:

| Field | Value |
| --- | --- |
| `exactness` | `exact` or `approximate` |
| `value` | What the query returns, or an unknown |

The values of the six queries are described in sections 4 to 9, as records of named
fields. A field is one of: a number; text; one of a fixed set of words; a list; another
record; or nothing ("none"), where the description says a field may be none.

### 2.2 Unknown, and its reasons

Wherever a description says a value or a field may be unknown, it is either the value
or an unknown carrying one of these reasons:

| Reason | When |
| --- | --- |
| `unrecognised-family` | The query, or the field, needs a family (section 3) and the story file is of none |
| `no-player` | The player cannot be found, or is not placed anywhere |
| `ambiguous-player` | More than one object fits the family's rule for the player |
| `inconsistent` | Two stored facts that the family keeps in step disagree; or the answer was asked of a different story file from the one the implementation was prepared for |
| `not-determinable` | The story file's contents do not settle it |
| `damaged` | A table could not be read (section 1.6) |

A field that can be unknown is never simply left out. Some fields hold a fact that a
family may not record at all; those take one of three words, `yes`, `no` or `unknown`,
and are described as such where they occur.

### 2.3 The JSON rendering

An implementation that offers its answers as text does so in JSON, as follows. The
record descriptions in sections 4 to 9 give the fields and their order.

- A record is a JSON object whose keys are the field names, in the order given.
- A number is a JSON number; text is a JSON string; a word from a fixed set is a JSON
  string holding the word; a list is a JSON array; none is `null`.
- An unknown is the object `{"unknown":"<reason>"}`, in place of the value.
- An answer is `{"exactness":"…","value":…}`.
- In `scope`, the field `player` is the string `"known"` or an unknown.
- In a conditional exit, `destination` and `destination_name` are `null`.
- An exit's keys are, in order: `direction`, `words`, `kind`, `destination`,
  `destination_name`, `door`, `state`, `dark_ahead`, `source`. `door` and `state` are
  `null` unless `kind` is `door` (section 12.5).
- `location`'s keys are `room`, `name`, `holder`, `lit`, `source`; `scope`'s are `light`,
  `source`, `player`, `objects`.
- An inventory entry's keys are, in order: `object`, `name`, `worn`, `state`, `seen`,
  `source`, `contents`. `state` is `null` for an object that holds nothing.

So an inventory that is not known is

```text
{"exactness":"exact","value":{"unknown":"no-player"}}
```

and one that is known to be empty is

```text
{"exactness":"exact","value":[]}
```

### 2.4 What a host writes

Two arrangements of answers are fixed here, so that tools can read any implementation.

**One line for each stop.** A host that reports as a game is played writes, each time
the game waits for a line or a key, one JSON object on one line:

| Field | Value |
| --- | --- |
| `spec` | The version of this specification the answers follow |
| `stop` | The number of this stop, counting from 1 |
| `wants` | `line` or `key` |
| `family` | `inform6`, `zil` or `unrecognised` |
| `pack` | What became of the game pack the host was given: one word (section 12.2) |
| `location`, `exits`, `inventory`, `scope` | The four answers |

**The tables.** A host that reports a story file's tables writes one JSON object with
`spec`, `family`, and either or both of `object_tree` and `dictionary`, each an answer.

### 2.5 Worked examples

From Zork I (family `zil`), after `open mailbox` and `take leaflet` at the start of the
game; and from Advent (family `inform6`), after `in` and `take lamp`. Long lists are cut
short here with an ellipsis, which is not part of the rendering.

`location`:

```text
{"exactness":"exact","value":{"room":64,"name":"West of House","holder":null,"lit":"unknown","source":"file"}}
{"exactness":"exact","value":{"room":38,"name":"Inside Building","holder":null,"lit":"yes","source":"file"}}
```

`inventory`:

```text
{"exactness":"exact","value":[{"object":76,"name":"leaflet","worn":"unknown","state":null,"seen":"yes","source":"file","contents":[]}]}
{"exactness":"exact","value":[{"object":43,"name":"brass lantern","worn":"no","state":null,"seen":"yes","source":"file","contents":[]}]}
```

`exits`. Zork I's house can be walked around, and its west side has no way in; in
version 3 a word is stored cut to six letters. Advent's building has a way west and a
way out, and stores a refusal for `in`, which is therefore absent:

```text
{"exactness":"exact","value":{"exits":[{"direction":31,"words":["n","north"],"kind":"fixed","destination":137,"destination_name":"North of House","door":null,"state":null,"dark_ahead":"unknown","source":"file"},{"direction":29,"words":["w","west"],"kind":"fixed","destination":87,"destination_name":"Forest","door":null,"state":null,"dark_ahead":"unknown","source":"file"}, … ,{"direction":25,"words":["se","southe"],"kind":"fixed","destination":209,"destination_name":"South of House","door":null,"state":null,"dark_ahead":"unknown","source":"file"}],"unknown":[]}}
{"exactness":"exact","value":{"exits":[{"direction":10,"words":["w","west"],"kind":"fixed","destination":30,"destination_name":"At End Of Road","door":null,"state":null,"dark_ahead":"unknown","source":"file"},{"direction":18,"words":["out"],"kind":"fixed","destination":30,"destination_name":"At End Of Road","door":null,"state":null,"dark_ahead":"unknown","source":"file"}],"unknown":[]}}
```

`scope`. ZIL records neither light nor scenery; Inform 6 records both:

```text
{"exactness":"approximate","value":{"light":"unknown","source":"file","player":"known","objects":[{"object":230,"name":"small mailbox","reach":"in-room","within":null,"visibility":"visible","scenery":"unknown","concealed":"unknown"}, … ,{"object":76,"name":"leaflet","reach":"carried","within":null,"visibility":"visible","scenery":"unknown","concealed":"unknown"}]}}
{"exactness":"approximate","value":{"light":"lit","source":"file","player":"known","objects":[{"object":32,"name":"stream","reach":"in-room","within":null,"visibility":"visible","scenery":"yes","concealed":"no"}, … ,{"object":42,"name":"tasty food","reach":"in-room","within":null,"visibility":"visible","scenery":"no","concealed":"no"}, … ,{"object":43,"name":"brass lantern","reach":"carried","within":null,"visibility":"visible","scenery":"no","concealed":"no"}]}}
```

`object_tree`, one object from each game. Advent's maze rooms print as "Maze" whatever
they are stored as:

```text
{"number":230,"name":"small mailbox","printed_name":"small mailbox","parent":64,"children":[76],"attributes":[13,18],"properties":[18,17,16,10]}
{"number":116,"name":"(Alike_Maze_1)","printed_name":"Maze","parent":null,"children":[],"attributes":[],"properties":[46,35,18,15,10,9,8,7,2]}
```

`dictionary`, two words from each game:

```text
{"word":"lamp","parts":["noun"],"direction":null}
{"word":"west","parts":["adjective","direction"],"direction":29}
{"word":"north","parts":["noun","direction"],"direction":7}
{"word":"save","parts":["verb","meta"],"direction":null}
```

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

**What is read from where.** Everything this section defines is worked out once from
the story file as shipped: the family, the directions with their words and their order,
the object that holds a ZIL game's rooms, the bounds S and R, and the objects that start
unplaced (section 7.2). Everything else in sections 4 to 9 is read from memory as it is
at the moment of the query, including whether an object is a room: in ZIL, that its
parent is now the object that holds the rooms.

**The order of a direction's words.** Wherever a direction's words are listed they are
shortest first, and words of the same length are in alphabetical order.

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
| `door` | 5 | | `n_to` … `out_to` | 7 … 18 |
| `light` | 9 | | `door_dir` | 21 |
| `open` | 14 | | `found_in` | 39 |
| `absent` | 1 | | `short_name` | 46 |
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

**Printed names.** If an object does not provide property 46 (`short_name`), its printed
name is its stored name. If it provides it as exactly two bytes: a value of 0 gives the
stored name; a string (below) gives that string's text; any other value is
`not-determinable`. A `short_name` of any other length is `not-determinable`.

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

Names are decoded as the Standard prescribes (§3), with the story file's abbreviations
and, where its header names them, its own alphabet table and Unicode table. If an
object's entry cannot be read its `name` is empty and its lists are empty.

**When a part is `damaged`:**

- `parent` is `damaged` if the stored number is beyond the last object.
- `children` is `damaged` if following the child and sibling links reaches a number
  beyond the last object, visits more objects than exist, or reaches an object whose own
  `parent` is some other object. A link of 0 ends the list. The two stored facts
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

**Parts of speech** are read from the first data byte of an entry, the byte after its
encoded text, and are listed in the order below.

| Part | Inform 6 | ZIL |
| --- | --- | --- |
| `noun` | bit 0x80 | bit 0x80 |
| `verb` | bit 0x01 | bit 0x40 |
| `adjective` | not recorded | bit 0x20 |
| `direction` | the word is one of a direction's words (section 3.1) | bit 0x10 |
| `preposition` | not recorded | bit 0x08 |
| `grammar-word` | bit 0x08 | not recorded |
| `plural` | bit 0x04 | not recorded |
| `meta` | bit 0x02 | not recorded |
| `buzz` | not recorded | bit 0x04 |

Inform sets bit 0x08 on every word that appears in a line of its grammar. Most are
prepositions, but not all, so the part is named for what the bit means. A `meta` verb is
a command to the game itself (save, quit) and not an action in the story. A `buzz` word
is one ZIL's parser ignores.

`direction` is the direction's number for a `direction` word: in ZIL the second data
byte when the low two bits of the first are set and the third otherwise (section 3.2);
in Inform 6 the direction whose words include this one.

A word is decoded from the Z-characters its entry stores, as the Standard prescribes
(§3) but without abbreviations; padding, and a shift or escape left incomplete at the
end, produce nothing. A word in the words of two directions takes the first, in the
family's order of directions.

A dictionary marked unsorted (a negative count, Standard §13.2) is listed in table
order like any other.

**The whole answer is `damaged`** if the dictionary's header cannot be read, if its
entry length is shorter than the encoded text an entry must hold, or if the table does
not lie wholly inside the story file.

## 6. `location`

The room the player is in. **Exact** for a recognised family.

| Field | Value |
| --- | --- |
| `room` | The object number of the room |
| `name` | The room's printed name (section 3); a part that may be unknown |
| `holder` | The object the player is in or on within the room (a vehicle, a cage); none when the player is directly in the room; a part that may be unknown |
| `lit` | `yes`, `no` or `unknown` |
| `source` | `file` or `pack` (section 12.9) |

### 6.1 Inform 6

- **The player** is the object with the stored name `(self object)`.
- **The room** is the object reached by following the player's `parent` until an object
  with no parent is reached. If the player itself has no parent, the answer is
  `no-player`: this is so before the game's start-up code has run.
- **`holder`** is the player's parent when that is not the room.
- **`lit`** is `no` when the location variable (section 3.1 rule 6) holds the number of
  `(darkness object)`, and `yes` otherwise. The library keeps the real room in the tree
  while it is dark, so the room is still reported.
- **Two checks.** The answer is `inconsistent` if the location variable holds neither
  the room nor `(darkness object)`, or if none of the 240 global variables holds the
  player's number. The second catches a game that has made another object the player
  (section 10).

### 6.2 ZIL

- **The room** is the object whose number is in global variable 0, provided it is a room
  (section 3.2 rule 4). The Z-machine requires a version 3 game to keep the location
  there for its status line (Standard §8.2.2). If global 0 holds anything else, the
  answer is `no-player`: this is so before the game's start-up code has run.
- **`holder`** needs the player, found by the rule of section 7.2. It is the player's
  parent when that is not the room, and is unknown for the reason the player is.
  The room is exact either way.
- **`lit`** is `unknown`. ZIL's compiler numbers attributes game by game, and nothing in
  a story file says which attribute or variable means light. A game pack that says
  which attribute means lit changes this (section 12.9).

### 6.3 How the answer relates to what the game prints

An implementation can be checked against the game's own text without reading the value
the query reads. When a game prints a room's name as a heading, it is the `name` of the
reported room. When the reported room changes, the game prints the new room's name on a
line of its own, except in three cases:

- the room is dark, and the game says so in place of the name;
- the player is in a vehicle, and the heading is the name followed by `, in the` and the
  vehicle;
- the old room and the new share a printed name, and the game treats them as one place.

## 7. `inventory`

What the player carries. **Exact** for a recognised family, given the player.

The answer is a list, in the tree's order, of the player's children. Each entry has:

| Field | Value |
| --- | --- |
| `object` | The object's number |
| `name` | Its printed name; a part that may be unknown |
| `worn` | `yes`, `no` or `unknown` |
| `state` | `open`, `closed` or `unknown`, or none |
| `seen` | `yes`, `no` or `unknown` |
| `source` | `file` or `pack` |
| `contents` | The same, for the object's own children |

`seen` says whether the game would show the player the object among what is carried.

An entry at the top level is seen `yes`.

An entry inside another is seen `yes` if the one it is in is seen `yes` and lets what it
holds be seen; `no` if the one it is in is seen `no`, or is known not to let what it
holds be seen; and `unknown` otherwise.

`state` is `open`, `closed` or `unknown` if the object holds something at the moment of
the query, and none if it holds nothing.

`source` is `pack` if `state` or `seen` would be different were no game pack in force
(section 12), and `file` otherwise.

`contents` lists everything the object holds, whatever its marks: the answer marks; it
does not filter. Choosing what to show a player is the caller's business.

An empty list means the player carries nothing, which is not the same as the inventory
being unknown.

### 7.1 Inform 6

The player is `(self object)`. The answer is unknown exactly when `location` is (section
6.1), for the same reason. `worn` is `yes` when the object has attribute 26.

An object that holds something has no `state` if it has attribute 20 (`supporter`);
otherwise its `state` is `open` if it has attribute 14 and `closed` if not. What it
holds can be seen if it is a supporter, is open, or has attribute 23 (`transparent`).

### 7.2 ZIL

ZIL keeps the player's object number in a variable whose position differs from game to
game, so the player is found by elimination:

1. Take the objects that, in the story file as shipped, have no parent and no children.
   A ZIL game creates its player unplaced and moves it into the first room at start-up.
2. Of those, keep each that is now inside the room of section 6.2: following its
   `parent` reaches the room before reaching an object with no parent or another object
   of step 1.
3. Of those, keep each whose number is, in memory now, the value of at least one of
   global variables 1 to 239.

If exactly one object remains, it is the player. If none, the answer is `no-player`. If
more than one, it is `ambiguous-player`: **a tie is never broken.** Other objects that a
game creates unplaced (things made or revealed during play) can lie in the room while a
parser variable holds their number, and then tie with the player.

`worn` is `unknown`.

With no game pack fact to decide it, the `state` of an object that holds something is
`unknown`, and what it holds is seen `unknown`: ZIL numbers "open" and "see-through"
game by game.

With `open-attribute: A` in force (section 12.3), the `state` is `open` if the object
has attribute A and `closed` if not, and what an open object holds can be seen.

With `transparent-attribute: T` in force (section 12.8), what an object with attribute
T holds can be seen, whatever its `state`.

In the reference implementation's tests, a complete game of Zork I has 332 command
prompts; the inventory is `ambiguous-player` at 6 of them and agrees with the game's own
listing at every other.

## 8. `exits`

The directions that lead out of the room `location` reports. **Exact.** Unknown as a
whole exactly when `location` is, for the same reason.

The answer has two lists.

`exits`, one entry for each direction that leads somewhere:

| Field | Value |
| --- | --- |
| `direction` | The direction's number: the number of the property a room gives that exit in (section 3). Stable within a story file |
| `words` | The dictionary words that name the direction, as stored, shortest first. May be empty |
| `kind` | `fixed`, `conditional` or `door` |
| `destination` | For a `fixed` or a `door` exit, the object number of the room it leads to |
| `destination_name` | For a `fixed` or a `door` exit, that room's printed name; a part that may be unknown |
| `door` | For a `door` exit, the object number of the door; otherwise none |
| `state` | For a `door` exit, `open` or `closed`; otherwise none |
| `dark_ahead` | `yes`, `no` or `unknown`: whether the room the exit leads to would be dark for the player as they are (section 12.10) |
| `source` | `file` or `pack` (section 12.5) |

`unknown`, one entry for each direction that could not be determined, with three
fields: `direction`, `words`, and `reason`, which is the reason's word itself (in JSON a
plain string, not an unknown object: the entry as a whole is the unknown).

A direction the room does not provide, or for which it stores only a refusal, is in
neither list. No direction appears twice. An exit that leads back to the room it is in
is reported like any other. Directions are listed in the family's order: for Inform 6
the order of the compass's children, for ZIL from the highest direction number down.

- A **`fixed`** exit is one whose destination the story file stores as a fact.
- A **`conditional`** exit is one the game decides by running its own code. The
  direction exists; where it leads, and whether the game will let the player through
  now, cannot be read. It carries no destination.
- A **`door`** exit is one through a door whose state can be read. It is reported only
  with a game pack in force (section 12.3).

### 8.1 Inform 6

For each direction, take the room's property of that number. If the room does not
provide it, the direction is not an exit. Otherwise, in this order:

| The property's value | Reported as |
| --- | --- |
| is not exactly two bytes | `unknown`, `not-determinable` |
| is 0 | not an exit |
| is the number of an object with attribute 5 (`door`) | `conditional` |
| is the number of an object with no parent that has attribute 9 (`light`) or provides at least one direction property | `fixed`, to that object |
| is the number of any other object | `conditional` |
| is a routine (section 3.1) | `conditional` |
| is a string (section 3.1) | not an exit: the string is the refusal the game prints |
| is not determinable (section 3.1) | `unknown`, `not-determinable` |

The door test comes before the room test because a door that stands in several rooms
has no parent, as a room has none.

### 8.2 ZIL

For each direction, take the room's property of that number. If the room does not
provide it, the direction is not an exit. Otherwise, by the length of the property:

| Length | Contents | Reported as |
| --- | --- | --- |
| 1 | a room | `fixed`, to that room |
| 2 | the address of a message | not an exit |
| 3 | a routine, then a zero byte | `conditional` |
| 4 | a room, a variable number, the address of a message | see below |
| 5 | a room, a door object, the address of a message, a zero byte | `conditional` |

A 4-byte exit is open while a variable is not zero. Its second byte is a Z-machine
variable number, in which 16 is the first global variable:

- if the byte is 16 or more and that global variable is not zero, the exit is `fixed`,
  to the room in the first byte, or `unknown` if that byte is not a room's number;
- if the byte is 16 or more and the variable is zero, it is not an exit;
- if the byte is less than 16 it names the stack or a local variable, which mean nothing
  outside a routine, and the exit is `conditional`.

Reading a variable is reading a stored fact, so this is exact. The variable is whatever
it holds at the moment of the query: some games recompute such a variable only when the
player next gives a command in that room, and the answer follows the stored value, not
what it would be recomputed to.

A 5-byte exit depends on whether a door is open. ZIL numbers its attributes game by
game, so "open" cannot be read, and the exit is `conditional`. A game pack that says
which attribute means open changes this (section 12.3), and one that names a variable
as untrusted changes the 4-byte exit (section 12.4).

The variable is tested before the room: a closed 4-byte exit is not an exit whatever its
first byte holds. A 1-byte exit, or an open 4-byte exit, whose first byte is not the
number of a room is `unknown`, `not-determinable`.

### 8.3 The game has the last word

The answer is what the room's own table says. A game's code runs when the player gives
the command, and may still refuse a listed exit or send the player somewhere else. In
particular, **code that runs on arriving in a room may move the player on**, so that a
move through a `fixed` exit ends in a room other than its `destination`. Advent has
one: two rooms both called "In Forest", where arriving in the first sends the player to
the second half the time. The exit into the first is `fixed`, to the first.

In the reference implementation's tests every move through a `fixed` exit arrived at its
destination, over complete games of Zork I (171 such moves) and Advent (67) and over
tours of several hundred more moves in each, except moves into that forest room.

## 9. `scope`

The objects the player could plausibly act on: what is in the room, what is in or on
those things, and what the player carries. **Always approximate**, and labelled so
whether or not it is known.

It is approximate because both families decide what is in scope, and what is lit, by
running their own code, which rule 1.1 forbids. The answer is worked out from the object
tree and the facts a family stores. It is unknown as a whole exactly when `location`
is, for the same reason.

| Field | Value |
| --- | --- |
| `light` | `lit`, `dark` or `unknown` |
| `player` | `known`, or unknown with the reason the player could not be found |
| `source` | `file` or `pack` (section 12.9) |
| `objects` | The entries, below |

Each entry:

| Field | Value |
| --- | --- |
| `object` | The object's number |
| `name` | Its printed name; a part that may be unknown |
| `reach` | `in-room`, `inside`, `on`, `carried` or `worn` |
| `within` | For `inside` and `on`, the object it is in or on, which is itself an entry |
| `visibility` | `visible`, or `unknown` where the family does not record whether its container is open |
| `scenery` | `yes`, `no` or `unknown` |
| `concealed` | `yes`, `no` or `unknown` |

No object appears twice. The player is never an entry, nor is the room: wherever the
player stands in the room's tree, inside a vehicle for instance, the player and all the
player holds are passed over there, and the vehicle's other contents are listed. The entries
marked `carried` or `worn` are exactly the top level of `inventory` (section 7).

**The answer marks; it does not filter.** Scenery, concealed things and things of
unknown visibility are listed with their marks. Choosing what to show a player is the
caller's business.

Entries are in this order: the room's children in the tree's order, each followed at
once by what is in or on it; then, for Inform 6, objects present by `found_in`, in
order of number, each followed by its contents; then what the player carries, each
followed by its contents.

### 9.1 Inform 6

- **Light.** `lit` or `dark`, from `location`'s `lit` (section 6.1). When `dark`, the
  room's contents are not listed: only what the player carries.
- **Contents.** An object's children are listed as `on` it if it has attribute 20
  (`supporter`); otherwise as `inside` it if it has attribute 14 (`open`) or 23
  (`transparent`); otherwise they are not listed. So what is in a closed, opaque
  container is left out, and so is what a creature holds.
- **Present in several rooms.** An object with no parent, without attribute 1
  (`absent`), whose property 39 (`found_in`) is an even number of bytes, every word of
  it an object number from 1 to the last object, one of them the room, is listed as
  `in-room`, unless it is already listed. Where `found_in`
  holds a routine nothing can be read, and the object is listed only if the library has
  already moved it into the room.
- **What is carried.** The player's children, `worn` if they have attribute 26 and
  `carried` otherwise, with their contents by the same rule.
- **Marks.** `scenery` is attribute 17; `concealed` is attribute 3. `visibility` is
  always `visible`.

### 9.2 ZIL

ZIL's compiler numbers attributes game by game, so a story file does not say which
attribute means open, lit, invisible or a surface.

- **Light** is `unknown`, unless a game pack says which attribute means lit (section
  12.9). The room's contents are listed regardless.
- **Contents.** Every object under the room in the tree is listed. An object inside
  another is `inside`, with `visibility` `unknown`: it may be in a closed container.
  An object directly in the room, or carried directly, is `visible`. Nothing is `on` or
  `worn`.
- **Marks.** `scenery` and `concealed` are `unknown`.
- **When the player is unknown** (section 7.2), `player` gives the reason and nothing is
  `carried`. If the reason is `ambiguous-player`, every object that tied is left out
  wherever it is, together with everything it holds; what it is inside is still listed: one of them is the player, whose possessions are
  not the room's. If it is `no-player`, the room's contents are listed in full.

### 9.3 What makes it approximate

- A game can add objects to scope, or take them out, by code.
- In Inform 6, an object placed by a `found_in` routine may not yet have been moved.
- In ZIL, light and open containers are not known at all.
- Objects a game keeps outside the room's tree but lets the player refer to anywhere
  (the sky, the player's own hands) are not listed.

## 10. Known limits

Where an implementation following this specification can be wrong, or cannot answer.

### 10.1 A change of player in an Inform 6 game

The library lets a game make another object the player. Its `player` variable has no
value in the story file as shipped, so this specification cannot locate it as it locates
`location`, and takes the player to be `(self object)`.

The second check of section 6.1 answers `inconsistent` when no variable holds the number
of `(self object)`, which is so after most changes of player. It does not catch a change
where some other variable still holds that number and the two objects are in the same
room. `inventory`, `holder` and the carried part of `scope` are then those of
`(self object)`, and are labelled exact. **This is the one place where an answer
labelled exact can be wrong.** It is to be closed before version 1 of this
specification.

### 10.2 Ties for the player in a ZIL game

The rule of section 7.2 can tie, and then `inventory`, `holder` and the carried part of
`scope` are unknown. This is a limit on how much is answered, not on whether what is
answered is right. In a complete game of Zork I it happens at about one prompt in fifty.

### 10.3 Translated Inform libraries

Three of the four names in section 3.1 rule 3 are in the library's parser and are the
same in every language. `compass` is in the library's language file. A game compiled
with a translated library that names it otherwise is `unrecognised`.

### 10.4 Versions of the library and of ZIL

Section 3.1 fixes property and attribute numbers by the order of the library's
declarations, checked against library 6.12. A library that declares them in another
order, or a game that declares an attribute of its own before including the library,
fails rules 4 or 5 and is `unrecognised`. `worn` (attribute 26) is on no object at any
point of the game the rules were checked against, and rests on the declaration order.

Section 3.2 describes version 3 story files and was checked against one of them. ZIL's
later versions lay out the dictionary's data differently and are `unrecognised`.

What differs between the games of one family, and so cannot be a rule of the family, is
what game packs are for (section 12).

### 10.5 Exits are the room's table

Section 8.3: a game's code may refuse a listed exit, or move the player elsewhere, when
the command is given or on arrival.

### 10.6 Scope is approximate

Section 9.3.

### 10.7 Other dictionaries

Section 5: only the dictionary the header names is read.

## 11. Versioning

The version has three parts, major, minor and patch.

- **Major** changes when a query or a field is removed, when the meaning of a field or
  of a reason changes, or when a rule changes in a way that makes a previously exact
  answer different. A reader written for one major version cannot rely on the next.
- **Minor** changes when a query, a field, a reason, a family, or a word in a fixed set
  is added. A reader written for an earlier minor version still reads what it knew, and
  must ignore fields and tolerate words it does not know.
- **Patch** changes for corrections of wording that alter no answer.

While the major version is 0 the specification is a draft, and a minor version may
change anything. The known limit of section 10.1 is to be closed before version 1.

A host states the version its answers follow in the `spec` field (section 2.4).

## 12. Game packs

A game pack is a file of facts about exactly one story file: facts that the story file
stores, but whose meaning differs from game to game within a family, so that no rule of
section 3 can read them. A host may give a reader one pack for the story it is
reading. Without a pack every answer is as sections 1 to 10 define it.

### 12.1 The file

A pack is text in UTF-8. Lines end in a line feed; a carriage return before it is
ignored.

A line that is empty, or holds only spaces, or starts with `#`, is skipped.

Every other line is a key, a colon, one space and a value, with nothing after the value.

The first four such lines are, in this order, `pack`, `sha256`, `release` and `serial`.

The lines after them are entries, in any order.

A reader that finds anything else refuses the whole pack: a key it does not know, a
value out of range, a key repeated that may not be, a line it cannot read, an identity
line missing or out of place. A pack is never applied in part.

| Key | Value | Times |
| --- | --- | --- |
| `pack` | `1`, `2` or `3`, the version of this format | once |
| `sha256` | 64 of the characters `0` to `9` and `a` to `f` | once |
| `release` | a decimal number, 0 to 65535 | once |
| `serial` | six characters, each a letter or a digit | once |
| `open-attribute` | a decimal number, 0 to 31 | at most once |
| `untrusted-variable` | a decimal number, 16 to 255 | any number of times; no value twice |
| `transparent-attribute` | a decimal number, 0 to 31; only in a pack of version 2 or later | at most once |
| `lit-attribute` | a decimal number, 0 to 31; only in a pack of version 3 | at most once |

A decimal number has no sign and no leading zero, other than `0` itself.

```text
# A pack for a story file that does not exist.
pack: 1
sha256: 0000000000000000000000000000000000000000000000000000000000000000
release: 1
serial: 000000
open-attribute: 7
untrusted-variable: 40
```

The format's version is 3. A reader of version 3 reads packs of versions 1, 2 and 3. A
pack that holds `transparent-attribute` states `pack: 2` or `pack: 3`, and one that
holds `lit-attribute` states `pack: 3`; in a pack of an earlier version the key is a
fault.

A reader refuses a pack of a version it does not know, so a reader of an earlier version
given a later pack behaves as it does with no pack.

A pack has nowhere to hold a condition, a script, or any text of a game.

### 12.2 Identity

A story file's identity is three things: the SHA-256 digest of the file as shipped; the
word at byte 2 of its header, its release; and the six bytes at 0x12 of its header,
read as characters, its serial.

A pack is **in force** for a story file exactly when all three equal the pack's.

A reader that was given a pack says what became of it, in one word:

| Word | When |
| --- | --- |
| `none` | No pack was given |
| `in-force` | The pack is well formed and its identity is the story file's |
| `not-matching` | The pack is well formed and its identity differs in any of the three |
| `refused` | The pack is not well formed (section 12.1), or the last rule below refuses it |

A host that writes one line for each stop (section 2.4) writes this word in the field
`pack`.

Only a pack in force changes any answer. In every other state every answer is what this
specification defines without a pack.

A pack that holds an entry is refused for a story file whose family is not `zil`: both
kinds of entry are about ZIL's exits.

A pack in force stays in force through restore, restart and undo. It belongs to the
story file, not to a game in progress.

### 12.3 `open-attribute`

`open-attribute: A` says that, in this story file, attribute A is set on a door while
the door is open. With it in force a 5-byte exit (section 8.2) is read as follows.

The exit's first byte is a room and its second a door.

If the second byte is not the number of an object, or the first is not the number of a
room, the direction is `unknown`, `not-determinable`.

Otherwise the exit's `kind` is `door`, its `destination` is the room, its `door` is the
object, and its `state` is `open` if the door has attribute A at the moment of the
query and `closed` if it has not.

A closed door is in the `exits` list. It is not "not an exit": the way is there, and
shut.

The same attribute decides the `state` of a carried object that holds something
(section 7.2). Its effect on exits is as above, whatever a pack says besides.

### 12.4 `untrusted-variable`

`untrusted-variable: N` says that the value stored in variable N is not to be relied
on: the game computes it afresh before using it. N is a Z-machine variable number as a
4-byte exit's second byte holds it, in which 16 is the first global variable.

A 4-byte exit (section 8.2) whose second byte is N is `conditional`, whatever variable
N holds.

A 4-byte exit whose second byte is any other number is read as it is without a pack.

### 12.5 The mark

Every exit has a field `source`. It is `pack` if the exit would be different in any
field were no pack in force, and `file` otherwise.

So every `door` exit is `pack`; a `conditional` exit that is conditional by section
12.4 is `pack`; every other exit is `file`; and when no pack is in force every exit is
`file`.

Every exit has the fields `door` and `state`, which are none unless its `kind` is
`door`.

The entries of the `unknown` list carry no mark.

```text
{"direction":22,"words":["e","east"],"kind":"door","destination":85,"destination_name":"…","door":243,"state":"closed","source":"pack"}
{"direction":20,"words":["n","north"],"kind":"conditional","destination":null,"destination_name":null,"door":null,"state":null,"source":"pack"}
{"direction":29,"words":["s","south"],"kind":"fixed","destination":12,"destination_name":"…","door":null,"state":null,"dark_ahead":"unknown","source":"file"}
```

### 12.6 What a pack may not do

A pack may change the answers to `location`, `scope`, `exits` and `inventory`, and to no
other query.

An answer given with a pack in force is still a read of the story file's state at the
moment of the query: sections 1.1, 1.2 and 1.7 hold unchanged, and nothing is run or
tried.

"Open" is the door's stored state. The game has the last word (section 8.3), and may
still refuse an open door.

### 12.7 What changed

From version 0.1.0 to 0.2.0:

- An exit has three more fields: `door`, `state` and `source`.
- `kind` has a third word, `door`.
- A host's line for each stop has one more field, `pack`.
- Sections 8.2 and 10.4 point here.

From version 0.2.0 to 0.3.0:

- An inventory entry has three more fields: `state`, `seen` and `source`.
- The pack format has a version 2, with one more key, `transparent-attribute`.
- A pack may change the answer to `inventory` as well as to `exits`.

From version 0.3.0 to 0.4.0:

- `location` and `scope` have one more field, `source`; an exit has one more,
  `dark_ahead`.
- The pack format has a version 3, with one more key, `lit-attribute`.
- A pack may change the answers to `location` and `scope`.

### 12.8 `transparent-attribute`

`transparent-attribute: T` says that, in this story file, attribute T is set on a thing
whose contents can be seen while it is closed. With it in force, what a carried object
with attribute T holds can be seen (section 7.2), whatever the object's `state`.

It changes nothing about exits, and nothing about an object that does not have the
attribute.

### 12.9 `lit-attribute`

`lit-attribute: L` says that, in this story file, attribute L is set on a room that is
lit of itself and on a thing that gives light.

A **light** is an object with attribute L, under the player's room in the object tree,
such that every object between it and the room is the player, is open, or can be seen
through (section 7.2).

With it in force, `location`'s `lit` is `yes` if the room has attribute L or a light is
present, and `no` otherwise, with two exceptions.

If the player cannot be told (section 7.2), the room has not attribute L, and no light
is seen outside the objects that tied, `lit` is `unknown`: what the player carries is
not known.

If an object with attribute L is inside another, no `open-attribute` is in force, and
nothing else makes the room lit, `lit` is `unknown`.

`scope`'s `light` is `lit`, `dark` or `unknown` as `location`'s `lit` is `yes`, `no` or
`unknown`. Its objects are listed as before.

`location` and `scope` each have a field `source`: `pack` if the light would be different
were no pack in force, and `file` otherwise.

### 12.10 Dark ahead

Every exit has a field `dark_ahead`: `yes`, `no` or `unknown`. It says whether the room
the exit leads to would be dark for the player as they are at the moment of the query.

For an exit with a `destination`, with `lit-attribute` in force, it is `no` if that room
has attribute L, or a light lies in it (a light by section 12.9, taken from that room),
or the player carries a light; `yes` if none of those; and `unknown` where an exception
of section 12.9 would apply.

For an exit with no `destination`, and wherever no `lit-attribute` is in force, it is
`unknown`.

An exit whose `dark_ahead` is `yes` or `no` has `source` `pack` (section 12.5).

Nothing is tried to answer. The game has the last word (section 8.3): it may refuse the
way, and what lights or darkens a room may change as the player moves.
