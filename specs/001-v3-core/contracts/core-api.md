# Contract: Core Library API

**Crate**: `zmachine-core` | **Plan**: [../plan.md](../plan.md)

This is the interface the CLI uses now and the C bindings will wrap later. Names are
indicative; the behavior is the contract. Entities are defined in
[../data-model.md](../data-model.md).

## Loading

```text
Machine::load(story_bytes, options) -> Result<Machine, LoadError>
```

- `options.seed`: optional seed, 0 to 2147483647. When absent, the host must supply
  `options.entropy`, a value from its own source; the core never reads a clock. Either
  value is placed directly in the generator; a small seed does not select predictable
  mode ([../research.md](../research.md) R2).
- `LoadError`: `UnsupportedVersion(n)`, `TooShort`, `BadHeader(field)`.
- Loading sets the header fields the interpreter owns (Standard §11.1). The same fields
  are re-applied after every restart and restore.
- A header length or checksum that does not match the file is not a load error; the
  game's `verify` reports it.

## Running

```text
machine.run(budget) -> Result<Event, HostError>
```

- Executes at most `budget` instructions, then returns one Event.
- Returns earlier when the game needs the host or stops.
- `HostError` is returned, and nothing changes, when `run` is called while an answer is
  owed or after `Finished` or `Fault`.

## Answering events

```text
machine.provide_line(text)            // after LineWanted
machine.save_result(stored: bool)     // after SaveRequested
machine.provide_restore(Some(bytes))  // after RestoreRequested
machine.provide_restore(None)         // host declined
machine.transcript_result(accepted)   // after TranscriptRequested
```

- `LineWanted` carries the maximum number of characters the game will accept, which in
  version 3 is one less than the first byte of its text buffer (Standard §15 `read`).
  `provide_line` drops anything beyond it and lowercases letters.
- `SaveRequested` carries complete Quetzal bytes. `save_result(true)` makes the game's
  save succeed; `false` makes it fail.
- `provide_restore` with bytes that are malformed or belong to another story leaves the
  machine untouched and makes the game's restore fail. On success, execution resumes
  inside the saved `save` instruction, reporting success.
- `transcript_result(false)` clears the transcription bit so the game sees it is off.
- Each answer is valid only immediately after its event; otherwise `HostError`.

## Output

```text
machine.take_output() -> String          // screen text since the last call; unwrapped
machine.take_transcript() -> String      // transcript text since the last call
machine.take_status() -> Option<StatusLine>   // Some only if it changed
machine.take_diagnostics() -> Vec<Diagnostic> // tolerated illegal operations
```

- Text is ZSCII translated to Unicode (Standard §3.8). Line ends are `\n`.
- Screen text does not include the player's input. Transcript text does, as
  [transcript-format.md](transcript-format.md) describes.
- The host may drain after every `run` or less often; nothing is lost in between.
- When the game quits leaving a line unfinished, the core ends it with a line feed in both
  the screen text and the transcript text.
- Transcript text is released a word at a time, so `take_transcript` may lag the screen
  by the word in progress until the game reads input or stops.

## Guarantees

1. **No I/O.** The crate opens no files, reads no clock, prints nothing.
2. **No `unsafe`.** Enforced with `#![forbid(unsafe_code)]`.
3. **No panics on any story file.** Every illegal operation becomes a `Fault` event or,
   for object 0, a diagnostic.
4. **Determinism.** Events and output are a pure function of story bytes, seed or
   entropy, and the answers given. Budget size does not affect them.
5. **Draining is passive.** The `take_` calls do not affect execution.

## Not in this contract

Reading the object tree, dictionary or variables from outside. That is the introspection
interface, a later feature.
