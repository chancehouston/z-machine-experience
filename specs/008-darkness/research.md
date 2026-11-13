# Research: Darkness

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Each finding is a decision, why, and what else was weighed. Figures marked "measured"
come from temporary tests run on 2026-11-12 and since removed; the tasks repeat each as
a kept test.

## R1. The rule for lit and dark

**Decision**: With `lit-attribute: L` in force, the player's room is lit if the room
has attribute L, or a light is present; otherwise dark. A **light** is an object with
attribute L, under the room in the object tree, that can be seen: every object between
it and the room is the player, or is open, or can be seen through.

**Measured on Zork I** with L = 19, open = 11, see-through = 12, the game's own answer
being whether its reply to `look` names the room:

| Where | Places | Lit, agreed | Dark, agreed | Differed |
| --- | --- | --- | --- | --- |
| The walkthrough | 330 | 330 | 0 | 0 |
| A walk of 24 commands into the attic and the cellar | 25 | 19 | 6 | 0 |

The lamp gains attribute 19 when switched on and loses it when switched off; carried or
put down, on, it lights a dark room.

**Alternatives considered**: the game's own flag (R8); a rule without the containers
(a lamp shut in a closed sack would light the room, which the game denies).

## R2. Where the rule cannot say

**Decision**: The answer is `unknown` when it would turn on something that cannot be
read:

- the interface cannot tell which object is the player (I§7.2), the room has not the
  attribute, and no light is seen that is not under one of the objects that tied;
- a thing with the attribute is inside another, the pack has no `open-attribute`, and
  nothing else lights the room.

**Measured**: at the 6 prompts of the walkthrough where the player is ambiguous, the
rooms concerned are lit by other means in most cases; where they are not, the rule
would have guessed wrongly five times in the walks of R3 had it treated "player
unknown" as "carries nothing".

## R3. Dark ahead

**Decision**: Each entry of the exits answer gains `dark_ahead`: `yes`, `no` or
`unknown`. For an exit with a known room (`fixed`, or `door`): `no` if that room has
attribute L, or a light lies in it, or the player carries a light; `yes` if none of
those; `unknown` where R2 applies. For a `conditional` exit: `unknown`.

**Measured on Zork I**, by walking each fixed exit on a copy and asking the copy to
`look`:

| Where | Marked dark, and was | Not marked, and was lit | Differed |
| --- | --- | --- | --- |
| The walkthrough | 14 | 723 | 1 (and 5 where the player was ambiguous, which R2 makes `unknown`) |
| The 24-command walk | 4 | 42 | 0 |

The one difference: the player carried a light and walked into a room where, in the
same turn, another character took it. A timed event; a cause for the ledger.

**Rationale**: FR-004. It is the present-tense rule applied to another room, with the
player's own lights carried along. Nothing is tried.

## R4. The marks

**Decision**: The interface specification goes to 0.4.0.

- `location` gains `source`: `pack` when `lit` would be different with no pack.
- `scope` gains `source`, by the same rule for its `light`.
- An exit's existing `source` is `pack` when any of its fields would be different with
  no pack, which now includes `dark_ahead`. So with a `lit-attribute` in force every
  exit with a known room is `pack`. This is the rule of I§12.5 as already written; its
  reach grows.

**What it costs**: the corpus tells a "distrusted" exit by `kind: conditional` with
`source: pack`; a conditional exit has no room, its `dark_ahead` is `unknown`, and its
source is unaffected, so that reading still holds. A task checks it.

