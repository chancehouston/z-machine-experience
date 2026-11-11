# Contract: Transcript Format

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md) R1, R6

A transcript is the file the host writes from the core's transcript stream (output
stream 2) once the game turns transcription on. For the same story file, command script
and seed it must equal, byte for byte, the transcript file written by:

```text
dfrotz -m -q -w 80 -S 0 -s <seed> <story> < <script>
```

The script starts with the game's `script` command, then a line naming the transcript
file. `zm --script` consumes that line the same way, so one script drives both.

## Rules (observed with dfrotz 2.55)

1. The transcript begins with the first text printed after transcription is turned on.
2. Game text is written unwrapped: one paragraph, one line. Lines end with a single line
   feed.
3. The status line is not part of the transcript.
4. Each line of player input is written immediately after whatever the game printed
   before asking (normally `>`), followed by a line feed.
5. Lines consumed as file names (for the transcript, a save or a restore) are not
   written, and neither are the interpreter's prompts for them.
6. If input ends while the game is waiting, the transcript ends with the game's prompt
   and no line feed.
7. If the game quits, the transcript ends with whatever the game last printed.

## To confirm at the first fidelity task

- Whether input is recorded exactly as typed or lowercased.
- What is written for an input line longer than the game's buffer.
- Whether anything is written when the game turns transcription off.

Each answer is added above with the dfrotz command that shows it.

## What is not compared

The host's terminal output. `zm` wraps it for reading and prints its own prompts for file
names; dfrotz's terminal output is a screen diff with the status row. Neither is a
transcript.
