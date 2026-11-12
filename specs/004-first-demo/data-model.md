# Data Model: First Demo on a Phone

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

## Session

One game being played. Owned by the experience layer.

| Holds | Notes |
| --- | --- |
| The machine | The core's `Machine`, loaded from the story file |
| The introspector | Made once for the story file |
| The turns | Every turn of the current game |
| The generation | A number that goes up each time a new game is started from the app, so the app can tell a fresh game from more of the old one. A `restart` typed by the player does not change it |
| The command address | The address at which the game first asked for a line (research R6) |
| The last status | The game's status line as last drawn; the core hands it over once, so the layer keeps it |
| The story file's bytes | Kept for starting a new game |

A story file the core refuses gives no session, but an error with the core's message.

## Turn

| Field | Type | Notes |
| --- | --- | --- |
| `number` | from 0 | 0 is the opening, before any command |
| `command` | text or none | The line given to the game; none for turn 0 |
| `source` | `typed`, `tap` or none | How the command was given |
| `text` | text | Everything the game printed in reply, unaltered, including the prompt that ends it |

A turn exists only when the game has run to its next stop, so its text is complete.

## Status

| Field | Type |
| --- | --- |
| `place` | text: the room's name from the game's status line |
| `score`, `moves` | numbers, for a game that keeps score; otherwise none |
| `hours`, `minutes` | numbers, for a game that keeps time; otherwise none |

None before the game has drawn a status line. After the game ends it stays as last
drawn.

## Compass

Twelve **places**: `north`, `northeast`, `east`, `southeast`, `south`, `southwest`,
`west`, `northwest`, `up`, `down`, `in`, `out`. Each has:

| Field | Type | Notes |
| --- | --- | --- |
| `place` | one of the twelve | |
| `label` | text | What to show for it: "North" |
| `state` | `open`, `maybe`, `closed`, `unknown`, `idle` | Below |
| `command` | text or none | The line a tap sends; present exactly when the state is `open` or `maybe` |
| `leads_to` | text or none | The destination's name, for `open`, where it is known |
| `spoken` | text | What a screen reader says: "North, to Forest Path" |

| State | Meaning | When |
| --- | --- | --- |
| `open` | Leads somewhere for certain | A `fixed` exit matched to this place |
| `maybe` | The game decides | A `conditional` exit matched to this place |
| `closed` | Does not lead anywhere | No exit matched, and the exits are known |
| `unknown` | Cannot be told | The exits answer is unknown; this direction is in its `unknown` list; or two exits matched this place |
| `idle` | Nothing is offered now | The game is not waiting for a command |

**Extras**: exits that match no place. Each has a `label` (its longest word), a `state`
(`open` or `maybe`), a `command` (the same word), `leads_to` and `spoken`. An exit the
introspection interface reports with no words at all cannot be named or sent; it is left
out of the extras and makes the overall state `unknown`.

**Overall**:

| Field | Values | Notes |
| --- | --- | --- |
| `offered` | yes or no | No when the game is not waiting for a command; every place is then `idle` and there are no extras |
| `known` | yes or no | No when the exits answer is unknown, or an exit could not be placed or named. Meaningful only when `offered` |
| `note` | text or none | What to show under the compass: "Exits unknown: type a direction", "No exits here", or none |

## Waiting state

`command`, `answer`, `ended` or `fault` (research R6). `fault` carries a message for the
player.

## Snapshot

What the app is given.

| Field | Type |
| --- | --- |
| `generation` | number |
| `turn_count` | how many turns the current game has |
| `waiting` | the waiting state |
| `message` | text or none: for `fault` |
| `status` | Status or none |
| `turns` | the turns from the one asked for onward |
| `compass` | the twelve places, the extras, and the overall fields |

`generation` and `turn_count` together are the **stamp** the app sends back with a tap
or a typed line.

## Validation rules

- A place or extra has a `command` exactly when its state is `open` or `maybe`.
- No two places, and no place and extra, have the same command.
- `leads_to` is present only for `open`.
- When `offered` is no, every place is `idle`; when it is yes, none is.
- `turns` are in order and `turn_count` is one more than the last turn's number.
- Turn 0 has no command; every other turn has one.
- The two notes that can be shown are different texts (FR-014).
