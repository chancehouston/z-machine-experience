# Fidelity artefacts for Advent

The reference for versions 5 and 8: a full game of Advent whose transcript must match,
byte for byte, the one Frotz 2.55 writes, for the version 5 build and for the version 8
build. These files contain or derive from the game's text and are covered by
`../SOURCE.md`, including its license terms.

| File | What it is |
| --- | --- |
| `walkthrough.txt` | The command script: 173 lines, from the opening to winning the game |
| `seed.txt` | The random seed the walkthrough is tuned to: 1 |
| `reference_z5.scr`, `reference_z8.scr` | The transcripts dfrotz 2.55 writes for each build. They are identical |
| `undo.txt` | Thirty moves, then thirty undos |
| `undo_z5.scr`, `undo_z8.scr` | The transcripts dfrotz writes: 25 undos succeed and 5 are refused |
| `saves/make_n.txt` | Plays the walkthrough to save point `n` (1 to 3) and saves as `save_n.qzl` |
| `saves/frotz_z5_n.qzl`, `frotz_z8_n.qzl` | The save dfrotz wrote at that point, for each build |
| `saves/cont_n.txt` | In a fresh run: restores `save_n.qzl`, then plays the next twenty commands |
| `saves/cont_z5_n.scr`, `cont_z8_n.scr` | The transcript dfrotz wrote for that continuation |

## The walkthrough

It was written for this project by reading the game's source (`../advent.inf`) and
playing against dfrotz a section at a time; no published command list was copied. It wins
with 350 points out of 350 in 168 turns and answers `quit` to the game's final question.

- Its first two lines are `script` and the transcript's file name, and it starts no
  second transcript: Frotz would ask for another file name, and this interpreter would not
  (`specs/002-v5-v8/contracts/transcript-format.md`).
- It includes `version`, whose reply prints the interpreter's number and letter. The
  reference shows `(1F)`, so the comparison is run with `--interpreter-version F`.
- It uses no `save`, `restore`, `undo`, `help` or `info`, has no backslash, and no line
  longer than 80 characters.
- It leans on this port's `go to <room already visited>` command, which keeps the lamp
  lit for about 120 of its 330 turns. At seed 1 a dwarf appears three times without doing
  harm and the pirate never appears.
- Sixteen `wait` commands near the end run out the timer between "cave closing soon" and
  the move to the repository. The count is exact for this script.

Run under `dfrotz -Z 2`, which reports every error a story file commits, both transcripts
are unchanged: the route triggers no warnings.

## The save points

After commands 44, 85 and 128 of the walkthrough. As with Zork I, a save does not carry
the random number state, so a restored game does not replay the rest of the walkthrough;
what must match is the reference interpreter restoring the same save in a fresh run at
the same seed.

## Regenerating

```text
scripts/regenerate-references.sh
```

needs dfrotz 2.55 and rewrites every `.scr` and `.qzl` file here. They must come out
unchanged.
