# Contract: The Screen

**App**: Lamplight | **Plan**: [../plan.md](../plan.md) | What it is given:
[../data-model.md](../data-model.md)

The app draws a snapshot. It keeps no knowledge of the game beyond the last snapshot.

## Layout, portrait

| Region | Shows |
| --- | --- |
| Top bar | The status: the place, and score and moves or the time. An "About" button |
| Transcript | Every turn in order: the command, marked as the player's, then the game's text. Scrolls; the newest turn is brought into view when it arrives |
| Compass | The eight points in a ring; up and down, in and out beside it; extras in a row beneath |
| Field | A text field and a send button, above the keyboard when it is showing |

When the keyboard is showing, the compass may shrink but the newest text and the field
stay visible (FR-019).

## The compass

| State | Appearance | Tap | Spoken label |
| --- | --- | --- | --- |
| `open` | Filled | Sends | "North, to Forest Path" |
| `maybe` | Outlined, with a question mark | Sends | "North, may be blocked" |
| `closed` | Faint | Nothing | "North, no exit" |
| `unknown` | Faint, dashed | Nothing | "North, unknown" |

When the compass's overall state is `unknown`, a line under it reads "Exits unknown:
type a direction". When every place is `closed` and the game is waiting for a command, a
line reads "No exits here". The two are never the same text (FR-014).

`open` and `maybe` differ by shape and symbol as well as colour (FR-009).

## Waiting states

| State | Compass | Field | Else |
| --- | --- | --- | --- |
| `command` | As reported | Live, placeholder "Type a command" | |
| `answer` | Offers nothing | Live, placeholder "Type your answer" | |
| `ended` | Offers nothing | Replaced by a "New game" button | "The game has ended." |
| `fault` | Offers nothing | Replaced by a "New game" button | The message |

While a command is being worked out, every control that sends is disabled (FR-016).

## Commands in the transcript

A tapped command and a typed one look the same: the line itself, set apart from the
game's text. The player is always shown what was sent (FR-011).

## Identifiers for automated tests

Each control has a stable accessibility identifier: `compass.north` … `compass.out`,
`compass.extra.0` …, `field.command`, `button.send`, `button.newGame`, `status.place`,
`status.score`, `transcript`, `compass.note`. Each compass control's accessibility value
is its state word.

## About

The app's name, a line saying it is a demonstration, and the bundled story's license
notice and source, as the repository records them. The word "Zork" appears only inside
that notice and in the game's own text.
