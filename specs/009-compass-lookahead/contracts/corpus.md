# Contract: The Corpus's Proof of Look-Ahead

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

Changes to `specs/005-corpus-testing/contracts/formats.md`. Each numbered statement
becomes a test that cites it.

## The measure

1. At every place, for each direction whose report is conditional, distrusted or
   unknown, the corpus asks the experience layer for that way's finding, with the same
   limits the session uses.
2. Apart from that, it runs its own trial of the direction on a copy given a value for
   chance that is none of the three look-ahead used.
3. A finding of *walked to R* is borne out by a trial that moved to R; *refused* by a
   trial that stayed. Any other pairing is not borne out. A way with no finding is
   counted as left, and judged by nothing.
4. The corpus's existing trial of every direction, and every count made from it, is as
   before.

## A report

One line is added after `unknown`:

```text
lookahead walked 4 refused 6 left 0 not-borne-out-known 0 not-borne-out-unexplained 0
```

5. `walked`, `refused` and `left` sum to the number of ways asked. The two
   `not-borne-out` counts are among `walked` and `refused`.
6. Every other line of every report is byte for byte what it was. The summary gains the
   same five numbers at the end of each of its lines.
7. A report with a line it does not know is refused, as today; a report without the
   new line is read as an earlier report and fails the comparison, so every kept report
   is rewritten once.

## The ledger

An entry for a finding not borne out:

```text
story: advent-zil
room: 204
direction: 28
lookahead: refused
outcome: moved 203
cause: The game decides by a draw; three copies stayed and the fourth did not.
```

8. `lookahead` is `walked N` or `refused`, and stands where `reported` stands in other
   entries; an entry has one or the other. `lookahead` and `outcome` must together be a
   pairing that is not borne out.
9. A run with a finding not borne out and no such entry exits with failure and names
   the story, room and direction. An entry that no run needs is reported, as today.

## The program

10. `corpus run`, `write` and `explain` take no new argument. `explain` for a
    direction that has a finding prints the finding and the trial's outcome, and no
    text of the game.
11. What is kept holds numbers and the fixed words above, and nothing of any game.
