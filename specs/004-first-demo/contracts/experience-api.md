# Contract: The Experience Layer

**Crate**: `zmachine-experience` | **Plan**: [../plan.md](../plan.md) | Shapes:
[../data-model.md](../data-model.md)

```text
pub struct Session { … }

impl Session {
    /// Loads the story file and runs it to its first stop.
    pub fn new(story: Vec<u8>, seed: Option<u32>, entropy: u32) -> Result<Session, StartError>;

    /// Gives the game a line the player typed. Refused if the game is not waiting for one.
    pub fn send_line(&mut self, line: &str) -> Result<(), Refused>;

    /// The player tapped a place on the compass, or an extra by its index.
    /// Sends that place's command if, and only if, the compass last reported offers one.
    pub fn tap(&mut self, place: Place) -> Result<(), Refused>;
    pub fn tap_extra(&mut self, index: usize) -> Result<(), Refused>;

    /// Abandons the game and starts a new one from the same story file.
    pub fn restart(&mut self);

    pub fn waiting(&self) -> Waiting;
    pub fn status(&self) -> Option<&Status>;
    pub fn compass(&self) -> &Compass;
    pub fn turns(&self) -> &[Turn];
    pub fn generation(&self) -> u64;

    /// Everything the screen needs, with the turns from `since` onward, as JSON.
    pub fn snapshot_json(&self, since: usize) -> String;
}

pub enum Refused { NotWaiting, NotOffered }
pub struct StartError { pub message: String }
```

## Rules

1. **One way in.** `send_line`, `tap`, `tap_extra` and `restart` are the only functions
   that change a session. `tap` and `tap_extra` end in the same call as `send_line`: the
   game receives a line of text and nothing else (constitution Principle III).
2. **A tap is refused, not guessed.** If the place is not `open` or `maybe` in the
   compass the session last reported, `tap` returns `NotOffered` and the game receives
   nothing (FR-012). A tap while the game is not waiting for a command returns
   `NotWaiting`.
3. **Every command is recorded.** The line sent, and whether it was typed or tapped, is
   in the turn it produced (FR-011, SC-006).
4. **Runs to the next stop.** Each of the four functions returns when the game is next
   waiting, has ended, or has faulted. A request from the game to save, restore or keep
   a transcript is answered by the layer and is not a stop (research R7).
5. **Bounded.** A run past 50 million instructions without a stop ends in `fault`.
6. **The compass is the introspection answer, redrawn after every stop**, by the mapping
   of research R5. The layer adds no knowledge of the game.
7. **Deterministic.** The same story file, seed and sequence of calls give the same
   snapshots.
8. **No panics** on any story file or any sequence of calls.
9. **No `unsafe`, no I/O, no game named** in the crate's source.

## Snapshot JSON

```text
{"generation":1,"waiting":"command","message":null,
 "status":{"place":"West of House","score":0,"moves":0,"hours":null,"minutes":null},
 "turn_count":2,
 "turns":[{"number":1,"command":"north","source":"tap","text":"North of House\nYou are facing…"}],
 "compass":{"known":true,"reason":null,
   "places":[{"place":"north","state":"open","command":"north","leads_to":"Forest Path"}, … twelve in all …],
   "extras":[]}}
```

Fields are always present; none is `null`. The twelve places are always listed, in the
order of the data model.
