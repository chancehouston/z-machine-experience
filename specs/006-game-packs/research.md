# Research: Game Packs

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Each finding is a decision, why, and what else was weighed. Figures marked "measured"
come from a temporary test run on 2026-11-12 and since removed; the tasks repeat each as
a kept test.

## R1. The pack file's form

**Decision**: Plain text in UTF-8, one `key: value` to a line, `#` for a comment line,
blank lines ignored. Four lines of identity and format first, then entries:

```text
# A game pack (docs/introspection.md, section 12).
pack: 1
sha256: 37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79
release: 119
serial: 880429
open-attribute: 11
untrusted-variable: 99
untrusted-variable: 136
```

Every value is a number or hexadecimal digits, but for the serial, which is the six
characters the file holds (digits in every story the corpus has). The grammar is in
[contracts/pack-format.md](contracts/pack-format.md).

**Rationale**: It is the form the corpus manifest already uses, so the project has one
way of writing small data files and one style of refusal ("line N: ..."). A grammar of
keys and numbers makes "no text of a game" a property of the format: there is nowhere
to put a word (FR-003, SC-008). A person can write one by hand (FR-004, SC-009).

**Alternatives considered**: JSON (needs a parser in a crate with no dependencies, and
admits strings); TOML (a new dependency); a table of all packs in one file (one bad
line would refuse every pack, and FR-005 refuses a pack as a whole).

## R2. Who works out the story's identity

**Decision**: The introspection crate does: it computes the SHA-256 of the story file
as shipped, and reads the release (header word at 2) and serial (six bytes at 0x12)
from the same bytes. `sha2` 0.10 becomes a dependency of `zmachine-introspect`; today it
is a development dependency of that crate.

**Rationale**: The identity rule is the pack's first protection (constitution, "applies
only when all three match"). If a host supplied the digest, a host's mistake could put
a pack in force for the wrong file, and no test of the introspection crate could show
otherwise. Hashing is arithmetic, not I/O, so Principle V is kept.

**Alternatives considered**: the host supplies the digest (above); a SHA-256 written in
the crate (a hundred lines of security-shaped code with no reason to exist); the
header checksum in place of a digest (16 bits; the constitution says SHA-256).

**Cost to be measured**: one hash of the story at the start of a session. Zork I is
86,838 bytes. The task that adds it measures the app's launch time before and after.

## R3. How a pack reaches the interface

**Decision**: A second constructor, `Introspector::with_pack(view, text)`, beside
`Introspector::new(view)`, which is unchanged. It never fails. What became of the pack
is asked with `Introspector::pack()`, which answers one of four words:

| Word | When |
| --- | --- |
| `none` | No pack was given |
| `in-force` | A pack was given, is well formed, and its identity matches |
| `not-matching` | Well formed, but digest, release or serial differs |
| `refused` | Malformed, of an unknown format version, or with an entry the story's family has no use for |

Only `in-force` changes any answer. A separate function, `Pack::parse(text)`, gives the
line and the fault, for the check.

**Rationale**: FR-005 (no query may fail because of a pack), FR-006 (the user can ask),
FR-034 (the interface reads no files: its host hands it text).

