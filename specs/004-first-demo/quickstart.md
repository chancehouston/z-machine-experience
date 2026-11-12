# Quickstart: Validating the First Demo

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Commands are run from the repository root. Steps 1 to 3 need only Rust. Steps 4 onward
need a Mac with Xcode 27 or later, XcodeGen (`brew install xcodegen`) and the Rust
targets for iOS:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

## 1. Everything in Rust

```bash
cargo test --workspace
```

Expected: all tests pass, including every test from the earlier features, unchanged.

## 2. A tap is a typed command (User Story 5)

```bash
cargo test -p zmachine-experience --test walkthrough
```

Expected: pass. The Zork I walkthrough, with a tap sent wherever the compass offered the
move, gives the reference text from the first command to the end, byte for byte; at
least 150 commands went as taps; and at every command prompt the compass agreed with the
introspection interface asked of an identical game.

## 3. The C interface

```bash
cargo test -p zmachine-ffi
```

Expected: pass.

## 4. Build the app

```bash
scripts/build-ios.sh
```

Expected: the bindings are built for the simulator and the device, and
`apps/ios/Lamplight.xcodeproj` is generated.

## 5. Run it in the simulator (User Stories 1 to 4)

```bash
xcodebuild -project apps/ios/Lamplight.xcodeproj -scheme Lamplight -destination 'platform=iOS Simulator,name=iPhone 18 Pro,OS=27.2' -derivedDataPath temp/ios CODE_SIGNING_ALLOWED=NO build
```

Then install and launch the built app in the simulator (or open the project in Xcode and
press Run).

Expected: the game's opening text, "West of House" at the top, and a compass with north,
south and west lit among others. Tapping north shows `north` in the text, then "North of
House"; the compass changes. Typing `open mailbox` in the field works. Typing `quit`
dims the compass while the game asks whether you mean it; `y` ends the game and offers
"New game".

## 6. The automated UI tests

```bash
scripts/test-ios.sh 27
```

```bash
scripts/test-ios.sh 26
```

Expected: one line per test, each passed, on iPhone 18 Pro with iOS 27.2 and on iPhone
17 Pro with iOS 26.0. The simulator's model names differ from one system version to the
next, so the script names both in each destination. A run takes about fifteen minutes.
`testScreenshots` is skipped unless a folder is given for the pictures:

```bash
TEST_RUNNER_SHOTS=$PWD/specs/004-first-demo/screenshots TEST_RUNNER_SHOTS_SUFFIX=light scripts/test-ios.sh 26 -only-testing:LamplightUITests/LamplightUITests/testScreenshots
```

## 7. Timing

```bash
cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture
```

Expected: pass; the slowest command with its snapshot is well under a millisecond.
