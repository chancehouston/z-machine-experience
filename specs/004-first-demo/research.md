# Research: First Demo on a Phone

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

What was checked on this machine is marked **Checked**. What rests on general knowledge
of the tools is marked **Believed**, and is confirmed or corrected by the first tasks.

**Revised after independent review.** One reviewer read the plan against the spec, the
constitution and the code; another built a throwaway app outside the repository to test
the toolchain claims, and recounted the claims about the walkthrough. Two findings were
blockers: the rule for telling a question from a command prompt was wrong (R6), and the
walkthrough comparison could not be made as described (R14). The build approach was
confirmed end to end, with four corrections (R8, R9).

## R1. What this Mac can build

**Checked**: Xcode 27.2 with the iPhone simulator SDK 27.2; simulator runtimes for iOS
26.0, 27.0 and 27.2; Swift 6.4; the Rust targets `aarch64-apple-ios` and
`aarch64-apple-ios-sim` installed; XcodeGen 2.46 installed. The core
and introspection crates build for `aarch64-apple-ios-sim` as they are.

No Android SDK, Gradle or Kotlin. Android is out of scope by the project owner's
decision.

## R2. Decisions by the project owner (2026-11-11)

- iPhone only; done when it runs in the iOS Simulator.
- One screen: text, compass, text field. No inventory, no tappable objects.
- Nothing remembered between launches.
- The app is called **Lamplight** on the home screen.
- The oldest system supported is **iOS 26**.
- A command prompt is told from a question **by where in its code the game is waiting**
  (R6).

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
of eight functions. Putting the bindings in the experience crate would spread `unsafe`
into it.

## R4. What crosses the boundary

**Decision**: The experience layer gives the app one **snapshot**: everything the screen
needs, as JSON text. The app decodes it and draws it. The app gives back one of three
things: a typed line, a tap on a named compass place, or "start a new game".

The C interface is eight functions (contract: [contracts/c-abi.md](contracts/c-abi.md)):
create a session, free it, send a line, tap a place, tap an extra, start anew, take a
snapshot, free a string.

