# Quickstart: Validating the Version 3 Core

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

These are the checks that show the feature works end to end. Commands are run from the
repository root.

## Prerequisites

- Rust 1.99 or later (`rustc --version`).
- dfrotz 2.55, for the fidelity comparison:

  ```bash
  brew install frotz
  ```

  Only the test that regenerates the reference transcript needs it; the comparison itself
  runs against the committed transcript.
- czech, fetched into the gitignored `stories/local/` directory:

  ```bash
  scripts/fetch-stories.sh
  ```

  This downloads czech 0.8 from the IF Archive and verifies it against a recorded SHA-256
  digest. Zork 1 needs no fetching: it is committed at `stories/zork1/zork1.z3`, with its
  release, serial number and digest in `stories/zork1/SOURCE.md`.

## 1. Everything at once

```bash
cargo test --workspace
```

Expected: all tests pass. Tests that need czech or dfrotz when it is missing are listed
as ignored with the reason, never as passed.

## 2. Play a game (User Story 1)

```bash
cargo run -p zmachine-cli -- stories/zork1/zork1.z3
```

Expected: the opening text of Zork 1 and a prompt. `open mailbox` and `read leaflet`
respond correctly; `quit` then `y` exits with status 0.

## 3. Repeatable scripted run (User Story 2)

```bash
cargo run -p zmachine-cli -- --script tests/fidelity/zork1.walkthrough --seed 4242 stories/zork1/zork1.z3 > temp/a.txt
```

```bash
cargo run -p zmachine-cli -- --script tests/fidelity/zork1.walkthrough --seed 4242 stories/zork1/zork1.z3 > temp/b.txt
```

```bash
cmp temp/a.txt temp/b.txt
```

Expected: `cmp` prints nothing. The seed shown is a placeholder; the pinned seed is
recorded with the walkthrough.

## 4. czech (User Story 3)

```bash
cargo test -p zmachine-cli --test czech
```

Expected: pass, with czech's own summary reporting zero failures.

## 5. Match the reference interpreter (User Story 4)

```bash
cargo test -p zmachine-cli --test fidelity_zork1
```

Expected: pass. On failure, the output names the byte offset of the first difference and
the command that preceded it. The final lines of the transcript show the game won with
350 points.

## 6. Save and restore (User Story 5)

```bash
cargo test -p zmachine-cli --test save_restore
```

Expected: pass. For each of five save points in the walkthrough, restoring and continuing
gives the same remaining transcript as an uninterrupted run.

## 7. Misbehaving story files

```bash
cargo test -p zmachine-cli --test malformed
```

Expected: pass. Each of at least twenty malformed or faulting images produces a
diagnostic and a non-zero exit status, and none crashes the host.
