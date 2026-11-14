# Lamplight for iPhone

The SwiftUI shell of the first demo. It draws what the experience layer
(`crates/zmachine-experience`) gives it and passes taps and typed lines back. It holds no
rule of any game.

## What is generated, and by what

`scripts/build-ios.sh`, run from the repository root, makes everything here that is not
committed:

| Generated | From |
| --- | --- |
| `Frameworks/ZMX.xcframework` | `crates/zmachine-ffi`, built for the simulator and for a device |
| `Lamplight/Resources/story.z3` | The story file the script names, after checking its digest against its source record |
| `Lamplight/Resources/story-license.txt` | That story's source and license, from the same folder |
| `Lamplight.xcodeproj` | `project.yml`, by XcodeGen |

## Needs

Xcode 27 or later, XcodeGen (`brew install xcodegen`), and the Rust targets
`aarch64-apple-ios` and `aarch64-apple-ios-sim`.

## Build, run, test

From the repository root:

```bash
scripts/build-ios.sh
```

```bash
xcodebuild -project apps/ios/Lamplight.xcodeproj -scheme Lamplight -destination 'platform=iOS Simulator,name=iPhone 18 Pro,OS=27.2' CODE_SIGNING_ALLOWED=NO build
```

```bash
scripts/test-ios.sh 27
```

For iOS 26.0 the destination is `platform=iOS Simulator,name=iPhone 17 Pro,OS=26.0`, and
the test script takes `26`. A destination must name the system version as well as the
model: model names differ from one version to the next.

## Files

| File | Holds |
| --- | --- |
| `Lamplight/Engine.swift` | The one owner of the session; the only file that touches the C interface |
| `Lamplight/Snapshot.swift` | The snapshot's fields, decoded from the layer's JSON |
| `Lamplight/Game.swift` | What is on screen: the turns so far and the newest snapshot |
| `Lamplight/GameView.swift` | The screen: status, the map's band, what is carried, transcript, field, end states |
| `Lamplight/MapBand.swift` | The map as a band above the reply and opened out to the sheet: the control for each way out, the drag, opening and closing |
| `Lamplight/MapDrawing.swift` | How places, lines, stubs, the lamp and each way's state are drawn, and the pencil's hand |
| `Lamplight/ItemCard.swift` | The card for a carried item |
| `Lamplight/ItemToken.swift` | An item's token, drawn from the layer's word for its kind |
| `Lamplight/MapLayout.swift` | How far apart positions are, where a way's control sits, and its symbol; the only file with direction words |
| `Lamplight/AboutView.swift` | The About sheet |
| `Lamplight/KeptFile.swift` | Where the game in progress is kept on the device; it holds the layer's bytes and does not read them |
| `Lamplight/Announcer.swift` | Posts the layer's announcement for a turn to the system's screen reader |
| `LamplightUITests/` | Tests of the screen, by accessibility identifier |

## The map and the keyboard

The screen is the map (`specs/012-map-first-slice/`). The compass is gone: the ways
out of the place the player is in are controls round that place on the map, with the
states and spoken labels the compass had.

- **The band** is 200 points tall, between the status line and the reply. It shows
  the sheet the player is on, moved so that their place is in the middle. Every
  position, name, state and word in it is the layer's (`map` in the snapshot); the app
  works out nothing.
- **A way names where it leads only once it has been walked.** Until then its control
  says "North", not "North, to Forest Path".
- **A way the layer does not offer is not drawn.** The compass drew it as a faint dot.
- **A place seen only in the dark shows only the way back**: a way the layer says
  leads to a place the player walked there from. No other way has been seen there.
- **Opened out**, by the button at the map's corner or by pulling the band down, the
  sheet has the transcript's room as well, the keyboard goes down, and the sheet can
  be dragged as far as its outermost places. A way can be walked from there. The
  button, or a touch on the command field, brings the band and the keyboard back.
