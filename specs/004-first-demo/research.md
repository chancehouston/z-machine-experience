# Research: First Demo on a Phone

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

What was checked on this machine is marked **Checked**. What rests on general knowledge
of the tools is marked **Believed**, and is confirmed or corrected by the first tasks.

## R1. What this Mac can build

**Checked**: Xcode 27.2 with the iPhone simulator SDK 27.2; simulator runtimes for iOS
26.0, 27.0 and 27.2; Swift 6.4; the Rust targets `aarch64-apple-ios` and
`aarch64-apple-ios-sim` installed; XcodeGen 2.46 and cbindgen 0.29 installed. The core
and introspection crates build for `aarch64-apple-ios-sim` as they are.

No Android SDK, Gradle or Kotlin. Android is out of scope by the project owner's
decision.

## R2. Decisions by the project owner (2026-11-11)

- iPhone only; done when it runs in the iOS Simulator.
- One screen: text, compass, text field. No inventory, no tappable objects.
- Nothing remembered between launches.
- The app is called **Lamplight** on the home screen.
- The oldest system supported is **iOS 26**.

## R3. Three new pieces, and where each lives

**Decision**:

| Piece | Where | Language | May use `unsafe` |
| --- | --- | --- | --- |
| Experience layer | `crates/zmachine-experience` | Rust | No |
| C bindings | `crates/zmachine-ffi` | Rust, built as a static library | Yes, each use commented |
| iPhone app | `apps/ios` | Swift, SwiftUI | No |

**Rationale**: The constitution's layering (Principle V) is shells → experience layer →
introspection → core, with `unsafe` only in the C-binding crate and logic both platforms
need in Rust. Keeping the bindings in a crate of their own means the experience layer is
ordinary safe Rust that `cargo test` exercises without a phone (FR-027), and the bindings
are a thin, mechanical wrapper.

**Alternatives considered**: UniFFI would generate the Swift side, but it brings a tree
of dependencies and generated code into a project that so far has none, for an interface
of seven functions. Putting the bindings in the experience crate would spread `unsafe`
into it.

## R4. What crosses the boundary

**Decision**: The experience layer gives the app one **snapshot**: everything the screen
needs, as JSON text. The app decodes it and draws it. The app gives back one of three
things: a typed line, a tap on a named compass place, or "start a new game".

The C interface is seven functions (contract: [contracts/c-abi.md](contracts/c-abi.md)):
create a session, free it, send a line, tap, start anew, take a snapshot, free a string.

