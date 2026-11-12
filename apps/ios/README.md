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
