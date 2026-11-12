# Contract: Command-Line Host, changes for versions 5 and 8

**Binary**: `zm` | **Plan**: [../plan.md](../plan.md)

Additions to `specs/001-v3-core/contracts/cli.md`. Everything there still holds.

## New options

| Option | Meaning | Default |
| --- | --- | --- |
| `--upper` | Show what the game drew outside the main text: whenever the game waits for input and the screen has changed, print the non-empty rows of the canvas above the prompt | off |
| `--style MODE` | `ansi` renders bold, italic and reverse with terminal escape codes; `plain` prints text only; `auto` picks `ansi` when standard output is a terminal | `auto` |
| `--interpreter-version C` | The single letter games may print as the interpreter's version | `Z` |

`--width` now also sets the screen width a version 5 or 8 game sees. A width of 0 still
disables wrapping, and the game is then told 80.

## Behavior

- **Story files** of versions 3, 5 and 8 are played. Any other version is refused with
  exit status 2 and a message naming those three.
- **Keypresses** follow the reference interpreter's rules, so one script drives both.
  When the game wants a single key, `zm` takes the next unused character of the current
  input line, reading another line when that one is used up. An empty line is the return
  key. Characters left unused are dropped when the game next asks for a whole line, and a
  key request drops anything left over from a line. Interactively this means a key is
  typed and then return is pressed; `zm` does not put the terminal into raw mode.
- **Special keys** are written with dfrotz's escapes inside an input line: `\_` return,
  `\[` escape, `\?` delete, `\^` `\.` `\<` `\>` the up, down, left and right arrows, `\1`
  to `\9` and `\0` function keys 1 to 10, and `\\` a backslash.
  An escape dfrotz does not list is reported on standard error and dropped, as dfrotz
  does. **Only `\_`, `\[`, `\?` and `\\` mean the same to both interpreters.** dfrotz 2.55
  documents the arrow and function-key escapes, but its UTF-8 build reads their codes
  back as malformed UTF-8 and gives the game a question mark (one for a whole run of
  them); `zm` gives the game the keys §3.8.4 defines. A script meant for both must not
  use them (recorded by probe, `crates/zmachine-core/tests/fixtures/keys/`).
- **In a line of input**, as opposed to a key, `\\` is one backslash and any other
  special key is reported and dropped: a line ends with return.
- **Lines beginning with a backslash and a lower-case letter** are commands to dfrotz
  itself, not input. `zm` refuses such a line with a message, so that a script cannot mean
  different things to the two.
- **A line ended by a function key** is not something `zm` can produce; lines end with
  return. The core supports it for hosts that can.
- **The screen** is printed only with `--upper`, between two rules, to standard output.
  It never appears in a transcript.
- **Transcripts, saves and restores** work as for version 3, with the same file-name
  rules. A transcript's file name is asked for once in a session, as the Standard says
  (§7.1.1.2). dfrotz asks every time a version 5 game starts one, so a script meant for
  both must start a transcript only once.
- **Styles** never appear in a transcript and are not written when standard output is
  not a terminal unless `--style ansi` is given.

## Exit status

Unchanged: 0 the game quit or input ended; 1 the story file faulted or exceeded
`--max-instructions`; 2 a usage error or a story file that was refused.

## Examples

```text
zm stories/advent/advent.z5
zm --upper --width 60 stories/advent/advent.z8
zm --script stories/advent/fidelity/walkthrough.txt --seed 11 --interpreter-version F --dir temp stories/advent/advent.z5
```
