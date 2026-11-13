# Research: Resume the Game

Each entry: the decision, why, and what else was weighed. Figures marked *measured*
were taken on 2026-11-13 on the development Mac, release build, with a throwaway test
that was removed afterwards. Nothing has been measured on a phone.

## R1. How a game is brought back: by playing its commands again

**Decision**: What is kept is the story's identity, the chance the game was started
with, and every input the player gave, in order. A game is brought back by starting
the story with that chance and giving it those inputs again, through the same code
that gave them the first time.

**Why**:

- **It is exact by construction.** The core's random generator is a function of the
  value the host supplies at the start (`random.rs`: `host`, `state`), and the core
  has no timed input and reads no clock. The same story, the same chance and the same
  inputs give the same game. The constitution's rationale for Principle III already
  says so: "every session is reproducible from its command script". FR-007 and FR-008
  are then not something to be achieved but something already true, to be tested.
- **Everything else comes back with it.** The transcript, the status, the compass,
  the light, what is carried, whether the last turn fell dark, and the announcement
  are all made by the session as it plays. Nothing of them need be kept or restored
  separately, so nothing of them can come back wrong.
- **It is fast enough.** *Measured*: Zork I's walkthrough less its last two lines, 329
  commands, plays in 19 ms with look-ahead off and 46 ms with it on. That is about
  0.06 ms a command. At that rate 2,000 commands are about 120 ms on the Mac. A phone
  is slower; even at five times slower it is well inside two seconds.
- **FR-025 is met without work.** The commands are what is kept.
- **It is small.** A command is a few bytes. *Estimate*: 2,000 turns are 20 to 30 KB.

**Weighed and set aside**:

