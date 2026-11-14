//! The card for something the player carries: what the game says of it when
//! it is examined, found by a peek, and what can be done with it
//! (specs/014-inventory-card/contracts/card.md).

use crate::announce;
use crate::strip::{ItemState, StripItem};
use zmachine_core::{Event, Machine};

/// Said in place of a description the peek could not have.
pub const NOTE_NO_DESCRIPTION: &str = "The game did not describe it. Type to ask.";
/// The label of the action that drops an item.
pub const LABEL_DROP: &str = "Drop";

/// Instructions a copy runs between checks on its limit.
const STEP: u64 = 1_000;

/// Something to do with an item: one command, about that item alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    /// What a screen reader says for it.
    pub spoken: String,
    /// The line it sends.
    pub command: String,
}

/// What the layer gives for one carried item at one turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    /// The game's printed name for the item.
    pub name: String,
    pub state: ItemState,
    /// The game's reply to examining it, from a copy of the game; none where
    /// the copy did not come back to the command prompt.
    pub description: Option<String>,
    /// The layer's words in place of a description.
    pub note: Option<&'static str>,
    /// What a screen reader is given when the card opens.
    pub spoken: String,
    pub actions: Vec<Action>,
}

/// What a copy of `machine` prints in reply to `line`: its text as a spoken
/// reply is trimmed, and only if the copy stops wanting a line at the command
/// prompt (`command_address`) within `limit` instructions and has said
/// something. `machine` itself is not touched (L§4, L§5).
pub(crate) fn peek(
    machine: &Machine,
    line: &str,
    command_address: u32,
    limit: u64,
) -> Option<String> {
    let mut copy = machine.clone();
    copy.provide_line(line).ok()?;
    let mut text = String::new();
    let mut ran = 0;
    loop {
        let event = copy.run(STEP).ok()?;
        text.push_str(&copy.take_output());
        ran += STEP;
        match event {
            Event::BudgetExhausted if ran < limit => continue,
            Event::LineWanted { .. } if copy.waiting_at() == Some(command_address) => break,
            // A question, a key, a save, a restore, a transcript, the end, or
            // the limit: nothing of what the copy said is given.
            _ => return None,
        }
    }
    let reply = announce::reply(&text, true);
    (!reply.is_empty()).then(|| reply.to_string())
}

/// The actions on an item's card: Drop, for an item not inside another, where
/// the story has the word (L§7).
pub(crate) fn actions(item: &StripItem, drop: Option<&str>) -> Vec<Action> {
    let mut actions = Vec::new();
    if let (false, Some(drop), Some(word)) = (item.inside, drop, item.word()) {
        actions.push(Action {
            label: LABEL_DROP.to_string(),
            spoken: format!("{LABEL_DROP} {}", item.name),
            command: format!("{drop} {word}"),
        });
    }
    actions
}

/// The card for `item`, with the description the peek gave.
pub(crate) fn card(item: &StripItem, description: Option<String>, drop: Option<&str>) -> Card {
    let said = description.as_deref().unwrap_or(NOTE_NO_DESCRIPTION);
    Card {
        name: item.name.clone(),
        state: item.state,
        spoken: format!("{}. {said}", item.name),
        note: description.is_none().then_some(NOTE_NO_DESCRIPTION),
        description,
        actions: actions(item, drop),
    }
}