**Alternatives considered**: a separate mark for `dark_ahead` (a second `source` on one
entry, with the first then meaning "every other field"); no mark on exits (against the
constitution's "Marked").

## R5. The pack format

**Decision**: One new key, `lit-attribute: N`, 0 to 31, at most once, for the ZIL
family. The format's version becomes 3, by the rule feature 007 set: a pack that holds
the new key says `pack: 3`; a reader of 3 reads 1, 2 and 3; an older reader refuses it
and behaves as with no pack.

**Rationale**: FR-001, FR-008. Three versions in two days is the price of that rule;
the alternative, adding keys without a version, leaves an older reader's refusal
looking like a malformed file.

## R6. How the corpus asks the game

**Decision**: The manifest gains a key, `look`: the command that asks the game to
describe where the player is. Where it is given, a throwaway copy is asked at every
place, and the room is "described" if the room's own printed name occurs in the reply.
The reply is dropped. A place where the interface cannot name the room is not compared.

**Rationale**: FR-009, FR-010, FR-015. It is feature 007's way of reading an inventory
listing, applied to one name.

**Its weakness**: a game that prints the room's name while refusing to describe it in
the dark would make every dark place look lit. Zork I does not.

## R7. What is compared, and the verdicts

| Compared | When | Agreed | Differed |
| --- | --- | --- | --- |
| Light at a place | every place, where `lit` is yes or no | lit and described, or dark and not | otherwise |
| Dark ahead | a trial that arrives in the room its exit named, where `dark_ahead` is yes or no | marked and not described, or unmarked and described | otherwise |

A difference with no cause in the ledger fails the run. The ledger gains a third form
of entry, with `light` and `described`, and a fourth with `direction`, `ahead` and
`described`.

**Made in the dark** (FR-013): at a place the interface reports as dark, a trial whose
verdict would be `wrong` is counted `in-the-dark` instead. Confirmed reports stay
confirmed. The report says how many.

**The report**, only for a story with `look`, gains two lines: one for light at places
and one for dark ahead, each with compared, agreed, differed-known and
differed-unexplained; the first also with the places agreed dark, the rooms seen both
dark and lit, and the trials counted in the dark.

## R8. The game's own flag

**Measured**: Zork I's global variable 67 was not zero exactly when the game was lit,
over the 24-command walk (6 dark, 18 lit).

**Decision**: Not read. It says what is so now and cannot say what lies ahead; it would
be a second kind of entry for a fact the first already gives; and a stored flag in
this same game has been found one command out of date (feature 006). The proof needs
no second check: the game's own behaviour is the check.

## R9. Proof

**Decision**: `lit-attribute` is `contradicted` if any difference of R7 has no cause;
otherwise `proven` if at least one place agreed dark and at least one room was seen
both dark at one place and lit at another (so a light, and not the room, made the
difference); otherwise `not-proven`.

**Where the places come from**: the crawl already wanders into dark rooms (that is
where Zork I's 39 ledger entries came from), and the walkthrough crosses the same rooms
with a lamp. If that is not enough, an aside is added; asides gain the light
comparison. **SC-003** is a test over all 32 attributes.

## R10. The experience layer and the app

**Decision**: The snapshot gains `light`, beside `compass` and `inventory`:

| Field | Value |
| --- | --- |
| `dark` | `true`, `false`, or `null` where it is not known |
| `note` | "It is dark" when dark, else none |

Each compass place and extra gains `dark_ahead`: true or false (unknown is false: no
mark is shown without knowing). Its `spoken` gains ", into darkness". The compass's
states, commands and existing note are untouched: in the dark the compass is the
room's own (FR-019).

**The app** shows the note on a line of its own above the compass, and draws a way
that is dark ahead as the open disc with a small dark badge at its corner, where
"may be blocked" has its question mark. **How it looks is settled with pictures for
the owner**; the task builds this version and stops.

## R11. Mini-Zork

**Decision**: Tried where its file is. Its crawl reaches a dark room (the twelve
ledger entries of feature 006 are there). It gets the entry only if one attribute
alone is left standing by the test of all 32; otherwise it is left as it is (FR-017).

## R12. Speed

**Decision**: The rule walks the room's objects once for `location`, and once more for
each exit with a known room. Rooms hold few objects. The timing tests of features 003
and 004 run with Zork I's pack as it then is, and must hold (FR-024).
