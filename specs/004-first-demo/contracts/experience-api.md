# Contract: The Experience Layer

**Crate**: `zmachine-experience` | **Plan**: [../plan.md](../plan.md) | Shapes:
[../data-model.md](../data-model.md)

```text
pub struct Session { … }

pub enum Seed { Fixed(u32), Fresh(u32) }   // Fresh carries a number from the host's random source

/// The game and turn a command was made against.
pub struct Stamp { pub generation: u64, pub turn_count: usize }

impl Session {
    /// Loads the story file and runs it to its first stop.
    pub fn new(story: Vec<u8>, seed: Seed) -> Result<Session, StartError>;

    /// Gives the game a line the player typed.
    pub fn send_line(&mut self, at: Stamp, line: &str) -> Result<(), Refused>;

    /// The player tapped a place on the compass, or an extra by its index.
    pub fn tap(&mut self, at: Stamp, place: Place) -> Result<(), Refused>;
    pub fn tap_extra(&mut self, at: Stamp, index: usize) -> Result<(), Refused>;

    /// Abandons the game and starts a new one from the same story file.
    pub fn new_game(&mut self, seed: Seed);

    pub fn stamp(&self) -> Stamp;
    pub fn waiting(&self) -> Waiting;
    pub fn status(&self) -> Option<&Status>;
    pub fn compass(&self) -> &Compass;
    pub fn turns(&self) -> &[Turn];

    /// Everything the screen needs, with the turns from `since` onward, as JSON.
    pub fn snapshot_json(&self, since: usize) -> String;
}

pub enum Refused { Stale, NotWaiting, NotOffered, BadLine }
pub struct StartError { pub message: String }
```

## Rules

1. **One way in.** `send_line`, `tap`, `tap_extra` and `new_game` are the only functions
   that change a session. The first three end in one and the same private function, which
   gives the game a line of text and nothing else (constitution Principle III).
2. **A stale command is refused.** If `at` is not the session's current stamp, the
   function returns `Stale` and the game receives nothing (FR-016). The stamp changes
   with every turn and every new game.
3. **A tap is refused, not guessed.** If the place is not `open` or `maybe` in the
   current compass, `tap` returns `NotOffered` (FR-012). A tap or a line while the game
   has ended or faulted returns `NotWaiting`.
4. **A line is one line.** A line containing a line feed or carriage return is refused
   with `BadLine`. An empty line is accepted: games answer one.
5. **Every command is recorded**, with whether it was typed or tapped, in the turn it
   produced (FR-011, SC-006).
6. **Runs to the next stop.** Each of the four functions returns when the game is next
   waiting, has ended, or has faulted. A request from the game to save, restore or keep
   a transcript is answered by the layer (not stored; nothing to restore; declined) and
   is not a stop.
7. **Bounded.** A run past 50 million instructions without a stop ends in `fault`.
8. **Text is unaltered.** A turn's text is what the core handed over for the lower
   window since the last stop, byte for byte.
9. **The compass is the introspection answer, redrawn after every stop**, by the mapping
   of research R5. When the game is not waiting for a command the compass is not
   offered (research R6). The layer adds no knowledge of any game.
10. **Deterministic.** The same story file, seeds and sequence of calls give the same
    snapshots.
11. **No panics** on any story file or any sequence of calls.
12. **No `unsafe`, no I/O, no game named** in the crate's source. A fresh seed is a
    number the caller supplies; the layer has no random source of its own.

## The words the layer owns

The twelve places, their labels, and the English word each sends are a table in the
layer (research R5). The spoken texts are built from it:

| State | Spoken |
| --- | --- |
| `open`, destination known | "North, to Forest Path" |
| `open`, destination unknown | "North" |
| `maybe` | "North, may be blocked" |
| `closed` | "North, no exit" |
| `unknown` | "North, unknown" |
| `idle` | "North, not available now" |

## Snapshot JSON

```text
{"generation":1,"turn_count":2,"waiting":"command","message":null,
 "status":{"place":"North of House","score":0,"moves":1,"hours":null,"minutes":null},
 "turns":[{"number":1,"command":"north","source":"tap","text":"North of House\nYou are facing…\n\n>"}],
 "compass":{"offered":true,"known":true,"note":null,
   "places":[{"place":"north","label":"North","state":"open","command":"north","leads_to":"Forest Path","spoken":"North, to Forest Path"}, … twelve in all …],
   "extras":[]}}
```

Every field is always present; one with no value is `null`, never left out. The twelve
places are always listed, in the order of the data model.
