# Z-Machine Experience

A faithful Z-machine interpreter in Rust, with a modern touch interface on top, for iPhone
and Android.

Classic text adventures are played by typing. This project keeps the games exactly as
their authors wrote them and adds a layer that lets you play by touch: tap an exit to
walk, tap an object to pick it up. Typing is always still available.

"Z-Machine Experience" is a working name.

## Status

The interpreter works and is tested for Z-machine versions 3, 5 and 8, and so does the
read-only introspection interface that the touch layer is built on. A first demo of the
touch interface runs in the iPhone simulator: an app called Lamplight that plays Zork I
with a compass of exits you can tap.

What the core does today:

- **Plays version 3, 5 and 8 games** from the command line: Zork I (version 3) and
  Adventure (versions 5 and 8) from start to finish.
- **Matches Frotz byte for byte.** A full, winning game of Zork I (350 points, 333
  commands) produces a transcript identical to the one Frotz 2.55 writes. So does a full,
  winning game of Adventure (350 points, 171 commands), compiled both as version 5 and as
  version 8.
- **Passes the conformance programs.** czech: 349 of 349 tests in version 3 and 406 of
  406 in versions 5 and 8. praxix: all tests passed.
- **Saves, restores and undoes.** Saves are in the standard Quetzal format and
  interchange with Frotz in both directions, in all three versions. Undo keeps the 25
  most recent positions, as Frotz does.
- **Models the screen.** Version 5 games draw a status line, menus and boxed quotations
  outside their main text. The core records what was drawn, with text styles, for a host
  to show as it likes; `zm --upper` prints it above the prompt.
- **Survives bad input.** Thousands of damaged story files and save files are run in the
  tests; each is refused or faulted with a message, and none crashes the host.

What the introspection interface does today:

- **Answers six questions about a running game** without changing it: the object tree,
  the dictionary, where the player is, what they carry, which ways lead out, and what is
  within reach. Its contract is a written specification,
  [docs/introspection.md](docs/introspection.md), that another interpreter could
  implement.
- **Recognises two families of game from their contents alone**: Infocom's ZIL (tested
  on Zork I) and Inform 6 with its standard library (tested on Adventure). Nothing in it
  names a game.
- **Agrees with the games.** At every prompt of both full walkthroughs the location is
  the room the game names and the inventory is what the game lists; every exit it
  reports with a destination led there; and its exits match lists made by hand from the
  games' sources for 49 rooms of Zork I and 50 of Adventure.
- **Says "unknown" when it does not know.** It never guesses and never breaks a tie.
- **Changes nothing.** With every question asked at every prompt, transcripts and save
  files are byte for byte the same.

What the first demo does today:

- **Plays a whole game on an iPhone, in the simulator.** The opening text appears at
  launch with the place, score and moves above it.
- **Shows the ways out as a compass.** Twelve places: the eight points, up, down, in and
  out. A way that is open is a filled disc, and says where it leads when that is known.
- **Tries the uncertain ways first.** Where a game's own code decides whether a way may
  be taken, the layer gives three throwaway copies of the game the direction and sees
  what they do. If all three walk to the same room, the way is drawn open. If all three
  stay put, it is drawn as no way at all. If they differ (the game decided by chance), or
  a copy's game ends or asks a question, the way keeps an outlined disc and a question
  mark. The game being played is never touched.
- **A tap is a typed command.** Tapping north sends the word `north` to the game and
  shows it in the text, exactly as if it had been typed. The whole Zork I walkthrough,
  with 186 of its commands sent as taps, gives the same text as Frotz, byte for byte.
- **Typing always works**, and the compass steps aside for the keyboard.
- **Speaks each reply to a screen reader.** With VoiceOver on, the game's reply is
  spoken as it arrives, without the listener leaving the control they used: the reply,
  then "It is dark" on the turn the dark falls. A new reply cuts off one still being
  spoken. The words are made by the experience layer; the app posts them and composes
  none. Heard on an iPhone on 2026-11-13; the tests check only what the app asks the
  system to speak.
