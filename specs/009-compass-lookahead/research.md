# Research: Compass Look-Ahead

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Each finding is a decision, why, and what else was weighed. Figures marked "measured"
come from temporary code run on 2026-11-13 and since removed: the corpus's own loop over
places, with each unsettled way tried on seven copies (one with the game's own draws of
chance, six given other draws) and each trial timed. The tasks repeat each figure as a
kept test or a kept report.

## R1. What copies find on the ways the interface cannot settle

**Measured** on the seven stories of the corpus that have such ways and whose files are
present (Advent's version 8 build was not run apart from its version 5 build; Zork II
and III have none at the places the corpus reaches). A "trial" is one unsettled way at
one place; a place met twice counts twice.

| Story | Trials | All seven copies walked, to one room | All seven stayed | Copies differed |
| --- | --- | --- | --- | --- |
| Zork I | 405 | 264 | 141 | 0 |
| Advent (Inform) | 37 | 23 | 14 | 0 |
| Anchorhead | 19 | 3 | 16 | 0 |
| Advent (ZIL) | 14 | 0 | 1 | 13 |
| Mini-Zork | 10 | 4 | 6 | 0 |
| Sampler | 8 | 2 | 6 | 0 |
| Adventureland | 2 | 2 | 0 | 0 |
| All | 495 | 298 | 184 | 13 |

Zork I's 405 are more than the 79 its report counts as conditional: the rest are ways
on a variable its pack names as untrusted, which the report counts apart and the
compass draws with the same question mark.

Three things follow.

- **482 of 495 (97%) are settled**, so SC-002's "nine in ten" stands, and is now a
  measured figure and not a hope.
- **Chance decided 13, all in one story.** In the ZIL build of Advent the forest sends
  the player to one of two rooms, or nowhere, by a draw.
- **No copy's game ended, asked a question or faulted** on an unsettled way: 0 of 3,465
  runs. User Story 3 is specified and must be built and tested on built stories, but
  the corpus as it stands has no such case to show.

## R2. How many copies, and with which draws

**Decision**: Three copies for each unsettled way: one left with the game's own draws,
two given other draws. A way has a finding only when all three give the same one.

**Measured**: by that rule 482 trials have a finding and 13 do not. Each of the 482
findings was then set against the four further copies: all four agreed, 482 times of
482.

**Why one copy keeps the game's own draws**: that copy does exactly what the player's
game will do if the way is touched now, since it is the same game given the same
command. A finding therefore never contradicts the very next move. The other two are
there so that the compass does not read the dice: without them a way through the forest
would be shown open or shut turn by turn, each time correctly, telling the player a
draw the game has not yet made for them.

**What three copies cannot do**: a way refused one time in a hundred will nearly always
look settled. No number of copies cures that. FR-018's independent trial counts such
misses, and each needs its cause in the ledger.

**Alternatives considered**: one copy with the game's own draws (always right for the
next move, and an oracle for every draw; rejected for that); two copies (a way decided
by an even draw looks settled half the time); seven (no gain measured over three).

## R3. Giving a copy other draws

