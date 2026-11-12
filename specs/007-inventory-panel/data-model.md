# Data Model: Inventory Panel

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

## Carried (the inventory answer's entry, changed)

New fields in **bold**.

| Field | Value |
| --- | --- |
| `object`, `name`, `worn`, `contents` | As before. `contents` still lists everything inside |
| **`state`** | `open`, `closed` or `unknown`, for a thing that holds something now; otherwise none |
| **`seen`** | `yes`, `no` or `unknown`. `yes` for every top-level entry |
| **`source`** | `pack` when `state` or `seen` would be different with no pack; otherwise `file` |

Rules (research R1, R6):

- An entry inside another is seen `yes` only if the one it is in is seen `yes` and is
  open, a supporter, or see-through. If the one it is in is seen `no`, it is `no`.
- Where a ZIL story has no pack fact to decide, `state` and `seen` are `unknown`.
- With no pack in force every entry's `source` is `file`.

## Pack (changed)

| Key | Value | Times |
| --- | --- | --- |
| `pack` | `1` or `2` | once |
| `transparent-attribute` | 0 to 31; only in a pack of version 2 | at most once |

Everything else is as in feature 006. A pack of version 1 that holds
`transparent-attribute` is malformed.

## Story (corpus manifest, changed)

| Key | Rules |
| --- | --- |
| `inventory` | The command that asks the game what the player carries. At most once. Without it nothing is compared |
| `aside` | May repeat. Commands separated by ` / `, given from the start of a fresh game after its opening answers. At each prompt the inventory is compared; no exit is tried. Needs `inventory` |

## Comparison

One carried object at one place, set against the game's own listing.

| Seen | Listed | Verdict |
| --- | --- | --- |
| `yes` | listed | agreed |
| `no` | unlisted | agreed |
| `yes` | unlisted | differed |
| `no` | listed | differed |
| `unknown` | either | not judged |

A place where the player carries something and nothing carried is listed is "no
listing" and is not judged. A place where nothing is carried is "empty".

## Ledger entry (second form)

| Field | Rules |
| --- | --- |
| `story`, `room`, `object` | An id and two numbers |
| `seen` | `seen` or `kept` |
| `listed` | `listed` or `unlisted`; together with `seen` it must make a difference |
| `cause` | Prose, not empty, with no name or word of a committed story |

## Strip (the snapshot's `inventory`)

| Field | Value |
| --- | --- |
| `known` | Whether the inventory could be told |
| `note` | Words for "nothing carried" or "not known", or none |
| `items` | Strip items, in the game's order |
| `spoken` | The whole strip, for a screen reader |

**Strip item**: `name` (the game's printed name), `state` (`plain`, `open`, `closed`),
`contents` (strip items). Only what is seen `yes` is an item.

| Carried entry | Strip item |
| --- | --- |
| holds nothing | `plain` |
| `state` `open` | `open`, with its seen contents |
| `state` `closed`, contents seen (see-through) | `closed`, with its contents |
| `state` `closed`, contents not seen | `closed`, with none |
| `state` `unknown` | `plain`, with none |

## What is kept in the repository

| File | Change |
| --- | --- |
| `packs/zork1.pack` | Version 2, with `transparent-attribute` |
| `stories/corpus/manifest.txt` | `inventory` for the ZIL stories it is proven on; one `aside` for `zork1` |
| `stories/corpus/reports/zork1.txt`, `nopack/zork1.txt` | One more line, and detail lines if any |
| `stories/corpus/ledger.txt` | Entries of the second form, if the comparison finds differences with causes |