**Rationale**: A snapshot as text keeps the boundary to pointers to bytes, which is the
part of a C interface that is hard to get wrong. Swift decodes JSON with a few lines of
`Codable`. The introspection crate already renders JSON by hand, so no dependency is
added. And it makes SC-011 checkable: the app receives finished facts ("north: open,
sends `north`") and has nothing to decide.

A tap is sent as the place tapped, not as text. The experience layer looks the place up
in the compass it last reported and sends that place's command only if the place is lit.
So the app cannot send a command the layer did not offer (FR-012), and a tap that
arrives late, after the room has changed, is refused, not misapplied.

**Alternatives considered**: A C struct per screen element means dozens of accessors and
lifetimes to manage across the boundary. Passing the whole transcript every time grows
with the game; the snapshot instead carries only the turns since one the app names.

## R5. From an exit to a place on the compass, and the word a tap sends

**Problem**: The introspection interface reports a direction by number, with the words
the story file stores for it. Zork I stores `n`, `north`, `ne`, `northe`. The compass
needs to know which direction is north-east, and a tap should not show the player
`northe`.

**Decision**: The experience layer has a fixed table of the twelve places and the
English word for each: north, northeast, east, southeast, south, southwest, west,
northwest, up, down, in, out. An exit belongs to a place when its words include that
place's word, cut to the length the story file's dictionary keeps (6 characters in
version 3, 9 in versions 5 and 8). A tap on the place sends the full English word.

An exit that matches no place is offered as an extra, labelled with and sending its
longest word (Zork I has `land`).

**Rationale**: The game's parser cuts what the player types to the same length before
looking it up, so `northeast` typed in full is the same command as `northe`. The table is
of English direction words, not of any game: it is the experience layer's knowledge of
what a compass is, as the introspection layer's is of what a family is.

**Checked**: The Zork I walkthrough writes every movement as a full word: 188 of its 331
commands are one of these twelve words (or `land`), none abbreviated. So a tap sends
exactly the text the walkthrough types, and the transcript of a replay with taps can be
compared with the reference byte for byte (SC-004, FR-031).

**If two exits match one place** (a story file whose words collide after cutting), that
place is shown as unknown: the layer does not choose.

## R6. What the game is waiting for

**Decision**: After each run the experience layer reports one of:

| State | When | Compass | Text field |
| --- | --- | --- | --- |
| `command` | The game wants a line and its text ends with the `>` prompt | Live | Live |
| `answer` | The game wants a line but did not print the prompt (a yes-or-no question) | Offers nothing | Live |
| `ended` | The game quit | Offers nothing | Hidden; "New game" offered |
| `fault` | The story file did something illegal, or ran past the instruction ceiling | Offers nothing | Hidden; "New game" offered |

A request for a single key cannot occur in a version 3 game. If it does (a later game),
the layer reports `answer` and sends the first character typed.

**Rationale**: FR-015. The prompt test is the one the introspection tests used to tell a
command prompt from any other stop, at all 332 prompts of the walkthrough.

## R7. Save, restore, transcript

**Decision**: The experience layer answers the game's request to save with "not stored",
to restore with "nothing to restore", and to start a transcript with "declined". The
game prints its own failure message and carries on. The layer does this itself; the app
never sees the request.

**Rationale**: FR-023, and the project owner's decision that nothing is remembered.

## R8. Never freezing the screen

**Decision**: The app calls the bindings from a background task, never from the main
thread, and one call at a time: a second tap while one is in flight is ignored
(FR-016). The experience layer runs the game to its next stop with a ceiling of 50
million instructions; past that it reports `fault` with a message that the game is not
responding.

**Rationale**: A walkthrough command takes well under a millisecond, so in practice the
screen never waits. The background task and the ceiling exist for the story file that
loops (FR-025), which the core already bounds for the same reason.

**Believed**: Swift 6's strict concurrency accepts an actor that owns the session's
pointer. The first app task confirms it.

## R9. Building the app without clicking in Xcode

**Decision**: The Xcode project is generated by XcodeGen from `apps/ios/project.yml`,
which is committed; the generated `.xcodeproj` is not. A script builds the bindings for
the simulator and the device and wraps them, with their header, as an XCFramework that
the project links. `xcodebuild` builds and tests from the command line.

**Rationale**: A hand-edited `.xcodeproj` is a large opaque file that cannot be reviewed
and that tools rewrite. A twenty-line `project.yml` can be.

**Checked**: XcodeGen is installed. **Believed**: `xcodebuild -create-xcframework`
accepts a static library with a headers directory containing a module map. Confirmed by
the first build task; if not, the project links the static library directly with a
header search path.

**Device builds**: the script also builds `aarch64-apple-ios`, so that running on a
physical iPhone later needs only a signing identity in Xcode. It is not tested in this
feature.

## R10. Testing a phone app

**Decision**: Four levels.

1. **The experience layer, with `cargo test`**: unit tests of the compass mapping and the
   waiting states on built story files; the walkthrough replayed with taps (SC-004,
   SC-005); restart, save, restore, script, faults. This is where behaviour is proven.
2. **The bindings, with `cargo test`**: every function called as C would call it,
   including with null and with a freed session where that must be refused; no leaks
   under a loop of create and free.
3. **The app, with XCUITest in the simulator**: launch, read the opening, tap north, see
   the command and the reply, see the compass change, type a command, end the game and
   start anew, largest text size (FR-032, SC-008, SC-010).
4. **By eye**: screenshots from the simulator at the largest and default text sizes and
   in light and dark appearance, kept with the feature's notes.

**Timing (SC-001, SC-003, SC-007)** is measured inside the experience layer for the
part that is the game's (run to next stop, snapshot), and by XCUITest's launch metric for
the cold launch. Simulator timing is noisier than a device's; the thresholds are met
with a wide margin or the plan is wrong.

**Checked**: The simulator can be driven from this session (launch, tap, screenshot).

## R11. Continuous integration

**Decision**: The existing workflow keeps running `cargo test` on Linux and macOS, which
now includes the experience layer and the bindings. A third job, macOS only, builds the
app and runs its UI tests if the runner has a new enough Xcode, and is allowed to be
skipped with a notice otherwise.

**Rationale**: GitHub's hosted runners trail the newest Xcode. A job that fails because
the runner is old tells nothing about the code. **Believed**, not checked: which Xcode
the runners have today. The task that adds the job finds out and records it.

## R12. The license notice and the name

- The app has an "About" sheet with the story file's MIT license text and its source, as
  `stories/zork1/SOURCE.md` records them (FR-029).
- The app's display name is Lamplight. Its bundle identifier is a placeholder,
  `app.lamplight.demo`, to be replaced with one the owner controls before it goes on a
  device.
- A test reads the app's own sources and configuration for the word "Zork" outside the
  bundled story file and its license notice (FR-028).

## R13. Open matters

1. **Whether CI can build the app** (R11).
2. **A spec correction, made in this plan**: none needed for SC-004, since the
   walkthrough types full direction words (R5). Had it abbreviated, the comparison would
   have had to be of replies only.
3. **Rotation** is fixed to portrait for the demo; the spec's edge case asks only that
   rotation not break anything, and not rotating meets it. Say if landscape matters.
