# Contract: Transcript Format

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md) R1, R6

A transcript is the standard output of `zm --script` and must equal, byte for byte, the
standard output of the reference command for the same story file, script, seed and width:

```text
dfrotz -m -p -q -w <width> -s <seed> <story> < <script>
```

## Settled

- Encoding is plain ASCII; characters outside ASCII are transliterated as the reference
  does with `-p`.
- Lines end with a single line feed.
- Text is wrapped at word boundaries to the given width.
- No MORE prompts and no startup banner appear.

## To be established by observation

dfrotz is not yet installed here and its manual does not specify the following. The first
fidelity task runs dfrotz on a small story file and on the opening of Zork 1, records the
answers in this section, and only then is the formatter written.

1. Whether a command read from piped input is echoed after the prompt.
2. Whether and how the version 3 status line appears in the output.
3. Exactly where a line breaks when a word ends at the width, and how trailing spaces are
   treated.
4. What is written when input ends while the game is waiting.
5. What is written when the game quits.
6. How a fault is reported, and whether any of it goes to standard output.

Each answer is recorded with the dfrotz command that demonstrates it.

## If the reference cannot be reproduced

If dfrotz writes something that cannot be derived from the core's events (for example,
text that depends on terminal state), the difference is not papered over in the
formatter. It is raised with the project owner and, if accepted, entered in the
divergence ledger.
