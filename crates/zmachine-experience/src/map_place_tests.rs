//! Where places go on a sheet
//! (specs/012-map-first-slice/contracts/map.md, part P, cited P§).

use crate::Place;
use crate::map_place::{Cell, Sheet, along, below, beside, corner, delta, ladder, straight};

fn sheet(taken: &[Cell]) -> Sheet {
    let mut sheet = Sheet::default();
    for (n, cell) in taken.iter().enumerate() {
        sheet.cells.insert(*cell, n as u16 + 1);
    }
    sheet
}

// The page has x to the right and y downward; north is up the page.
#[test]
fn the_eight_directions_are_steps_on_the_page() {
    let steps: Vec<Option<Cell>> = Place::ALL.iter().map(|p| delta(*p)).collect();
    assert_eq!(
        steps,
        [
            Some((0, -1)),
            Some((1, -1)),
            Some((1, 0)),
            Some((1, 1)),
            Some((0, 1)),
            Some((-1, 1)),
            Some((-1, 0)),
            Some((-1, -1)),
            None,
            None,
            None,
            None
        ]
    );
}

// P§2: one step along the direction when that is free.
#[test]
fn a_new_place_goes_one_step_along_the_way() {
    let s = sheet(&[(0, 0)]);
    for place in &Place::ALL[..8] {
        let d = delta(*place).unwrap();
        assert_eq!(along(&s, (0, 0), d), d, "{place:?}");
    }
    assert_eq!(along(&sheet(&[(5, 5)]), (5, 5), (1, 0)), (6, 5));
}

// P§2: two steps, then three, when the nearer are taken.
#[test]
fn it_goes_further_along_when_the_near_place_is_taken() {
    assert_eq!(along(&sheet(&[(0, 0), (1, 0)]), (0, 0), (1, 0)), (2, 0));
    assert_eq!(
        along(&sheet(&[(0, 0), (1, 0), (2, 0)]), (0, 0), (1, 0)),
        (3, 0)
    );
    assert_eq!(
        along(&sheet(&[(0, 0), (-1, -1)]), (0, 0), (-1, -1)),
        (-2, -2)
    );
    // A place two steps along does not stop one step being used.
    assert_eq!(along(&sheet(&[(0, 0), (2, 0)]), (0, 0), (1, 0)), (1, 0));
}

// P§2: with all three taken, the nearest free place to one step, on the side
// the direction points to; ties by lowest y, then lowest x.
#[test]
fn with_three_taken_it_goes_to_the_nearest_free_place_on_that_side() {
    let east = [(0, 0), (1, 0), (2, 0), (3, 0)];
    // Around (1, 0): the ring's free places on the east side of (0, 0) are
    // (1, -1), (2, -1), (1, 1), (2, 1). The lowest y, then the lowest x.
    assert_eq!(along(&sheet(&east), (0, 0), (1, 0)), (1, -1));
    let mut more = east.to_vec();
    more.extend([(1, -1), (2, -1)]);
    assert_eq!(along(&sheet(&more), (0, 0), (1, 0)), (1, 1));
    // With the whole east side of the first ring taken, a place on the other
    // side of that ring is nearer than any on the next, and is used.
    more.extend([(1, 1), (2, 1)]);
    assert_eq!(along(&sheet(&more), (0, 0), (1, 0)), (0, -1));
}

// P§7: whatever is asked, the place given is free.
#[test]
fn the_place_given_is_always_free() {
    // A seeded walk that fills a sheet in every direction.
    let mut s = sheet(&[(0, 0)]);
    let mut at = (0, 0);
    let mut state = 12345u32;
    for n in 0..300u16 {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let choice = (state >> 16) % 11;
        let cell = match choice {
            0..=7 => along(&s, at, delta(Place::ALL[choice as usize]).unwrap()),
            8 => ladder(&s, at, true, &[(1, -1), (0, -1)]),
            9 => beside(&s, at),
            _ => below(&s, at),
        };
        assert!(!s.cells.contains_key(&cell), "{cell:?} at step {n}");
        s.cells.insert(cell, n + 2);
        if !state.is_multiple_of(3) {
            at = cell;
        }
    }
    assert_eq!(s.cells.len(), 301);
}

// P§3: up goes above and to the right, down below and to the left.
#[test]
fn up_and_down_go_beside_the_place_left() {
    let s = sheet(&[(0, 0)]);
    assert_eq!(ladder(&s, (0, 0), true, &[]), (1, -1));
    assert_eq!(ladder(&s, (0, 0), false, &[]), (-1, 1));
}

// P§3: never on a compass way the place has, at any distance along it.
#[test]
fn a_ladder_is_not_put_on_a_compass_way_of_the_place() {
    let s = sheet(&[(0, 0)]);
    // The place has a way northeast: up goes to the nearest place off that
    // ray. Of the ring around (1, -1), lowest y then lowest x: (0, -2).
    assert_eq!(ladder(&s, (0, 0), true, &[(1, -1)]), (0, -2));
    // With north as well, (0, -2) lies on a ray too.
    assert_eq!(ladder(&s, (0, 0), true, &[(1, -1), (0, -1)]), (1, -2));
    let got = ladder(&s, (0, 0), false, &[(-1, 1), (0, 1), (-1, 0)]);
    assert!(
        ![(-1, 1), (-2, 2), (0, 1), (0, 2), (-1, 0), (-2, 0)].contains(&got),
        "{got:?}"
    );
}

