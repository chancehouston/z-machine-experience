//! Laying a sheet out around one place, on places and ways made by hand
//! (specs/016-map-untangle/contracts/untangle.md, cited R§).

use crate::map_redraw::{Way, around, measure, tidy};
use std::collections::{BTreeMap, BTreeSet};

const N: (i32, i32) = (0, -1);
const NE: (i32, i32) = (1, -1);
const E: (i32, i32) = (1, 0);
const S: (i32, i32) = (0, 1);
const W: (i32, i32) = (-1, 0);

/// A way walked from `from` to `to` in the direction `delta`. Its rank is
/// the direction's place in the compass's order.
fn way(from: u16, delta: (i32, i32), to: u16) -> Way {
    let rank = [N, NE, E, (1, 1), S, (-1, 1), W, (-1, -1)]
        .iter()
        .position(|d| *d == delta)
        .unwrap();
    Way {
        from,
        delta,
        rank,
        to,
    }
}

// R§1, R§2: the anchor keeps its position, and each place a way was walked
// out to is exactly along that way.
#[test]
fn the_places_joined_to_the_anchor_are_along_their_ways() {
    let order = [1, 2, 3, 4];
    let ways = [way(1, N, 2), way(1, E, 3), way(1, S, 4)];
    let at = around(&order, &ways, 1, (5, 7));
    assert_eq!(at[&1], (5, 7));
    assert_eq!((at[&2], at[&3], at[&4]), ((5, 6), (6, 7), (5, 8)));
}

// R§2: where two ways lead to one place, the first in the compass's order
// decides; a position taken is passed over for the next along the way.
#[test]
fn the_first_way_decides_and_a_taken_position_is_passed() {
    let order = [1, 2, 3];
    // North and north-east both lead to 2; east leads to 3.
    let ways = [way(1, NE, 2), way(1, N, 2), way(1, E, 3)];
    let at = around(&order, &ways, 1, (0, 0));
    assert_eq!(at[&2], (0, -1), "north comes before north-east");
    // 4 was walked to north from 1 as well, by a way that varies: it goes
    // one further.
    let order = [1, 2, 4];
    let ways = [way(1, N, 2), way(1, N, 4)];
    let at = around(&order, &ways, 1, (0, 0));
    assert_eq!((at[&2], at[&4]), ((0, -1), (0, -2)));
}

// R§3: a place joined only by a way walked into the anchor is put opposite
// to that way; and the rings go outward from there.
#[test]
fn a_way_walked_in_puts_its_place_opposite_and_rings_go_outward() {
    let order = [1, 2, 3, 4];
    // 2 was only ever left southward into 1; 3 lies east of 2; 4 joins nothing.
    let ways = [way(2, S, 1), way(2, E, 3)];
    let at = around(&order, &ways, 1, (0, 0));
    assert_eq!(at[&2], (0, -1));
    assert_eq!(at[&3], (1, -1));
    // What no ring reaches is put somewhere free.
    let places: BTreeSet<(i32, i32)> = at.values().copied().collect();
    assert_eq!(places.len(), 4, "{at:?}");
}

// What is counted: a join is straight when a way walked along it lies along
// the page with no place between.
#[test]
fn joins_are_counted_as_bent_and_crossing() {
    let ways = [way(1, E, 2), way(3, S, 4)];
    let mut at: BTreeMap<u16, (i32, i32)> =
        [(1, (0, 0)), (2, (2, 0)), (3, (1, -1)), (4, (1, 1))].into();
    let m = measure(&at, &ways);
    assert_eq!((m.bent, m.crossings), (0, 1));
    at.insert(2, (2, 1));
    assert_eq!(measure(&at, &ways).bent, 1);
    // A place between makes a way not lie along.
    let ways = [way(1, E, 2)];
    let at: BTreeMap<u16, (i32, i32)> = [(1, (0, 0)), (2, (2, 0)), (3, (1, 0))].into();
    assert_eq!(measure(&at, &ways).bent, 1);
}

// R§4, R§6: with some places held, the others are moved to fewer bent joins
// and then fewer crossings; the held do not move; no two share a position.
#[test]
fn the_rest_are_tidied_and_the_held_stay() {
    let order = [1, 2, 3, 4, 5];
    // 1 and 2 are held. 3 is east of 2 and has been put far off; 4 is south
    // of 3, 5 north of 3.
    let ways = [way(1, E, 2), way(2, E, 3), way(3, S, 4), way(3, N, 5)];
    let mut at: BTreeMap<u16, (i32, i32)> = [
        (1, (0, 0)),
        (2, (1, 0)),
        (3, (0, 2)),
        (4, (2, -1)),
        (5, (3, 3)),
    ]
    .into();
    let held: BTreeSet<u16> = [1, 2].into();
    let before = measure(&at, &ways);
    assert_eq!(before.bent, 3);
    let spent = tidy(&mut at, &order, &ways, &held, 3_000);
    assert!(spent > 0 && spent <= 3_000);
    assert_eq!(
        (at[&1], at[&2]),
        ((0, 0), (1, 0)),
        "the held have not moved"
    );
    let after = measure(&at, &ways);
    assert_eq!(after.bent, 0, "{at:?}");
    assert_eq!(after.crossings, 0);
    let places: BTreeSet<(i32, i32)> = at.values().copied().collect();
    assert_eq!(places.len(), 5);
}

// R§9: the search stops at its budget, and what it leaves is sound and no
// worse than it began.
#[test]
fn the_tidying_stops_at_its_budget() {
    let order = [1, 2, 3, 4, 5];
    let ways = [way(1, E, 2), way(2, E, 3), way(3, S, 4), way(3, N, 5)];
    let start: BTreeMap<u16, (i32, i32)> = [
        (1, (0, 0)),
        (2, (1, 0)),
        (3, (0, 2)),
        (4, (2, -1)),
        (5, (3, 3)),
    ]
    .into();
    let mut at = start.clone();
    let spent = tidy(&mut at, &order, &ways, &[1, 2].into(), 5);
    assert!(spent <= 5, "{spent}");
    assert!(measure(&at, &ways).bent <= measure(&start, &ways).bent);
    let places: BTreeSet<(i32, i32)> = at.values().copied().collect();
    assert_eq!(places.len(), 5);
    // With nothing to mend, nothing moves.
    let mut tidy_already: BTreeMap<u16, (i32, i32)> =
        [(1, (0, 0)), (2, (1, 0)), (3, (2, 0))].into();
    let same = tidy_already.clone();
    tidy(
        &mut tidy_already,
        &[1, 2, 3],
        &[way(1, E, 2), way(2, E, 3)],
        &[1].into(),
        3_000,
    );
    assert_eq!(tidy_already, same);
}
