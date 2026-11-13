# Data Model: Compass Look-Ahead

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Nothing here is stored. Each value lives for one prompt, save the corpus's counts.

## Unsettled way

A way on the compass, or beside it, that the layer would report as `maybe` or `unknown`
and that has a command to send.

| Field | Meaning |
| --- | --- |
| command | The text a touch would type |
| where | A place of the compass, or an extra's position in the list |

**Not unsettled**: a place left `unknown` because two exits claim it (nothing is chosen
between them, and so nothing is tried); a closed door a pack reports; any way that is
`open` or `closed`.

## Copy

A clone of the player's machine at a command prompt.

| Field | Meaning |
| --- | --- |
| number | 0, 1 or 2 |
| chance | Copy 0: the game's own. Copies 1 and 2: a value worked out from the count of commands sent and the copy's number |

A copy is given one line, run until it stops or its limit is spent, asked where its
player is, and dropped. Its output is taken and discarded unread.

## Outcome (of one copy)

| Outcome | When |
| --- | --- |
| walked to room R | Stopped at the command prompt; its player is in room R, which is not the room it began in, and both are known |
| stayed | Stopped at the command prompt; its player is in the room it began in |
| nothing | Anything else: stopped elsewhere (a question, a key, a save, a restore, a transcript), ended, faulted, ran past its limit, or its room is not known before or after |

## Finding (of one way)

| Finding | When | Carries |
| --- | --- | --- |
| walked | All three copies walked to the same room | The room's name, and its light, as copy 0 answers `location` after arriving |
| refused | All three copies stayed | Nothing |
| none | Otherwise, or the prompt's total limit was spent before the way was begun | Nothing |

## What the compass then says

| Was | Finding | Becomes | `leads_to` | `dark_ahead` |
| --- | --- | --- | --- | --- |
| `maybe` or `unknown` | walked | `open` | the room's name, if it has one | true if `lit` is `no` |
| `maybe` or `unknown`, on the compass | refused | `closed` | none | false |
| `maybe` or `unknown`, an extra | refused | left out of the list | | |
| `maybe` or `unknown` | none | unchanged | | |

The compass's note that some ways are unknown stays only while a direction is still
unanswered after look-ahead.

## Limits

| Limit | Counted in | Reaching it |
| --- | --- | --- |
| One copy | Instructions | That copy's outcome is nothing |
| One prompt, all copies | Instructions | Ways not yet begun have no finding |

## In the corpus

| Value | Meaning |
| --- | --- |
| found walked, found refused, left | Counts of unsettled ways at the places of a story, by finding |
| borne out | A finding that the corpus's own trial, given a value for chance that no copy used, repeats: the same room, or stayed |
| not borne out | A finding the trial does not repeat. Fails the run unless the ledger names its cause |
| ledger entry | Story, room, direction, the word `lookahead`, and a cause in the project's own words |
