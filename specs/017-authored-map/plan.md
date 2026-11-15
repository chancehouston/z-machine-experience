# Implementation Plan: An Authored Map for Zork I

**Branch**: `017-authored-map` | **Date**: 2026-11-15 | **Spec**: [spec.md](spec.md)

## Summary

A presentation file is a small text file beside a story's game pack: its story's
identity, sheets with optional names, and for each place a sheet and a position. The
experience layer reads it when a game is opened, refuses it whole if anything is
amiss, and gives it to the map. The map puts a place the file has at its authored
sheet and position when the player first stands in it, keeps authored positions
clear of places the file does not have, names a sheet from the file, marks authored
places in what it reports, and offers no redraw on an authored sheet. The app
bundles the file with the story and hands it over unread.

Two stages, two pull requests: (1) the mechanism and the trial file (above ground
and the first region below), ending in the owner's phone trial; (2) the rest of
Zork I's rooms, planned after that trial.

## Strongest objection

**The trial file is authored by me, and my test of it is a count.** I can make a
layout with no crossings and few bent joins and still have a page that does not read
as the place. The published maps are good because people who knew the game drew
them. I will draft from the ways walked, adjust by hand with those maps open, and
show the owner pictures before the phone trial; but the measure that matters,
whether it reads, is the owner's alone, and a second or third file may be needed
before it does.

**Second: the way the game is opened changes again.** The layer's entry and the
binding each gain an argument for the file's text, and every caller follows. This
is the third time (pack, kept game, now this).

**Third: a place between.** A join is straight when no drawn place lies between its
ends. On an authored sheet a room not yet found may sit on a line; when the player
finds it, a join that was straight becomes bent. The file's author has to keep rooms
off the lines of others, and a test counts the cases on the walkthrough.

## Technical Context

**Language/Version**: Rust 1.99; Swift for the app. **Dependencies**: none new.
**Storage**: one file per story in `presentations/`, committed. **Scale**: about 250
lines in the layer with as many of tests; a changed binding; a few lines in the app
and its build script; the file itself, about 40 lines for the trial.

## Constitution Check

| Principle | How this plan stands |
| --- | --- |
| I. Faithful Core | Not touched. **Pass.** |
| II. Test-First | Each clause a test, seen failing first; the record says what happened. **Pass**, to be shown. |
| III. Typed Commands | Nothing is sent. **Pass.** |
| IV. Introspection | No query changes. The file is not a pack and holds no fact. **Pass.** |
| V. Strict Layering | The file is read and applied in the layer. The app holds its bytes and reads none. **Pass.** |
| VI. Licensing | The file holds numbers and sheet names of the project's own. **Pass.** |
| VII. Authored Presentation | Identity: digest, release, serial, all three checked. Drawing only: a place is put, never shown, by the file (a test at every turn of the walkthrough). Names: sheets only. No behaviour: no query or input changes (the transcript with and without the file is the same). Our own work: positions drafted from ways walked and adjusted by hand; a note beside the file records what was consulted. Checked: each entry a room of the story, no two in one position. Marked: `authored` on each place. Absent means unchanged: every earlier test passes with no file. **Pass**, with the copying question named in the specification left open. |

## Project Structure

```text
specs/017-authored-map/        spec, plan, research, data-model, quickstart, contracts/presentation.md, tasks
presentations/README.md        the form; what a file may hold
presentations/zork1.present    the trial file
presentations/zork1.md         what was consulted, and how the positions were made
crates/zmachine-experience/src/presentation.rs, presentation_tests.rs   # reading and refusing
crates/zmachine-experience/src/map.rs, map_tests.rs                     # authored places, sheets, names, no redraw
crates/zmachine-experience/src/lib.rs, json.rs                          # opening with a file; `authored`
crates/zmachine-experience/tests/presentation.rs                        # Zork I: every turn; orders; measures
crates/zmachine-ffi/                                                    # zmx_session_open takes the file
scripts/build-ios.sh, apps/ios/Lamplight/Game.swift, Engine.swift
```

## Decisions for the owner in this plan

1. **Two stages**, the trial first.
2. **The file's form** (contracts/presentation.md): plain lines, as a pack's are.
3. **A place the file has is always on a position**, never attached small at a
   corner; the Attic and Up a Tree get positions of their own.
4. **Pictures before the phone**: the trial file is shown to the owner as a drawing
   of the whole trial region before it is built into the app.
