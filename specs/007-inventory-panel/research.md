# Research: Inventory Panel

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Each finding is a decision, why, and what else was weighed. Figures marked "measured"
come from temporary tests run on 2026-11-12 and since removed; the tasks repeat each as
a kept test.

## R1. The rule for what is seen

**Decision**: A thing the player carries directly is seen. A thing inside a carried
thing is seen if the thing it is in is itself seen and is open or see-through.
Otherwise it is not seen, or, where the family does not record the facts and no pack
gives them, unknown.

**Measured on Zork I's whole walkthrough**, with attribute 11 as "open" and nothing as
see-through: 332 command prompts; the interface answers at 326; at 47 of those the
player carries something inside something. Set against the game's own listing on a
throwaway copy at every one of the 326, the rule agrees everywhere but two prompts,
and at neither of those did the game list an inventory at all (R4). So "open" as the
pack already states it is right for containers along the whole game.

## R2. "See-through" cannot be proven on the walkthrough

**Measured**: along Zork I's walkthrough the player never carries a closed,
see-through container with something in it. Naming attribute 12 as see-through, or
naming any of 26 other attributes, or naming none, all give the same result (the two
prompts of R4). Only attributes 3, 8, 9, 17 and 18 are contradicted (40, 38, 11, 40 and
40 prompts).

**Decision**: The manifest gains a key, `aside`, which may repeat: a short walk from the
start of a fresh game, its commands separated by ` / `. At each prompt of an aside the
inventory is compared, and nothing else is done: no exit is tried, so the story's exit
figures do not change. Zork I gets one aside that goes into the kitchen and takes the
bottle, closed, with the water in it.

**Measured for that aside** (by hand, with the same commands): the bottle has
attributes 12, 17 and 18, and 3 once touched; closed, the game lists the water in it;
the sack beside it has 17, 18 and 26, and closed, the game lists nothing in it. So with
the aside, attribute 12 is the only one the comparison leaves standing: 17, 18 and 3
are contradicted on the walkthrough, and every other attribute keeps the water back
where the game lists it.

**Rationale**: FR-013 needs a place where the entry matters. A second walkthrough file
would be a large thing for one bottle; a line in the manifest is the project's own
words, as `first` is.

**Alternatives considered**: proving it from what the game lists in a room (the bottle
on the table shows its water too), which needs the room's listing read and is a
different and messier measure; leaving the entry out of Zork I's pack, which the owner
decided against; taking it on trust, which the constitution forbids.

## R3. How the game's listing is read without keeping its text

**Decision**: The corpus gives a throwaway copy the story's inventory command and reads
the reply only to ask, for each object the player carries at any depth, whether the
object's printed name occurs in it. The names come from the story file at that moment,
by the interface's own `name`. An object is "listed" if its name occurs (as many times
as objects of that name are seen, where two share a name). Nothing of the reply is
kept: the report holds counts, and for a difference the room, the object's number, and
the two words `seen`/`kept` and `listed`/`unlisted`.

**Rationale**: FR-011, FR-016. It needs no knowledge of how any game lays out its
listing: no heading, no indentation, no article. Feature 003's test compared names
with the listing's lines and needed the game's heading; this does not.

**Its weakness**: a kept-back thing whose name happens to occur in the name of a listed
thing would be read as listed. Not met on Zork I. Such a case is a difference with a
cause, for the ledger.

**The command** is the manifest's new key `inventory` (FR-012). A story without the key
has no comparison, and its report is unchanged.

## R4. A reply that is not a listing

**Measured**: two of the 326 prompts. At one the room repeats what is typed instead of
obeying; at the other the game has ended and is asking whether to restart.

**Decision**: When the player carries something and no carried thing's name occurs in
the reply, the reply is "no listing": it is counted, and judged neither right nor
wrong. When the player carries nothing there is nothing to compare, and the prompt is
counted as "empty".

## R5. The inventory answer's new parts

**Decision**: The interface specification goes to version 0.3.0. Each entry of the
inventory answer gains three fields:

| Field | Value |
| --- | --- |
| `state` | `open`, `closed` or `unknown` for a thing that holds something now; none for a thing that holds nothing |
| `seen` | `yes`, `no` or `unknown`. Always `yes` at the top level |
| `source` | `file`, or `pack` when `state` or `seen` would be different with no pack |

`contents` is unchanged: everything inside is still listed, with its marks. "The answer
marks; it does not filter" (I§9), and choosing what to show is the caller's business.

**Rationale**: FR-001 to FR-005. The marks reuse words the interface already has
(`yes`/`no`/`unknown`; `open`/`closed`; `file`/`pack`).

