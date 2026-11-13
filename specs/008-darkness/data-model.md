# Data Model: Darkness

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

## Pack (changed)

| Key | Value | Times |
| --- | --- | --- |
| `pack` | `1`, `2` or `3` | once |
| `lit-attribute` | 0 to 31; only in a pack of version 3 | at most once |

## Light

An object with the lit attribute that can be seen from the room: every object between
it and the room is the player, is open, or can be seen through (research R1).

## Location and Scope (changed)

| Field | Value |
| --- | --- |
| `lit` (location), `light` (scope) | With a `lit-attribute` in force for a ZIL story: yes/lit, no/dark, or unknown (research R2). Otherwise as before |
| **`source`** (both) | `pack` when the light would be different with no pack; otherwise `file` |

## Exit (changed)

| Field | Value |
| --- | --- |
| **`dark_ahead`** | `yes`, `no` or `unknown` (research R3) |
| `source` | As before; now also `pack` when `dark_ahead` is `yes` or `no` |

## Story (corpus manifest, changed)

| Key | Rules |
| --- | --- |
| `look` | The command that asks the game to describe where the player is. At most once. Without it nothing about light is compared |

## Light comparison

| The interface said | The game | Verdict |
| --- | --- | --- |
| lit | described the room | agreed |
| dark | did not | agreed |
| lit | did not | differed |
| dark | described it | differed |
| unknown | either | not judged |

The same table serves dark ahead, with "not marked" for lit and "marked" for dark, on
the copy a trial left in the room its exit named.

## Verdict (changed)

One more: `in-the-dark`, for a trial at a place reported dark whose verdict would have
been `wrong` (research R7).

## Ledger entry (third and fourth forms)

| Form | Fields |
| --- | --- |
| Light at a place | `story`, `room`, `light` (`lit` or `dark`), `described` (`yes` or `no`), `cause` |
| Dark ahead | `story`, `room`, `direction`, `ahead` (`dark` or `lit`), `described` (`yes` or `no`), `cause` |

In each, the two words must make a difference.

## The snapshot's `light`, and a compass place (changed)

| Field | Value |
| --- | --- |
| `light.dark` | true, false, or none |
| `light.note` | "It is dark", or none |
| a place's or an extra's `dark_ahead` | true or false |
| its `spoken` | gains ", into darkness" when `dark_ahead` |

## What is kept in the repository

| File | Change |
| --- | --- |
| `packs/zork1.pack` | Version 3, with `lit-attribute` |
| `stories/corpus/manifest.txt` | `look` for `zork1`; an aside if the proof needs one |
| `stories/corpus/ledger.txt` | Zork I's entries whose cause is darkness removed; entries of the new forms where a difference has a cause |
| `stories/corpus/reports/zork1.txt`, `nopack/zork1.txt` | Two more lines; fewer wrong reports with the pack |
