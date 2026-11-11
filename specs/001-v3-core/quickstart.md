# Quickstart: Validating the Version 3 Core

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

These are the checks that show the feature works end to end. Commands are run from the
repository root.

## Prerequisites

- Rust 1.99 or later (`rustc --version`).
- Nothing else for the main test suite: Zork 1 and czech are committed under `stories/`,
  and so are the reference transcripts.
- Only to regenerate the reference files, dfrotz 2.55:

  ```bash
  brew install frotz
  ```

## 1. Everything at once

```bash
cargo test --workspace
```

Expected: all tests pass. The tests that regenerate reference files with dfrotz are
listed as ignored.

## 2. Play a game (User Story 1)

```bash
cargo run -p zmachine-cli -- stories/zork1/zork1.z3
```

Expected: the opening text of Zork 1 and a prompt. `open mailbox` and `read leaflet`
respond correctly; `quit` then `y` exits with status 0.

## 3. Repeatable scripted run (User Story 2)

```bash
mkdir -p temp/a temp/b
```

```bash
cargo run -p zmachine-cli -- --script stories/zork1/fidelity/walkthrough.txt --seed "$(cat stories/zork1/fidelity/seed.txt)" --dir temp/a stories/zork1/zork1.z3
```

```bash
cargo run -p zmachine-cli -- --script stories/zork1/fidelity/walkthrough.txt --seed "$(cat stories/zork1/fidelity/seed.txt)" --dir temp/b stories/zork1/zork1.z3
```

```bash
diff -r temp/a temp/b
```

Expected: `diff` prints nothing. Each directory holds the transcript the walkthrough
started with its `script` command.

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
the command that preceded it. The end of the transcript shows the game won with 350
points.

With dfrotz installed, confirm it still produces the committed reference files:

```bash
cargo test -p zmachine-cli -- --ignored
```

## 6. Save and restore (User Story 5)

```bash
cargo test -p zmachine-cli --test save_restore
```

Expected: pass. For each of five save points, restoring the save in a fresh run and
playing its continuation gives the committed reference transcript, for saves written by
Frotz and by `zm`.

## 7. Misbehaving story files

```bash
cargo test -p zmachine-cli --test malformed
```

Expected: pass. Each of at least twenty malformed or faulting images produces a
diagnostic and a non-zero exit status, and none crashes the host.
