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

Every object of the game, with its name, parent, children, attributes and properties.

## 5. `dictionary`

Every word the game's parser knows, with its parts of speech where the family records
them.

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
