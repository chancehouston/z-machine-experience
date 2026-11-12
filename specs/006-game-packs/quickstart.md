# Quickstart: Validating Game Packs

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Commands are run from the repository's root and need only Rust, except step 8, which
needs Xcode and a simulator.

## 1. Everything in Rust

```bash
cargo test --workspace
```

Expected: all pass. The corpus target prints a line for each story and for each pack; a
pack whose story is absent reads `ignored, story absent`.

## 2. The packs are well formed

```bash
cargo run -q -p zmachine-corpus -- check
```

Expected: exit status 0. Any ledger entry on a marked answer is printed, for reading.

## 3. A door is open or closed (User Story 1)

```bash
cargo test -p zmachine-introspect --test packs
```

Expected: pass. On Zork I with its pack, a room with a door reports `door`, `closed`,
then `open` after the door is opened, the same room both times, `source` `pack`. With no
pack, with a pack of another digest, release or serial, and with a malformed pack, the
answers are those of version 0.1.0.

## 4. The entries are proven (User Story 2)

```bash
cargo run -q --release -p zmachine-corpus -- run zork1
```

Expected: a line for each entry ending `proven`, then `zork1 ok`.

```bash
cargo test -p zmachine-corpus --test proof
```

Expected: pass. Each of the other 31 attributes is contradicted on Zork I; an entry
seen in one state only is not proven; an untrusted variable that changes nothing is
unneeded.

## 5. No stale flag in the ledger (User Story 3)

```bash
grep -c "sets afresh" stories/corpus/ledger.txt
```

Expected: `0`.

## 6. Without packs, nothing changed (SC-005, SC-006)

```bash
cargo test -p zmachine-corpus --test nopack
```

Expected: pass. With packs set aside each story's verdict counts are feature 005's, and
each walkthrough's transcript is the same with and without its pack.

## 7. The published format (User Story 4)

```bash
cargo test -p zmachine-introspect --test spec_agreement
```

Expected: pass. Every statement of section 12 is cited by a test.

```bash
cargo test -p zmachine-corpus --test no_text
```

Expected: pass. No pack holds a word of a game.

Where Mini-Zork's file is present:

```bash
cargo run -q --release -p zmachine-corpus -- run minizork
```

Expected: its `open-attribute` entry `proven`, then `minizork ok`.

## 8. The compass (User Story 5)

```bash
scripts/test-ios.sh 27
```

Expected: pass, including a test that walks to a closed door, sees it as closed, opens
it by typing, and sees it as open. Then by hand in the simulator: the closed door can
be told from "may be blocked" at a glance, and VoiceOver says "closed door".
