# Contract: Transcript Format, additions for versions 5 and 8

**Plan**: [../plan.md](../plan.md)

The twelve rules of `specs/001-v3-core/contracts/transcript-format.md` hold for versions 5
and 8, with the additions below. Each was observed with dfrotz 2.55 running Advent or a
small test program, by this project or by its independent reviewer; the probe programs
become committed fixtures as the version 3 ones did.

The reference command is unchanged:

```text
dfrotz -m -q -w 80 -S 0 -s <seed> <story> < <script>
```

## Rules

13. **A single keypress is not written.** Neither the key nor a line feed.
14. **Text printed while output is redirected into memory is not written** (§7.1.2.2). A
    word in progress when the redirection begins is completed first.
15. **Upper-window text is not written**, and it does not interrupt a lower-window word:
    `pend`, then text in the upper window, then `ing done` is recorded as `pending done`.
16. **Text printed with the screen stream turned off is written.** The transcript does not
    depend on stream 1. Inform's quotation boxes rely on this: the box goes to the upper
    window and a bracketed copy is printed with stream 1 off, for the transcript alone.
17. **Clearing a window writes nothing.**
18. **Pre-loaded input is written with what was typed.** If the game printed `pre>abc`,
    placed `abc` in the buffer and the player typed `DEF`, the transcript has `pre>abcDEF`.
    The reference interpreter removes the `abc` the game printed and writes it again with
    the input; the result is the same text.
19. **With buffering off, text is written a character at a time.** Rule 12, a word at a
    time, applies while `buffer_mode` is 1, which is its starting value. After
    `buffer_mode 0` each character is recorded, or not, according to whether transcription
    is on at the moment it is printed.
20. **Characters outside ASCII are written as UTF-8.**
21. **A timed request for input is recorded as an untimed one.**

## Differences from the reference interpreter

These are not in the divergence ledger, because none can occur in a transcript used for
comparison. Each is enforced by a check on the comparison scripts.

- **Starting a transcript twice.** Frotz asks for a file name each time a version 5 game
  starts one; this interpreter asks once, as the Standard says (§7.1.1.2). After the
  first, Frotz takes the next script line as a name and this interpreter takes it as a
  command. Comparison scripts start a transcript once.
- **ZSCII 0 with buffering off.** Frotz writes a zero byte to the transcript; this
  interpreter writes nothing, as the Standard says (§3.8.2.1).
- **Undo beyond the reference's depth** cannot differ: both keep 25 positions.
- **Games that test the header's style or timed-input bits** may take a different course
  ([../data-model.md](../data-model.md), "What the interpreter writes into the header").
