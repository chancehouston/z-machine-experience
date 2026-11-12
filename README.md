# Z-Machine Experience

A faithful Z-machine interpreter in Rust, with a modern touch interface on top, for iPhone
and Android.

Classic text adventures are played by typing. This project keeps the games exactly as
their authors wrote them and adds a layer that lets you play by touch: tap an exit to
walk, tap an object to pick it up. Typing is always still available.

"Z-Machine Experience" is a working name.

## Status

The interpreter works and is tested for Z-machine versions 3, 5 and 8, and so does the
read-only introspection interface that the touch layer will be built on. The touch
interface itself does not exist yet.

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

The project proceeds in three phases:

1. **Faithful core.** Done: versions 3, 5 and 8, with czech and praxix passing.
2. **Introspection interface.** Done: the specification at version 0.1.0 and its
   implementation, for Infocom/ZIL and Inform 6 games.
3. **First demo.** A game boots on a phone, room text shows, and a tappable compass of
   exits works end to end. Not started.

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

## Layout

| Path | What it is |
| --- | --- |
| `crates/zmachine-core` | The interpreter. No dependencies, no `unsafe`, no I/O of its own. |
| `crates/zmachine-introspect` | The introspection interface: read-only queries over a running game. |
| `crates/zmachine-cli` | `zm`, the command-line host. |
| `crates/zmachine-testkit` | Builds small story files in memory, for tests. |
| `stories/` | Story files that may be redistributed, each with its license record. |
| `stories/zork1/fidelity/` | The Zork I walkthrough and what Frotz makes of it. |
| `stories/advent/fidelity/` | The same for Adventure in versions 5 and 8, with undo and save files. |
| `stories/*/introspection/` | Exits and scope worked out by hand from each game's source, for the tests. |
| `docs/introspection.md` | The introspection specification. |
| `docs/divergences.md` | Every intentional difference from Frotz. Currently none. |
| `specs/001-v3-core/` | The specification, plan and task list for the version 3 core. |
| `specs/002-v5-v8/` | The same for versions 5 and 8, with the screen model's contract. |
| `specs/003-introspection/` | The same for the introspection interface. |

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
license and source recorded beside it. Four are included:

- **Zork I**, whose source was released under the MIT License in November 2025. See
  `stories/zork1/SOURCE.md`, including a caveat about the compiled file.
- **czech**, a conformance test program by Amir Karger, whose license permits
  redistribution with its notice. See `stories/czech/SOURCE.md`.
- **Advent**, the Inform 6 port of Colossal Cave, as a test fixture for versions 5 and 8.
  It is not under this repository's MIT license: it may be redistributed unmodified and
  not for profit, and is not cleared for bundling in an app. See
  `stories/advent/SOURCE.md`.
- **praxix**, a conformance test program by Zarf and Dannii, in the public domain. See
  `stories/praxix/SOURCE.md`.

Commercial games are never committed or bundled. The MIT License on Zork's code does not
grant the Zork trademark.

## Development

The project uses [GitHub Spec Kit](https://github.com/github/spec-kit) for spec-driven,
test-first development. The governing principles are in the
[constitution](.specify/memory/constitution.md).

## License

[MIT](LICENSE)
