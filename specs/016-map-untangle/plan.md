# Implementation Plan: Untangling the Map

**Branch**: `016-map-untangle` | **Date**: 2026-11-15 | **Spec**: [spec.md](spec.md)

(Written again after the owner changed the feature's direction. The first plan laid
a sheet out by itself; this one does it around the player, when asked.)

## Summary

The layer gains a redraw of the player's sheet: their place stays; the places it
joins are put along the ways walked; the rest are placed outward ring by ring and
then tidied by a bounded search that holds the near places still. The redraw is an
entry of the session of its own, sends nothing, and is recorded in the kept game at
its place among the inputs. The snapshot's map says when it can be asked for. The
app shows a button, and glides places and lines to their new positions.

One stage, one pull request against `main`.

## Strongest objection

**The map stops being a function of what was sent to the game.** Until now, the
turns alone gave the map, which is why a game brought back had its map for nothing.
A redraw is a second kind of event. It goes into the kept game (form 2), the replay
must apply it at the right moment, and every test that says "the same turns give
the same map" now means "the same turns and redraws". This is the cost of the
owner's fifth decision, and it is the part of this feature most likely to hide a
fault: a redraw applied one input early or late gives a different map.

**Second: the tidying's budget is a guess.** In the experiment it took between 700
and 1,800 candidate layouts on a sheet of 13. The plan allows 3,000 and measures.

## Technical Context

**Language/Version**: Rust 1.99; Swift for the app. **Dependencies**: none new.
**Storage**: the kept game gains a line; its form becomes 2, and form 1 is still
read. **Scale**: about 250 lines in the layer with as many of tests; one binding;
about 100 lines in the app.

## Constitution Check

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | Not touched. **Pass.** |
| II. Test-First | Each clause a test, seen failing first; the record says what happened. **Pass**, to be shown. |
| III. Typed Commands | A redraw sends the game nothing. **Pass.** |
| IV. Read-Only, no guessing | Positions come from walked ways alone. A position is drawing, not a claim about the game. **Pass.** |
| V. Strict Layering | Positions, and when a redraw is on offer, are the layer's. The app shows a button and glides what the snapshot gives. **Pass.** |
| VI. Licensing | Nothing of a story is added. **Pass.** |

## Changes to earlier contracts

- **Feature 012, P§2, P§4** ("no other placed place ever moves"): still true between
  redraws; a redraw moves places.
- **Feature 012, P§9** (the same turns give the same positions): the same turns and
  redraws.
- **Feature 011, K (the kept game's form)**: form 2 adds the line `redraw`. A reader
  takes form 1 and form 2.

## Project Structure

```text
specs/016-map-untangle/      spec, plan, research, data-model, quickstart, contracts/untangle.md, tasks
crates/zmachine-experience/src/map_redraw.rs, map_redraw_tests.rs   # around a place; the tidying
crates/zmachine-experience/src/map.rs                               # applying it; the offer in the view
crates/zmachine-experience/src/kept.rs, kept_tests.rs               # form 2
crates/zmachine-experience/src/lib.rs, json.rs                      # Session::redraw_map; the key
crates/zmachine-experience/tests/map.rs, resume.rs, speed.rs
crates/zmachine-corpus/tests/map.rs
crates/zmachine-ffi/                                                # zmx_session_redraw_map
specs/011-resume-game/contracts/kept-game.md, specs/012-map-first-slice/contracts/map.md
apps/ios/Lamplight/Snapshot.swift, Engine.swift, Game.swift, MapBand.swift, MapDrawing.swift
apps/ios/LamplightUITests/LamplightUITests.swift
```

## Decisions for the owner in this plan

1. **The kept game's form becomes 2** (the strongest objection).
2. **The layer's words for the control**: "Redraw the map around here".
3. **The control is at the map's lower left**, in the band and opened out.
4. **A press is answered within 50 ms**, not the 10 ms of a command (FR-013).
5. **Lines glide with their places**: every join is drawn as a shape whose ends can
   be animated, bent ones too.