- **The other sheets** (`specs/015-map-sheets/`). Opened out, the map says which
  sheet is shown ("Sheet 2 of 11 · Cellar", the layer's words) and can be turned
  to any other: by a touch on a way that leaves the sheet, which turns to the
  sheet it leads to with the place it leads to in the middle; or by the two
  arrows, which go round the sheets in the order they were found. A sheet the
  player is not on has no lamp and no way to walk, and a control that returns to
  where they are. Turning a sheet sends the game nothing (`zmx_session_sheet`
  only reads). The band always shows the player's own sheet, and closing the map
  or any new turn returns to it. Identifiers: `map.sheet.label`,
  `button.sheet.before`, `button.sheet.after`, `button.sheet.return`,
  `map.lead.<n>`.
- **Drawn as by hand.** A place's outline is a little uneven, worked out from the
  place's id so that it is the same each time; lines bow slightly. A new place and
  its line are drawn in over a quarter of a second and the view glides to the
  player's place; with Reduce Motion both happen at once.
- **A touch on the place the player is in looks around**
  (`specs/013-room-look/`). The layer finds the story's own word for looking and
  says in the snapshot (`map.look`) when the place can be touched; the app holds no
  such word. The touch is sent by `zmx_session_tap_place` and comes back as a
  tapped command with the game's full description. It costs whatever the game
  charges for a look. Other places do nothing when touched. A drag that begins on
  the place still moves the sheet.
- **A touch on a carried item opens its card** over the map
  (`specs/014-inventory-card/`): the game's name for it, what the game says when
  it is examined, and a Drop button.
  The description comes from a copy of the game: the layer runs the story's own
  word for examining there, and the game itself is not touched, so no move is
  spent and nothing is added to the transcript. Drop is sent as a tapped command.
  The card has a drawn edge and sits in from the band's sides. Closing is the
  easy thing: any touch that is not on an action closes it (its control, the card
  itself, the band beside it, the item again), and it goes at the next turn. An
  action is small and has to be aimed at. The close control is at the card's
  bottom left with the actions to its right. A long description scrolls, with a
  mark (`card.more`) while there is more below.
- **The card says what the item is for.** After the close control and its clear
  space come the actions the layer gives, in the layer's order: what can be
  undone first (Open or Close, Turn on or Turn off, Read, Wear or Take off), then
  Drop, Eat and Drink. Each is offered only where the item's own flags say it
  applies and in the state it is in; none is tried beforehand, and the game may
  still refuse. The row scrolls sideways where there are many.
- **Each item has a token** for its kind, on its card and on its chip in the
  strip: a lamp (with rays when lit), a box (lid down or lifted), a page, food or
  drink, clothing, or a plain mark. The kind is the layer's word
  (`Lamplight/ItemToken.swift` is the one file that knows those words). The app holds no command
  word; `zmx_session_card` and `zmx_session_tap_action` are the two entries.
  Identifiers for tests: `item.<index>`, `card`, `card.name`, `card.text`,
  `card.action.<n>`, `card.close`.
- **The marker is a lamp** at the corner of the player's place: lit, out where the
  layer reports the place dark, or plain where the layer cannot say.
- **The keyboard is up** whenever the game waits for a line. It goes down only when
  the player scrolls back through the transcript, and comes up again when they touch
  the field or send. What is carried, and the note of darkness, sit under the band and
  are visible with the keyboard up.
- **A reply is shown from its first line.** One taller than its room has a mark at the
  foot of the transcript: there is more below.
- **A touch on a way** is sent to the layer by the way's place in the snapshot's list
  (`zmx_session_tap_way`), and comes back as a tapped command in the transcript.

Identifiers for tests: `map` (the band; its label is the layer's summary),
`map.frame`, `map.here`, `map.place.<id>`, `map.note`, `map.lamp` (its value is the
layer's word), `button.openMap`, `button.closeMap`, `compass.<way>` and
`compass.extra.<n>` for the ways (kept from the compass), `transcript.more`. With the
launch argument `-showMapHand` there are also `map.outline.<id>` (the numbers a
place's outline is drawn from) and `map.motion` (`full` or `reduced`); `-reduceMotion`
draws the map as the system's Reduce Motion does.

## The kept game

The app brings back the game that was in progress when it was last open
(`specs/011-resume-game/`). This replaces the first demo's rule that each launch is a
new game (feature 004, FR-005).

- **What is kept** is made by the experience layer and stored as it is: which story,
  the chance the game began with, and each input the player gave. It holds no text of
  the game. Its form is `specs/011-resume-game/contracts/kept-game.md`.
- **Where**: `Application Support/Kept/<story>.kept` in the app's own space, left out
  of backups. Removing the app removes it. It is never committed or bundled.
- **When**: after every turn, before the reply is shown and before another command is
  taken, written whole and moved into place.
- **Starting over** is in the About sheet and asks first. "New game" after an ending
  does not ask.
- **A notice** above the command field, in the layer's words, says once when an
  earlier game could not be brought back, or when the game cannot be stored.

Launched with `-fresh`, the app gives up what it kept before it starts; the UI tests
pass it so that each begins with a new game. A debug build launched with `-damageKept`
overwrites the kept file first, for the test of a game that cannot be brought back.

## For a screen reader

When a turn arrives and VoiceOver is running, the app posts the snapshot's
`announcement` to the system, marked to cut off speech in progress
(`specs/010-screen-reader-announcements/`). It posts nothing otherwise.

A simulator cannot be listened to. Launched with `-speakAlways`, the app behaves as if
a screen reader were running and adds an element, `announcement.last`, whose label is
the text it last posted and whose value is how many times it has posted. The UI tests
read it. Without the argument the element does not exist.

VoiceOver changes what a touch does: one tap moves to an item and speaks it, and a
double-tap acts on it.
