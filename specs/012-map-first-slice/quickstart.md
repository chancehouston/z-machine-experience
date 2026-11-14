# Quickstart: Checking the Map

The clauses are in `contracts/map.md`; the stages in `research.md`, R11.

## Stage 1: the layer

```bash
cargo test --workspace
```

Expect, among the rest: every row of the witnessed table; placement on built stories;
the recorded pages of Zork I and Adventure unchanged; SC-003 and SC-004 asserted on
both; a game brought back at ten points showing the same map (SC-008); every corpus
story walked with no two places in one position (FR-044).

```bash
cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture
```

Expect printed, and within their limits: a command with its snapshot and map over
both walkthroughs; bringing back 2,000 turns with the map; the size of the largest
`map` (FR-039).

To see a recorded page change after a change to the rules:

```bash
git diff stories/zork1/map stories/advent/map
```

## Stage 2: the band

```bash
scripts/build-ios.sh
```

```bash
scripts/test-ios.sh 27
```

```bash
scripts/test-ios.sh 26
```

Expect the earlier tests to pass with their listed changes, and the new ones: walking
to the Kitchen by touching ways; the keyboard up across five commands; the strip and
the darkness note with the keyboard up; each way's spoken label; no compass.

By hand, on the owner's phone: play twenty rooms of Zork I. Is the band tall enough
to be worth looking at? Does the keyboard stay put? Record what is said.

## Stage 3: the sheet and the hand

The same three commands. Expect the new tests: opening the map out and back; the
sheet dragged to its far corner; Reduce Motion; the largest text size.

By hand, on the owner's phone (SC-009): twenty rooms again, opening the map out. Does
the page read as the places walked? What would have been drawn differently by hand?
Record what is said.
