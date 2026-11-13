# Contract: Look-Ahead in the Core, the Layer and the Published Interface

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md) |
**Data**: [../data-model.md](../data-model.md)

Each numbered statement becomes a test that cites it.

## The core

1. A machine has one new method by which its host gives it a new value for chance.
   After it, the generator draws as a machine loaded with that value as its seed would,
   from the start of that sequence.
2. If the game has put the generator in its predictable mode, the method leaves it
   there, and the game's draws are what they would have been.
3. A machine on which the method is never called behaves, byte for byte, as before this
   feature. Every fidelity test passes unchanged.

## The layer: one way

4. A look-ahead is made on a clone of the player's machine at a command prompt. The
   clone is given exactly the command the way would send.
5. Three copies are made for a way. Copy 0 keeps the game's chance. Copies 1 and 2 are
   each given a value that depends only on the count of commands the session has sent
   and the copy's number, and the two values differ.
6. A copy's outcome is *walked to R*, *stayed* or *nothing*, as the data model's table
   says. A copy that stops for a save, a restore or a transcript is given none of them.
7. A way's finding is *walked* when all three copies walked to one room, *refused* when
   all three stayed, and *none* otherwise.
8. After any number of look-aheads the player's machine is equal to what it was: its
   memory, its stack, its place in the program, its output not yet taken, its status
   line, and its next draws of chance.
9. Nothing a copy printed is in the session's turns, transcript or status.

## The layer: the compass

10. Look-ahead is made only at a stop where the game waits for a command, and only for
    a place or an extra whose state would be `maybe` or `unknown` and that has a
    command. A place that is `unknown` because two exits claim it is not tried.
11. A way found *walked* is reported `open`; its `leads_to` is the name copy 0's
    `location` gives, if it gives one; its `dark_ahead` is true exactly when that
    answer's `lit` is `no`.
12. A place found *refused* is reported `closed`: it has no command and its spoken
    words are those of any closed place. An extra found *refused* is not in the list.
13. A way with no finding is reported as it would have been without this feature.
14. The note that some ways are unknown is present only if, after look-ahead, a
    direction is still unanswered.
15. The words shown and spoken for each state are those the layer gave that state
    before this feature. The snapshot's JSON has the same keys in the same order. The C
    interface is unchanged.
16. Asking for the compass again at the same prompt gives an equal compass.
17. A story the interface does not recognise gets no look-ahead, and its compass is
    what it was.

## The layer: limits

18. One copy runs at most a fixed count of instructions; past it, its outcome is
    *nothing*.
19. All copies at one prompt run at most a fixed total; a way not begun when the total
    is spent has no finding. Ways are begun in the compass's own order, then the
    extras in theirs.
20. With look-ahead on, a command and its snapshot take less than 10 ms at every prompt
    of every walkthrough the project holds, by `time_limit`.

## What `docs/introspection.md` is to say (version 0.5.0)

A new section after 10.5, and a line in the list of changes:

21. An answer of `conditional` or `unknown` for an exit says what the room's table
    says and no more. A host may settle it by a trial: give a copy of the machine the
    direction's word, and see whether the copy's `location` changes.
22. Such a trial must leave the machine being played unchanged in every way the game
    or this interface can observe, including its next random numbers; and a host that
    reports a trial's result must not report it as an answer of this interface.
23. No query, field or rule of the interface changes.

## The app

24. No source file of the app changes. Its tests gain: Zork I's living room shows no
    way down before the trap door is opened and an open way down, to the cellar and
    into darkness, after; with the lamp on, the same way is not marked dark.
25. Each existing test that fixed `maybe` for a way look-ahead now settles is changed
    to the settled state, and is listed in the tasks' closing notes.

## As built

- **Statement 10.** Only a way whose state is `maybe` is tried. A place the layer
  reports `unknown` has no command to send, so there is nothing to give a copy, and it
  is left. Statement 14 follows: the note of unknown ways is never removed by
  look-ahead. Where every way was known and look-ahead leaves none to take, the note
  says there are no exits.
- **Statement 5.** The count is the number of turns the session holds when the compass
  is made. The two values come from a mixing of all its bits; a first, simpler mixing
  left the two copies' draws related, and a way decided by an even draw was settled 83
  times in 200 where 50 is expected. The test of statement 7 caught it.
- **Statements 18 and 19.** One copy may run 540,000 instructions, and all copies at
  one prompt 540,000 before a further way is begun. What a copy ran is counted in steps
  of a thousand. Measured over the two walkthroughs the project holds: the longest
  single copy ran 27,000 and the busiest prompt 207,000 in all, both in Advent; in
  Zork I, 3,000 and 12,000.
- **Statement 6.** The toy has no way that asks to save, restore or keep a transcript;
  that a copy given none of them comes to nothing is in the code and has no test of
  its own.
- **Statement 21.** The new section is 10.8, after 10.7, so that no section's number
  changed.
- **The session** has two additions a host does not need: it can be given other
  limits, and it says what look-ahead ran at the last stop. Tests and the corpus use
  them. `lookahead::OFF` sets look-ahead aside.
- **The toy game** of the corpus's tests moved to the testkit, so that the layer's
  tests could use it; it gained a way decided by an even draw and one by a rare draw.
