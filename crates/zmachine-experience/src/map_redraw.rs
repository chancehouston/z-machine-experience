//! Laying a sheet of the map out again around one place
//! (specs/016-map-untangle/contracts/untangle.md, part R).
//!
//! The player asks for it. The place they are in keeps its position; the
//! places it is joined to are put exactly where the ways walked say; the rest
//! follow outward, a ring at a time, and are then moved about to leave fewer
//! joins bent and crossing while the near places are held still. Nothing here
//! uses chance or a clock, so the same sheet redrawn around the same place
//! comes out the same every time.

use crate::map_place::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// A compass way the player has walked between two places of one sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Way {
    pub from: u16,
    /// The step on the page for the way's direction.
    pub delta: Cell,
    /// The direction's place in the compass's order.
    pub rank: usize,
    pub to: u16,
}

/// How many steps along a way a place may be put from the place it is
/// joined to, before it is left for later.
const REACH: i32 = 3;

/// How far out the search for a free position goes.
const RINGS: i32 = 64;

/// How a layout is judged, least first: joins that do not lie along a way
/// walked; pairs of joins that cross; the summed length of the joins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Measure {
    pub bent: usize,
    pub crossings: usize,
    pub length: i32,
}

/// Whether `to` is exactly along `delta` from `from`, with no place between.
fn lies_along(at: &BTreeMap<u16, Cell>, taken: &BTreeSet<Cell>, way: &Way) -> bool {
    let (Some(&from), Some(&to)) = (at.get(&way.from), at.get(&way.to)) else {
        return false;
    };
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let steps = dx.abs().max(dy.abs());
    if steps == 0 || (way.delta.0 * steps, way.delta.1 * steps) != (dx, dy) {
        return false;
    }
    (1..steps).all(|k| !taken.contains(&(from.0 + way.delta.0 * k, from.1 + way.delta.1 * k)))
}

/// The pairs of places joined by a way, each with the ways between them.
fn joins(ways: &[Way]) -> BTreeMap<(u16, u16), Vec<&Way>> {
    let mut joins: BTreeMap<(u16, u16), Vec<&Way>> = BTreeMap::new();
    for way in ways.iter().filter(|way| way.from != way.to) {
        joins
            .entry((way.from.min(way.to), way.from.max(way.to)))
            .or_default()
            .push(way);
    }
    joins
}

/// Whether the segments `p`-`q` and `r`-`s` cross between their ends.
fn cross(p: Cell, q: Cell, r: Cell, s: Cell) -> bool {
    let side = |o: Cell, a: Cell, b: Cell| {
        ((a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)).signum()
    };
    side(p, q, r) * side(p, q, s) < 0 && side(r, s, p) * side(r, s, q) < 0
}

/// How the places at `at` are judged, for the ways among them.
pub(crate) fn measure(at: &BTreeMap<u16, Cell>, ways: &[Way]) -> Measure {
    let taken: BTreeSet<Cell> = at.values().copied().collect();
    let joins = joins(ways);
    let mut bent = 0;
    let mut length = 0;
    let mut segments: Vec<(u16, u16, Cell, Cell)> = Vec::with_capacity(joins.len());
    for (&(a, b), between) in &joins {
        let (Some(&pa), Some(&pb)) = (at.get(&a), at.get(&b)) else {
            continue;
        };
        if !between.iter().any(|way| lies_along(at, &taken, way)) {
            bent += 1;
        }
        length += (pa.0 - pb.0).abs().max((pa.1 - pb.1).abs());
        segments.push((a, b, pa, pb));
    }
    let mut crossings = 0;
    for (n, s) in segments.iter().enumerate() {
        for t in &segments[n + 1..] {
            let apart = s.0 != t.0 && s.0 != t.1 && s.1 != t.0 && s.1 != t.1;
            if apart && cross(s.2, s.3, t.2, t.3) {
                crossings += 1;
            }
        }
    }
    Measure {
        bent,
        crossings,
        length,
    }
}

/// The places a place is joined to and the step toward each: by the ways
/// walked out of it, in the compass's order; then by the ways walked into
/// it, each of which puts its place opposite. A place is named once, by the
/// first of these that names it (R§2, R§3).
fn neighbours(ways: &[Way], of: u16) -> Vec<(Cell, u16)> {
    let mut out: Vec<&Way> = ways
        .iter()
        .filter(|way| way.from == of && way.to != of)
        .collect();
    out.sort_by_key(|way| way.rank);
    let mut into: Vec<&Way> = ways
        .iter()
        .filter(|way| way.to == of && way.from != of)
        .collect();
    into.sort_by_key(|way| way.rank);
    let mut named: Vec<(Cell, u16)> = Vec::new();
    for (delta, other) in out.iter().map(|way| (way.delta, way.to)).chain(
        into.iter()
            .map(|way| ((-way.delta.0, -way.delta.1), way.from)),
    ) {
        if !named.iter().any(|(_, seen)| *seen == other) {
            named.push((delta, other));
        }
    }
    named
}

