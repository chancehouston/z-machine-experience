# Contract: The Corpus's Files

**Plan**: [../plan.md](../plan.md) | What they hold: [../data-model.md](../data-model.md)

Three kinds of file, all plain text in UTF-8 with lines ending in a line feed. A line
that starts with `#` is a comment in the manifest and the ledger. Reports have no
comments. A file that does not follow its format is refused, with the line number.

## The manifest: `stories/corpus/manifest.txt`

Entries separated by one blank line. Each line of an entry is `key: value`. Keys may come
in any order; none may repeat within an entry.

```text
id: zork1
sha256: 37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79
size: 86838
version: 3
family: zil
source: https://github.com/historicalsource/zork1 COMPILED/zork1.z3
standing: MIT, Copyright (c) 2025 Microsoft; see stories/zork1/SOURCE.md
tier: committed
path: stories/zork1/zork1.z3
seed: 11
walkthrough: stories/zork1/fidelity/walkthrough.txt

id: curses
sha256: <recorded at the first fetch>
size: <the same>
version: 5
family: inform
source: https://ifarchive.org/if-archive/games/zcode/curses.z5
standing: no license attached; personal use only under the archive's terms
tier: fetched
path: stories/local/corpus/curses.z5
seed: 1
opening: no
opening: <a second answer, if the game asks twice>
```

| Key | Required | Value |
| --- | --- | --- |
| `id` | yes | `[a-z0-9-]+`, unique |
| `sha256` | yes | 64 lowercase hexadecimal digits, unique |
| `size` | yes | decimal bytes |
| `version` | yes | `3`, `5` or `8` |
| `family` | yes | `zil`, `inform` or `none` |
| `source` | yes | an address, and after a space a path within it if it is a repository |
| `standing` | yes | one line |
| `tier` | yes | `committed` or `fetched` |
| `path` | yes | relative to the repository's root. `fetched` must be under `stories/local/corpus/`; `committed` must not be |
| `seed` | yes | 0 to 2147483647 |
| `walkthrough` | no | a path; only with `tier: committed` |
| `opening` | no, may repeat | one answer to a line or a key; this key alone may repeat, in order. `(space)` is one space and `(return)` an empty line |
| `rooms`, `trials` | no | the crawl's budget for this story |

**The manifest check** fails when: a required key is missing; a key is not known; two
entries share an `id` or a `sha256`; a `committed` story's file is absent, or its digest,
size, version or family differs, or there is no `SOURCE.md` beside it that holds the
digest, or no license text beside it; a `fetched` path is tracked by the repository; a
present `fetched` story's size, version or family differs.

## The ledger: `stories/corpus/ledger.txt`

Entries separated by one blank line.

```text
story: zork1
room: 82
direction: 28
reported: fixed 83
outcome: moved 84
cause: The game's own code moves the player on at arrival; introspection specification
  section 8.3.
```

| Key | Value |
| --- | --- |
| `story` | a manifest `id` |
| `room`, `direction` | decimal |
| `reported` | `fixed N` or `none` |
| `outcome` | `moved N`, `stayed`, `question`, `ended` or `fault` |
| `cause` | prose; may continue on following lines that start with two spaces |

All six are required. `reported` and `outcome` must together make a wrong verdict. Two
entries may not share story, room, direction, reported and outcome.

## A report: `stories/corpus/reports/<id>.txt`

Every line is one of the forms below, in this order. `N` is a decimal number. Nothing
else may appear: this is what makes "no game text" checkable (research R9).

```text
story <id>
sha256 <64 hex>
version <3|5|8> family <zil|inform|none> seed N
places N walkthrough N crawl N
rooms visited N named-not-reached N
budget rooms N trials N reached <no|rooms|trials>
trials N wordless N
fixed confirmed N wrong-known N wrong-unexplained N
none confirmed N wrong-known N wrong-unexplained N other N
conditional N moved N stayed N question N ended N fault N
unknown N moved N stayed N question N ended N fault N
no-verdict N
detail
<room> <direction> <reported> <outcome> N
...
```

- `other` under `none` counts trials that ended, asked a question or faulted.
- After `conditional N` and `unknown N`, the five outcome counts add up to `N`.
- The four verdict lines and `no-verdict` add up to `trials`.
- A detail line is `N N (fixed N|none|conditional|unknown) (moved N|stayed|question|ended|fault|lost) N`:
  the room, the direction, what was reported, the outcome, and how many trials. One line
  for each case not confirmed, sorted by room, then direction, then the rest as text.
- A story whose family is `none` has `trials 0` and no detail lines.

## The summary: `stories/corpus/reports/summary.txt`

```text
story <id> <family> places N rooms N trials N fixed N/N none N/N conditional N/N unknown N wrong-known N wrong-unexplained N
...
family <zil|inform|none> stories N places N rooms N trials N fixed N/N none N/N conditional N/N unknown N wrong-known N wrong-unexplained N
...
all stories N places N rooms N trials N fixed N/N none N/N conditional N/N unknown N wrong-known N wrong-unexplained N
skipped <id>
...
```

- `fixed A/B`: A confirmed of B reported fixed. `none A/B`: A confirmed of B reported as
  no exit.
- `conditional A/B`: of B trials of exits reported conditional, A moved the player. This
  is the figure for "how often the compass said may be blocked when the way was open".
- `unknown N`: trials of directions introspection could not answer.
- Stories in the order of the manifest; families in the order `zil`, `inform`, `none`.
- A skipped story has a `skipped` line and no `story` line, and is in no total.

## Room for more

Everything above is about exits. A later kind of check (what is in reach, what is
carried, light) keeps a report of its own beside this one, `<id>.<kind>.txt`, with its
own lines, and marks its ledger entries with a further key, `check: <kind>`; an entry
without that key is about exits. No file written now needs rewriting when one is added
(FR-029).
