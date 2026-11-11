# Z-Machine Experience App — Handoff

Decided in voice with Chance, November 11, 2026. Starting point for Claude Code.

## Goal

A shipped iPhone and Android app: a faithful Z-machine interpreter with a modern touch "world experience" layer on top. Working name: "a better Zork." Pure text won't hold modern attention; the new interface is the product, the interpreter is the foundation.

## Architecture

- **Core:** faithful Z-machine in Rust. Text in, text out. Versions 3 and 5 only; no v6 graphics.
- **Introspection interface:** a specified way for a Z-machine to expose live internals (object tree, player location, objects in scope, exits, inventory, dictionary). Written as a spec that other interpreters could adopt. Lives in the main repo as Markdown for now; split out only once it stabilizes.
- **Experience layer:** reads state through the introspection interface and drives the UI. Synthesizes typed commands from taps (tap lantern → "take lantern"; tap exit → "north"). Typing remains available as a fallback.
- **Shells:** SwiftUI and Jetpack Compose over the Rust core via C bindings (same pipeline as the rr-dr60-emu project).

## Phases

1. **Faithful core.** Spec Kit / spec-driven development, test-first. Definition of faithful: passes the standard czech and praxix test programs, and a scripted Zork 1 walkthrough produces a transcript matching Frotz byte for byte.
2. **Introspection interface.** Write the spec, implement it on our core.
3. **First demo.** Zork boots, room text shows, a tappable compass of exits works end to end on a phone.

## Decisions already made

- Rust for the shared core (Chance's preference).
- Engine always takes typed commands; the user may not type them.
- Core is already solved by Frotz, Bocfel, etc.; we reimplement for ownership and to control the interface. Not a gap-filler on its own.
- Keep the interface generic enough that it could in principle sit on other Z-machines (stretch goal, not day one).

## Open items

- Repo name.
- Story file licensing: Zork is still Activision's; use it for dev/testing, bundle free Inform games for the shipped app.
- Whether the introspection spec eventually becomes its own repo.

## Working style

- Chance checks in at regular intervals during the day; ask questions and wait rather than guess.
- Lead with the strongest objection; distinguish real agreement from inability to fault an idea.
- Test first.