- **Brings the game back where it was left.** The app keeps the game in progress and,
  when it is opened again, carries straight on: the whole transcript, the ways out and
  what is carried, with no question asked. What it keeps is small: which story it is,
  the chance the game was started with, and every command given. The experience layer
  brings the game back by starting the story with that chance and giving it those
  commands again, then checks a digest of the transcript; a game that cannot be brought
  back (the file is damaged, or is another story's) gives a new game and one plain
  message. Zork I's walkthrough, brought back after every one of its commands, is the
  same game each time. "Start over", in the About sheet, asks before a game in progress
  is given up. In the simulator so far; not yet tried on a phone.
- **The layer keeps a map (not yet on the screen).** As the game is played the
  experience layer records the places the player has stood in and the moves they made,
  and puts each place on a page as it is first reached, where it then stays
  (`specs/012-map-first-slice/`). It holds only what the player has witnessed: no room
  they have not been in, and no name of where a way leads until they have walked it. A
  snapshot carries the sheet the player is on. Along Zork I's walkthrough the page has
  83 places on 11 sheets, with 57 of its 60 joins straight; the recorded pages are in
  `stories/zork1/map/` and `stories/advent/map/`. The app does not draw it yet: that
  is the feature's next stage, which replaces the compass.
- **The app holds no rule of any game.** It draws what the experience layer reports and
  sends back taps and typed lines; tests read its source to hold it to that.

What the demo does not do: answer the game's own `save` and `restore` (they are
declined politely; the app keeps the game by itself), keep more than one game of a
story, or run on Android.
The compass goes dark whenever the game asks a question, and at one ordinary prompt of
Zork I (the Loud Room, where the game reads its command from a different place in its
code).

The project proceeds in three phases:

1. **Faithful core.** Done: versions 3, 5 and 8, with czech and praxix passing.
2. **Introspection interface.** Done: the specification at version 0.1.0 and its
   implementation, for Infocom/ZIL and Inform 6 games.
3. **First demo.** Done, in the iPhone simulator: a game boots, room text shows, and a
   tappable compass of exits works end to end.

## Try it

You need Rust 1.99 or later. Zork I and Adventure are included.

```text
cargo run --release -p zmachine-cli -- stories/zork1/zork1.z3
```

```text
cargo run --release -p zmachine-cli -- --upper stories/advent/advent.z5
```

To watch the interpreter play the whole game from a script:

```text
mkdir -p temp
cargo run --release -p zmachine-cli -- --script stories/zork1/fidelity/walkthrough.txt --seed 11 --dir temp stories/zork1/zork1.z3
```

`zm --help` lists the options: a script of commands, a random seed for repeatable runs,
the wrapping width, and a directory for transcripts and saves. For version 5 and 8 games
there are three more:

| Option | What it does |
| --- | --- |
| `--upper` | Prints what the game has drawn outside its main text (status line, menu, quotation box) above the prompt, whenever it has changed |
| `--style ansi\|plain\|auto` | Shows bold, italic and reverse text with terminal escape codes, or not; `auto` does when writing to a terminal |
| `--interpreter-version C` | The letter a game may print as the interpreter's version; `Z` unless set |

When a game wants a single key, type it and press return; an empty line is the return
key.

To watch what the introspection interface reports as you play, or to dump a story file's
tables:

```text
cargo run --release -p zmachine-cli -- --inspect - stories/zork1/zork1.z3
```

```text
cargo run --release -p zmachine-cli -- --tree --dictionary stories/advent/advent.z5
```

| Option | What it does |
| --- | --- |
| `--inspect FILE` | Each time the game waits for input, writes one line of JSON to FILE: the location, exits, inventory and objects in reach. `-` writes to standard error |
| `--tree` | Prints the object tree as JSON and exits |
| `--dictionary` | Prints the dictionary as JSON and exits |

## Lamplight, the iPhone demo

You need a Mac with Xcode 27 or later, XcodeGen (`brew install xcodegen`) and the Rust
targets for iOS (`rustup target add aarch64-apple-ios aarch64-apple-ios-sim`).

```text
scripts/build-ios.sh
```

builds the Rust code for iOS, copies the story into the app and generates
`apps/ios/Lamplight.xcodeproj`. Open that in Xcode, pick an iPhone simulator running
iOS 26 or later, and press Run. To run the app's UI tests from the command line:

```text
scripts/test-ios.sh 27
```

(`26` runs them on iOS 26.0.) They take about ten minutes.

## Tests

```text
cargo test --workspace
```

This needs nothing but Rust. The story files, the reference transcripts and the reference
save files are in the repository.

A few more tests check the committed reference files against Frotz itself. They need
dfrotz from Frotz 2.55 (`brew install frotz` on macOS) and are skipped otherwise:

```text
cargo test --workspace --release -- --ignored
```

## The corpus

How often is introspection right about exits? `crates/zmachine-corpus` measures it: at
each place a game is brought to, every direction is tried on a throwaway copy of the
game, and what happened is set against what introspection reported. On fourteen story
files and 11,310 trials, 96.6% of the exits reported as leading to a room did, and 98.1%
of the directions reported as not an exit were not; every wrong report has its cause
written down. The figures, what they mean and what they do not show are in
[`stories/corpus/README.md`](stories/corpus/README.md).

