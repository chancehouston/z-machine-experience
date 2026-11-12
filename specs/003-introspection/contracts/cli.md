# Contract: Command-Line Host, additions for introspection

**Binary**: `zm` | **Plan**: [../plan.md](../plan.md)

Additions to the contracts of features 001 and 002. Everything there still holds.

## New options

| Option | Meaning |
| --- | --- |
| `--inspect FILE` | Each time the game waits for a line or a key, append one line of JSON to FILE: the family and the answers to location, exits, inventory and scope. `-` writes to standard error |
| `--tree` | Print the object tree as JSON to standard output and exit without playing |
| `--dictionary` | Print the dictionary as JSON to standard output and exit without playing |

## Behavior

- **`--inspect` output never mixes with the game's text.** It goes to its own file, or
  to standard error, never to standard output and never into a transcript.
- **One line per stop.** Each line is a complete JSON object:

  ```text
  {"spec":"0.1.0","stop":12,"family":"zil","location":{...},"exits":{...},"inventory":{...},"scope":{...}}
  ```

  `stop` counts requests for input from 1. Each answer has the form the specification
  defines, including `unknown` with its reason and `exactness`.
- **The file is resolved like the story file**, not inside `--dir`: it is the
  developer's own output, not something the game named.
- **`--tree` and `--dictionary`** load the story file, print, and exit with status 0. A
  file that cannot be loaded is refused as usual with status 2. They may be given
  together; the output is then one JSON object with both.
- **Playing is unaffected.** With or without `--inspect`, standard output, the
  transcript and any save file are byte for byte the same.

## Exit status

Unchanged.

## Examples

```text
zm --inspect - stories/zork1/zork1.z3
zm --inspect temp/advent.jsonl --script stories/advent/fidelity/walkthrough.txt --seed 1 --dir temp stories/advent/advent.z5
zm --tree stories/advent/advent.z8
```
