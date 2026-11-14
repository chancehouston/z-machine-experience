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
| `Lamplight/MapBand.swift` | The map as shown above the reply: the sheet about the player's place, and a control for each way out |
| `Lamplight/MapDrawing.swift` | How places, lines, stubs and each way's state are drawn |
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
- **A place seen only in the dark shows no way out**, since none has been seen there;
  the way back is typed.
- **The keyboard is up** whenever the game waits for a line. It goes down only when
  the player scrolls back through the transcript, and comes up again when they touch
  the field or send. What is carried, and the note of darkness, sit under the band and
  are visible with the keyboard up.
- **A reply is shown from its first line.** One taller than its room has a mark at the
  foot of the transcript: there is more below.
- **A touch on a way** is sent to the layer by the way's place in the snapshot's list
  (`zmx_session_tap_way`), and comes back as a tapped command in the transcript.

Identifiers for tests: `map` (the band; its label is the layer's summary),
`map.frame`, `map.here`, `map.place.<id>`, `map.note`, `compass.<way>` and
`compass.extra.<n>` for the ways (kept from the compass), `transcript.more`.

Opening the band out to the whole sheet, and the hand-drawn look, are the feature's
third stage and are not built.

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
