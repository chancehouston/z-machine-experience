# Implementation Plan: The Other Sheets of the Map

**Branch**: `015-map-sheets` | **Date**: 2026-11-14 | **Spec**: [spec.md](spec.md)

## Summary

The layer can already draw the sheet the player is on. It learns to draw any sheet,
by number, in the same form, with the marks that say whether it is the player's own
and which sheets are before and after it. The app asks for a sheet when the player
turns to it in the opened-out map, shows it in place of their own, and returns to
their own when the map is closed or a turn comes.

One stage, one pull request, opened against `main` once the inventory card's stage 2
is merged. Not stacked.

## Strongest objection

**The app gains a second piece of state the snapshot does not hold**, after the
card: which sheet is being looked at. It is dropped at every new turn and when the
map closes, as the card is. If that rule slips, the band could show a sheet the
player is not on while they type, with no ways to walk. The band is given the
player's own sheet by a separate path from the one that draws the opened-out map,
and a test holds it to that.

**Second: a sheet asked for on its own costs a call for every turn of the page.**
The whole atlas could have been sent with every snapshot instead. On Zork I that is
eleven sheets of up to 7 KB at every stop, for something looked at rarely. A sheet
is made in under 0.1 ms, so asking is cheap.

## Technical Context

**Language/Version**: Rust 1.99; Swift for the app. **Dependencies**: none new.
**Storage**: none. **Testing**: as the earlier features. **Scale**: about 80 lines
in the layer, one binding, about 150 lines in the app, with as many of tests.

## Constitution Check

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | Not touched. **Pass.** |
| II. Test-First | Each clause a test, seen failing first; the record says what happened. **Pass**, to be shown. |
| III. Typed Commands | Turning a sheet sends nothing. **Pass.** |
| IV. Read-Only, no guessing | A sheet shows what was witnessed, as the player's own does. Nothing is inferred about a sheet's present state. **Pass.** |
| V. Strict Layering | Which sheets there are, their order, their labels, where a way leads and what is spoken are the layer's. The app shows a sheet and remembers which it was asked to show. **Pass.** |
| VI. Licensing | Nothing of a story is added. **Pass.** |

## Project Structure

```text
specs/015-map-sheets/   spec, plan, research, data-model, quickstart, contracts/sheets.md, tasks
crates/zmachine-experience/src/map.rs, map_tests.rs     # a view of any sheet
crates/zmachine-experience/src/json.rs, lib.rs          # its keys; Session::sheet
crates/zmachine-experience/tests/map.rs                 # Zork I's sheets; nothing changes
crates/zmachine-ffi/                                    # zmx_session_sheet
apps/ios/Lamplight/Snapshot.swift, Engine.swift, Game.swift
apps/ios/Lamplight/MapBand.swift, MapDrawing.swift, GameView.swift
apps/ios/LamplightUITests/LamplightUITests.swift
```

## Decisions for the owner in this plan

1. **A sheet is asked for, not sent with every snapshot** (the second objection).
2. **The layer's words**: the label "Sheet 2 of 11 · Cellar" (or "Sheet 2 of 11");
   spoken for another sheet, "Map. Sheet 2 of 11, Cellar. You are not on this
   sheet."; the arrows' and the return control's spoken words are the label of the
   sheet each turns to.
3. **On another sheet every place's unwalked ways are drawn as stubs**, the first
   place found on it included: on the player's own sheet their own place has
   controls in their stead.
