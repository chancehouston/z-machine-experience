# Contract: The Kept Game

Three parts: the bytes (K), the layer's interface (L), the app's part (A).

## K. The bytes

UTF-8 text. Lines end with a single line feed. Nothing follows the last line's line
feed.

```text
lamplight-kept 1
story 3f2a…(64 hex digits)
chance fresh 2891337402
inputs 3
transcript 9c41…(64 hex digits)
typed open mailbox
tap north
typed 
check 77d0…(64 hex digits)
```

- **K§1** Line 1 is `lamplight-kept ` and the form's number. This contract is form 1.
- **K§2** `story` is the SHA-256 of the story file, lower-case hex.
- **K§3** `chance` is `fixed` or `fresh` and a number from 0 to 4294967295; for
  `fixed`, at most 2147483647.
- **K§4** `inputs` is the count of input lines that follow `transcript`.
- **K§5** `transcript` is the SHA-256, lower-case hex, of the text of every turn in
  order, each followed by one zero byte. The opening turn is the first.
- **K§6** An input line is `typed ` or `tap ` and then the input's text to the end of
  the line. The text may be empty; the space after the word is still there.
- **K§7** `check` is the SHA-256, lower-case hex, of every byte before the `c` of
  `check`.
- **K§8** Anything that departs from K§1 to K§7 is not a kept game: a missing line, a
  line out of order, a count that does not match, a digest that is not 64 hex digits,
  bytes that are not UTF-8, a carriage return, bytes after the end.
- **K§9** The same session gives the same bytes: nothing in them depends on the time
  or the device.

## L. The layer

- **L§1** `Session::open_kept(story, pack, chance, kept)` gives a session and an outcome:
  `new`, `brought back` or `could not` (data-model.md). `chance` is used for a new
  game; a game brought back uses the chance it was kept with.
- **L§2** A kept game is brought back only when, in this order: its check matches
  (K§7); its form is 1; its story digest is the story's; its inputs, given in order to
  a new game with its chance, are each accepted; the game does not fault and does not
  exceed the instruction ceiling on the way; and the transcript's digest then matches
  (K§5).
- **L§3** While the inputs are given, look-ahead is off. At the last stop it runs with
  its ordinary limits (research R2).
- **L§4** Bringing back gives the game exactly the kept inputs and nothing else
  (FR-009).
- **L§5** A session brought back and a session that was never closed, given the same
  later commands, give the same replies and the same snapshots, except `generation`
  (FR-007, FR-008).
- **L§6** `Session::kept()` gives the bytes for the game as it stands: every answered
  input, and the digest of the turns those inputs gave. After a fault it gives the
  bytes it gave before the faulting command.
- **L§7** On `could not`, the session's `notice` is the layer's words that the earlier
  game could not be brought back. No other outcome sets it.
- **L§8** `Session::keep_failed()` sets `notice` to the layer's words that the
  player's place is not being kept, the first time it is called in a game, and does
  nothing after that until a new game.
- **L§9** A notice is in the snapshot until the next turn is recorded. The notice of
  L§7, which comes with a game's first turn, is at the front of that snapshot's
  `announcement`. The notice of L§8 comes after a turn that has been spoken already,
  and changes no announcement.
- **L§10** `start_over` in the snapshot is the layer's question while the game waits
  for a command or an answer, and `null` when it has ended or faulted.
- **L§11** `Session::new_game(chance)` empties the answered inputs and takes the new
  chance; `kept()` then gives a kept game with no inputs.
- **L§12** No function of this contract panics or runs without bound on any bytes.
  Bringing back is bounded by the instruction ceiling for each input, as a command is.

### The C interface

- **L§13** `zmx_session_open(story, length, seed, has_seed, entropy, pack, kept,
  kept_length, outcome, error)` as `zmx_session_new_with_pack`, with the kept bytes
  (null for none) and an `outcome` written as 0 (`new`), 1 (`brought back`) or 2
  (`could not`).
- **L§14** `zmx_session_kept(session)` gives the kept game as a string the caller
  frees with `zmx_string_free`: the form K is text and holds no zero byte. It gives
  null for a session that has stopped unexpectedly, and what was stored is then left.
  (The plan had a buffer and a length; a string needs no second way of freeing.)
- **L§15** `zmx_session_keep_failed(session)`.
- **L§16** The functions that exist keep their meaning.

## A. The app

- **A§1** At launch the app reads the kept file for its story, if there is one, and
  hands its bytes to L§13. It does not look inside them.
- **A§2** After every command, new game or launch, before the snapshot is shown and
  before another command is accepted, the app asks for L§14 and writes the bytes to
  the kept file atomically. If the write fails it calls L§15 and shows the snapshot
  that follows.
- **A§3** The kept file is in the app's Application Support directory under `Kept`,
  named for the story's resource name, and is marked to be left out of backups.
- **A§4** The About sheet has a "Start over" button, with the identifier
  `button.startOver`. When the snapshot's `start_over` is text, choosing it shows a
  confirmation whose message is that text; confirming starts a new game and closes
  the sheet. When it is
  `null` the button starts a new game without asking.
- **A§5** A snapshot's `notice`, when text, is shown above the command field, with the
  identifier `notice`.
- **A§6** With the launch argument `-fresh` the app removes the kept file before A§1.
- **A§7** The app composes none of the words of L§7, L§8 or L§10.