## Game packs

Some facts a game stores mean different things from game to game: which attribute says
a door is open is number 11 in Zork I and 10 in Mini-Zork. No rule for a whole family of
games can read them. A game pack is a small text file that states such facts for exactly
one story file, named by its SHA-256, release and serial. Its format is section 12 of
[`docs/introspection.md`](docs/introspection.md), so any interpreter can read the packs
in [`packs/`](packs/). A pack holds numbers and no text of a game; every entry is proven
against its story by the corpus's trials, and one that a trial contradicts fails the
build. With no pack, every answer is what it was.

```text
cargo run --release -p zmachine-cli -- --pack packs/zork1.pack --inspect - stories/zork1/zork1.z3
```

With Zork I's pack a door is reported as a door, open or closed, with the room behind
it, where without one the answer is "conditional". The iPhone app carries the pack of
the story it carries: its compass draws a closed door as a door, and an open one as an
open way.

A pack also says what of the player's inventory can be seen: which attribute means a
container is open, and which means it can be seen through. The app shows what is
carried in a row above the compass, always, and shows what the game's own inventory
listing would: a closed sack with nothing in it, a closed glass bottle with its water.
That is proven too. At every place the corpus asks a throwaway copy of the game for its
inventory and sets what it lists against what the interface says is seen; the reply is
read for the names of the things carried and then dropped, and nothing of it is kept.

And a pack says which attribute means lit. With it the interface says whether the
player's room is dark, and for each way out whether the room it leads to would be dark
for the player as they are. The app shows the layer's words, "It is dark", above the
compass, which stays the room's own; a way into darkness carries a small badge and is
spoken as such. Without a pack nothing is claimed. The corpus proves it by asking a
throwaway copy of the game to describe the room, at every place and after every walk.

Look-ahead (feature 009) is measured the same way. At every place, what the three
copies found for each uncertain way is set against a fourth trial with other draws of
chance. Of 210 such ways across the corpus, 191 are settled (105 found open, 86 found
shut) and 19 keep their question mark. Three findings, all in one game's random forest,
were not borne out, and each has its cause in the ledger.

```text
cargo run -q --release -p zmachine-corpus -- check
cargo run -q --release -p zmachine-corpus -- run
```

Five of the stories are in the repository. The other nine are free to download but not to
redistribute: they are fetched by name with `scripts/fetch-corpus.sh` into a folder the
repository ignores, and are never committed. Where they are absent their tests are
reported as ignored, not passed.

## Layout

| Path | What it is |
| --- | --- |
| `crates/zmachine-core` | The interpreter. No dependencies, no `unsafe`, no I/O of its own. |
| `crates/zmachine-introspect` | The introspection interface: read-only queries over a running game. |
| `crates/zmachine-experience` | The experience layer: a session of play, its transcript and its compass. No `unsafe`, no I/O, no game's name. |
| `crates/zmachine-ffi` | C bindings to the experience layer. The only crate that uses `unsafe`. |
| `apps/ios` | Lamplight, the SwiftUI app. Presentation only. |
| `scripts/` | Building and testing the app; regenerating the reference files. |
| `crates/zmachine-cli` | `zm`, the command-line host. |
| `crates/zmachine-testkit` | Builds small story files in memory, for tests. |
| `crates/zmachine-corpus` | Testing tools: tries a game's exits on copies and judges what introspection reported. Nothing depends on it. |
| `stories/corpus/` | The corpus: its manifest, the ledger of known wrong reports, and each story's figures. |
| `packs/` | Game packs: a few facts about one story file each, in the format of `docs/introspection.md` section 12, each proven by the corpus. |
| `stories/` | Story files that may be redistributed, each with its license record. |
| `stories/zork1/fidelity/` | The Zork I walkthrough and what Frotz makes of it. |
| `stories/advent/fidelity/` | The same for Adventure in versions 5 and 8, with undo and save files. |
| `stories/*/introspection/` | Exits and scope worked out by hand from each game's source, for the tests. |
| `docs/introspection.md` | The introspection specification. |
| `docs/divergences.md` | Every intentional difference from Frotz. Currently none. |
| `specs/001-v3-core/` | The specification, plan and task list for the version 3 core. |
| `specs/002-v5-v8/` | The same for versions 5 and 8, with the screen model's contract. |
| `specs/003-introspection/` | The same for the introspection interface. |
| `specs/004-first-demo/` | The same for the first demo, with the C interface, the screen's contract and screenshots. |
| `specs/005-corpus-testing/` | The same for corpus testing, with the formats of its files and its commands. |
| `specs/006-game-packs/` | The same for game packs. |
| `specs/007-inventory-panel/` | The same for the inventory strip. |
| `specs/008-darkness/` | The same for darkness: lit or dark, and dark ahead. |
| `specs/009-compass-lookahead/` | The same for look-ahead: an uncertain way tried on copies of the game. |
| `specs/010-screen-reader-announcements/` | The same for what a screen reader speaks when a turn arrives. |

