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
zmx_session_new_with_pack(story, len, entropy, pack)    pack: a C string, or null
```

A null or unreadable pack is no pack. The snapshot's compass may hold the state word
`door` for a place; such a place is offered and has a command. No other part of the
snapshot changes.

## The app

The app's resources hold `story.pack` beside `story.z3`. At a new session the app reads
it, if it is there, and passes its text. A closed door is drawn in a way that differs
from the other four states without relying on colour, and is read aloud as "(direction),
closed door".

## `zm`

`--pack FILE` on the commands that report introspection. The line for each stop has
`"pack":"<state>"` after `family`. A file that cannot be read is an error before the
game starts; a pack that is refused or does not match is not an error (the field says
so).

## The corpus program

| Command | Change |
| --- | --- |
| `corpus check` | Also checks every file under `packs/` (research R9), and prints each ledger entry whose reported value is `open R` or `closed R` |
| `corpus run [ID...]` | A story with a pack is run with it, then with packs set aside; each entry's proof is printed as `ID entry KEY VALUE: OUTCOME`; any outcome but `proven` fails |
| `corpus write [ID...]` | Writes `reports/<id>.txt` and, for a story with a pack, `reports/nopack/<id>.txt` |
| `corpus explain` | Unchanged; it runs with the pack |

Exit status as before: 0 when nothing failed.

## The corpus's files

**Manifest**: one new key, `first`, which may repeat (data-model.md).

**Ledger**: `reported` may be `open R` or `closed R`. An entry whose reported value is
`distrusted` is refused: a conditional report is never wrong.

**Report**: one new line after the heading, in every report:

```text
pack none
pack in-force entries 3 marked 61 confirmed 60 wrong 1 none-other 0 conditional 0
pack not-matching
pack refused
```

Detail lines may hold the reported values `open R`, `closed R` and `distrusted`. The
verdict counts still add up to the trials.

**Summary**: unchanged in form. Its figures are those of the runs with packs.

## Tests that change because a form changes (SC-012)

To be listed exactly by the task that makes each change. Expected:

- the introspection crate's tests of the exits JSON (three more keys in every exit);
- `zm`'s tests of the line for each stop (the `pack` field);
- the corpus's format tests and kept reports (the `pack` line);
- the corpus ledger (five entries removed, FR-029).

No test of the core, and no test of a transcript, changes.