**`state` only for a thing that holds something**: whether an empty thing is a
container at all is one more fact a ZIL story numbers for itself, and the strip has no
use for it. The spec's edge case about an open, empty container is amended to match.

**Alternatives considered**: filtering `contents` to what is seen (hides facts from
other callers, and breaks every reader of 0.2); a second query (two answers to keep in
step).

## R6. The rules by family

| Family | A thing that holds something is | What is in it is seen |
| --- | --- | --- |
| Inform 6 | `open` with attribute 14, else `closed`; none if it is a supporter (attribute 20) | if it is a supporter, or open, or has attribute 23 (`transparent`) |
| ZIL, pack with `open-attribute` | `open` with that attribute, else `closed` | if open, or the pack has `transparent-attribute` and it has that attribute |
| ZIL, no `open-attribute` | `unknown` | `unknown` |

Inform's row is the rule I§9.1 already states for scope. With only a
`transparent-attribute` and no `open-attribute`, a thing with the attribute has its
contents seen and its state `unknown`; every other thing is as with no pack.

## R7. The pack format

**Decision**: One new key, `transparent-attribute: N`, 0 to 31, at most once, for the
ZIL family. The format's version becomes 2: a version 2 reader reads versions 1 and 2;
a pack that uses the new key says `pack: 2`. A version 1 reader refuses `pack: 2`, and
so behaves as with no pack, which is the safe side. Zork I's pack becomes version 2;
Mini-Zork's stays 1.

**Rationale**: FR-007, FR-009. A version 1 reader would refuse the unknown key anyway;
saying 2 makes the reason plain.

## R8. Proof

**Decision**, for a story whose manifest gives an inventory command:

- Any difference between "seen" and "listed" with no cause in the ledger fails the
  run, pack or no pack.
- `transparent-attribute` is `proven` when there is no such difference and at least one
  thing is seen and listed whose seeing rests on the entry: it is inside a closed thing
  that has the attribute. With no such thing it is `not-proven`.
- `open-attribute` keeps its proof by doors (feature 006) and must also have no
  unexplained difference. Where the walk has both a closed container whose contents are
  kept and unlisted, and an open one whose contents are seen and listed, that is
  recorded as its evidence on containers; a story with no carried containers needs
  none.

**SC-003** is shown by a test over all 32 attributes on Zork I with its aside.

## R9. The ledger and the report

**Decision**: A ledger entry may be of a second form, for an inventory difference:
`story`, `room`, `object`, `seen` (`seen` or `kept`), `listed` (`listed` or
`unlisted`), `cause`. A report of a story with an inventory command gains one line,
`inventory compared N agreed N differed-known N differed-unexplained N no-listing N
empty N unknown N`, and a detail line for each difference. A story without the command
has neither, so its report is unchanged.

## R10. The strip in the experience layer

**Decision**: The snapshot gains `inventory`, beside `compass`:

| Field | Value |
| --- | --- |
| `known` | false when the inventory answer is unknown |
| `note` | "Carrying nothing", or "Not known: type to ask", or none |
| `items` | The items shown, in the game's order |
| `spoken` | What a screen reader says for the whole strip |

Each item: `name`; `state`, one of `plain`, `open`, `closed`; `contents`, the items
shown inside it, each of the same form. Only things marked seen `yes` are items. A
container whose state is `unknown` is `plain`, and nothing in it is an item (FR-020).

**Rationale**: FR-019 to FR-027. The app composes no text: names, notes and spoken
words are the layer's. When the game is not waiting for a command the app dims the
strip as it dims the compass, from the snapshot's `waiting`, so the layer need not say
it twice.

## R11. The strip in the app

**Decision**: One row between the transcript and the compass, inside the same "not
typing" condition as the compass. A sideways-scrolling row of chips. A container with
shown contents is a chip followed by its contents' chips, joined by a hairline so they
read as one group; a closed container has the outlined-panel mark the compass uses for
a closed way, so "shut" looks the same in both places. Where the row runs past the
edge, the last chip is cut by the edge and a fade marks that there is more.

**How it looks is settled with pictures for the owner**, as the closed way was. The
task builds the first version and stops for the owner's eye.

## R12. Mini-Zork

**Decision**: Its manifest entry gains the inventory command, so its "open" entry is
checked on containers where the file is present. It gets a `transparent-attribute` only
if a short aside can be found that proves one; otherwise none, and its strip keeps
back what is in a closed container (FR-018).

## R13. Speed

**Decision**: The layer asks for the inventory once at each stop, where before it asked
only for the exits. Feature 003 measured all six queries together at about 0.15 ms a
prompt. The timing tests of features 003 and 004 are extended to include it
(SC-010).
