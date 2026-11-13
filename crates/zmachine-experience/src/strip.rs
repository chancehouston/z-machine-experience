//! The strip: what of the inventory to show a player, and what a screen reader says
//! for it (specs/007-inventory-panel/data-model.md "Strip"; research R10).

use zmachine_introspect::{Answer, Carried, ContainerState, Tri};

/// How an item of the strip is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemState {
    /// A thing that holds nothing to show, or whose state is not known.
    Plain,
    /// A thing that is open, with what it holds.
    Open,
    /// A thing that is closed.
    Closed,
}

impl ItemState {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemState::Plain => "plain",
            ItemState::Open => "open",
            ItemState::Closed => "closed",
        }
    }
}

/// Something the player carries and would be shown by the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripItem {
    /// The game's printed name for it.
    pub name: String,
    pub state: ItemState,
    /// The things shown inside it.
    pub contents: Vec<StripItem>,
}

/// What to show of the inventory at a stop.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Strip {
    /// False when what is carried could not be told.
    pub known: bool,
    /// What to show in place of items, if anything.
    pub note: Option<&'static str>,
    pub items: Vec<StripItem>,
    /// What a screen reader says for the whole strip.
    pub spoken: String,
}

/// The note shown when the player carries nothing.
pub const NOTE_NOTHING: &str = "Carrying nothing";
/// The note shown when what is carried is not known.
pub const NOTE_NOT_KNOWN: &str = "Not known: type to ask";

impl Strip {
    /// The strip for an inventory answer, as the introspection interface gives
    /// it. Only what is marked as seen is an item.
    pub(crate) fn of(inventory: &Answer<Vec<Carried>>) -> Strip {
        let Ok(carried) = &inventory.value else {
            return Strip {
                known: false,
                note: Some(NOTE_NOT_KNOWN),
                items: Vec::new(),
                spoken: NOTE_NOT_KNOWN.to_string(),
            };
        };
        let items = shown(carried);
        if items.is_empty() {
            return Strip {
                known: true,
                note: Some(NOTE_NOTHING),
                items,
                spoken: NOTE_NOTHING.to_string(),
            };
        }
        let phrases: Vec<String> = items.iter().map(phrase).collect();
        Strip {
            known: true,
            note: None,
            spoken: format!("Carrying: {}", phrases.join("; ")),
            items,
        }
    }
}

/// The entries the game would show, each with what it would show inside it. An
/// entry that is not seen, or not known to be, is left out with all it holds.
fn shown(carried: &[Carried]) -> Vec<StripItem> {
    carried
        .iter()
        .filter(|entry| entry.seen == Tri::Yes)
        .map(|entry| StripItem {
            name: entry
                .name
                .clone()
                .unwrap_or_else(|_| "something".to_string()),
            state: match entry.state {
                Some(ContainerState::Open) => ItemState::Open,
                Some(ContainerState::Closed) => ItemState::Closed,
                Some(ContainerState::Unknown) | None => ItemState::Plain,
            },
            contents: shown(&entry.contents),
        })
        .collect()
}

/// What a screen reader says for an item: its name, whether it is open or
/// closed, and what it holds.
fn phrase(item: &StripItem) -> String {
    let mut out = item.name.clone();
    match item.state {
        ItemState::Open => out.push_str(", open"),
        ItemState::Closed => out.push_str(", closed"),
        ItemState::Plain => {}
    }
    if !item.contents.is_empty() {
        let inside: Vec<String> = item.contents.iter().map(phrase).collect();
        out.push_str(", holding ");
        out.push_str(&listed(&inside));
    }
    out
}

/// "a", "a and b", "a, b and c"; with a comma before "and" where a part has
/// commas of its own.
fn listed(parts: &[String]) -> String {
    let and = if parts.iter().any(|p| p.contains(',')) {
        ", and "
    } else {
        " and "
    };
    match parts {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{}{and}{last}", rest.join(", ")),
    }
}