**Alternatives considered**: a pack set after construction (the interface would then
hold something that changes, against its own rule 1.7's spirit); a result type from the
constructor (every host would need a path for "the pack was bad", and FR-005 says the
game goes on regardless).

## R4. What changes in an exits answer

**Decision**: The interface specification goes to version 0.2.0. An exit gains three
fields, present on every exit of every family:

| Field | Value |
| --- | --- |
| `door` | The door's object number, or none |
| `state` | `open` or `closed`, or none |
| `source` | `file` or `pack` |

and `kind` gains a third word, `door`. A `door` exit has `destination`, `door` and
`state`. `source` is `pack` exactly when the entry would have been different with no
pack: on every `door` exit, and on a `conditional` exit that is conditional only because
its variable is named as untrusted. Everything else is `file`.

**Rationale**: FR-010 (a closed door stays listed and is told apart from no exit),
FR-012 (the mark). The specification's own rule (I§2.2) is that a field is never simply
left out, so the fields appear with or without a pack. By I§11 a new field and a new
word are a minor change, and a reader of 0.1 "must ignore fields and tolerate words it
does not know".

**What it costs**: every exits answer in JSON gains three keys, pack or no pack. Tests
of earlier features that fix that text change with it. FR-013 is read as "every value
that was there is unchanged", and SC-012 lists the tests changed. See the plan's
strongest objection.

**Alternatives considered**: `fixed` for an open door and "not an exit" for a closed
one (hides the door from the compass exactly when the player needs to see it, and makes
a closed door look like a wall); a mark on the whole answer (an answer with one door and
five plain exits would be wholly marked); leaving the fields out with no pack (against
I§2.2, and a reader could not tell "no door" from "an implementation of 0.1").

## R5. The rule for a door exit

**Decision**: In a ZIL story with an `open-attribute` entry in force, a 5-byte exit is
read as: first byte the room, second byte the door.

| Condition | Reported as |
| --- | --- |
| the second byte is not an object's number | `unknown`, `not-determinable` |
| the first byte is not a room's number | `unknown`, `not-determinable` |
| the door has the attribute | `door`, `open`, to the room |
| the door has it not | `door`, `closed`, to the room |

**Measured on Zork I's walkthrough**: 57 trials of door exits, at three room-and-door
pairs, two doors. With attribute 11: 52 trials had it set and all 52 arrived in exactly
the room the table names; 5 had it clear and all 5 stayed. Every one of the other 31
attributes disagrees with at least 5 of the 57 (the closest, attributes 14 and 22,
agree on 52), so a pack naming any other attribute is contradicted (SC-004).

**Measured earlier on Mini-Zork**: opening a door set attribute 10 and nothing else,
after which the exit let the player through.

## R6. The rule for an untrusted variable

**Decision**: The entry is `untrusted-variable: N`, where N is the Z-machine variable
number exactly as the exit's second byte stores it (16 to 255; 16 is the first global).
A 4-byte exit whose variable is named is `conditional`, with `source: pack`, whatever
the variable holds. The spec's word "flag" is this variable.

**Rationale**: The number is written as the story file holds it, so a pack's author
copies it and converts nothing.

**Measured on Zork I**: the five stale-flag entries of the corpus ledger are three
exits: room 129 direction 22 (variable 99), and room 249 directions 20 and 30 (both
variable 136). Eleven different variables guard 4-byte exits along the walkthrough;
these two are the ones the ledger blames. Whether either also guards an exit in another
room was not measured: if so, that exit becomes conditional too, which is the honest
answer for a value that cannot be trusted.

## R7. The compass

**Decision**: The experience layer's place gains one state, `door`, for a closed door.
An open door is the existing `open` state, with its destination, like any fixed exit.

| Exit | Place state | Offered | Spoken |
| --- | --- | --- | --- |
| `door`, `open` | `open` | yes | "East, to (the room's name)" |
| `door`, `closed` | `door` | yes | "East, closed door" |
| `conditional` | `maybe` | yes | "East, may be blocked" (unchanged) |

Touching a closed door types the direction (FR-031). The room behind a closed door is
not spoken or shown.

**Rationale**: FR-030. The existing state named `closed` already means "no exit", so
the new state takes another word; renaming the old one would change the snapshot for
nothing. Not naming the room behind a closed door is a choice for the owner (plan,
"Decisions"): the interface gives the room, but a player has not yet seen through the
door.

**How it looks in the app** is fixed by a task with a picture for the owner's eye,
as feature 004 did for the compass. It must differ from the other four states without
relying on colour alone.

## R8. How the pack reaches the app

**Decision**: `Session::with_pack(story, seed, pack)` in the experience layer beside
`Session::new`; one new C function, `zmx_session_new_with_pack`, beside
`zmx_session_new`, which is unchanged. The app carries `story.pack` beside `story.z3`
in its resources and hands over its text. If the resource is absent the app starts as
it does today.

**Rationale**: FR-032, FR-033. A new function leaves the existing C contract and its
tests alone. The app holds no game's name in code today (a rule a test enforces), and
naming the pack after the story's resource keeps that.

## R9. Where packs live, and the check

**Decision**: `packs/<id>.pack` at the repository's root, where `<id>` is the story's
id in the corpus manifest. `corpus check` reads every file there and refuses:

- a file that `Pack::parse` refuses, with file and line;
- a pack whose digest is no story's in the manifest;
- two packs with one digest;
- a pack whose file name is not its story's id;
- an entry its story's family has no use for (in this feature, any entry for a story
  not of family `zil`);
- where the story is present, a release or serial that is not the file's.

