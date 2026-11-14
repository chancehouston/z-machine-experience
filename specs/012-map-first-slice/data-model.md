# Data Model: The Map, First Slice

All of it is in the experience layer (`map` module). Nothing is stored: it is made
again from the turns (research R1).

## Observation

What the map is given at each stop (research R2).

| Field | Value |
| --- | --- |
| `sent` | The input just given, if any: its text, and typed or tapped |
| `waiting` | Command, answer, ended or fault |
| `room` | The room's number, or not known |
| `name` | The room's printed name, where known |
| `dark` | Yes, no or unknown |
| `compass` | The compass as settled, with its extras |
| `jumped` | Whether the core's count of jumps rose since the last stop |

## Place

| Field | Value | Rule |
| --- | --- | --- |
| `id` | The room's number | Identity (W§1, FR-003) |
| `name` | Text, or none | None until seen not dark (W§2) |
| `number` | From 1, or none | Given when a second place has the name (W§15) |
| `sheet` | Which sheet | Changes at most once, by P§4 |
| `x`, `y` | Whole numbers | Changes at most once, by P§4, and then only by the group's shift |
| `attached_to` | A place, or none | Set by P§3; cleared when its group moves |
| `group` | A group, or none | P§4 |
| `ways` | The ways last seen here | W§3, W§4 |
| `put_by_word` | Yes or no | Whether P§6 put it; for P§6's exception |

## Way

Of a place, as last seen from it.

| Field | Value |
| --- | --- |
| `way` | One of the compass's twelve names, or an extra's label |
| `state` | Open, uncertain, door |
| `command` | The line that walks it |
| `dark_ahead` | Yes or no |
| `label`, `spoken_base` | The layer's words for it, without a destination |

## Line

| Field | Value | Rule |
| --- | --- | --- |
| `a`, `b` | Two places | A move that happened (W§5, W§6, W§9) |
| `out` | The way walked from `a`, or word | |
| `back` | The way walked from `b`, or none | Set the first time it is walked back |
| `label` | The command, for a word | As sent |
| `kind` | Straight, bent, ladder, word | P§8; worked out when asked, not kept |

One line for a pair of places and a way out. A second way between the same two
places is a second line.

## Refusal

A place and a way: W§8.

## Sheet

| Field | Value |
| --- | --- |
| `number` | From 1, in the order made |
| `cells` | Position to place; no two places in one (P§7) |
| `started_by_word` | Whether P§6 made it |

## Group

The places attached, directly or through each other, to one place of a sheet: P§3,
P§4. It has at most three places; the fourth dissolves it.

## Map

| Field | Value |
| --- | --- |
| `places` | By id, ordered |
| `order` | Ids in the order first reached |
| `lines`, `refusals` | In the order made |
| `sheets` | In the order made |
| `here` | A place, or not known (W§13) |
| `last` | The place and whether dark at the last command stop, for judging the next move (W§17) |

## States of a move

```text
at a command stop, with the command sent at the last one:

  jumped ────────────────────────────────► no line; new place, if new, on a new sheet
  here not known, or last not known ─────► no line; place added if new, on a new sheet
  same place ── direction, not offered ──► a refusal (W§8)
             └─ anything else ───────────► nothing
  another place ── direction ── way gave another destination ──► carried (W§9)
                │             └─ otherwise ────────────────────► a line in that direction
                └─ any other word ─────────────────────────────► a line by that word
```
