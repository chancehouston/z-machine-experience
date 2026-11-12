# Advent: source and license record

"ADVENTURE", the Inform 6 port of Colossal Cave: by Will Crowther (1976) and Don Woods
(1977), reconstructed by Donald Ekman, David M. Baggett (1993) and Graham Nelson (1994),
with later changes by Roger Firth (2004, 2006). Release 9, serial 060321.

**This directory is not under the repository's MIT license.** It is here as a test
fixture only, and is not cleared for bundling in any shipped application.

| | |
| --- | --- |
| Source | `std/demos/advent.inf` in the Inform 6 distribution, <https://ifarchive.org/if-archive/infocom/compilers/inform6/source/inform-6.44-r7.tar.gz> |
| Revision | Inform 6.44 release 7; tarball SHA-256 `3e1f94e5c726f5f59cf086756dc14e66a3fd281ddbac75ead611d45aa57e4bdb` |
| License text | `COPYING-inform.txt` beside this file: the distribution's `COPYING`, unmodified |

## Files

| File | Origin | SHA-256 |
| --- | --- | --- |
| `advent.inf` | From the distribution, unmodified | `5b078ced72d2036f54f40eb8da7a02a9b1e1a1315243539ca04fc5ffb79babd0` |
| `advent.z5` | Compiled by this project, 152,064 bytes | `9df430b9454ebee51de43d10af6c5616ad56b8f4f59389130c132449d99e9b05` |
| `advent.z8` | Compiled by this project, 156,160 bytes | `9dc8e334af1e4487707a2bd351d8121417d5864ac04ae9e883503527917d6c4e` |
| `COPYING-inform.txt` | From the distribution, unmodified | `df031f0aaa235597a75d308bd406757f4ca347f6c98a4e6a7a039e3608709bb9` |

## How the story files were built

Inform 6.44 for MacOS (11th September 2025) with the standard library 6.12.8 from the same
distribution, installed with `brew install inform6`:

```text
inform +include_path=<distribution>/std/lib -v5 advent.inf advent.z5
inform +include_path=<distribution>/std/lib -v8 advent.inf advent.z8
```

Compiling again gives identical files.

## License, and how far it reaches

`advent.inf` states no license. The distribution's `COPYING` says the status of its demos
"is what it says in each file", and quotes the Inform Designer's Manual, under which the
"example games" may be "freely distributed to anybody who wants a copy, provided that: (a)
distributed copies are not substantially different from those archived by the author, (b)
this and other copyright messages are always retained in full, and (c) no profit is
involved." The Inform library compiled into the story files is under that license or the
Artistic License 2.0.

Relying on that clause for this file is this project's reading, not a statement by the
rights holders. Three things weaken it: the file itself is silent; the grant is Graham
Nelson's, while this release also contains later changes by another author and is a port
of someone else's game; and the clause speaks of distributing copies like the author's,
while this directory holds new builds and, under `fidelity/`, complete transcripts.

The project owner accepted Advent as a test-only reference game on these terms on
2026-11-11. If the reading proves wrong, this directory is to be removed from the
repository's history and the fidelity tests for versions 5 and 8 pointed at another game.

## Files derived from this game

Everything under `fidelity/` contains or is derived from the game's text and is covered
by this record.
