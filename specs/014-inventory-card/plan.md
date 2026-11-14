# Implementation Plan: The Inventory Card

**Branch**: `014-inventory-card` | **Date**: 2026-11-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/014-inventory-card/spec.md`

## Summary

The introspection interface gains, for each carried thing, the dictionary words
that name it. The experience layer finds the story's words for examining and
dropping once, marks in the snapshot which shown items can be tapped, and on a
touch makes a **card**: it runs the examine command on a copy of the game, takes
what the copy printed if the copy came back to the command prompt, and returns the
card as JSON by a new entry. The card's actions are sent by another new entry as
tapped lines. The app shows the card over the map's band.

Two stages, two pull requests. This plan designs stage 1 fully and stage 2 in
outline; stage 2 is planned again, with its own research, when stage 1 is merged.

## Strongest objection

**Stage 1 changes the introspection interface, which five features rest on.** Every
carried thing gains its naming words. In an Infocom story the property that holds
them is not named anywhere: it is found by its shape (the one property whose every
holder holds a list of dictionary entries). The probe found exactly one such
property in each of the eleven recognised files, but a rule found by measuring is
a rule that a twelfth file can break. Where no single property stands out, the
interface says the words are not known, items cannot be tapped, and nothing is
guessed. A pack entry can name the property for a story where the shape fails.

**Second: the card is the first thing the app shows that is not in the snapshot.**
It is asked for and returned on its own, and lives in the app until the next turn.
That is new state in the shell. It is kept honest by the stamp: a card is for one
turn, and the app drops it at any other.

**Third: the peek's text is trimmed by the rule the screen reader's text is.** If
that rule cuts something a reader wanted, the card cuts it too.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024); Swift for the app

**Primary Dependencies**: None new.

**Storage**: None. A card is never kept.

**Testing**: `cargo test` on built stories, Zork I, Adventure, the corpus; the
app's UI tests on iOS 26 and 27; the owner's phone after each stage.

**Target Platform**: Linux and macOS; iOS 26 and 27.

**Project Type**: Libraries, two command-line programs and an iPhone app.

**Performance Goals**: A card within 10 ms (SC-004). The snapshot at each stop
within a twentieth of what it costs now (FR-020): the words are read at each stop
for carried things only.

**Constraints**: A peek leaves the game exactly as it was. The app composes no
text and holds no command word. Nothing is examined before it is tapped.

**Scale/Scope**: Stage 1: about 80 lines in the introspection crate, 200 in the
layer, two bindings, 150 in the app, with as many of tests.

## Constitution Check

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | The core is not changed. A copy is a clone of the machine, as look-ahead makes. **Pass.** |
| II. Test-First | Each clause of the contract is a test, seen failing first; the record says what happened. **Pass**, to be shown. |
| III. The Engine Takes Typed Commands | An action is one tapped line, visible and kept. A peek sends a typed line to a copy, never to the game. The game itself receives nothing the player did not see sent. **Pass.** |
| IV. Introspection Is Read-Only | The naming words and the verbs are read from the story file. The peek's text is the game's own. What cannot be read is said to be unknown. Stage 2's flags in a pack are facts of the file, as the pack's present entries are; no content is authored. **Pass.** |
| V. Strict Layering | The words, the peek, the card and every label are the layer's. The app shows a card and sends touches. The token is drawing, which is the shell's. **Pass.** |
| VI. Story File Licensing | Nothing of a story is added to the repository. Corpus reports give counts, not text. **Pass.** |

**Does the peek need an amendment?** Feature 009 settled that trying a command on
a copy complies with Principle IV. The peek shows the copy's text, which 009 did
not. The text is the game's and nothing is inferred from it, so this plan takes it
as within the principle; the owner chose the peek knowing it. Said here so it is
not passed over.

## Project Structure

```text
specs/014-inventory-card/
├── spec.md, plan.md, research.md, data-model.md, quickstart.md
├── contracts/card.md
├── checklists/requirements.md
└── tasks.md                          # stage 1's; stage 2's are added later

docs/introspection.md                 # section 7: the words that name a thing
crates/zmachine-introspect/src/
├── lib.rs, inform.rs, zil.rs         # Carried.words; the naming property
├── json.rs                           # "words"
└── pack.rs                           # stage 2: the flags' entries
crates/zmachine-experience/src/
├── look.rs → verbs.rs                # the words for looking, examining, dropping
├── strip.rs                          # which items can be tapped
├── card.rs, card_tests.rs            # the card, the peek, the actions
├── json.rs                           # the strip's marks; the card as JSON
└── lib.rs                            # Session::card, Session::tap_action
crates/zmachine-experience/tests/card.rs
crates/zmachine-corpus/tests/card.rs
crates/zmachine-ffi/                  # zmx_session_card, zmx_session_tap_action
apps/ios/Lamplight/
├── Snapshot.swift, Engine.swift, Game.swift
├── InventoryStrip.swift              # items as buttons
├── ItemCard.swift                    # the card, and the token
└── GameView.swift, MapBand.swift     # the card over the band
```

## Decisions for the owner in this plan

1. **The interface changes** (the strongest objection), with the pack as the remedy
   where a story's shape fails.
2. **The layer's words**, fixed here unless changed:
   - (removed 2026-11-14 by the owner's ruling) on every card: "A look ahead. The
     game has not been told."
   - where there is no description: "The game did not describe it. Type to ask."
   - the action: "Drop", spoken "Drop" and the item's name.
3. **The card's text is the reply as a screen reader hears it**: without the prompt
   and the white space round it.
4. **Stage 2 is not designed in detail here.** Its outline is in research R9.

## Complexity Tracking

No violation to justify.
