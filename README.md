# Z-Machine Experience

A faithful Z-machine interpreter in Rust, with a modern touch interface on top, for iPhone
and Android.

Classic text adventures are played by typing. This project keeps the games exactly as
their authors wrote them and adds a layer that lets you play by touch: tap an exit to
walk, tap an object to pick it up. Typing is always still available.

"Z-Machine Experience" is a working name.

## Status

Early. The project principles are written; there is no interpreter code yet. Work is
proceeding in three phases:

1. **Faithful core.** A Z-machine interpreter that passes the czech and praxix test
   programs and reproduces a Frotz transcript of a scripted walkthrough byte for byte.
2. **Introspection interface.** A written spec, and an implementation on the core, for
   reading live game state.
3. **First demo.** A game boots on a phone, room text shows, and a tappable compass of
   exits works end to end.

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

Z-machine versions 3, 5 and 8. Version 6 (graphics) is out of scope.

## Story files

This repository contains no story files, and commercial games such as Zork are never
committed or bundled. To run the tests that use one, you supply your own copy locally;
those tests are reported as skipped when the file is absent.

## Development

The project uses [GitHub Spec Kit](https://github.com/github/spec-kit) for spec-driven,
test-first development. The governing principles are in the
[constitution](.specify/memory/constitution.md).

## License

[MIT](LICENSE)
