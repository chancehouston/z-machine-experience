# Contract: Command-Line Host

**Binary**: `zm` (crate `zmachine-cli`) | **Plan**: [../plan.md](../plan.md)

## Synopsis

```text
zm [--script FILE] [--seed N] [--width N] [--save-dir DIR] STORY
zm --version
zm --help
```

| Option | Meaning | Default |
| --- | --- | --- |
| `STORY` | Path to a version 3 story file | required |
| `--script FILE` | Read commands from FILE instead of the terminal; `-` means standard input | interactive |
| `--seed N` | Random seed, 0 to 4294967295 | unpredictable |
| `--width N` | Wrap output at N columns; 0 disables wrapping | 80 |
| `--save-dir DIR` | Where save files are written and read | current directory |

## Behavior

- **Interactive**: prints game text to standard output, reads lines from standard input.
- **Scripted**: takes each line of the script as one input line. Standard output is the
  transcript, in the format of [transcript-format.md](transcript-format.md). When the
  script ends while the game wants input, the run ends normally.
- **Save and restore**: the game's save writes `<story-name>.qzl` in the save directory;
  restore reads it. A missing or unreadable file makes the game report failure.
- Diagnostics go to standard error and never to standard output.

## Exit status

| Status | Meaning |
| --- | --- |
| 0 | The game quit, or the script ended |
| 1 | The story file faulted; the fault and address are on standard error |
| 2 | Usage error, or the story file could not be read or was refused |

## Examples

```text
zm stories/zork1/zork1.z3
zm --script tests/fidelity/zork1.walkthrough --seed 4242 stories/zork1/zork1.z3 > out.txt
```
