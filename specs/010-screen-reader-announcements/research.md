# Research: Screen Reader Announcements

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Each finding is a decision, why, and what else was weighed. Figures marked "measured"
come from a temporary test run on 2026-11-13 and since removed; the tasks repeat each as
a kept test.

## R1. How a turn's text ends

**Measured** over the two walkthroughs the project holds, through the experience layer,
the last characters of every turn's text by what the game was then waiting for:

| Story | Turns | Waiting for a command | Waiting for an answer | Ended |
| --- | --- | --- | --- | --- |
| Zork I | 332 | 329, every one ending `>` | 2, both ending `>` | 1, ending in a line feed |
| Advent (Inform, version 5) | 172 | 170, every one ending `>` | 1, ending `>` | 1, ending in a line feed |

In each story one turn has nothing left once the prompt and the space around it are
taken away. The longest reply is 1,035 characters in Zork I and 1,233 in Advent.

**Decision**: The reply is the turn's text with the white space at its end removed,
then one final `>` removed if the game is waiting for a line, then the white space at
both ends removed. Where the game is not waiting for a line (it has ended, or wants a
key) no `>` is removed.

**Why**: in every turn measured where the game waits for a line, the last character is
the game's prompt, for a question as for a command. A `>` anywhere else in a reply is
the game's text and is kept.

**Alternatives considered**: removing `>` only at a command prompt (a question's prompt
would be read aloud as "greater than"); a rule from the game's own prompt routine
(there is none to read: the prompt is printed text).

## R2. Where the announcement is made

**Decision**: The experience layer makes it, once for each turn, and the snapshot
carries it: a field `announcement`, the text to speak, or null. It is made when the
turn is recorded and belongs to that turn.

**Why**: FR-012. The app composes nothing, and Android would read the same field.

**What it holds**, in order, joined by a line feed, each part left out if empty:

1. the reply (R1);
2. the layer's note of darkness, if this turn fell dark (R3);
3. the layer's words for an ended game, if the game ended on this turn (R4).

Where the game has stopped with a fault, the announcement is the fault's message alone.
If no part is left, the announcement is null.

**Alternatives considered**: giving the parts apart for the app to join (the app would
then compose); giving it with each turn in the list of turns (a host asks for turns
"since" a number, and would have to choose one; the newest turn's is the only one
ever spoken).

## R3. Fell dark

**Decision**: The session remembers whether the player was in the dark at the stop
before. A turn fell dark when the layer's light now says dark and it did not at the
stop before. The opening turn of a game falls dark if it is dark. A stop where light is
unknown, or where the game is not waiting for a command (feature 008 says nothing of
light there), counts as not dark, so that walking out of a question into the same dark
room says the note again.

**Why**: the owner's choice that the note is said when it appears. The last clause
follows from what the screen does: at a question the note is not shown, and after it
the note appears again.

## R4. The words for an ended game

**Decision**: They move from the app's source to the layer. When the game has ended,
the snapshot's `message` holds "The game has ended.", as it holds a fault's message
when the game has faulted, and the app shows `message` in both cases.

**Why**: FR-012 cannot be met while the app holds a sentence that is to be spoken. The
words on the screen do not change.

**Cost**: tests that fix `message` as null for an ended game change, and are listed.

## R5. Asking the system to speak

**Decision**: When a turn is shown and a screen reader is running, the app posts the
snapshot's `announcement` as a system announcement marked to interrupt speech in
progress. It posts nothing when the field is null, when no screen reader is running,
or when the snapshot shown brought no new turn.

**Not verified, and why**: that an announcement so marked cuts off an earlier one, and
is itself not cut off by the screen reader speaking the control just used (the send
button, or a way on the compass), is how the system is documented to behave and is
**not something this project has heard**. The simulator has no screen reader to listen
to. The tasks put this before the owner on a device early, with a stop, because the
whole feature rests on it.

**Alternatives considered**: moving the screen reader's focus to the newest turn
(FR-009 forbids it: the listener would lose the command field every turn); marking the
transcript as a region that speaks its changes (it would speak the player's own
command, and the whole of a turn's text including the prompt, against FR-003 and
FR-007).

## R6. Testing what cannot be heard

**Decision**: Three layers of proof.

- **The layer**: tests of `announcement` on built stories and on Zork I (FR-015), and
  over the whole of Zork I's walkthrough that it is each turn's text without the prompt
  (SC-006).
- **The app**: with a launch argument, as `-showLaunchTime` already is, the app behaves
  as if a screen reader were running and shows what it last asked to be spoken, and
  how many times it has asked, in an element the tests read. Without the argument the
  element does not exist and, with no screen reader, nothing is posted (FR-010,
  SC-005).
- **The owner's ear**: the walk of SC-001 and SC-002 on a phone (FR-017).

**What the app's test cannot show**: that the system spoke, or what it cut off.

## R7. Limits

**Decision**: No new limit. The announcement is made from text the turn already holds
and one remembered fact; the timing tests of features 003 and 004 run unchanged and
must hold.
