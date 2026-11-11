# Contract: Core Library API

**Crate**: `zmachine-core` | **Plan**: [../plan.md](../plan.md)

This is the interface the CLI uses now and the C bindings will wrap later. Names are
indicative; the behavior is the contract. Entities are defined in
[../data-model.md](../data-model.md).

## Loading

```text
Machine::load(story_bytes, options) -> Result<Machine, LoadError>
```

- `options.seed`: optional 32-bit seed. When absent, the host must supply
  `options.entropy`, a 32-bit value from its own source; the core never reads a clock.
- `LoadError`: `UnsupportedVersion(n)`, `TooShort`, `BadHeader(field)`.
- Loading sets the interpreter-owned header bits (Standard §11.1): status line available,
  screen splitting unavailable, variable-pitch font not default.
- A wrong checksum or length is not a load error (the game's `verify` reports it).

## Running

```text
machine.run(budget) -> Event
```

- Executes at most `budget` instructions, then returns one Event.
- Returns earlier when the game needs the host or stops.
- Calling `run` while an answer is owed, or after `Finished` or `Fault`, returns a
  `HostError` and changes nothing.

## Answering events

```text
machine.provide_line(text)            // after LineWanted
machine.save_result(stored: bool)     // after SaveRequested
machine.provide_restore(Some(bytes))  // after RestoreRequested
machine.provide_restore(None)         // host declined
```

- `provide_line`: text beyond the game's buffer is dropped; letters are lowercased;
  characters outside ZSCII input are replaced with `?` (Standard §3.8, §15 `read`).
- `provide_restore` with bytes that are malformed or belong to another story leaves the
  machine untouched and makes the game's restore branch fail.
- Each answer is valid only immediately after its event; otherwise `HostError`.

## Output

```text
machine.take_output() -> String        // text printed since the last call; unwrapped
machine.take_status() -> Option<StatusLine>   // Some only if it changed since last call
```

- Output is ZSCII translated to Unicode (Standard §3.8). Newlines are `\n`.
- The host may drain after every `run` or less often; nothing is lost in between.

## Guarantees

1. **No I/O.** The crate opens no files, reads no clock, prints nothing.
2. **No `unsafe`.** Enforced with `#![forbid(unsafe_code)]`.
3. **No panics on any story file.** Every illegal operation becomes `Event::Fault`.
4. **Determinism.** The sequence of events and output is a pure function of story bytes,
   seed (or entropy), the answers given, and nothing else. Budget size does not affect it.
5. **No hidden state changes.** `take_output` and `take_status` do not affect execution.

## Not in this contract

Reading the object tree, dictionary or variables from outside. That is the introspection
interface, a later feature.
