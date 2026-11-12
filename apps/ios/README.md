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
| `Lamplight/GameView.swift` | The screen: status, transcript, field, end states |
| `Lamplight/CompassLayout.swift` | Where each of the twelve places goes, and its symbol |
| `Lamplight/CompassView.swift` | How each state is drawn |
| `Lamplight/AboutView.swift` | The About sheet |
| `LamplightUITests/` | Tests of the screen, by accessibility identifier |
