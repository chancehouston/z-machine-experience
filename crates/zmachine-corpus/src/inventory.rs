//! Comparing what the interface says is seen of the inventory with what a game's
//! own listing gives (specs/007-inventory-panel/research.md R3, R4, R8).

use zmachine_core::Machine;
use zmachine_introspect::{Carried, ContainerState, Introspector, Tri};

/// One carried object at one place, set against the game's own listing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Compared {
    pub object: u16,
    /// What the interface said: `Some(true)` seen, `Some(false)` kept back,
    /// `None` unknown.
    pub seen: Option<bool>,
    /// Whether the object's name is in the game's listing.
    pub listed: bool,
    /// It is seen through a closed thing: because that thing is see-through.
    pub through_closed: bool,
}

/// What a place gave when a copy of the game was asked for its inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    /// The interface could not say what is carried.
    Unknown,
    /// Nothing is carried, so there is nothing to compare.
    Empty,
    /// Something is carried and the reply names none of it: not a listing.
    NoListing,
    Listed(Vec<Compared>),
}

/// Asks a copy of `machine`, which must be waiting at a command prompt, for its
/// inventory with `command`, and sets what the reply lists against what the
/// interface marks as seen. The reply is read and dropped. `machine` is not
/// touched.
pub fn ask(machine: &Machine, introspector: &Introspector, command: &str) -> Asked {
    let view = machine.view();
    let Ok(carried) = introspector.inventory(&view).value else {
        return Asked::Unknown;
    };
    if carried.is_empty() {
        return Asked::Empty;
    }
    // Every object carried, at any depth, in the answer's order, with its name.
    let mut all: Vec<(Compared, String)> = Vec::new();
    flatten(&carried, false, &mut all);

    let mut copy = machine.clone();
    let reply = match copy
        .provide_line(command)
        .and_then(|()| copy.run(crate::trial::LIMIT))
    {
        Ok(_) => copy.take_output().to_lowercase(),
        Err(_) => String::new(),
    };
    // An object is listed if its own printed name is in the reply. Of several
    // objects of one name, as many are listed as the name occurs, those the
    // interface says are seen first.
    let mut order: Vec<usize> = (0..all.len()).collect();
    order.sort_by_key(|&i| all[i].0.seen != Some(true));
    let mut used: Vec<(String, usize)> = Vec::new();
    for i in order {
        let name = all[i].1.to_lowercase();
        if name.is_empty() {
            continue;
        }
        let occurs = reply.matches(name.as_str()).count();
        let taken = match used.iter_mut().find(|(n, _)| *n == name) {
            Some((_, taken)) => taken,
            None => {
                used.push((name, 0));
                &mut used.last_mut().expect("just pushed").1
            }
        };
        all[i].0.listed = *taken < occurs;
        *taken += 1;
    }
    if !all.iter().any(|(compared, _)| compared.listed) {
        return Asked::NoListing;
    }
    Asked::Listed(all.into_iter().map(|(compared, _)| compared).collect())
}

/// Every entry of an inventory, depth first, with what the interface said of
/// it. `closed` is whether the thing it is in is closed.
fn flatten(carried: &[Carried], closed: bool, out: &mut Vec<(Compared, String)>) {
    for entry in carried {
        let seen = match entry.seen {
            Tri::Yes => Some(true),
            Tri::No => Some(false),
            Tri::Unknown => None,
        };
        out.push((
            Compared {
                object: entry.object,
                seen,
                listed: false,
                through_closed: closed && seen == Some(true),
            },
            entry.name.clone().unwrap_or_default(),
        ));
        flatten(
            &entry.contents,
            entry.state == Some(ContainerState::Closed),
            out,
        );
    }
}
