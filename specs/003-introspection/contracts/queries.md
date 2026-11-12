# Contract: The Six Queries

**Plan**: [../plan.md](../plan.md) | Shapes: [../data-model.md](../data-model.md) |
Rules per family: [../research.md](../research.md)

This is the working contract for the feature. Its content becomes
`docs/introspection.md`, the published specification, which adds the JSON rendering and
is written for a reader who has not seen this project.

## Rules for every query

1. **Reads only.** No query writes to the game's memory, stack, random state, screen or
   output, and none runs the game's code.
2. **Any time.** A query may be asked whenever the interpreter has stopped for the host,
   including before the game has run, after it has ended, and after a fault. It answers
   from the state as it stands.
3. **Unknown is an answer.** It differs from empty: no exits is a list of none; exits
   unknown is a reason.
4. **No guessing.** If the family's rules do not settle an answer from the story file's
   contents, it is unknown. A tie is unknown.
5. **Exactness is declared.** Every answer says `exact` or `approximate`.
6. **Never fails.** A damaged table gives `damaged` for what could not be read. No query
   panics, loops without bound, or reads outside the story file.
7. **Deterministic.** The same story file in the same state gives the same answer.

## The family

Not a query: a fact about the story file, `inform6`, `zil` or `unrecognised`, decided
once from the file as shipped and given with every answer. Rules: research R1 and R2. A
file that satisfies neither set of rules in full is unrecognised; the rules are not
scored.

## 1. `object_tree`

Every object, by number. Exact. Works for every story file.

- The last object is the one before the first property table.
- A walk of children stops after as many steps as there are objects; the remainder of
  that list is `damaged`.

## 2. `dictionary`

Every word of the main dictionary, in the dictionary's order. Exact. The word list works
for every story file; `parts` are unknown for an unrecognised family (research R10).

Only the dictionary the header names is read. A game's secondary dictionaries are not
discoverable from the header and are not reported.

## 3. `location`

The room the player is in. Exact for a recognised family.

| Family | Room | Unknown when |
| --- | --- | --- |
| Inform 6 | Top of the player's parent chain | The player has no parent (`no-player`); the `location` variable disagrees, or no variable holds the player (`inconsistent`) |
| ZIL | Global 0 | Global 0 is not a room (`no-player`) |

`holder` is the player's parent when that is not the room, and `none` when it is. For
ZIL it needs the player (query 4's rule) and is unknown when the player is; the room is
still exact.

**Known limit (Inform), a recorded deviation**: a game that makes another object the
player is caught only when no variable still holds the original player's number
(research R3).

## 4. `inventory`

What the player carries, as a tree. Exact for a recognised family, given the player.

| Family | Player | Unknown when |
| --- | --- | --- |
| Inform 6 | The object named `(self object)` | As for location |
| ZIL | The rule of research R4 | None fits (`no-player`); more than one fits (`ambiguous-player`) |

An empty list means the player carries nothing.

## 5. `exits`

For the room that `location` gives: a list of exits, and a list of directions that could
not be determined. Exact. Unknown as a whole when the location is unknown.

Rules: research R5 (ZIL), R6 and R7 (Inform). An exit a stored value opens (ZIL's flag
exits) is a fixed exit while the value says open and absent while it says closed.

- A direction the room stores a refusal for is in neither list.
- A direction is reported once, with every word that names it.
- Exits are what the room's own table says. A game may still refuse a listed exit, or
  move the player somewhere else, by code that runs when the command is given. The
  interface cannot see that; a `fixed` exit states where the table points.
- An exit that leads to the room it is in is reported like any other.

## 6. `scope`

The objects the player could plausibly act on. **Always `approximate`**: both families
decide scope and light by running code, which rule 1 forbids.

Rules: research R9.

- Listed: what is in the room, what is in or on those things where that is visible or
  cannot be told, and what the player carries, likewise.
- Not listed: the player; the room; in Inform, the contents of a closed opaque
  container, and the room's contents when it is dark.
- Each entry says how it is reached and, where the family records it, whether it is
  scenery or concealed. The interface marks; it does not filter. Choosing what to show a
  player is the caller's job.
- Unknown as a whole when the location is unknown. When only the player is unknown
  (ZIL), the answer's `player` field says so, the room's contents are listed, nothing is
  marked carried, and the objects that tied for the player are left out together with
  what they hold: one of them is the player, and the player is never an entry.

**What makes it approximate**, to be stated in the specification: objects the game adds
to scope by code; objects present in several rooms by a routine; a game's own rules that
hide or reveal things; and, in a family that does not record them, light and whether a
container is open.