/// The nearest free position to `near`: by rings outward; within a ring the
/// lowest y, then the lowest x.
fn free_near(taken: &BTreeSet<Cell>, near: Cell) -> Cell {
    for ring in 1..=RINGS {
        for y in -ring..=ring {
            for x in -ring..=ring {
                let cell = (near.0 + x, near.1 + y);
                if x.abs().max(y.abs()) == ring && !taken.contains(&cell) {
                    return cell;
                }
            }
        }
    }
    (near.0 + RINGS + 1, near.1)
}

/// Positions for the places in `order` (the order they were found), laid
/// out around `anchor`, which is put at `at` (research R1).
pub(crate) fn around(order: &[u16], ways: &[Way], anchor: u16, at: Cell) -> BTreeMap<u16, Cell> {
    let mut put: BTreeMap<u16, Cell> = [(anchor, at)].into();
    let mut taken: BTreeSet<Cell> = [at].into();
    let mut ring = vec![anchor];
    while !ring.is_empty() {
        let mut next = Vec::new();
        for from in ring {
            let here = put[&from];
            for (delta, other) in neighbours(ways, from) {
                if put.contains_key(&other) || !order.contains(&other) {
                    continue;
                }
                let free = (1..=REACH)
                    .map(|k| (here.0 + delta.0 * k, here.1 + delta.1 * k))
                    .find(|cell| !taken.contains(cell));
                // With no room along the way it is left for another place
                // that joins it, or for the end.
                if let Some(cell) = free {
                    put.insert(other, cell);
                    taken.insert(cell);
                    next.push(other);
                }
            }
        }
        ring = next;
    }
    for &place in order {
        if let std::collections::btree_map::Entry::Vacant(slot) = put.entry(place) {
            let cell = free_near(&taken, at);
            slot.insert(cell);
            taken.insert(cell);
        }
    }
    put
}

/// The places the anchor is joined to by a way either way, with the anchor:
/// what a redraw holds still while it tidies the rest.
pub(crate) fn near(ways: &[Way], anchor: u16) -> BTreeSet<u16> {
    let mut held: BTreeSet<u16> = [anchor].into();
    held.extend(neighbours(ways, anchor).into_iter().map(|(_, other)| other));
    held
}

/// Moves the places not in `held` to leave fewer joins bent, then fewer
/// crossing, then shorter joins: each in turn is tried at every position
/// within the sheet's bounds and one step beyond, changing places with any
/// that is there and not held; the best strictly better layout is kept; and
/// so again until a pass changes nothing or `budget` layouts have been
/// tried (research R2). Returns how many were tried.
pub(crate) fn tidy(
    at: &mut BTreeMap<u16, Cell>,
    order: &[u16],
    ways: &[Way],
    held: &BTreeSet<u16>,
    budget: usize,
) -> usize {
    let mut best = measure(at, ways);
    let mut spent = 0;
    loop {
        let mut moved = false;
        for &place in order {
            if held.contains(&place) || !at.contains_key(&place) {
                continue;
            }
            let (xs, ys): (Vec<i32>, Vec<i32>) = at.values().copied().unzip();
            let (left, right) = (xs.iter().min().unwrap() - 1, xs.iter().max().unwrap() + 1);
            let (top, bottom) = (ys.iter().min().unwrap() - 1, ys.iter().max().unwrap() + 1);
            let here = at[&place];
            let mut choice: Option<(Cell, Option<u16>)> = None;
            for y in top..=bottom {
                for x in left..=right {
                    let cell = (x, y);
                    if cell == here {
                        continue;
                    }
                    let holder = at
                        .iter()
                        .find(|(_, other)| **other == cell)
                        .map(|(id, _)| *id);
                    if holder.is_some_and(|holder| held.contains(&holder)) {
                        continue;
                    }
                    if spent >= budget {
                        break;
                    }
                    spent += 1;
                    at.insert(place, cell);
                    if let Some(holder) = holder {
                        at.insert(holder, here);
                    }
                    let now = measure(at, ways);
                    if now < best {
                        best = now;
                        choice = Some((cell, holder));
                    }
                    at.insert(place, here);
                    if let Some(holder) = holder {
                        at.insert(holder, cell);
                    }
                }
            }
            if let Some((cell, holder)) = choice {
                at.insert(place, cell);
                if let Some(holder) = holder {
                    at.insert(holder, here);
                }
                moved = true;
            }
        }
        if !moved || spent >= budget {
            return spent;
        }
    }
}
