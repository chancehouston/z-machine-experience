# Data Model: Screen Reader Announcements

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Nothing is stored. The session holds two more values between stops.

## Reply

| From | Rule |
| --- | --- |
| The newest turn's text | White space at its end removed; then one final `>` removed if the game is waiting for a line (a command or an answer); then white space at both ends removed |

Nothing else of the game's text is changed.

## In the session

| Value | Meaning |
| --- | --- |
| was dark | Whether the layer's light said dark at the stop before this one. False at the start of a game, where light is unknown, and at any stop that is not a command prompt |
| announcement | What is to be spoken for the newest turn, or none |

## Announcement

Made once, when a turn is recorded. Its parts in order, joined by one line feed, each
left out if empty:

| Part | Present when | Words |
| --- | --- | --- |
| Reply | The reply is not empty | The game's |
| Darkness | Light says dark now, and was dark is false | The layer's note, "It is dark" |
| Ending | The game ended on this turn | The layer's, "The game has ended." |

Where the game has stopped with a fault, the announcement is the fault's message and
nothing else. With no part present it is none.

## In the snapshot

| Field | Was | Is |
| --- | --- | --- |
| `announcement` | absent | text or null, the last key |
| `message` | a fault's message, or null | a fault's message; "The game has ended." when the game has ended; or null |

## In the app

| Value | Meaning |
| --- | --- |
| speaking | True when the system says a screen reader is running, or the launch argument says to behave so |
| last asked, times asked | What the app last posted and how often; shown to tests only with the launch argument |

The app posts when a snapshot it shows brought a new turn, or a new game, and
`announcement` is not null and speaking is true.
