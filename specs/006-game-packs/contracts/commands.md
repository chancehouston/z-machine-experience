# Contract: Programs, Functions and Files

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

## The introspection crate

```text
Introspector::new(view) -> Introspector                 unchanged
Introspector::with_pack(view, text: &str) -> Introspector
Introspector::pack(&self) -> PackState                  None | InForce | NotMatching | Refused
Pack::parse(text: &str) -> Result<Pack, PackError>      PackError { line, what }
```

`with_pack` never fails and never panics, whatever the text. `Exit` gains `door:
Option<u16>`, `state: Option<DoorState>` and `source: Source`; `ExitKind` gains `Door`.

## The experience layer and the C interface

```text
Session::new(story, seed)                               unchanged
Session::with_pack(story, seed, pack: Option<&str>)
zmx_session_new(...)                                    unchanged
zmx_session_new_with_pack(story, len, seed, has_seed, entropy, pack, error)
                                                        pack: a C string, or null
```

A null or unreadable pack is no pack. The snapshot's compass may hold the state word
`door` for a place; such a place is offered and has a command. No other part of the
snapshot changes.

## The app

The app's resources hold `story.pack` beside `story.z3`. Like the story, it is copied
there by `scripts/build-ios.sh` and is not committed; the script copies it only if it
names the story it is copied beside. At a new session the app reads it, if it is there,
and passes its text unread. A closed door is drawn as an outlined upright panel with the
direction's arrow, where the other states are discs, a dot or a dashed ring, and is read
aloud as "(direction), closed door".

## `zm`

`--pack FILE` on the commands that report introspection. The line for each stop has
`"pack":"<state>"` after `family`. A file that cannot be read is an error before the
game starts; a pack that is refused or does not match is not an error (the field says
so).

## The corpus program

| Command | Change |
| --- | --- |
| `corpus check` | Also checks every file under `packs/` (research R9), and prints each ledger entry whose reported value is `open R` or `closed R` |
| `corpus run [ID...]` | A story with a pack is run with it, then with packs set aside, then once more for each untrusted variable with that entry left out; each entry's proof is printed as `ID entry KEY VALUE: OUTCOME`; any outcome but `proven` fails. A pack that does not pass the check fails the run before anything is run |
| `corpus write [ID...]` | Writes `reports/<id>.txt` and, for a story with a pack, `reports/nopack/<id>.txt` |
| `corpus explain` | Unchanged; it runs with the pack |

Exit status as before: 0 when nothing failed.

## The corpus's files

**Manifest**: one new key, `first`, which may repeat (data-model.md). A story with
`first` is walked by its opening answers and then these commands, one to a stop.

**Ledger**: `reported` may be `open R` or `closed R`. An entry whose reported value is
`distrusted` is refused: a conditional report is never wrong.

**Report**: a run that was given a pack says so, in lines between `no-verdict` and
`detail`. A run with no pack writes none of them, so its report is byte for byte what it
was before this feature (as built; the plan had one line in every report).

```text
pack in-force entries 3
open confirmed 52 wrong-known 0 wrong-unexplained 0
closed confirmed 6 wrong-known 0 wrong-unexplained 0 other 0
distrusted 24 moved 9 stayed 15 question 0 ended 0 fault 0
```

or the one line `pack not-matching` or `pack refused`. The trials these lines count are
not counted in the lines above them. Detail lines may hold the reported values `open R`,
`closed R` and `distrusted`, and only in a report whose pack is in force. The verdict
counts still add up to the trials.

**Summary**: unchanged in form. Its figures are those of the runs with packs. A door
reported open is counted with the exits reported as leading to a room, a door reported
closed with those reported as not an exit, and a distrusted exit with the conditional.

## Tests that change because a form changes (SC-012)

To be listed exactly by the task that makes each change. Expected:

- the introspection crate's tests of the exits JSON (three more keys in every exit);
- `zm`'s tests of the line for each stop (the `pack` field);
- the corpus's format tests and kept reports (the `pack` line);
- the corpus ledger (five entries removed, FR-029).

No test of the core, and no test of a transcript, changes.
