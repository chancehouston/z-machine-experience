//! The strip: what of the inventory to show a player, and what a screen reader says
//! for it (specs/007-inventory-panel/data-model.md "Strip"; research R10).

use zmachine_introspect::{Answer, Carried, ContainerState, Tri, Uses};

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

/// What kind of thing an item is, for its token
/// (specs/014-inventory-card/contracts/card.md, L§13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Plain,
    /// A light, lit or not.
    Lamp,
    Container,
    /// Something to read.
    Page,
    /// Something to eat or to drink.
    Food,
    Clothing,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Plain => "plain",
            Kind::Lamp => "lamp",
            Kind::Container => "container",
            Kind::Page => "page",
            Kind::Food => "food",
            Kind::Clothing => "clothing",
        }
    }

    /// The first kind that applies; and for a lamp whether it is lit, where
    /// that is known.
    fn of(uses: &Uses) -> (Kind, Option<bool>) {
        let yes = |part: Tri| part == Tri::Yes;
        if yes(uses.lamp) {
            let lit = match (uses.on, uses.light) {
                (Tri::Yes, _) | (_, Tri::Yes) => Some(true),
                (Tri::No, _) | (_, Tri::No) => Some(false),
                _ => None,
            };
            (Kind::Lamp, lit)
        } else if yes(uses.container) {
            (Kind::Container, None)
        } else if yes(uses.read) {
            (Kind::Page, None)
        } else if yes(uses.eat) || yes(uses.drink) {
            (Kind::Food, None)
        } else if yes(uses.wear) {
            (Kind::Clothing, None)
        } else {
            (Kind::Plain, None)
        }
    }
}

/// Something the player carries and would be shown by the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripItem {
    /// The game's printed name for it.
    pub name: String,
    pub state: ItemState,
    /// Its place among the shown items, counted from 0 in the strip's order
    /// with what a thing holds after it
    /// (specs/014-inventory-card/contracts/card.md, S§1).
    pub index: usize,
    /// Whether it can be tapped for its card now.
    pub card: bool,
    /// The word a command names it by, if it has one.
    pub(crate) word: Option<String>,
    /// Whether it is inside another shown thing.
    pub(crate) inside: bool,
    /// What kind of thing it is, for its token.
    pub kind: Kind,
    /// For a lamp, whether it is lit, where that is known.
    pub lit: Option<bool>,
    /// What it is for, as the interface reports it.
    pub(crate) uses: Uses,
    pub(crate) worn: Tri,
    /// The things shown inside it.
    pub contents: Vec<StripItem>,
}

impl StripItem {
    /// The word a command names the item by: the word of its printed name
    /// that the game knows it by, or failing that the first of its words.
    pub fn word(&self) -> Option<&str> {
        self.word.as_deref()
    }

    /// This item and all it shows inside it, in the strip's order.
    fn each(&self) -> Vec<&StripItem> {
        let mut all = vec![self];
        for item in &self.contents {
            all.extend(item.each());
        }
        all
    }
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
    #[cfg(test)]
    pub(crate) fn of(inventory: &Answer<Vec<Carried>>) -> Strip {
        Strip::with(inventory, 9)
    }

    /// As [`Strip::of`], for a story whose dictionary keeps `kept` characters
    /// of a word.
    pub(crate) fn with(inventory: &Answer<Vec<Carried>>, kept: usize) -> Strip {
        let Ok(carried) = &inventory.value else {
            return Strip {
                known: false,
                note: Some(NOTE_NOT_KNOWN),
                items: Vec::new(),
                spoken: NOTE_NOT_KNOWN.to_string(),
            };
        };
        let items = shown(carried, kept, false, &mut 0);
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

impl Strip {
    /// The strip with its items marked as ones that can be tapped, where
    /// `cards` says cards are on offer now and the item has a word
    /// (specs/014-inventory-card/contracts/card.md, L§2).
    pub(crate) fn offering(mut self, cards: bool) -> Strip {
        fn mark(items: &mut [StripItem], cards: bool) {
            for item in items {
                item.card = cards && item.word.is_some();
                mark(&mut item.contents, cards);
            }
        }
        mark(&mut self.items, cards);
        self
    }

    /// The shown item at this place among them (S§1).
    pub fn item(&self, index: usize) -> Option<&StripItem> {
        self.items
            .iter()
            .flat_map(StripItem::each)
            .find(|item| item.index == index)
    }
}

/// The word a command names a thing by (research R2): the word of its printed
/// name whose form as the dictionary keeps it is one of the thing's words,
/// given in full; or the first of its words.
fn word_for(name: &str, words: &[String], kept: usize) -> Option<String> {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .find(|part| {
            let cut: String = part.chars().take(kept).collect();
            words.contains(&cut)
        })
        .or_else(|| words.first().cloned())
}

/// The entries the game would show, each with what it would show inside it. An
/// entry that is not seen, or not known to be, is left out with all it holds.
fn shown(carried: &[Carried], kept: usize, inside: bool, next: &mut usize) -> Vec<StripItem> {
    carried
        .iter()
        .filter(|entry| entry.seen == Tri::Yes)
        .map(|entry| {
            let name = entry
                .name
                .clone()
                .unwrap_or_else(|_| "something".to_string());
            let index = *next;
            *next += 1;
            let (kind, lit) = Kind::of(&entry.uses);
            StripItem {
                kind,
                lit,
                uses: entry.uses,
                worn: entry.worn,
                word: entry
                    .words
                    .as_deref()
                    .ok()
                    .and_then(|words| word_for(&name, words, kept)),
                name,
                index,
                card: false,
                inside,
                state: match entry.state {
                    Some(ContainerState::Open) => ItemState::Open,
                    Some(ContainerState::Closed) => ItemState::Closed,
                    Some(ContainerState::Unknown) | None => ItemState::Plain,
                },
                contents: shown(&entry.contents, kept, true, next),
            }
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
