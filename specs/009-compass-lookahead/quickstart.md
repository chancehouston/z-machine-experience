# Quickstart: Validating Compass Look-Ahead

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Run from the repository's root. Step 7 needs Xcode and a simulator.

## 1. Everything in Rust

```bash
cargo test --workspace
```

## 2. The core's new method changes nothing unless called

```bash
cargo test -p zmachine-core random && cargo test -p zmachine-cli --test fidelity_zork1 --test fidelity_advent
```

Expected: pass. Both walkthroughs give the same bytes as before.

## 3. One way, three copies (User Stories 1 to 3)

```bash
cargo test -p zmachine-experience --lib lookahead
```

Expected: pass. On built stories: a way allowed, a way refused, a way decided by
chance (never settled), a way that ends the game, one that asks a question, one that
runs past its limit; and the player's machine equal before and after each.

## 4. The compass and the snapshot

```bash
cargo test -p zmachine-experience --lib compass && cargo test -p zmachine-experience --lib json
```

Expected: pass. The snapshot's keys are what they were.

## 5. The proof, on every story present (User Story 5)

```bash
cargo run -q --release -p zmachine-corpus -- run
```

Expected: every story `ok`. Then:

```bash
grep "^all" stories/corpus/reports/summary.txt
```

Expected: the line ends with the five look-ahead numbers. Research R2 measured 298
walked, 184 refused, 13 left and none not borne out on the seven stories it ran; the
kept figure includes Advent's version 8 build and may differ by its share.

## 6. The limit holds

```bash
cargo test --release -p zmachine-experience --test speed -- --ignored
```

Expected: pass: under 10 ms for a command and its snapshot, with look-ahead on.

## 7. In the app (User Stories 1 and 4)

```bash
scripts/test-ios.sh 27
```

Expected: pass, including a test that sees no way down in the living room, moves the
rug and opens the trap door, and sees an open way down that is spoken as leading to
the cellar and into darkness; and with the lamp on, not into darkness.
