# Data Model: Corpus Testing of Introspection

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md) | File formats:
[contracts/formats.md](contracts/formats.md)

Everything here lives in the corpus crate and in files under `stories/corpus/`. Nothing
here is seen by the core, the introspection interface, the experience layer or the app.

## Story

One story file in the corpus. An entry of the manifest.

| Field | Type | Rules |
| --- | --- | --- |
| `id` | short name: lowercase letters, digits, hyphens | Unique. Names the report file |
| `sha256` | 64 hexadecimal digits | Unique. The story's identity (FR-017) |
| `size` | bytes | Must equal the file's |
| `version` | 3, 5 or 8 | Must equal the file's first byte |
| `family` | `zil`, `inform` or `none` | Must equal what introspection detects |
| `source` | address | Where the file comes from |
| `standing` | one line | The license standing, in words |
| `tier` | `committed` or `fetched` | Exactly one |
| `path` | repository path | For `committed`: the file, which must have a `SOURCE.md` beside it holding the digest, and a license text (FR-018). For `fetched`: under `stories/local/corpus/` |
| `seed` | number | The random seed for every run |
| `walkthrough` | repository path, or none | Only for a committed story |
| `opening` | lines, or none | Answers given before the first command prompt (research R6) |
| `rooms`, `trials` | numbers, or none | The crawl's budget, when not the default (research R5) |

A story is **present** when a file at `path` has the digest. An absent story's test is
skipped (FR-021).

## Direction

A direction of a story: a number, and the word a trial sends for it (research R4). Found
once per story from its dictionary. A direction with no word is never tried.

## Place

A game stopped at a command prompt.

| Field | Notes |
| --- | --- |
| The machine | A copy the place owns |
| `room` | The room's object number, as introspection reports it; or unknown |
| `origin` | `line N` of the walkthrough, or `crawl` with the directions taken from where it started |
| The reports | What introspection says of each direction here (below) |

## Reported

What introspection said of one direction at one place.

| Value | From |
| --- | --- |
| `fixed R` | An exit of kind fixed with destination R |
| `conditional` | An exit of kind conditional |
| `unknown` | The direction is in the unknown list, or the whole answer is unknown |
| `none` | The direction is in neither list |

## Trial and outcome

One direction tried on a copy at one place (research R2).

| Outcome | Meaning |
| --- | --- |
| `moved R` | At a command prompt, in room R, which is not the room it started in |
| `stayed` | At a command prompt, in the same room |
| `question` | Waiting for something that is not a command |
| `ended` | The game quit |
| `fault` | The machine faulted, or ran 50 million instructions without stopping |
| `lost` | At a command prompt, but the location is unknown before or after |

## Verdict

From what was reported and the outcome (spec FR-006). Exactly one for each trial.

| Reported | Outcome | Verdict |
| --- | --- | --- |
| `fixed R` | `moved R` | `confirmed` |
| `fixed R` | `stayed`, `moved` elsewhere, `question`, `ended`, `fault` | `wrong` |
| `none` | `stayed` | `confirmed` |
| `none` | `moved R` | `wrong` |
| `none` | `question`, `ended`, `fault` | `none-other` |
| `conditional` | any but `lost` | `conditional` |
| `unknown` | any but `lost` | `unknown` |
| any | `lost` | `no-verdict` |
| `fixed R`, from room R itself | `stayed` | `no-verdict` |

A `wrong` verdict is either **known** (the ledger holds its case) or **unexplained**
(the test fails).

## Case

What identifies a trial's result apart from where in the walk it happened: the story,
the room it started in, the direction, what was reported, and the outcome. Many trials
may share a case (the same room visited at many prompts). The ledger and the report's
detail are keyed by case.

## Ledger entry

| Field | Rules |
| --- | --- |
| `story` | A manifest `id` |
| `room`, `direction` | Numbers |
| `reported`, `outcome` | As above; together they must give the verdict `wrong` |
| `cause` | Prose, not empty. Must not hold any name or word of a committed story (research R9) |

An entry must match at least one trial of a run in which its story was present; an entry
that matches none fails the test (FR-014). An entry for an absent story is not checked.

## Report

The figures for one story on one run. Derived wholly from the trials; holds no text of
the game.

| Part | Holds |
| --- | --- |
| Heading | `id`, digest, version, family, seed |
| Reach | Places, rooms visited, rooms named and not reached, the budget and whether it was reached |
| Totals | Trials; wordless directions; the count of each verdict; `wrong` split into known and unexplained |
| Outcomes | For `conditional` and for `unknown`: the count of each outcome |
| Detail | One line for each case whose verdict is not `confirmed`: room, direction, reported, outcome, how many trials |

Rule: the verdict counts add up to the trials (FR-007).

## Summary

One line of the totals for each story that ran, one for each family, one for all, and a
list of the stories skipped. Each story's line equals its report's totals.

## What is kept in the repository

| File | Kept | Checked against a fresh run |
| --- | --- | --- |
| Manifest, ledger | Yes | They are inputs |
| Report of a committed story | Yes | Always |
| Report of a fetched story | Yes | Where the story is present |
| Summary | Yes | When every story is present; otherwise each present story's line |
| A fetched story file, or any text of it | Never | |