**Decision**: The core gains one method by which the host gives a machine a new value
for its source of chance, as it gives one at loading. A game that has put the generator
into its predictable mode stays in it. The values given to the two copies are worked
out from the count of commands sent and the copy's number, so the same prompt always
gets the same values (the spec's "a second look gives the same answer").

**Why in the core**: the generator's state is private to the core, and Principle V
already says randomness is supplied by the host through an explicit interface. Loading
takes a seed; this is the same supply at a later moment. Nothing a game can observe
changes unless the host calls it, and the session never calls it on the player's game.

**Alternatives considered**: reaching the state from outside (there is no way, by
design); making the copies by loading the story afresh with another seed and replaying
every command (cost grows with the length of play, and the replay would diverge at the
first draw).

## R4. Where look-ahead lives

**Decision**: In the experience layer, as its own module. The session calls it when it
builds the compass at a command prompt. The corpus crate comes to depend on the
experience layer, so that what it measures is the code the app runs; its own `trial`
stays as it is and serves as the independent check of FR-018.

**Why not in the introspection crate**: that interface reads one machine and runs
nothing (Principle IV). A look-ahead runs a machine. `docs/introspection.md` gains a
short section saying that a conditional answer may be settled by a trial on a copy, and
what such a trial must leave untouched (FR-022); the interface's answers do not change.

**Alternatives considered**: sharing one `trial` between the layer and the corpus (the
proof would then check the code against itself); a new crate (one module does not earn
one).

## R5. What a finding says of the room, and of light

**Decision**: The room's name and whether it is dark are asked of the copy that kept
the game's own draws, by the ordinary `location` query, after it has arrived. Dark
ahead is marked when that answer's `lit` is `no`.

**Why**: the copy is standing in the room, carrying what the player carries. This is
the interface's present-tense answer about a real state, not feature 008's forecast
from here. It needs a pack only because `lit` does (FR-004, FR-012).

**Alternatives considered**: feature 008's rule applied to the room's number (it would
miss a light lost or gained during the walk, which the copy has already lived through).

## R6. What changes in the snapshot, and in the app

**Decision**: Nothing in the snapshot's form. A way found walked is reported with state
`open`, its `leads_to` and `dark_ahead`; a way found refused with state `closed` on the
compass, and an extra found refused is left out of the list; a way with no finding is
`maybe` or `unknown` as before. The spoken words are those the layer already gives each
state. The app's code does not change; it gains tests.

**Why**: the spec's assumption that a settled way looks like any other, and FR-014.

**The compass's note**: "some ways are unknown" is shown today when any direction could
not be answered. It is shown after look-ahead only if one is still unanswered.

## R7. Limits

**Decision**: Two limits, both counted in instructions and not in time, so that the
same prompt always gives the same compass:

- one copy may run a fixed count of instructions before it has no finding;
- all copies at one prompt share a total; a way not begun when it is spent has no
  finding.

The counts are set in the tasks from a kept measurement of the corpus: the single-copy
limit at twenty times the longest trial seen; the total as large as the time test
allows. That test is feature 004's: a command and its snapshot within 10 ms. It runs
with look-ahead on and must hold unchanged.

**Measured** (the owner's Mac, release build, one trial including making the copy):

| Story | Median | Slowest |
| --- | --- | --- |
| Zork I | 47 µs | 131 µs |
| Advent (Inform, version 5) | 633 µs | 847 µs |
| Anchorhead | 295 µs | 759 µs |

At three copies a way costs under 0.4 ms in Zork I and up to 2.6 ms in the slowest
story measured. Against 10 ms that is room for every way of any place in Zork I, the
game the app carries, and for only three or four ways at a place in the Inform build of
Advent, on a Mac. **This is tight.** Where a place has more unsettled ways than the
total allows, the later ones keep their question mark; how many places that touches is
not yet measured, and SC-002's nine in ten must be shown to hold with the limit in
force, not without it. A phone's margin is smaller again and is measured in the tasks,
not assumed.

**Alternatives considered**: a limit in milliseconds (the compass would differ from one
run to the next on a slow day).

## R8. The proof

**Decision**: At each place the corpus asks the layer's look-ahead for a finding on
each unsettled way, and separately runs its own `trial` on a copy given a value for
chance that look-ahead does not use. A finding is **borne out** when the trial walked
to the same room, or stayed, as the finding says. One that is not borne out fails the
run unless the ledger names its cause. Each report gains one line: found walked, found
refused, left, not borne out. Every existing line of every report is unchanged, which
is what "look-ahead set aside" (FR-020) comes to.

**Measured**: see R2; by this measure today's corpus would report 298, 184, 13 and 0.

## R9. Refused ways, counted

**Measured**: 184 of the 482 findings are refusals: by the owner's choice of
2026-11-13 those ways leave the compass. In Zork I that is 141 of 405. This is the
largest visible change the feature makes, and the plan's strongest objection is about
it.
