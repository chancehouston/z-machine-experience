# Quickstart: Validating the Inventory Panel

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Run from the repository's root. Step 7 needs Xcode and a simulator.

## 1. Everything in Rust

```bash
cargo test --workspace
```

Expected: all pass.

## 2. What is seen (User Story 1)

```bash
cargo test -p zmachine-introspect --lib inventory_marks
```

Expected: pass. On built stories: a closed thing keeps its contents back, an open one
does not, a see-through one does not; a thing in a thing is seen only if both let it
be; with no pack a ZIL story's marks are unknown and every earlier value is unchanged.

## 3. On Zork I, with the game as the measure (User Story 2)

```bash
cargo run -q --release -p zmachine-corpus -- run zork1
```

Expected: every entry `proven`, among them `transparent-attribute 12`, then `zork1 ok`.
The report's `inventory` line has no difference unexplained.

## 4. A wrong fact fails

```bash
cargo test -p zmachine-corpus --test inventory
```

Expected: pass. Each of the other 31 attributes named as see-through fails on Zork I;
an entry with no place that needs it is not proven; a difference with its cause in the
ledger passes.

## 5. Nothing changed without a pack, and no transcript changed

```bash
cargo test -p zmachine-corpus --test nopack
```

Expected: pass.

## 6. The published text

```bash
cargo test -p zmachine-introspect --test spec_agreement
```

Expected: pass, with every new statement cited.

## 7. The strip (User Stories 3 and 4)

```bash
scripts/test-ios.sh 27
```

Expected: pass, including a test that starts with "Carrying nothing", takes a closed
container and sees it closed with nothing in it, opens it and sees what is in it, and
one drawn from hand-made snapshots for "not known". Then by hand: the strip is there
without being asked for, and VoiceOver reads it.