**Rationale**: A snapshot as text keeps the boundary to pointers to bytes, which is the
part of a C interface that is hard to get wrong. Swift decodes JSON with a few lines of
`Codable`. The introspection crate already renders JSON by hand, so no dependency is
added. And it makes SC-011 checkable: the app receives finished facts ("north: open,
sends `north`") and has nothing to decide.

A tap is sent as the place tapped, not as text, together with the turn the app was
showing when it was made. The experience layer sends that place's command only if the
turn is still the current one and the place is lit. So the app cannot send a command the
layer did not offer (FR-012), and a tap that arrives late is refused even when the new
room has an exit the same way (FR-016). A typed line carries the turn too.

Every label the app shows for a place, including what is spoken for it, is in the
snapshot. The app's own knowledge of the compass is where each of the twelve places goes
on the screen (SC-011).

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

**Problem**: The compass must offer nothing while the game asks a question (FR-015). A
game does not mark its questions. The first plan said a command prompt is a stop whose
text ends with `>`; Zork I prints `>` before its yes-or-no questions too ("Do you wish
to leave the game? (Y is affirmative): >"), and both reviewers showed a tap answering
one.

**Decision of the project owner**: tell them apart by where in its code the game is
waiting. Each request for a line is made by one instruction of the story file, at one
address. The address of the **first** request for a line in a game is taken to be its
command prompt. A request from that address is `command`; a request from any other
address is `answer`.

| State | When | Compass | Text field |
| --- | --- | --- | --- |
| `command` | The game wants a line, at the address of its first request | Live | Live |
| `answer` | The game wants a line at any other address, or wants a single key | Offers nothing | Live |
| `ended` | The game quit | Offers nothing | Replaced by "New game" |
| `fault` | The story file did something illegal, ran past the instruction ceiling, or the layer itself failed | Offers nothing | Replaced by "New game" |

**Checked**, with a temporary probe since removed: over the Zork I walkthrough and a
script of quit and restart questions, 329 command prompts wait at one address; the quit
question, the restart question and the end-of-game question each wait at another. Advent
shows the same: 170 command prompts at one address, its final question at another.

**Where the rule is wrong, and which way**:

- **Zork I's Loud Room** has a reader of its own. One command prompt of the walkthrough
  waits there; the rule calls it `answer` and the compass offers nothing. The player
  types, which works.
- **A game that asks a question before its first command** would have that question's
  address taken as the command prompt. Its real command prompts would then be `answer`:
  the compass would never light, and the question would have a live compass once.
- Both failures leave typing working. The first is the safe direction. The second is
  not, and no committed game does it.

**What it needs**: one read-only addition to the core, `Machine::waiting_at()`, giving
the address of the instruction that is waiting for a line or a key, and nothing
otherwise. The core's behaviour does not change.

A request for a single key cannot occur in a version 3 game. If it does, the layer
reports `answer` and sends the first character of what is typed.

## R7. Save, restore, transcript

**Decision**: The experience layer answers the game's request to save with "not stored",
to restore with "nothing to restore", and to start a transcript with "declined". The
layer does this itself; the app never sees the request, and it is not a stop.

**Checked**: Zork I prints "Failed." after a declined restore. It prints "Here begins a
transcript of interaction with…" whether or not the host keeps one, so FR-023 is worded
as "whatever the game then prints is shown", not "the game reports the failure".

## R8. Never freezing the screen, and owning the session in Swift

**Decision**: The app calls the bindings from an actor, never from the main thread. The
experience layer refuses a command that names a turn other than the current one, so a
second tap while one is in flight is refused by the layer and not only by a disabled
button (FR-016). The layer runs the game to its next stop with a ceiling of 50 million
instructions; past that it reports `fault` with a message that the game is not
responding.

**Checked by the spike**: under Swift 6 with strict concurrency and warnings as errors,
an actor holding the session's pointer does **not** compile with an ordinary `deinit`
(`cannot access property 'session' with a non-Sendable type 'OpaquePointer' from
nonisolated deinit`). It compiles cleanly, and runs on iOS 26.0 and 27.2, with
`isolated deinit`. That is what the app uses. No `@unchecked Sendable` is needed.

## R9. Building the app without clicking in Xcode

**Decision**: The Xcode project is generated by XcodeGen from `apps/ios/project.yml`,
which is committed; the generated `.xcodeproj` is not. A script builds the bindings for
the simulator and the device and wraps them, with their header and a module map, as an
XCFramework that the project links. The same script copies the story file and its
license into the app's resources. `xcodebuild` builds and tests from the command line.

**Checked by the spike**, with a throwaway crate and app outside the repository:

- `xcodebuild -create-xcframework -library <sim .a> -headers include -library <device
  .a> -headers include` works, with `include/` holding the header and a
  `module.modulemap`. Swift imports it by module name.
- A generated project with an app target and a UI-test target builds for the simulator
  unsigned (`CODE_SIGNING_ALLOWED=NO`) and with default signing; an unsigned device
  build also succeeds. The spike's `project.yml` is the starting point for the real one.
- A resource declared as a source entry with `buildPhase: resources` lands at the bundle
  root and is read with `Bundle.main`.

**Corrections from the spike**:

- **The destination must name the system version.** `name=iPhone 17 Pro` alone resolves
  to the newest system, where that model does not exist. The working destinations on
  this Mac are `platform=iOS Simulator,name=iPhone 18 Pro,OS=27.2` and
  `platform=iOS Simulator,name=iPhone 17 Pro,OS=26.0`.
- **Simulator builds exclude x86_64** (`EXCLUDED_ARCHS[sdk=iphonesimulator*]: x86_64`),
  since the bindings are built for Apple-silicon simulators only. Without it a generic
  simulator build fails at link. An Intel Mac cannot run the simulator build; that is
  accepted.
- **Portrait only draws an Xcode warning either way**: without `UIRequiresFullScreen`
  it warns that all orientations must be supported, and with it that the key is
  deprecated from iOS 26. Neither fails the build. The app is portrait only and carries
  the first warning; supporting rotation is left for a later feature.
- **Numbers in SwiftUI text are written with `Text(verbatim:)`**, or they are formatted
  with grouping ("1,234") and a test that reads them fails.

**Device builds**: the script also builds `aarch64-apple-ios`, so that running on a
physical iPhone later needs only a signing identity in Xcode. It is not tested in this
feature.

## R10. Testing a phone app

**Decision**: Four levels.

1. **The experience layer, with `cargo test`**: the compass mapping and the waiting
   states on built story files; the walkthrough replayed with taps (R14); new games,
   declined requests, faults, stale taps. This is where behaviour is proven.
2. **The bindings, with `cargo test`**: every function called as C would call it,
   including with null; a loop of create and free; the header checked against the
   library's exported symbols with `nm`. The crate is built as a static library for the
   app and as an ordinary library for these tests.
3. **The app, with XCUITest in the simulator**: launch, read the opening, tap north, see
   the command and the reply, see the compass change, type a command, answer a question
   with the compass dark, end the game and start anew ten times, leave the app and
   return, the largest text size, a story file that is missing (FR-032, FR-006, SC-008,
   SC-009, SC-010).
4. **By eye**: screenshots from the simulator at the default and largest text sizes, in
   light and dark appearance, kept with the feature's notes.

**What the UI tests need from the app**, fixed in [contracts/screen.md](contracts/screen.md):
an identifier on every turn's command and text, not only on the scrolling container,
which does not expose its off-screen rows; and launch arguments for a fixed seed, for a
missing story file, and to show the measured launch time. A fault during play cannot be
provoked from the screen with a healthy story file; it is tested in Rust, and the
screen's handling of the `fault` state is tested with the missing story file.

**Timing**: the layer's time per command and per snapshot is measured in Rust in a
release build (SC-003, SC-007). The launch time (SC-001) is measured by the app itself,
from the start of its process to the first snapshot being shown, and exposed to the UI
test under a launch argument. XCUITest's own launch metric measures the first frame, not
the opening text, and is not used.

**Checked by the spike**: a UI test that launches the app, reads a label, taps a button
and sees the label change passes on iOS 27.2 and on iOS 26.0. The test itself takes
about 8 seconds; a whole `xcodebuild test` takes 25 to 40 seconds, and 77 from a clean
build.

## R11. Continuous integration

**Decision**: The existing workflow keeps running `cargo test` on Linux and macOS, which
now includes the experience layer and the bindings. A third job, macOS only, builds the
app and runs its UI tests if the runner has a new enough Xcode, and is allowed to be
skipped with a notice otherwise.

**Rationale**: GitHub's hosted runners trail the newest Xcode. A job that fails because
the runner is old tells nothing about the code. **Believed**, not checked: which Xcode
the runners have today. The task that adds the job finds out and records it.

## R12. The story file in the bundle, the license notice, and the name

- The build script copies `stories/zork1/zork1.z3` into the app's resources as
  `story.z3`, and `stories/zork1/LICENSE` as `story-license.txt`. The copies are
  generated and not committed, so the repository still holds the story file once, beside
  its record (constitution Principle VI). The script checks the file's SHA-256 against
  `stories/zork1/SOURCE.md` before copying.
- The app has an "About" sheet with that license text and the source the record names
  (FR-029).
- The app's display name is Lamplight. Its bundle identifier is a placeholder,
  `app.lamplight.demo`, to be replaced with one the owner controls before it goes on a
  device.
- A test reads `apps/ios` (sources, `project.yml`, and any text resource but the license
  notice) for the word "Zork", in any case, and fails if it finds it (FR-028). The build
  script is outside `apps/ios` and names the path it copies from.

## R12a. Randomness

**Decision**: The app draws a number from the system's random source and passes it to
the layer when it creates a session and again for each "New game". A new game is a new
machine loaded from the story file with that number, not the game's own restart, so
successive games in one launch differ. A fixed seed, for tests, is from 0 to
2,147,483,647, as the core accepts; a larger one is refused.

If the player types `restart`, the game restarts itself. The core then reseeds from the
number the session was created with, so that game repeats the randomness of the
session's first. This is the core's existing behaviour and is left as it is.

## R13. Open matters

1. **Whether CI can build the app** (R11). Not checked.
2. **Rotation**: portrait only, with one Xcode warning (R9). Say if landscape matters.
3. **The second failure of the prompt rule** (R6): a game that asks a question before
   its first command. No committed game does. It would need a different rule.
4. **Not tested by the spike**: a physical device, any signing identity, the largest
   text size, and the 9-character cut of versions 5 and 8.

## R14. Measuring "a tap is a typed command" on the whole game

**Problem**: The first plan said the replay's transcript would equal
`stories/zork1/fidelity/reference.scr` byte for byte. That file is the game's own
transcript stream, which the walkthrough turns on with `script` and a file name on its
first two lines. The experience layer declines transcripts (R7) and has no such stream,
and the file-name line would reach the game as a command.

**Decision**: The comparison is made with what the layer itself records, with no mode
that exists only for the test.

- A turn's `text` is everything the game printed in reply, unaltered, including the
  prompt that ends it. The layer strips nothing.
- The replay skips the walkthrough's first two lines and gives the other 331.
- What a terminal would have shown from the first command on is `>` followed, for each
  turn after the opening, by the command, a line feed, and the turn's text.
- That must equal the reference from its first command line (`>north`) to its end,
  apart from the line feed the reference interpreter adds when a game quits with a line
  unfinished, which the layer's text has in the same place or the test accounts for by a
  stated rule.

**Believed, and the first thing the test establishes**: that for Zork I the text the
core sends to the screen and the text it sends to a transcript are the same from the
first command on. Feature 001 compared the transcript stream with Frotz, not the screen
text. If they differ anywhere, each difference is listed and explained from the
transcript rules of feature 001, and a difference that cannot be explained stops the
task for the project owner.

**Checked**: 188 of the 331 commands are full direction words, none abbreviated (east
42, north 30, west 28, south 27, down 24, up 20, southwest 7, southeast 5, northeast 3,
northwest 2). At 186 of them the direction is a reported exit, so they can be taps; the
other two (`east` in the Drafty Room, `west` in the Stone Barrow) are not reported as
exits and are typed. One command prompt falls at the Loud Room's own reader (R6) and
its command is typed whatever it is. SC-004 asks for at least 150.

**The compass check (SC-005)** asks a second machine. The test keeps an identical game
beside the session, gives it the same lines, and at each command prompt asks the
introspection interface for its exits; the session's compass must light exactly those,
by a mapping from words to places written in the test, not shared with the layer. At
the walkthrough lines that `stories/zork1/introspection/exits.txt` covers, the compass
must also agree with that hand-made list.
