# Z-Machine Experience

A faithful Z-machine interpreter in Rust, with a modern touch interface on top, for iPhone
and Android.

Classic text adventures are played by typing. This project keeps the games exactly as
their authors wrote them and adds a layer that lets you play by touch: tap an exit to
walk, tap an object to pick it up. Typing is always still available.

"Z-Machine Experience" is a working name.

## Status

The version 3 interpreter works and is tested. The touch interface does not exist yet.

What the version 3 core does today:

- **Plays version 3 games** from the command line, including Zork I from start to finish.
- **Matches Frotz byte for byte.** A full, winning game of Zork I (350 points, 333
  commands) produces a transcript identical to the one Frotz 2.55 writes.
- **Passes czech**, the standard conformance test program: 349 of 349 tests.
- **Saves and restores** in the standard Quetzal format. Saves interchange with Frotz in
  both directions.
- **Survives bad input.** Thousands of damaged story files and save files are run in the
  tests; each is refused or faulted with a message, and none crashes the host.

The project proceeds in three phases:

1. **Faithful core.** Version 3 is done. Versions 5 and 8, and the praxix test program,
   are next.
2. **Introspection interface.** A written spec, and an implementation on the core, for
   reading live game state. Not started.
3. **First demo.** A game boots on a phone, room text shows, and a tappable compass of
   exits works end to end. Not started.

## Try it

You need Rust 1.99 or later. Zork I is included.

```text
cargo run --release -p zmachine-cli -- stories/zork1/zork1.z3
```

To watch the interpreter play the whole game from a script:

```text
mkdir -p temp
cargo run --release -p zmachine-cli -- --script stories/zork1/fidelity/walkthrough.txt --seed 11 --dir temp stories/zork1/zork1.z3
```

`zm --help` lists the options: a script of commands, a random seed for repeatable runs,
the wrapping width, and a directory for transcripts and saves.

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
| `crates/zmachine-cli` | `zm`, the command-line host. |
| `crates/zmachine-testkit` | Builds small story files in memory, for tests. |
| `stories/` | Story files that may be redistributed, each with its license record. |
| `stories/zork1/fidelity/` | The Zork I walkthrough and what Frotz makes of it. |
| `docs/divergences.md` | Every intentional difference from Frotz. Currently none. |
| `specs/001-v3-core/` | The specification, plan and task list for the version 3 core. |

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

## Scope

Z-machine versions 3, 5 and 8. Version 3 is implemented; 5 and 8 are planned. Version 6
(graphics) is out of scope.

"Faithful" has a precise meaning here. The Z-Machine Standard 1.1 defines correct
behavior, and Frotz 2.55 is the reference the interpreter's output is compared with.
Where the two disagree the interpreter follows the Standard, and the difference is
recorded in `docs/divergences.md` with the section that justifies it.

## Story files

A story file is committed here only when its license permits redistribution, with the
license and source recorded beside it. Two are included:

- **Zork I**, whose source was released under the MIT License in November 2025. See
  `stories/zork1/SOURCE.md`, including a caveat about the compiled file.
- **czech**, a conformance test program by Amir Karger, whose license permits
  redistribution with its notice. See `stories/czech/SOURCE.md`.

Commercial games are never committed or bundled. The MIT License on Zork's code does not
grant the Zork trademark.

## Development

The project uses [GitHub Spec Kit](https://github.com/github/spec-kit) for spec-driven,
test-first development. The governing principles are in the
[constitution](.specify/memory/constitution.md).

## License

[MIT](LICENSE)
