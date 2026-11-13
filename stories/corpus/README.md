# The corpus

How often the introspection interface is right about exits, measured on fourteen story
files. Built by feature 005 (`specs/005-corpus-testing/`); feature 006
(`specs/006-game-packs/`) added game packs and their proof. The figures below are those
of `reports/summary.txt` as written on 2026-11-12, with the packs in force.

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
| A door, open, to room X (with a game pack) | arrived in X | confirmed |
| A door, open, to room X | anything else | wrong |
| A door, closed | stayed where it was | confirmed |
| A door, closed | arrived somewhere | wrong |

A wrong report fails the tests until its cause is written in `ledger.txt`.

Places are reached by replaying a walkthrough where the repository holds one (Zork I and
Advent), and otherwise by a crawl that walks outward through every direction that moved
the player. The manifest may give a story a few first commands (`first`), each of which
makes a place before the crawl starts.

## The files

| File | Holds |
| --- | --- |
| `manifest.txt` | Each story: its SHA-256, size, source, license standing, whether it is committed or fetched |
| `ledger.txt` | Each known wrong report, by story, room and direction, with its cause; and each known difference between what is seen of the inventory and what the game lists |
| `reports/<id>.txt` | One story's figures, and each case that was not confirmed |
| `reports/summary.txt` | One line for each story, each family, and all |
| `reports/nopack/<id>.txt` | For a story that has a game pack: its figures with packs set aside |
| `../../packs/<id>.pack` | A story's game pack (`docs/introspection.md`, section 12) |

Reports hold numbers and identifiers only: no room's name, no object's name and no line
of any game's text. Rooms and directions are by number. The formats are in
`specs/005-corpus-testing/contracts/formats.md`.

## The figures

| | Stories | Places | Rooms | Trials | Reported as leading to a room: confirmed | Reported as not an exit: confirmed | Reported conditional: let the player through |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ZIL family | 6 | 418 | 170 | 5,420 | 997 of 1,045 (95.4%) | 4,132 of 4,252 (97.2%) | 58 of 103 (56%) |
| Inform family | 5 | 496 | 284 | 5,890 | 1,348 of 1,382 (97.5%) | 4,335 of 4,379 (99.0%) | 51 of 95 (54%) |
| No family | 3 | 3 | 3 | 0 | | | |
| All | 14 | 917 | 457 | 11,310 | 2,345 of 2,427 (96.6%) | 8,467 of 8,631 (98.1%) | 109 of 198 (55%) |

In this table a door reported open counts with the exits reported as leading to a room,
a door reported closed with those reported as not an exit, and an exit on an untrusted
variable with the conditional. Each story's own report counts them apart.

**Of the exits reported conditional, how many let the player through?** In both
families a little over one in two. Before game packs the ZIL figure was about three in
four (101 of 138): most of those were Zork I's doors standing open, which its pack now
reports as open.

**Of all directions tried, how many could introspection not answer?** None was reported
unknown in a story it recognised. But three of the fourteen stories it did not recognise
at all, and on those every direction is unanswered: no trial was run on them.

**Every wrong report is explained.** There are 183 wrong trials, all in the ledger. Not
one is a misreading of a room's exit table. Each is the game's code doing something the
table does not say:

- the player is in the dark, where a walk in any direction is fatal;
- a timed event moves the player, or the player's vehicle, whatever was typed;
- the player is in a vehicle whose own routine refuses the walk, or the destination can
  be entered only in a vehicle;
- an obstacle's routine, or one that runs before the walk, refuses or redirects it,
  sometimes by chance;
- the destination's routine sends the player on at arrival.

Feature 005 found one more: a flag exit whose flag the room sets afresh when a command
begins, so that the stored value is one command out of date. Zork I has two rooms where
it is stale. Its game pack now names those two variables as untrusted, the exits that
hang on them are reported conditional, and those five entries have left the ledger.

## Game packs

A game pack states facts about one story file that no rule of its family can read
(`docs/introspection.md`, section 12). Two stories have one, in `packs/`:

| Story | Entry | Proven by |
| --- | --- | --- |
| `zork1` | attribute 11 means open | 58 trials of doors: 52 open and 6 closed, all confirmed |
| `zork1` | variables 99 and 136 are untrusted | each has a report that is wrong without it |
| `zork1` | attribute 12 means see-through | the game's own inventory listing, below |
| `minizork` | attribute 10 means open | 6 trials of doors: 4 open and 2 closed, all confirmed |
| `zork1` | attribute 19 means lit | the game describing the room or not, below; no other attribute passes |
| `zork1` | attribute 7 means hidden | without it the game and the interface differ about light; attribute 30 passes as well (`packs/README.md`) |
| `minizork` | attribute 19 means lit | the same; no other attribute passes |

`corpus run` proves every entry, and fails on one that is contradicted, not proven or
unneeded. An "open" entry is proven only when a door has been seen both open and closed
with no wrong report on a door left unexplained. A pack that names any other of Zork I's
32 attributes is contradicted.

What Zork I's pack changed, over the same 4,628 trials:

