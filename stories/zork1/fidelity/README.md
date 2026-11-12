# Fidelity artefacts for Zork I

These files are the reference for the project's headline test: a full game of Zork I
whose transcript must match, byte for byte, the one Frotz 2.55 writes. They contain or
derive from the game's text and are covered by `../SOURCE.md`.

| File | What it is |
| --- | --- |
| `walkthrough.txt` | The command script: 333 lines, from the opening to winning the game |
| `seed.txt` | The random seed the walkthrough is tuned to: 11 |
| `reference.scr` | The transcript dfrotz 2.55 writes for that script and seed |

| `saves/make_n.txt` | Plays the walkthrough to save point `n` (1 to 5) and saves as `save_n.qzl` |
| `saves/frotz_n.qzl` | The save file dfrotz wrote at that point |
| `saves/cont_n.txt` | In a fresh run: restores `save_n.qzl`, then plays the next twenty commands |
| `saves/cont_n.scr` | The transcript dfrotz wrote for that continuation |

## The walkthrough

It was written for this project by playing against dfrotz a section at a time; no
published command list was copied. It wins with 350 points in 341 moves and ends by
entering the stone barrow and answering `quit` to the game's final question.

The first two lines are the game's own `script` command and the transcript's file name,
so the same file drives dfrotz and `zm`. No line begins with a backslash, which dfrotz
would take as a command of its own, and none is longer than 28 characters.

The script depends on the seed. Combat and the thief's wandering are random, so at
another seed the troll may not fall to the first blow and the thief may be elsewhere. Two
`wait` commands are there only for timing: one in the Timber Room and one in the maze each
let the thief wander off before the player walks into him.

## Regenerating

```text
scripts/regenerate-references.sh
```

needs dfrotz 2.55 and rewrites `reference.scr` with:

```text
dfrotz -m -q -w 80 -S 0 -s 11 stories/zork1/zork1.z3 < stories/zork1/fidelity/walkthrough.txt
```

The file must come out unchanged. Run under `dfrotz -Z 2`, which reports every error a
story file commits, the transcript is also unchanged: the route triggers no warnings.

## The save points

The five points are after commands 30, 90, 160, 235 and 300 of the walkthrough. A save
does not carry the random number state, in the standard format or in Frotz, so a restored
game does not replay the rest of the walkthrough: the thief is somewhere else and blows
land differently. What must match is the reference interpreter doing the same thing. Each
continuation is run fresh at the pinned seed, so both interpreters restore into the same
random state, and the tests require:

- `zm` restoring `frotz_n.qzl` writes `cont_n.scr`;
- `zm` restoring a save it wrote itself with `make_n.txt` writes `cont_n.scr`;
- dfrotz restoring a save written by `zm` writes `cont_n.scr`.

The regeneration script rebuilds the saves and continuations too. dfrotz reproduces its
own save files byte for byte.
