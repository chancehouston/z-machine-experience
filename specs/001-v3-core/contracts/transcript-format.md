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
8. Input is written exactly as given: case and surrounding spaces are kept
   (`>OPEN Mailbox`, `>  look  `), even though the game receives it in lower case.
9. Text and input are written only while transcription is on. When the game turns it
   off, whatever it printed before clearing the bit is included (Zork I's "Here ends a
   transcript of interaction with" and its version lines) and nothing after. The prompt
   and command that follow are absent.
10. A file name is asked for only the first time. When the game turns transcription on
    again, writing continues at the end of the same file with no prompt, so the next
    input line goes to the game.

Rules 8 to 10 were observed with:

```text
printf 'script\nt1\nOPEN Mailbox\n  look  \nunscript\ninventory\nscript\nlook\n' \
  | dfrotz -m -q -w 80 -S 0 -s 4242 stories/zork1/zork1.z3
```

## A limit of the reference, not of the format

dfrotz keeps at most 198 characters of an input line and writes those to the transcript,
whatever the size of the game's own input buffer (observed with test stories whose
buffers hold 19, 99 and 249 characters). This interpreter writes the whole line as given.
Scripts used for comparison therefore keep every line under 198 characters.

## What is not compared

The host's terminal output. `zm` wraps it for reading and prints its own prompts for file
names; dfrotz's terminal output is a screen diff with the status row. Neither is a
transcript.
