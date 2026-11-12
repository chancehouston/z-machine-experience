# The corpus

How often the introspection interface is right about exits, measured on fourteen story
files. Built by feature 005 (`specs/005-corpus-testing/`). The figures below are those of
`reports/summary.txt` as written on 2026-11-12.

## What is measured

At each place a game is brought to, every direction the game has a word for is tried on a
throwaway copy of the game. What the copy did is set against what introspection reported
for that direction at that place:

| Introspection reported | The copy | Verdict |
| --- | --- | --- |
| Leads to room X | arrived in X | confirmed |
| Leads to room X | anything else | wrong |
| Not an exit | stayed where it was | confirmed |
| Not an exit | arrived somewhere | wrong |
| Conditional ("may be blocked") | anything | counted by what happened; never wrong |
| Unknown | anything | counted by what happened; never wrong |

A wrong report fails the tests until its cause is written in `ledger.txt`.

Places are reached by replaying a walkthrough where the repository holds one (Zork I and
Advent), and otherwise by a crawl that walks outward through every direction that moved
the player.

## The files

| File | Holds |
| --- | --- |
| `manifest.txt` | Each story: its SHA-256, size, source, license standing, whether it is committed or fetched |
| `ledger.txt` | Each known wrong report, by story, room and direction, with its cause |
| `reports/<id>.txt` | One story's figures, and each case that was not confirmed |
| `reports/summary.txt` | One line for each story, each family, and all |

Reports hold numbers and identifiers only: no room's name, no object's name and no line
of any game's text. Rooms and directions are by number. The formats are in
`specs/005-corpus-testing/contracts/formats.md`.

## The figures

| | Stories | Places | Rooms | Trials | Reported as leading to a room: confirmed | Reported as not an exit: confirmed | Reported conditional: let the player through |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ZIL family | 6 | 414 | 167 | 5,368 | 937 of 987 (94.9%) | 4,112 of 4,223 (97.4%) | 101 of 138 (73%) |
| Inform family | 5 | 496 | 284 | 5,890 | 1,348 of 1,382 (97.5%) | 4,335 of 4,379 (99.0%) | 51 of 95 (54%) |
| No family | 3 | 3 | 3 | 0 | | | |
| All | 14 | 913 | 454 | 11,258 | 2,285 of 2,369 (96.5%) | 8,447 of 8,602 (98.2%) | 152 of 233 (65%) |

**Of the exits reported conditional, how many let the player through?** In the ZIL
family about three in four; in the Inform family about one in two. Where the compass
says "may be blocked", the way is more often open than not.

**Of all directions tried, how many could introspection not answer?** None was reported
unknown in a story it recognised. But three of the fourteen stories it did not recognise
at all, and on those every direction is unanswered: no trial was run on them.

**Every wrong report is explained.** There are 227 wrong trials, all in the ledger. Not
one is a misreading of a room's exit table. Each is the game's code doing something the
table does not say:

- the player is in the dark, where a walk in any direction is fatal;
- a timed event moves the player, or the player's vehicle, whatever was typed;
- the player is in a vehicle whose own routine refuses the walk, or the destination can
  be entered only in a vehicle;
- an obstacle's routine, or one that runs before the walk, refuses or redirects it,
  sometimes by chance;
- the destination's routine sends the player on at arrival;
- a flag exit whose flag the room sets afresh when a command begins, so that the stored
  value is one command out of date.

The last is the one that bears on a decision already made: feature 003 chose to trust a
flag exit's stored flag. In Zork I there are two rooms where it is stale, in both
directions.

## What the figures do not show

- **Where the player is comes from introspection itself.** A trial is judged by the room
  before and after, as introspection reports it. On Zork I and Advent that report was
  checked separately by feature 003. On the other stories it was not, so their figures
  show that introspection is consistent with itself, which is less than right.
- **A move to the room one is already in cannot be told from staying.** An exit reported
  as leading to its own room has no verdict.
- **A room that reads its own line looks like a question.** One room of Zork I does; a
  trial that arrives there is counted as wrong, and the ledger says why.
- **The crawl only walks.** It opens nothing, takes nothing, lights nothing. Without a
  walkthrough it sees what can be walked to from the opening:

  | Story | Rooms visited | Named by an exit and not reached |
  | --- | --- | --- |
  | zork1 (walkthrough and crawl) | 109 | 1 |
  | advent-z5, advent-z8 (walkthrough and crawl) | 122 | 1 |
  | anchorhead | 31 | 1 |
  | sampler1 | 22 | 0 |
  | zork2 | 15 | 7 |
  | minizork | 12 | 0 |
  | adventureland | 8 | 0 |
  | advent-zil | 7 | 0 |
  | zork3 | 2 | 3 |
  | nine-oh-five | 1 | 0 |

  In `nine-oh-five` the player starts inside something, introspection cannot say which
  room that is, and all twelve trials have no verdict.
- **Three stories are of no family.** `curses` and `balances` are from an early Inform
  library, and `advent-crowther` from Inform 7. Introspection's rules do not recognise
  them, so nothing is measured on them. That is a finding about the rules' reach.
- **The sampler is one file of several games.** `sampler1` is run on the second excerpt
  of its menu only.

## Which stories are here

Five are committed, each with its source, digest and license beside it: `zork1`, `zork2`,
`zork3` (MIT License, with the caveat recorded in each `SOURCE.md`) and the two builds of
Advent (a test fixture only). These run in CI.

Nine are **fetched**: free to download from the IF Archive, with no license attached,
which under the archive's terms means personal use only. They are kept in
`stories/local/corpus/`, which the repository ignores, and are never committed; neither
is any of their text. Their tests are reported as "ignored, story absent" wherever the
files are not present, as in CI. The project owner approved each download by name.

## Running it

From the repository's root:

```bash
cargo run -q --release -p zmachine-corpus -- check
```

```bash
cargo run -q --release -p zmachine-corpus -- run
```

To fetch a story the manifest lists as fetched, by name:

```bash
scripts/fetch-corpus.sh minizork
```

To see what a game printed for one case, for finding a cause (this prints the game's
text, to the terminal only):

```bash
cargo run -q --release -p zmachine-corpus -- explain zork1 249 29
```
