# Contract: Command-Line Host

**Binary**: `zm` (crate `zmachine-cli`) | **Plan**: [../plan.md](../plan.md)

## Synopsis

```text
zm [--script FILE] [--seed N] [--width N] [--dir DIR] [--max-instructions N] STORY
zm --version
zm --help
```

| Option | Meaning | Default |
| --- | --- | --- |
| `STORY` | Path to a version 3 story file | required |
| `--script FILE` | Read input lines from FILE instead of the terminal; `-` means standard input | interactive |
| `--seed N` | Random seed, 0 to 2147483647 | taken from the operating system |
| `--width N` | Wrap terminal output at N columns; 0 disables wrapping | 80 |
| `--dir DIR` | Directory for transcript and save files | current directory |
| `--max-instructions N` | Stop with a fault after N instructions without the game asking for input | unlimited |

## Behavior

- **Interactive**: prints game text to standard output, reads lines from standard input.
- **Scripted**: takes each line of the script as one line of input. When the script ends
  while the game wants input, the run ends normally.
- **File names**: when the game starts a transcript, saves or restores, the next input
  line (typed, or from the script) is taken as the file name, as dfrotz does. Names are
  resolved inside `--dir`; a name containing a path separator is refused and the
  operation fails. An existing file is overwritten without asking.
- **Transcript**: once started, the transcript file receives the core's transcript
  stream, in the format of [transcript-format.md](transcript-format.md). A file name is
  asked for only the first time; if the game turns transcription off and on again,
  writing continues at the end of the same file.
- **Save and restore**: saves are Quetzal files. A missing, unreadable or refused file
  makes the game report failure and play continues.
- Diagnostics, prompts for file names, and fault reports go to standard error.

## Exit status

| Status | Meaning |
| --- | --- |
| 0 | The game quit, or the script ended |
| 1 | The story file faulted, or `--max-instructions` was exceeded; details on standard error |
| 2 | Usage error, or the story file could not be read or was refused |

## Examples

```text
zm stories/zork1/zork1.z3
zm --script stories/zork1/fidelity/walkthrough.txt --seed 4242 --dir temp stories/zork1/zork1.z3
```