## How it fits together

| Layer | What it does |
| --- | --- |
| Shells | SwiftUI and Jetpack Compose apps. Presentation only. |
| Experience layer | Turns taps into the command text a player would have typed. |
| Introspection interface | Read-only view of the object tree, location, exits, scope, inventory and dictionary. |
| Core | The Z-machine itself, in Rust. Text in, text out. |

Each layer depends only on the one below it. The shells reach the Rust code through C
bindings.

Two rules shape the design:

- **The engine only takes typed commands.** A tap on a lantern becomes "take lantern",
  and the command is shown to the player. Nothing changes game state behind the game's
  back.
- **Introspection never guesses.** Location, exits and scope are conventions of the
  Infocom and Inform libraries, not of the Z-machine, so they come from adapters per
  compiler family. When an adapter cannot answer, the interface falls back to typing.

The introspection interface is written as a spec that other interpreters could adopt.

What it cannot do, stated in the specification's section 10:

- **Zork's inventory is "unknown" at about one prompt in fifty.** ZIL does not say which
  variable holds the player, so the player is found by elimination, and the rule can
  tie. A tie is reported, never broken.
- **Zork's light and open containers are not known.** ZIL numbers attributes game by
  game. Everything in the room is listed, with what is inside other things marked as of
  unknown visibility.
- **An Inform game that swaps the player to another character** in the same room is not
  always detected. Adventure never does this.
- **An exit is what the room's table says.** A game's own code can still refuse it or
  send the player elsewhere; Adventure's forest does.

## Scope

Z-machine versions 3, 5 and 8, all implemented. Version 6 (graphics) is out of scope.
Colour, sound and timed input are reported to games as unavailable; a game that asks for
them carries on without.

"Faithful" has a precise meaning here. The Z-Machine Standard 1.1 defines correct
behavior, and Frotz 2.55 is the reference the interpreter's output is compared with.
Where the two disagree the interpreter follows the Standard, and the difference is
recorded in `docs/divergences.md` with the section that justifies it. The ledger is
empty: no transcript difference has had to be excused.

A few differences from Frotz cannot show in a compared transcript and are recorded in the
contracts instead (`specs/002-v5-v8/contracts/`):

- **A transcript's file name is asked for once.** Frotz asks a version 5 game again each
  time it restarts a transcript; the Standard says once (§7.1.1.2).
- **A boxed quotation is removed at the next input.** Frotz leaves it on its screen until
  it scrolls away. The core has no scrolling screen, so it hands the box to the host until
  the player next answers.
- **The interpreter's letter is `Z`**, where Frotz says `F`. The comparison is run with
  `--interpreter-version F`.
- **Arrow and function keys in scripts work.** dfrotz 2.55 documents escapes for them but
  gives the game a question mark.

## Story files

A story file is committed here only when its license permits redistribution, with the
license and source recorded beside it. Six are included:

- **Zork I**, whose source was released under the MIT License in November 2025. See
  `stories/zork1/SOURCE.md`, including a caveat about the compiled file.
- **Zork II** and **Zork III**, released the same way on the same day, for the corpus.
  See `stories/zork2/SOURCE.md` and `stories/zork3/SOURCE.md`; the same caveat applies.
- **czech**, a conformance test program by Amir Karger, whose license permits
  redistribution with its notice. See `stories/czech/SOURCE.md`.
- **Advent**, the Inform 6 port of Colossal Cave, as a test fixture for versions 5 and 8.
  It is not under this repository's MIT license: it may be redistributed unmodified and
  not for profit, and is not cleared for bundling in an app. See
  `stories/advent/SOURCE.md`.
- **praxix**, a conformance test program by Zarf and Dannii, in the public domain. See
  `stories/praxix/SOURCE.md`.

The corpus also uses nine story files that are free to download and are not committed;
see `stories/corpus/README.md`.

Commercial games are never committed or bundled. The MIT License on Zork's code does not
grant the Zork trademark.

## Development

The project uses [GitHub Spec Kit](https://github.com/github/spec-kit) for spec-driven,
test-first development. The governing principles are in the
[constitution](.specify/memory/constitution.md).

## License

[MIT](LICENSE)
