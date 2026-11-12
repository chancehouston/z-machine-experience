# Data Model: Introspection Interface

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

These are the shapes the queries answer with. The published specification
(`docs/introspection.md`) defines them for any implementer; this file is the working
model for this feature.

## Answer

Every query returns an Answer.

| Field | Values | Meaning |
| --- | --- | --- |
| `value` | the query's result, or absent | Absent when unknown |
| `unknown` | absent, or a reason | Why there is no value |
| `exactness` | `exact`, `approximate` | `approximate` only for scope |

**Unknown reasons** (one word each, so a caller can branch on them):

| Reason | When |
| --- | --- |
| `unrecognised-family` | A family-dependent query on a file of neither family |
| `no-player` | The player cannot be found, or is not placed anywhere |
| `ambiguous-player` | More than one object fits the family's rule for the player (ZIL) |
| `inconsistent` | Two stored facts that should agree do not (research R3) |
| `not-determinable` | The file's contents do not settle it (research R7) |
| `damaged` | A table could not be read: a loop, or a number out of range |

Parts of an answer can be unknown on their own: one direction among a room's exits, the
light in a scope answer, a word's parts of speech. In every rendering a part that can be
unknown is either its value or an unknown with a reason; it is never simply left out.

## Family

`inform6`, `zil` or `unrecognised`. Fixed for a story file; decided once when the
introspector is made (research R1, R2).

## Object

| Field | Type | Notes |
| --- | --- | --- |
| `number` | 1 to the last object | The Z-machine's object number |
| `name` | text | Its short name as stored; may be empty |
| `printed_name` | text, or unknown | What the game prints for it. The stored name, except in Inform where a `short_name` string replaces it; unknown where `short_name` is a routine |
| `parent` | object number or none | |
| `children` | list of object numbers | In the tree's order |
| `attributes` | list of numbers | Those that are set |
| `properties` | list of numbers | Those it provides, in table order |

The **object tree** answer is the list of all objects by number. It is never
family-dependent. A damaged link makes `parent` or `children` of the objects concerned
unknown; the rest are still given.

## Location

| Field | Type | Notes |
| --- | --- | --- |
| `room` | object number | |
| `name` | text, or unknown | The room's printed name |
| `holder` | object number, `none`, or unknown | What the player is in or on, within the room. Unknown when the player is (ZIL) |
| `lit` | `yes`, `no`, `unknown` | Inform: stored. ZIL: unknown |

## Inventory

A list of carried objects, each:

| Field | Type |
| --- | --- |
| `object` | object number |
| `name` | text |
| `worn` | `yes`, `no`, `unknown` |
| `contents` | list of the same shape |

## Exit

The **exits** answer is a list of exits and a list of directions that are unknown.

| Field | Type | Notes |
| --- | --- | --- |
| `direction` | number | The direction's property number; stable within a story file |
| `words` | list of text | Dictionary words that name it, as stored (truncated), shortest first. May be empty |
| `kind` | `fixed`, `conditional` | |
| `destination` | object number | Only when `fixed` |
| `destination_name` | text | Only when `fixed` |

A direction for which the room stores a refusal is absent from both lists.

## Scope

| Field | Type | Notes |
| --- | --- | --- |
| `light` | `lit`, `dark`, `unknown` | |
| `player` | `known`, or unknown with a reason | When unknown, nothing is marked carried, and the objects that tied for the player are left out with what they hold |
| `objects` | list of scope entries | |

Each entry:

| Field | Type | Notes |
| --- | --- | --- |
| `object` | object number | |
| `name` | text | |
| `reach` | `in-room`, `inside`, `on`, `carried`, `worn` | |
| `within` | object number or none | For `inside` and `on`; also the carried container |
| `visibility` | `visible`, `unknown` | `unknown` where the family does not record whether the container is open |
| `scenery` | `yes`, `no`, `unknown` | |
| `concealed` | `yes`, `no`, `unknown` | |

The player and the room itself are not entries. Exactness is always `approximate`.

## Dictionary word

| Field | Type | Notes |
| --- | --- | --- |
| `word` | text | As stored: truncated to 6 characters in version 3, 9 in versions 5 and 8 |
| `parts` | list of `noun`, `verb`, `adjective`, `direction`, `preposition`, `grammar-word`, `plural`, `meta`, `buzz`; or unknown | Unknown for an unrecognised family |
| `direction` | number or none | For a direction word, its direction number |

## Introspector

The object a host holds. Made from a machine's view; holds the family and what was
worked out once from the story file as shipped (the direction table, the routine and
string bounds, the candidates for the player). Holds no copy of the game's memory and
nothing that changes as the game runs, so the same state always gives the same answers
(FR-009).

## Validation rules

- An object number in any answer is between 1 and the last object.
- `fixed` exits have a destination; `conditional` exits do not.
- No direction appears twice in an exits answer, nor in both of its lists.
- No object appears twice in a scope answer or in an inventory answer.
- Every scope entry with `reach` of `inside` or `on` has a `within` that is itself an
  entry, the room excepted.
