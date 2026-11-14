//! The card for something the player carries: what the game says of it when
//! it is examined, found by a peek, and what can be done with it
//! (specs/014-inventory-card/contracts/card.md).

use crate::announce;
use crate::strip::{ItemState, Kind, StripItem};
use crate::verbs::Acts;
use zmachine_core::{Event, Machine};
use zmachine_introspect::Tri;

/// Said in place of a description the peek could not have.
pub const NOTE_NO_DESCRIPTION: &str = "The game did not describe it. Type to ask.";
/// The label of the action that drops an item.
pub const LABEL_DROP: &str = "Drop";
pub const LABEL_OPEN: &str = "Open";
pub const LABEL_CLOSE: &str = "Close";
pub const LABEL_TURN_ON: &str = "Turn on";
pub const LABEL_TURN_OFF: &str = "Turn off";
pub const LABEL_READ: &str = "Read";
pub const LABEL_WEAR: &str = "Wear";
pub const LABEL_TAKE_OFF: &str = "Take off";
pub const LABEL_EAT: &str = "Eat";
pub const LABEL_DRINK: &str = "Drink";

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
    /// What kind of thing it is, and for a lamp whether it is lit.
    pub kind: Kind,
    pub lit: Option<bool>,
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

/// The actions on an item's card, for an item not inside another (L§10 to
/// L§14): what the item's own flags say it is for, in the state it is in,
/// each only where the story has the words. The order is the owner's: what
/// can be undone is first, nearest the close control; Drop, eating and
/// drinking are last. Nothing here is tried: the game has the last word.
pub(crate) fn actions(item: &StripItem, acts: &Acts) -> Vec<Action> {
    let Some(word) = item.word().filter(|_| !item.inside) else {
        return Vec::new();
    };
    let uses = &item.uses;
    let yes = |part: Tri| part == Tri::Yes;
    let no = |part: Tri| part == Tri::No;
    let offered: [(bool, &'static str, &Option<String>); 10] = [
        (yes(uses.opens) && no(uses.open), LABEL_OPEN, &acts.open),
        (yes(uses.opens) && yes(uses.open), LABEL_CLOSE, &acts.close),
        (
            yes(uses.switch) && no(uses.on),
            LABEL_TURN_ON,
            &acts.turn_on,
        ),
        (
            yes(uses.switch) && yes(uses.on),
            LABEL_TURN_OFF,
            &acts.turn_off,
        ),
        (yes(uses.read), LABEL_READ, &acts.read),
        (yes(uses.wear) && no(item.worn), LABEL_WEAR, &acts.wear),
        (yes(item.worn), LABEL_TAKE_OFF, &acts.remove),
        (true, LABEL_DROP, &acts.drop),
        (yes(uses.eat), LABEL_EAT, &acts.eat),
        (yes(uses.drink), LABEL_DRINK, &acts.drink),
    ];
    offered
        .into_iter()
        .filter(|(applies, ..)| *applies)
        .filter_map(|(_, label, command)| {
            command.as_ref().map(|command| Action {
                label: label.to_string(),
                spoken: format!("{label} {}", item.name),
                command: format!("{command} {word}"),
            })
        })
        .collect()
}

/// The card for `item`, with the description the peek gave.
pub(crate) fn card(item: &StripItem, description: Option<String>, acts: &Acts) -> Card {
    let said = description.as_deref().unwrap_or(NOTE_NO_DESCRIPTION);
    Card {
        name: item.name.clone(),
        state: item.state,
        kind: item.kind,
        lit: item.lit,
        spoken: format!("{}. {said}", item.name),
        note: description.is_none().then_some(NOTE_NO_DESCRIPTION),
        description,
        actions: actions(item, acts),
    }
}