- **Keeping the machine's state** (the core already writes a Quetzal save). A Quetzal
  save does not hold the random generator's state, the session's transcript, or what
  the layer remembers (the command prompt's address, whether it was dark before). Each
  would need its own form and its own tests, and each is a way for the game brought
  back to differ from the one left. It also would not give the map its commands.
- **Both** (state for speed, commands for the map). Twice the machinery for a speed
  that is not needed.

**What is not known**: the figure on a phone, and the figure for 2,000 commands that
all do real work. In the measurement the game was asking a question after the
walkthrough, so the commands that filled it out to 2,000 were cheap; the 2,000 figure
above is worked out from the 329, not measured. FR-020 requires both to be measured,
on the simulator and on the owner's phone.

## R2. Look-ahead while playing the commands again

**Decision**: While the kept inputs are given again, look-ahead (feature 009) is off.
It runs once, at the last stop, with its ordinary limits.

**Why**: look-ahead tries ways on copies and never touches the game, so leaving it out
changes nothing the game does. Its chance is worked out from the count of turns
(`chance(sent, copy)`), so at the last stop it gives what it gave before the app
closed. *Measured*: it more than halves the time (46 ms to 19 ms).

## R3. Knowing that the game brought back is the game left

**Decision**: What is kept also holds a digest (SHA-256) of every turn's text in
order. After the inputs are given again, the digest of the new transcript is compared.
If they differ, or the game faults or stops responding on the way, the kept game
cannot be brought back (FR-018).

**Why**: R1 holds for one version of the interpreter. An update to the app may change
how the core behaves (a defect put right, a ledgered difference removed). The same
inputs could then give a different game, and FR-018 forbids showing it as the one
left. The digest is how that is known.

**The cost, stated plainly**: after an update that changes what a game printed in the
turns a player took, that player's kept game is given up, with the message. It should
be rare; the core's behaviour is fixed against Frotz by the fidelity tests. The
alternative is to show a game that may be quietly wrong.

**Weighed and set aside**: comparing only the last turn (cheaper, and misses a
difference that has not yet shown); bringing back as far as the first difference (the
player is somewhere they did not leave, with no way to tell them where).

## R4. The story's identity

**Decision**: the SHA-256 of the story file's bytes, as the game pack format already
uses (`docs/introspection.md`, section 12.2; `sha2` is already a dependency of
`zmachine-introspect`).

**Changed in building**: the plan had the experience crate take `sha2` itself. A test
that exists (`tests/crate_rules.rs`) holds that crate to depending only on the layers
beneath it. So `zmachine-introspect` now offers `pack::sha256_hex`, the digest it
already made for a pack's identity, and the experience layer uses that.

**Why**: FR-017 asks for "exactly". Release number and serial are not exact: the
corpus holds stories that share them.

## R5. The form of what is kept

**Decision**: one small text file per story, made whole by the layer at each turn and
handed to the app as bytes. Its form is in `contracts/kept-game.md`: a first line
naming the form and its version, the story's digest, the chance, the count of inputs,
the transcript's digest, the inputs one to a line, and a last line holding a digest
of all that came before it.

**Why text**: a command cannot hold a line break (`send_line` refuses one), so one
line per input needs no escaping beyond none; and a kept game can be read by eye when
something goes wrong.

**Why whole each time, not appended**: FR-011 asks that what is kept is always one
whole game. A whole file written beside the old one and moved over it is whole or
absent. An appended line can be half written. At 30 KB the cost of writing it whole is
small; FR-021 requires it measured.

**Why a last-line digest**: a file cut short or altered is known at once, before any
of it is played (FR-018, FR-029).

## R6. Which turns are kept

**Decision**: the session keeps its own list of inputs that were answered. An input is
added when the stop that follows it is not a fault. The opening turn has no input. A
new game empties the list.

**Why**: the spec's assumption that a fault is not kept. A game brought back from such
a list is the game before the faulting command, waiting for a command.

A game that ended is kept with the input that ended it, and so comes back ended.

## R7. When the app writes

**Decision**: after every command's snapshot is shown, and before `busy` is cleared,
the app asks the layer for the kept bytes and writes them (FR-010). The write is
atomic (to a new file, then moved over the old). It is also written once at launch
after a new game starts, and after starting over, before the new game is shown
(FR-015).

**Why before `busy` clears**: `Game.command` already refuses a second command while
one is being worked out. Keeping inside that window makes FR-010 a property of code
that exists.

**Where**: the app's Application Support directory, in a folder `Kept`, one file named
for the story's resource name. It is marked to be left out of backups, so that it is
not sent anywhere (FR-023). Removing the app removes it.

## R8. The layer's words, and where they are shown

**Decision**: the snapshot gains two keys, after `announcement`:

- `notice`: text or null. The layer's words for "the earlier game could not be brought
  back" and for "your place is not being kept". Each is set once and cleared when the
  next turn is recorded.
- `start_over`: text or null. The layer's question to put before starting over. It is
  text while a game is in progress and null when the game has ended or faulted, so the
  layer, not the app, decides when asking is needed (FR-014, FR-016).

**Why in the snapshot**: every word the app shows that is not a control's own label
already comes that way (`message`, `announcement`, the compass's `spoken`). The
buttons' labels ("Start over", "Cancel") are control labels and stay in the app, as
"New game" is.

**A notice is announced**: with a screen reader running, a notice is spoken before the
turn's announcement. The layer puts it at the front of `announcement` for that
snapshot, so the app's part does not change.

## R9. Where starting over is put

**Decision, for the owner to approve**: a "Start over" button in the About sheet,
which is already reached from the status bar at any time. Choosing it shows the
system's confirmation with the layer's question; confirming starts a new game and
closes the sheet.

**Why there**: it is rarely wanted, it gives something up, and the main screen is
about to be given to the map with the keyboard up. A control that is one sheet away
cannot be hit by accident.

**Weighed and set aside**: a button on the main screen (takes room, and invites
accidents); making the player type the game's own RESTART (it works, but not every
game has it, and a newcomer does not know it).

## R10. The app's tests

**Decision**: a launch argument `-fresh` makes the app remove the kept game before it
starts. The UI tests' shared `launch` helper passes it by default, so every existing
test starts from a new game as before; the tests of this feature launch without it on
their second launch.

Ending the app abruptly in a test is `XCUIApplication.terminate()`, which does not let
the app close politely.

## R11. What changes in tests that exist (FR-019 of earlier features, FR-031)

- `json_tests.rs`: the list of the snapshot's keys gains `notice` and `start_over`.
- `tests/session.rs`: checks that name the snapshot's last key move from
  `announcement` to `start_over`.
- `apps/ios/LamplightUITests`: the `launch` helper gains `-fresh`. No test's expected
  values change.
- `specs/004-first-demo/spec.md` FR-005 is replaced by this feature's FR-001; the
  earlier spec is left as written, and this plan records the replacement.
