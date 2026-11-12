# Data Model: First Demo on a Phone

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

## Session

One game being played. Owned by the experience layer.

| Holds | Notes |
| --- | --- |
| The machine | The core's `Machine`, loaded from the story file |
| The introspector | Made once for the story file |
| The transcript | Every turn so far |
| The generation | A number that goes up each time a new game is started, so the app can tell a fresh transcript from more of the old one |
| The story file's bytes and the seed | Kept for starting anew |

Created from a story file and a seed (or none, for a fresh one). A story file the core
refuses gives no session but an error with the core's message.

## Turn

| Field | Type | Notes |
| --- | --- | --- |
| `number` | from 0 | 0 is the opening, before any command |
| `command` | text or none | The line given to the game; none for turn 0 |
| `source` | `typed`, `tap` or none | How the command was given |
| `text` | text | What the game printed in reply, without the prompt that ends it |

The transcript is the list of turns. A turn's text is complete: the layer runs the game
to its next stop before the turn exists.

## Status

| Field | Type |
| --- | --- |
| `place` | text: the room's name from the game's status line |
| `score`, `moves` | numbers, for a game that keeps score |
| `hours`, `minutes` | numbers, for a game that keeps time |

None before the game has drawn a status line, and for a game that has none.

## Compass

Twelve **places**: `north`, `northeast`, `east`, `southeast`, `south`, `southwest`,
`west`, `northwest`, `up`, `down`, `in`, `out`. Each has:

| Field | Type | Notes |
| --- | --- | --- |
| `place` | one of the twelve | |
| `state` | `open`, `maybe`, `closed`, `unknown` | See below |
| `command` | text or none | The line a tap sends; present exactly when the state is `open` or `maybe` |
| `leads_to` | text or none | The destination's name, for `open` |

| State | Meaning | From the introspection answer |
| --- | --- | --- |
| `open` | Leads somewhere for certain | A `fixed` exit matched to this place |
| `maybe` | The game decides | A `conditional` exit matched to this place |
| `closed` | Does not lead anywhere | No exit matched, and the exits are known |
| `unknown` | Cannot be told | The exits answer is unknown; or this direction is in its `unknown` list; or two exits matched this place |

**Extras**: exits that match no place, each with a `label`, a `state` (`open` or
`maybe`), a `command` and `leads_to`.

**Overall**: `known` or `unknown` with the reason, so the app can say "exits unknown" in
place of an empty compass (FR-014).

When the waiting state is not `command`, every place is reported `closed` with no
command and there are no extras: the compass offers nothing (FR-015).

## Waiting state

`command`, `answer`, `ended` or `fault` (research R6). `fault` carries a message for the
player.

## Snapshot

What the app is given.

| Field | Type |
| --- | --- |
| `generation` | number |
| `waiting` | the waiting state |
| `message` | text or none: for `fault` |
| `status` | Status or none |
| `turns` | the turns from the one asked for onward |
| `turn_count` | how many turns there are in all |
| `compass` | the twelve places, the extras, and the overall state |

## Validation rules

- A place has a `command` exactly when its state is `open` or `maybe`.
- No two places, and no place and extra, have the same command.
- `leads_to` is present only for `open`.
- `turns` are in order and `turn_count` is one more than the last turn's number.
- Turn 0 has no command; every other turn has one.