**Rationale**: FR-023. Packs are published data for any interpreter, so they sit at the
root and not under the corpus. The corpus is what proves them, so the corpus checks
them.

## R10. What the corpus reports for a marked answer

**Decision**: `Reported` gains three values, which are exactly the marked answers:

| Reported | Outcome | Verdict |
| --- | --- | --- |
| `open R` | `moved R` | `confirmed` |
| `open R` | anything else but `lost` | `wrong` |
| `closed R` | `stayed` | `confirmed` |
| `closed R` | `moved` anywhere | `wrong` |
| `closed R` | `question`, `ended`, `fault` | `none-other` |
| `distrusted` | any but `lost` | `conditional` |

A report gains one line, `pack`, giving the pack's state and the trials on marked
answers by verdict (FR-022). The ledger accepts `open R` and `closed R` as reported
values. `corpus check` prints each ledger entry on a marked answer (FR-019), so that
the owner can read its cause; whether a cause "is the entry" cannot be decided by a
program.

## R11. Proof of an entry

**Decision**: Proof is part of `corpus run` for a story with a pack. Each entry ends as
one of `proven`, `contradicted`, `not-proven`, `unneeded`; only `proven` passes.

- **`open-attribute`**: `contradicted` if any trial reported `open` or `closed` is
  `wrong` with no ledger entry; else `not-proven` unless at least one trial reported
  `open` and one reported `closed` are `confirmed`; else `proven` (FR-017).
- **`untrusted-variable: N`**: the story is run a second time with packs set aside.
  `proven` if at least one exit that hangs on N is `wrong` in that run at a room and
  direction where the run with the pack reports `distrusted`; otherwise `unneeded`
  (FR-018).

**Rationale**: The second run is also what FR-021 and SC-005 need. For a story with a
pack the repository keeps two reports: `reports/<id>.txt` with the pack and
`reports/nopack/<id>.txt` without. The second is today's report with the one new line
(`pack none`), which a review can see.

**Cost**: Zork I and Mini-Zork run twice. To be measured against SC-009 of feature 005
(ten minutes for the committed stories on CI).

**SC-004** (every other attribute is contradicted) is shown by a test that judges only
the door-exit trials along the walkthrough, 57 trials for each of 31 attributes, not 31
whole runs.

## R12. A door in both states on Mini-Zork

**Decision**: The manifest gains a key, `first`, which may repeat: commands given at
the first command prompts, each making a place where trials run, before the crawl
starts from all the places so far. Mini-Zork gets the few commands that walk to a door
and open it.

**Rationale**: The crawl only walks, and reaches Mini-Zork's doors closed (three
conditional trials, all stayed). FR-017 needs both states. A walkthrough file would be
a second, larger thing for a fetched story; a few commands in the manifest are the
project's own words. The commands are found by a task on the owner's Mac, where the
file is; the plan does not name them.

**Risk**: if no short sequence opens a door in Mini-Zork, its entry stays `not-proven`
and, by FR-017, its pack cannot be merged. The task says so and stops for the owner.

## R13. No text of a game in a pack

**Decision**: A test reads every file under `packs/` against the grammar of R1 by its
own small reader, not the crate's parser: each line is a comment, blank, or a known key
and a value of digits or hexadecimal digits only (for `serial`, six letters or digits). Comments are searched for every word
of four letters or more in each committed story's dictionary and object names, with the
format's own words excepted (SC-008).

## R14. The published text

**Decision**: `docs/introspection.md` gains section 12, "Game packs", after
"Versioning", since section numbers are stable. Sections 2.3, 2.4, 8 and 8.2 change to
carry the new fields and to point to 12; section 10.4 gains a sentence. The version
line becomes 0.2.0, and section 12 ends with the list of what changed from 0.1.0
(FR-025). Every statement in section 12 is cited by a test as "I§12.n", checked by the
test that already checks citations for sections 1 to 11 (FR-026).

## R15. Per-game code

**Decision**: A test in the introspection, experience and FFI crates' sources searches
for any story's digest, release-and-serial pair, or id; none may appear outside tests
(FR-035). The existing rule for the app (no game's name in its code) stands.

## R16. The command-line host

**Decision**: `zm` gains `--pack FILE` wherever it reports introspection, and the
per-stop line gains a `pack` field holding the word of R3. Without the option the field
is `none`.

**Rationale**: FR-006 for a person at a terminal, and it is how quickstart shows User
Story 1 without the app.
