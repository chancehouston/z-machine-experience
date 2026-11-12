# zmachine-introspect

A read-only window onto a running Z-machine game: the object tree, the dictionary, where
the player is, what they carry, the exits, and what is within reach.

- **The contract is [`docs/introspection.md`](../../docs/introspection.md).** This crate
  is one implementation of it. Another interpreter could implement the same document.
- **It only reads.** Every query takes the machine by shared reference. No query runs the
  game's code.
- **It never guesses.** Where a story file's contents do not settle an answer, the answer
  is "unknown", with a reason.
- **It knows families, not games.** Story files compiled with Inform 6 and its standard
  library, and with Infocom's ZIL, are recognised from their contents. No source file here
  names a game, a serial number, a release or a checksum, and a test enforces that.

It depends only on `zmachine-core`, has no `unsafe` and does no input or output.
