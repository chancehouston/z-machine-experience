# Contract: The Screen

**App**: Lamplight | **Plan**: [../plan.md](../plan.md) | What it is given:
[../data-model.md](../data-model.md)

The app draws a snapshot. It keeps no knowledge of the game beyond the snapshots it has
been given. The only things it knows on its own are where each of the twelve places goes
on the screen and how each state looks.

## Layout, portrait only

| Region | Shows |
| --- | --- |
| Top bar | The status: the place, and score and moves or the time. An "About" button |
| Transcript | Every turn in order. Scrolls; the newest turn is brought into view when it arrives |
| Compass | The eight points in a ring around a centre; up and down to one side, in and out to the other; extras in a wrapping row beneath; the note beneath that |
| Field | A text field and a send button |

**With the keyboard showing**, the compass is hidden and the transcript and field take
its room; the field sits directly above the keyboard and the newest text directly above
the field (FR-019). A button beside the field puts the keyboard away and brings the
compass back. (It was first put on the keyboard's own bar; on iOS 26 the system floats
that bar's buttons over the send button.) Sending a line does not put the keyboard away.

**At large text sizes** the transcript, the status and the labels of extras grow with
the system setting. The twelve compass controls are symbols in targets of a fixed size,
at least 44 points square, and do not grow; they are identified to a screen reader by
their spoken text. Nothing overlaps at the largest accessibility size on the smallest
iPhone the supported systems run on (SC-010).

## The transcript

A turn is drawn as the game's text, in the order printed, with the command that led to
the next turn written directly after the prompt that ends it, as on a terminal, and set
apart as the player's (weight and colour). A tapped command and a typed one look the
same (FR-011). The app does not alter the text.

A transcript shorter than its room sits at the bottom of it, next to the compass and the
field, at launch and after "New game" alike.

## The compass

| State | Appearance | Tap |
| --- | --- | --- |
| `open` | Filled arrow | Sends |
| `maybe` | Outlined arrow with a question mark | Sends |
| `closed` | Faint dot | Nothing |
| `unknown` | Faint dashed outline | Nothing |
| `idle` | Faint dot; the whole compass dimmed | Nothing |

`open` and `maybe` differ by shape and symbol as well as colour (FR-009). The spoken
text and the note are the snapshot's, shown as given.

## Waiting states

| State | Compass | Field | Else |
| --- | --- | --- | --- |
| `command` | As reported | Live, placeholder "Type a command" | |
| `answer` | Not offered | Live, placeholder "Type your answer" | |
| `ended` | Not offered | Replaced by a "New game" button | "The game has ended." |
| `fault` | Not offered | Replaced by a "New game" button | The snapshot's message |

The send button is disabled while the field is empty. While a command is being worked
out, every control that sends is disabled; the layer would refuse a second command in
any case (FR-016).

## Lifecycle

The game lives in memory for as long as the app's process does. Going to the background
and returning changes nothing and sends nothing (FR-006). A new process starts a new
game (FR-005). "New game" asks the layer for one with a fresh random number.

## Starting

The app reads `story.z3` from its bundle, draws a random number, and creates a session.
If the file is missing, or the layer refuses it, the screen shows "The game could not be
started" with the layer's message, and nothing else (FR-024).

## What the app sends

- A tap: the place's identifier, or the extra's index, with the stamp of the snapshot on
  screen.
- A typed line: the field's text, with that stamp.
- "New game": a fresh random number.

The app composes no command. The only text it sends is what the player typed (SC-011).

## For automated tests

Accessibility identifiers: `compass.north` … `compass.out`, `compass.extra.0` …,
`compass.note`, `field.command`, `button.send`, `button.compass` (shown with the
keyboard), `button.newGame`, `button.about`, `about.demonstration`, `about.license`,
`status.place`, `status.score`, `status.moves`, `status.time`, `transcript`, and for each turn
`turn.N.text` and `turn.N.command` (the command that led to turn N). Each compass
control's accessibility value is its state word, and its label is the spoken text.

Launch arguments:

| Argument | Effect |
| --- | --- |
| `-seed N` | The game's randomness is fixed to N |
| `-story NAME` | Looks for the resource NAME in place of `story`; a name that does not exist exercises the could-not-start screen |
| `-showLaunchTime` | Adds an element `debug.launchMillis` whose label is the time in milliseconds from the start of the process to the first snapshot being shown |

The system's own argument for content size is used to test large text.

Debug builds only: when the environment variable `LAMPLIGHT_FIXTURE_SNAPSHOT` holds a
snapshot's JSON, the app draws it in place of starting a game, and sends nothing. It
exists to test how states are drawn that the bundled game reaches late or never: extras
and unknown exits.

## About

The app's name, a line saying it is a demonstration, and the bundled story's license
notice and source, as the repository records them. Each paragraph of the notice is
drawn as one run of text: the line ends inside a paragraph are for a wide page, and the
words are unchanged. The word "Zork" appears only inside
that notice and in the game's own text.