// P§3: the nearest free place when the first choice is taken.
#[test]
fn a_ladder_takes_the_nearest_free_place() {
    let s = sheet(&[(0, 0), (1, -1)]);
    assert_eq!(ladder(&s, (0, 0), true, &[]), (0, -2));
}

// P§5: in and out go to the nearest free place.
#[test]
fn in_and_out_go_beside() {
    assert_eq!(beside(&sheet(&[(0, 0)]), (0, 0)), (-1, -1));
    assert_eq!(
        beside(&sheet(&[(0, 0), (-1, -1), (0, -1)]), (0, 0)),
        (1, -1)
    );
}

// P§6: a run of moves by a word goes on downward.
#[test]
fn a_move_by_a_word_goes_on_below() {
    assert_eq!(below(&sheet(&[(0, 0)]), (0, 0)), (0, 1));
    assert_eq!(below(&sheet(&[(0, 0), (0, 1)]), (0, 0)), (-1, 0));
}

// P§8: straight when the far place is exactly along the way.
#[test]
fn a_line_is_straight_along_its_way() {
    let s = sheet(&[(0, 0), (1, 0), (3, 0), (2, 2), (2, 1)]);
    assert!(straight(&s, (0, 0), (1, 0), (1, 0), None));
    assert!(straight(&s, (0, 0), (1, 0), (1, 0), Some((-1, 0))));
    // Along a diagonal only when the steps are equal.
    assert!(straight(&s, (0, 0), (1, 1), (2, 2), None));
    assert!(!straight(&s, (0, 0), (1, 1), (2, 1), None));
}

// P§8: bent when off the way, when the way back does not agree, or when a
// place lies between.
#[test]
fn a_line_is_bent_otherwise() {
    let s = sheet(&[(0, 0), (1, 0), (3, 0), (0, 1)]);
    // Off the way: east, to a place that is south.
    assert!(!straight(&s, (0, 0), (1, 0), (0, 1), None));
    // Behind: east, to a place that is west of it.
    assert!(!straight(&s, (1, 0), (1, 0), (0, 0), None));
    // The way back does not agree: out east, back north.
    assert!(!straight(&s, (0, 0), (1, 0), (1, 0), Some((0, -1))));
    // A place lies between: (1, 0) is between (0, 0) and (3, 0).
    assert!(!straight(&s, (0, 0), (1, 0), (3, 0), None));
    // To itself.
    assert!(!straight(&s, (0, 0), (1, 0), (0, 0), None));
}

// P§3: an attached place goes in the gap at a corner of the place it was
// reached from: above-right for up, below-left for down.
#[test]
fn an_attached_place_goes_at_a_corner() {
    let s = sheet(&[(0, 0)]);
    assert_eq!(corner(&s, (0, 0), true, &[]), (0, -1));
    assert_eq!(corner(&s, (0, 0), false, &[]), (-1, 0));
    assert_eq!(corner(&s, (5, 3), true, &[]), (5, 2));
    assert_eq!(corner(&s, (5, 3), false, &[]), (4, 3));
}

// P§3: a corner is passed over if the place has a compass way toward it, or
// another attached place is there.
#[test]
fn a_corner_with_a_way_or_a_place_is_passed_over() {
    let mut s = sheet(&[(0, 0)]);
    // A way northeast: up goes above-left.
    assert_eq!(corner(&s, (0, 0), true, &[(1, -1)]), (-1, -1));
    // And northwest: below-left; and southwest: below-right.
    assert_eq!(corner(&s, (0, 0), true, &[(1, -1), (-1, -1)]), (-1, 0));
    assert_eq!(
        corner(&s, (0, 0), true, &[(1, -1), (-1, -1), (-1, 1)]),
        (0, 0)
    );
    // Down, with below-left taken by another attached place.
    s.insets.insert((-1, 0), 9);
    assert_eq!(corner(&s, (0, 0), false, &[]), (0, 0));
    // The place to the east shares two corners with this one: its up goes
    // to its own above-right, and its down, with below-left free, there.
    assert_eq!(corner(&s, (1, 0), true, &[]), (1, -1));
    assert_eq!(corner(&s, (1, 0), false, &[]), (0, 0));
    s.insets.insert((0, 0), 8);
    assert_eq!(corner(&s, (1, 0), false, &[]), (1, 0));
}

// P§3: with all four corners gone, the nearest free inset.
#[test]
fn with_no_corner_free_the_nearest_inset_is_used() {
    let mut s = sheet(&[(0, 0)]);
    for (n, inset) in [(0, -1), (-1, -1), (-1, 0), (0, 0)].into_iter().enumerate() {
        s.insets.insert(inset, n as u16 + 20);
    }
    let got = corner(&s, (0, 0), true, &[]);
    assert!(!s.insets.contains_key(&got));
    assert_eq!(got, (-1, -2));
}
