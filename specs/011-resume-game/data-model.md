# Data Model: Resume the Game

## Input

One thing the player gave the game that was answered.

| Field | Value |
| --- | --- |
| `text` | The line as sent. No line break. May be empty (return alone) |
| `source` | `typed` or `tap`, as the turn records it |

Whether the game took it as a line or as a single key is not recorded: the game
decides that again as it is played.

## Kept game

Made by the session; one per story on a device.

| Field | Value | Rule |
| --- | --- | --- |
| `form` | `1` | A reader that does not know the number does not read the rest |
| `story` | SHA-256 of the story file, 64 hex digits | Must equal the digest of the story the app holds |
| `chance` | `fixed N` or `fresh N` | What the game was started with (`Seed::Fixed`, `Seed::Fresh`) |
| `inputs` | Count, then the inputs in order | The count must equal the number of input lines |
| `transcript` | SHA-256 over every turn's text in order, the opening first | Compared after the inputs are given again |
| `check` | SHA-256 of every byte before the last line | Compared before anything else is done |

It holds no text of the game. The transcript's digest is a digest.

## Session (additions)

| Field | Value |
| --- | --- |
| `chance` | The `Seed` the game in progress was started with |
| `answered` | The inputs answered so far, in order (research R6) |
| `notice` | Words to show once, or none (research R8) |
| `keep_failed_told` | Whether "your place is not being kept" has been said in this game |

## Outcome of opening

What the layer reports when a session is made from a story and, perhaps, kept bytes.

| Outcome | When | Session holds |
| --- | --- | --- |
| `new` | No kept bytes were given | A new game; no notice |
| `brought back` | The kept game was read, played and matched | The game left; no notice |
| `could not` | The kept bytes were damaged, of another form, of another story, or did not play to a matching transcript | A new game with the given chance; the notice of FR-018 |

## States

```text
              no kept bytes
   launch ───────────────────────────► new game ──┐
     │                                             │ each answered turn:
     │ kept bytes                                  ▼ kept bytes made, app writes
     ├── read, played, matched ──────► game left ──┤
     │                                             │
     └── anything else ──► new game + notice ──────┘
                                                   │
        start over (confirmed) ◄───────────────────┤  inputs emptied, new chance,
              │                                    │  kept bytes written before shown
              └────────────────────────────────────┘
```

A fault adds no input. A game that ends adds the input that ended it.

## Snapshot (additions)

Two keys after `announcement`, in this order:

| Key | Value |
| --- | --- |
| `notice` | Text, or `null` |
| `start_over` | The question to ask before starting over, or `null` when no asking is needed |
