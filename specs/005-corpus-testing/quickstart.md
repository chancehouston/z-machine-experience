# Quickstart: Validating Corpus Testing

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Commands are run from the repository root and need only Rust, except step 6, which needs
`curl` and `shasum` and the project owner's approval of each file named.

## 1. Everything in Rust

```bash
cargo test --workspace
```

Expected: all tests pass, including every test from the earlier features, unchanged
(SC-010). The corpus target prints a line for each story; a fetched story that is absent
reads `ignored, story absent` (SC-005).

## 2. The manifest and the ledger are well formed

```bash
cargo run -q -p zmachine-corpus -- check
```

Expected: no output, exit status 0. Every committed story has its digest, source and
license recorded beside it (User Story 4).

## 3. One story's figures (User Story 1)

```bash
cargo run -q --release -p zmachine-corpus -- run zork1
```

Expected: `zork1 ok`. The report it was compared with is
`stories/corpus/reports/zork1.txt`: its verdict lines add up to its `trials` line
(SC-001).

## 4. Trials change nothing (SC-002)

```bash
cargo test -p zmachine-corpus --test untouched
```

Expected: pass. Each walkthrough the project holds gives the reference transcript, byte
for byte, with trials run at every prompt.

## 5. A wrong report fails until it is explained (User Story 2)

```bash
cargo test -p zmachine-corpus --test ledger
```

Expected: pass. On a small story built for the test, whose exit table says one thing and
whose code does another: the run fails and names the case; with the case in a ledger it
passes; with a ledger entry that matches nothing it fails; with an entry that has no
cause it fails.

## 6. Add a story that is not committed (User Story 4)

With the owner's approval for the file named:

```bash
scripts/fetch-corpus.sh minizork
```

Expected: the file is in `stories/local/corpus/`, and `git status` shows nothing new.

```bash
cargo run -q --release -p zmachine-corpus -- run minizork
```

Expected: `minizork ok`.

## 7. The whole corpus (User Story 5)

```bash
cargo run -q --release -p zmachine-corpus -- run
```

Expected: one line for each story, `ok` or `skipped`, and exit status 0. The table is
`stories/corpus/reports/summary.txt` (SC-008): for each family, how many exits reported
conditional let the player through, and how many directions introspection could not
answer.

## 8. The same twice (SC-004)

```bash
cargo run -q --release -p zmachine-corpus -- write && git status --short stories/corpus
```

Expected: no output from `git status`: writing the reports afresh changes nothing.

## 9. No game text in the reports (SC-006)

```bash
cargo test -p zmachine-corpus --test no_text
```

Expected: pass.
