# Quickstart: Validating the Introspection Interface

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Commands are run from the repository root. Rust 1.99 or later is the only requirement.

## 1. Everything at once

```bash
cargo test --workspace
```

Expected: all tests pass, including every test from features 001 and 002, unchanged.

## 2. Watch it while playing (User Stories 1 to 3)

```bash
cargo run -p zmachine-cli -- --inspect - stories/zork1/zork1.z3
```

Expected: the game plays as usual. After each prompt a line of JSON appears on standard
error naming the family (`zil`), the location (`West of House` at the start), its exits
with the words that name them, the inventory and the objects in scope. `north` moves to
the room the `north` exit named.

```bash
cargo run -p zmachine-cli -- --inspect - stories/advent/advent.z5
```

Expected: the same for Advent, with family `inform6`. After `enter building`, the
location is `Inside Building` and scope lists the keys, food, lamp and bottle.

## 3. The object tree and dictionary (User Story 4)

```bash
cargo run -p zmachine-cli -- --tree --dictionary stories/czech/czech.z5
```

Expected: JSON with every object and every dictionary word. czech is of neither family,
and both are still answered in full.

## 4. Nothing changes (User Story 5)

```bash
cargo test -p zmachine-introspect --test read_only
```

Expected: pass. Both walkthroughs, with every query asked at every stop, give the
committed reference transcripts, and the save scripts give the same save files as
without.

## 5. The answers agree with the games

```bash
cargo test -p zmachine-introspect --test zork1 --test advent
```

Expected: pass. At every prompt of both walkthroughs the reported location is the room
the game names; every move through a fixed exit arrives where the exit said; the
inventory matches the game's own, and is unknown at no more than 3% of Zork I's prompts
and none of Advent's; and the hand-made lists of exits for at least 25 rooms in each game
are matched.

## 6. Unrecognised and damaged files

```bash
cargo test -p zmachine-introspect --test families --test robustness
```

Expected: pass. czech and praxix are unrecognised and answer "unknown" to the
family-dependent queries; thousands of damaged files are queried without a panic.

## 7. The specification and the code agree (User Story 6)

```bash
cargo test -p zmachine-introspect --test spec_agreement
```

Expected: pass. `docs/introspection.md` lists exactly the queries the implementation
offers, at the same version.

## 8. Speed, and a blind reading of the specification

```bash
cargo test -p zmachine-introspect --release -- --ignored
```

Expected: pass. All six queries at one prompt of Advent take under 5 milliseconds.

SC-012 is not a command: a reviewer who has not seen the implementation is given
`docs/introspection.md` and the story files, predicts the answers at five positions in
each game, and the predictions are compared with `zm --inspect`. The result is recorded
in tasks.md.

