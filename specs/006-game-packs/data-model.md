# Data Model: Game Packs

**Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md) | Formats:
[contracts/pack-format.md](contracts/pack-format.md)

## Pack

A file of facts about one story file. Parsed by the introspection crate from text its
host hands it.

| Field | Type | Rules |
| --- | --- | --- |
| `pack` | number | The format's version. Must be `1`; any other is refused (FR-002) |
| `sha256` | 64 lowercase hexadecimal digits | The story file's digest |
| `release` | 0 to 65535 | The story's release number (header word at 2) |
| `serial` | six characters | The story's serial (header bytes 0x12 to 0x17) |
| `open-attribute` | 0 to 31, at most once | The attribute set on a door while it is open |
| `untrusted-variable` | 16 to 255, any number of times, none twice | A variable whose stored value is not to be trusted |

The first four are required, once each, and come before any entry. A pack may have no
entries. A key not listed is a fault. Any fault refuses the whole pack (FR-005).

`open-attribute` is 0 to 31 because both entries are for the ZIL family, whose stories
here are version 3. A wider range is a change to the format.

## Identity

The three facts by which a pack and a story are matched: digest, release, serial. The
interface computes the story's from the file as shipped (research R2). All three must
be equal (FR-001).

## Pack state

What became of the pack an interface was given. One word (research R3):

| State | Changes answers |
| --- | --- |
| `none` | no |
| `in-force` | yes |
| `not-matching` | no |
| `refused` | no |

A pack is also `refused` when it holds an entry and the story's family is not `zil`.

The state is fixed when the interface is made and does not change during play: restore,
restart and undo leave it as it is.

## Exit (changed)

An exit as the introspection interface reports it. New fields in **bold**.

| Field | Value |
| --- | --- |
| `direction`, `words` | As before |
| `kind` | `fixed`, `conditional`, or **`door`** |
| `destination`, `destination_name` | For `fixed` and **`door`**: the room and its printed name. Otherwise none |
| **`door`** | For `door`: the door's object number. Otherwise none |
| **`state`** | For `door`: `open` or `closed`. Otherwise none |
| **`source`** | `pack` when the entry would be different with no pack; otherwise `file` |

Rules:

- `kind` is `door` only with an `open-attribute` entry in force.
- `source` is `pack` on every `door` exit, and on a `conditional` exit that is
  conditional only because its variable is untrusted. On everything else it is `file`.
- With pack state other than `in-force`, every exit has `door` none, `state` none and
  `source` `file`, and its other fields are what version 0.1.0 gave (FR-013).

## Compass place (changed)

| State | Meaning | Offered |
| --- | --- | --- |
| `open` | Leads somewhere for certain; now also an open door | yes |
| `maybe` | The game decides | yes |
| **`door`** | A closed door | yes |
| `closed` | No exit | no |
| `unknown`, `idle` | As before | no |

## Reported (corpus, changed)

| Value | From |
| --- | --- |
| `fixed R`, `conditional`, `unknown`, `none` | As in feature 005, for an exit whose `source` is `file` |
| **`open R`** | A `door` exit, `open`, to R |
| **`closed R`** | A `door` exit, `closed`, to R |
| **`distrusted`** | A `conditional` exit whose `source` is `pack` |

The three new values are exactly the marked answers. Their verdicts are in research
R10.

## Proof

The outcome of testing one entry of one pack on one run (research R11).

| Outcome | Passes |
| --- | --- |
| `proven` | yes |
| `contradicted` | no |
| `not-proven` | no |
| `unneeded` | no |

## Story (corpus manifest, changed)

One new key:

| Key | Rules |
| --- | --- |
| `first` | May repeat, in order. A command given at a command prompt before the crawl starts. Each prompt it is given at is a place. Not allowed with `walkthrough` |

## What is kept in the repository

| File | Kept | Checked |
| --- | --- | --- |
| `packs/<id>.pack` | Yes, for a committed or a fetched story | By `corpus check` always; proven by `corpus run` where the story is present |
| `stories/corpus/reports/<id>.txt` | Yes | The run with packs |
| `stories/corpus/reports/nopack/<id>.txt` | Yes, for a story that has a pack | The run with packs set aside |
| The app's `story.pack` | A copy of the pack of the story the app carries | A test compares the two files |