| | Without the pack | With it |
| --- | --- | --- |
| Reported conditional | 113 | 55 |
| Doors reported open or closed | 0 | 58, all confirmed |
| On an untrusted variable (conditional) | 0 | 24 |
| Wrong | 101 | 57, each with a cause in the ledger |
| Wrong only because the player is in the dark | counted in the 101 | 39, told apart and needing no entry |

The 24 are a cost as well as a gain: 5 of those trials were wrong before, and 19 were
right, 6 of them in a third room that the same variable guards and that was never
wrong. The project owner chose to keep both entries: the stored value cannot be relied
on, wherever it is read.

## Light, set against whether the game describes the room

Feature 008 (`specs/008-darkness/`) added a third measure. Where the manifest gives a
story's look command, a throwaway copy of the game is asked to describe the room at
every place, and on the copy of every trial that arrived where its exit said. The room
counts as described if its own printed name is in the reply; the reply is then dropped.
That is set against what the interface says: lit or dark here, and dark ahead or not on
the way just walked. A difference with no cause in the ledger fails the run.

| | Zork I | Mini-Zork |
| --- | --- | --- |
| Places compared | 372 | 31 |
| Agreed | 370 | 31 |
| Not judged (the interface could not tell the player) | 2 | 0 |
| Places the game left dark | 6 | 4 |
| Ways walked and compared | 847 | 54 |
| Agreed | 836 | 54 |
| Differed, with a cause in the ledger | 6 | 0 |
| Not judged | 5 | 0 |

The six are one moment of Zork I's walkthrough: a character takes the player's light
during the move itself, so the way was lit when asked and dark on arrival.

A game in the dark does not follow its exit tables: a walk in any direction may end the
game. Those trials used to need a ledger entry each. With light known, a trial that
would be wrong at a place the interface reports dark is counted as `in-the-dark`
instead, and 51 entries have left the ledger (Zork I 39, Mini-Zork 12). The ledger now
holds 172 entries. Zork II, Zork III and the sampler have no pack entry for light, and
their darkness entries remain.

## The inventory, set against the game's own listing

Feature 007 (`specs/007-inventory-panel/`) added a second measure. Where the manifest
gives a story's inventory command, a throwaway copy of the game is asked for its
inventory at every place, and each thing the player carries, at any depth, is compared:
the interface says it is seen or kept back; the game's reply names it or does not. The
reply is read only for those names and is not kept. A difference with no cause in the
ledger fails the run.

On Zork I, over its walkthrough, the crawl and one short aside:

| | With the pack | With packs set aside |
| --- | --- | --- |
| Objects compared | 1,523 | 1,521 |
| Agreed | 1,523 | 1,463 |
| Differed | 0 | 0 |
| Not judged (the interface could not say) | 0 | 58 |

So "open" as the pack states it for doors is right for containers too, across the whole
game. "See-through" could not be proven on the walkthrough, which never carries a
closed thing that can be seen through with something in it; the manifest gives Zork I
an aside, five commands that do. With it, attribute 12 is proven and a pack naming any
of the other 31 is contradicted.

The same five commands were tried on Mini-Zork. They carry one such thing, and four of
its attributes fit the game's listing equally, with nothing else carried to tell them
apart. One object cannot single out a fact, so Mini-Zork has no see-through entry.

Mini-Zork's story file is fetched, so its pack is proven only where the file is present;
elsewhere its test is reported as "ignored, story absent". Its doors are reached closed
by walking, so its manifest entry gives three first commands that open one. Because that
changed its walk, its figures with packs set aside are not those of feature 005; Zork
I's are, trial for trial.

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
  | minizork (first commands and crawl) | 15 | 0 |
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

## Look-ahead, set against a trial apart from it

Feature 009 (`specs/009-compass-lookahead/`) added a fourth measure. The experience
layer settles a way the interface reports as conditional, or cannot answer, by trying
it on three throwaway copies of the game: one with the game's own draws of chance and
two with others. All three walking to one room is a finding of *walked*; all three
staying is *refused*; anything else leaves the way as it was. At every place the corpus
asks the layer for each such way's finding, with the limits the app uses, and then
makes a trial of its own on a copy given draws that none of the three had. A finding
the trial does not repeat fails the run until the ledger says why.

| | Ways asked | Found walked | Found refused | Left | Not borne out |
| --- | --- | --- | --- | --- | --- |
| ZIL family | 103 | 54 | 42 | 7 | 3 |
| Inform family | 107 | 51 | 44 | 12 | 0 |
| All | 210 | 105 | 86 | 19 | 3 |

- **191 of 210 are settled (91%).** Of the 198 that were reported conditional, 191
  (96%): the other 12 asked are the directions of one story the interface cannot
  answer at its only place, and no copy settles them.
- **7 are left because the game decides by chance**, all in the ZIL build of Advent,
  whose forest sends a walk to one of two rooms by a draw.
- **3 findings were not borne out**, in the same forest: three copies drew alike and
  the fourth did not. Each is in the ledger. Three copies cannot rule this out; the
  count is what the measure is for.
- **No copy's game ended, asked a question or ran away** on any of these ways.
- **Nothing else moved.** Every other line of every report is what it was.

A refused way is drawn as no way at all, by the owner's choice: 86 of the 191.

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
