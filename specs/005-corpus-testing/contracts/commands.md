# Contract: Commands

**Plan**: [../plan.md](../plan.md) | Files: [formats.md](formats.md)

Two things a person runs: the `corpus` program, and the fetch script. The tests use the
same code as the program and never the script.

## `corpus`

Built from `crates/zmachine-corpus`. Run from the repository's root.

```text
corpus check
corpus run [ID...]
corpus write [ID...]
corpus explain ID ROOM DIRECTION
corpus --help
```

| Command | Does | Exit status |
| --- | --- | --- |
| `check` | Checks the manifest and the ledger's form (formats.md). Prints each failure with its file and line | 0 if none; 1 otherwise |
| `run` | Runs each named story, or every present story, and compares its report with the one kept. Prints one line for each story: `ok`, `skipped`, `differs`, or `wrong` with each unexplained case. Then checks the ledger for entries that matched nothing, and the summary | 0 if every present story is `ok` and the ledger and summary hold; 1 otherwise |
| `write` | Runs as `run` does, and writes each present story's report and the summary in place of those kept | 0 unless a story could not be run |

- `explain` runs the story as `run` does and prints every trial that started in that
  room in that direction: where it was (`line N` of the walkthrough, or `crawl`), the
  word sent, the case, and **what the game printed in reply**. It is how a wrong case's
  cause is found. It is the one command that prints a game's text; its output is for the
  terminal and is never written to a file the repository tracks.
- An `ID` that is not in the manifest is an error: exit status 2, nothing run.
- `write` never writes a report for an absent story and never removes one. With some
  stories absent it rewrites only the present stories' lines of the summary.
- `write` does not touch the ledger. An unexplained wrong case is written into the
  report as `wrong-unexplained`; the next `run` fails on it until a person adds it to
  the ledger with a cause, or its cause is fixed.
- Output is to standard output. Except for `explain`, it holds ids, numbers and the
  keywords of the formats, and nothing a game printed.

## `scripts/fetch-corpus.sh`

```text
scripts/fetch-corpus.sh ID...
scripts/fetch-corpus.sh --propose ADDRESS
```

| Form | Does |
| --- | --- |
| `ID...` | For each named `fetched` story in the manifest: downloads from its `source` to its `path`, computes the SHA-256, and keeps the file only if it equals the manifest's; otherwise removes it and says so. A story already present with the right digest is left alone |
| `--propose ADDRESS` | Downloads the address into a new temporary folder and prints the file's name, size, SHA-256 and first byte (its version), for a person to write a manifest entry from. Adds nothing to the repository |

- It takes no "all" option: every story fetched is named by the person running it
  (FR-020, FR-022, FR-023).
- It downloads from the address given and nowhere else, sends nothing but the request,
  and identifies itself only as `curl`.
- It refuses an `ID` whose `tier` is `committed`.
- It is not run by any test or by CI.

## The tests

`cargo test -p zmachine-corpus` runs:

- the crate's unit tests, on stories built in memory;
- `check`, as a test;
- the corpus itself, as a test target with its own `main` (research R16): for each story
  in the manifest it prints one line in the form the standard test runner uses,
  `test <id> ... ok`, `test <id> ... FAILED`, or `test <id> ... ignored, story absent`,
  then the usual line of totals, and exits with a failure if any story failed.

A story's test being ignored never fails the run (SC-005).
