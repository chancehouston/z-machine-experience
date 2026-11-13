# Quickstart: Validating Darkness

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Run from the repository's root. Step 6 needs Xcode and a simulator.

## 1. Everything in Rust

```bash
cargo test --workspace
```

## 2. Lit and dark (User Story 1), and dark ahead (User Story 3)

```bash
cargo test -p zmachine-introspect --lib light
```

Expected: pass. On built stories: a lit room; a dark room; a light carried, put down,
shut in a closed thing, in an open one, in a see-through one; the two unknowns; and
each case again for the room a way leads to.

## 3. On Zork I, with the game as the measure (User Story 2)

```bash
cargo run -q --release -p zmachine-corpus -- run zork1
```

Expected: every entry `proven`, among them `lit-attribute 19`, then `zork1 ok`.

```bash
grep -c "in the dark" stories/corpus/ledger.txt
```

Expected: 48, where it was 87: Zork I's 39 are gone.

## 4. A wrong fact fails

```bash
cargo test -p zmachine-corpus --test light
```

Expected: pass. Each of the other 31 attributes fails on Zork I.

## 5. Nothing changed without a pack

```bash
cargo test -p zmachine-corpus --test nopack && cargo test -p zmachine-introspect --test packs
```

## 6. The warning (User Story 4)

```bash
scripts/test-ios.sh 27
```

Expected: pass, including a test that sees the way up from the kitchen marked, goes up
and sees the note, fetches the lamp, switches it on, and sees neither.
