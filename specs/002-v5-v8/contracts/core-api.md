# Contract: Core Library API, changes for versions 5 and 8

**Crate**: `zmachine-core` | **Plan**: [../plan.md](../plan.md)

This lists what is added to `specs/001-v3-core/contracts/core-api.md`. Everything there
still holds, and nothing in the existing interface changes shape: no existing type gains
a field and no existing function changes its signature. Names are indicative; the
behavior is the contract.

## Loading

```text
Machine::load(story_bytes, options) -> Result<Machine, LoadError>              // as before
Machine::load_with(story_bytes, options, settings) -> Result<Machine, LoadError>
```

- Both accept versions 3, 5 and 8. `LoadError::UnsupportedVersion(n)` for any other; its
  message names the three supported versions.
- `LoadError::TooLarge` when a version 5 or 8 file exceeds its version's limit. Version 3
  files are not checked, as before.
- `load` is `load_with` with default settings.
- `Settings`, all with defaults:
  - `screen`: columns and rows, default 80 by 24. Written into the header for versions 5
    and 8 and used as the size of the screen canvas.
  - `interpreter_number`, default 1, and `interpreter_version`, default `b'Z'`. Written
    into the header for versions 5 and 8.
  - `undo_slots`, default 25. With 0, `save_undo` stores -1, "undo is unavailable"
    (§15), and Flags 2 bit 4 is cleared (§6.1.4).
- Settings have no effect on a version 3 story file.

## Events and answers

```text
Event::KeyWanted                                  // a single keypress is wanted
machine.provide_key(key) -> Result<(), HostError>
machine.preloaded_input() -> &str                 // during LineWanted only
machine.provide_line_ended_by(text, key) -> Result<(), HostError>
```

- `provide_key` is valid only after `KeyWanted`. A character with no ZSCII code returns
  `HostError::UnrepresentableKey` and the machine keeps waiting.
- `Event::LineWanted { max_len }` is unchanged. In versions 5 and 8 `max_len` is how many
  more characters the buffer can take after any text already in it.
- `preloaded_input()` is the text a game placed in its input buffer beforehand (§15
  `read`). It is empty in version 3 and usually empty otherwise.
- `provide_line(text)`: what the player typed. It is stored after any pre-loaded text
  (§15: the interpreter "writes the new characters after those already there"), and the
  game is told the line ended with return (13).
- `provide_line_ended_by(text, key)` does the same but tells the game the line was ended
  by `key`, which must be one the game listed in its terminating characters table
  (§10.5.2.1); otherwise `HostError::NotATerminator` and the machine keeps waiting.
- A request for input with a time limit raises the same events as one without, and the
  routine it names is never called.
- `SaveRequested` and `RestoreRequested` are unchanged for the host. In versions 5 and 8
  the game receives 1 or 0 from a save, and 2 when it resumes from a restore. The forms
  of `save` and `restore` with operands store 0 and raise no event (§7.6.5.1 allows an
  interpreter not to offer auxiliary files).

## Output

```text
machine.take_output() -> String                 // unchanged: plain lower-window text
machine.take_styled_output() -> Vec<StyledRun>  // the same text, with styles
machine.screen() -> &Screen                     // the canvas, upper window height, cursor
machine.take_screen_changed() -> bool           // true once after each change
```

- `take_output` and `take_styled_output` drain one buffer. A host uses one or the other.
- `StyledRun { text, style, clears_window }`. Adjacent text with the same style is one
  run. A run with `clears_window` set has no text and marks the point where the game
  cleared the lower window.
- `Style { reverse, bold, italic, fixed_pitch }`.
- `Screen` gives `width()`, `rows()`, `upper_height()`, `cursor()`, and `cell(row, col)`
  as `Option<(char, Style)>`, `None` for a cell the game has not written. Details,
  including the overlay, are in [screen-model.md](screen-model.md).
- `take_status()` continues to report the version 3 status line and returns `None` for
  versions 5 and 8.
- Text a game redirects into memory reaches no host buffer and no transcript.
- For version 3 the screen is never written to: the game is told there is no upper
  window, as before.

## Undo

Undo needs nothing from the host beyond the `undo_slots` setting.

## Guarantees

The five guarantees of the version 3 contract hold for all three versions. Added:

6. **Version 3 is unchanged.** For a version 3 story file, every event, every drained
   buffer and every save file is byte for byte what the first feature produced, through
   the same functions with the same signatures.
7. **Bounded memory.** The screen canvas is fixed at the size the host set, undo keeps at
   most `undo_slots` positions, and a memory stream can write only inside the story's own
   dynamic memory. No story file can make the core allocate without limit.
