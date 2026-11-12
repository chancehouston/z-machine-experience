# Quickstart: Validating Versions 5 and 8

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

The checks that show this feature works end to end. Commands are run from the repository
root. Rust 1.99 or later is the only requirement for all but the optional step.

## 1. Everything at once

```bash
cargo test --workspace
```

Expected: all tests pass, including every test from the version 3 feature. A handful of
those were edited where they asserted that a version 5 or 8 file is refused, or counted
the decoder's instructions; tasks.md, "What was done differently", lists each one.

## 2. Version 3 is unchanged

```bash
cargo test -p zmachine-cli --test fidelity_zork1 --test czech --test save_restore
```

Expected: pass. `git diff main -- crates/zmachine-cli/tests/fidelity_zork1.rs
crates/zmachine-cli/tests/czech.rs crates/zmachine-cli/tests/save_restore.rs stories/zork1`
shows czech and save_restore gaining cases for versions 5 and 8 and nothing else
changing.

## 3. Play a version 5 game, and the same game in version 8 (User Stories 1 and 2)

```bash
cargo run -p zmachine-cli -- --upper stories/advent/advent.z5
```

Expected: the opening of Adventure, with a status line shown above the prompt. `in`,
`take lamp`, `undo`, `score` respond correctly; `help` opens a menu drawn on the whole
screen, where a key is typed and return pressed (`q` leaves it); `quit` then `y` exits
with status 0.

```bash
cargo run -p zmachine-cli -- --upper stories/advent/advent.z8
```

Expected: the same game, behaving the same.

## 4. The upper window (User Story 3)

```bash
cargo test -p zmachine-core --test screen
```

Expected: pass. For each of the ten test programs in the screen contract, the canvas the
host reads equals the expected canvas, cell for cell, including a boxed quotation that is
readable at the next input and gone after it.

## 5. Conformance (User Story 4)

```bash
cargo test -p zmachine-cli --test praxix --test czech
```

Expected: pass. praxix reports that all tests passed; czech reports zero failures for
versions 3, 5 and 8.

## 6. Match the reference interpreter (User Story 5)

```bash
cargo test -p zmachine-cli --test fidelity_advent
```

Expected: pass for both `advent.z5` and `advent.z8`. On failure the output names the byte
offset of the first difference and the command before it. Each transcript ends with the
game won on its maximum score.

## 7. Undo, save and restore (User Story 6)

```bash
cargo test -p zmachine-cli --test undo --test save_restore
```

Expected: pass. Of thirty undos after thirty moves, twenty-five succeed, as in the
reference transcript; saves interchange with the reference transcripts at three points in
each version.

## 8. Optional: against dfrotz itself

With dfrotz 2.55 installed (`brew install frotz`):

```bash
cargo test --workspace --release -- --ignored
```

Expected: pass. dfrotz still writes every committed reference file, restores saves this
interpreter wrote, and each walkthrough finishes in under five seconds.
