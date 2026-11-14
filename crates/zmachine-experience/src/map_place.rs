//! Where places go on a sheet of the map
//! (specs/012-map-first-slice/contracts/map.md, part P).
//!
//! A sheet is a sparse grid of whole-numbered positions, x to the right and y
//! downward, so that north is up the page. A place is put once and left; these
//! functions only ever answer with a position that is free.

use crate::Place;
use std::collections::BTreeMap;

/// A position on a sheet.
pub(crate) type Cell = (i32, i32);

/// One page of the map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Sheet {
    /// Which place is at each position. No two places share one (P§7).
    pub cells: BTreeMap<Cell, u16>,
    /// The places attached to others, on a grid of their own that lies half a
    /// step off the first: inset (i, j) is centred half a step right of and
    /// below position (i, j), in the gap where four positions meet. A place
    /// drawn small there touches none of the four, so attached places take
    /// no position from the places that are walked to (P§3).
    pub insets: BTreeMap<Cell, u16>,
    /// Whether the sheet was started by a move that was not along a way
    /// (P§6).
    pub by_word: bool,
}

/// The step on the page for one of the eight compass ways; none for the rest.
pub(crate) fn delta(place: Place) -> Option<Cell> {
    Some(match place {
        Place::North => (0, -1),
        Place::Northeast => (1, -1),
        Place::East => (1, 0),
        Place::Southeast => (1, 1),
        Place::South => (0, 1),
        Place::Southwest => (-1, 1),
        Place::West => (-1, 0),
        Place::Northwest => (-1, -1),
        Place::Up | Place::Down | Place::In | Place::Out => return None,
    })
}

/// How far out the search for a free position goes. A sheet holds tens of
/// places; a ring this wide always has room.
const RINGS: i32 = 64;

/// The free position nearest to `ideal` that `allowed` admits: `ideal`
/// itself, or else the best of the nearest ring that has any. Within a ring
/// the highest `score` wins; ties go to the lowest y, then the lowest x.
fn nearest(
    sheet: &Sheet,
    ideal: Cell,
    score: impl Fn(Cell) -> i32,
    allowed: impl Fn(Cell) -> bool,
) -> Cell {
    let usable = |cell: Cell| !sheet.cells.contains_key(&cell) && allowed(cell);
    if usable(ideal) {
        return ideal;
    }
    for ring in 1..=RINGS {
        let mut best: Option<(i32, Cell)> = None;
        // Rows from the top, each from the left, so that the first of equal
        // score is the one with the lowest y and then the lowest x.
        for oy in -ring..=ring {
            for ox in -ring..=ring {
                if ox.abs().max(oy.abs()) != ring {
                    continue;
                }
                let cell = (ideal.0 + ox, ideal.1 + oy);
                if usable(cell) && best.is_none_or(|(s, _)| score(cell) > s) {
                    best = Some((score(cell), cell));
                }
            }
        }
        if let Some((_, cell)) = best {
            return cell;
        }
    }
    // Not reached: no sheet fills sixty-four rings.
    (ideal.0 + RINGS + 1, ideal.1)
}

/// For a new place reached by a compass way `d` from `from` (P§2): one, two
/// or three steps along it, the first that is free; else the nearest free
/// position to one step, preferring the side the way points to.
pub(crate) fn along(sheet: &Sheet, from: Cell, d: Cell) -> Cell {
    for k in 1..=3 {
        let cell = (from.0 + d.0 * k, from.1 + d.1 * k);
        if !sheet.cells.contains_key(&cell) {
            return cell;
        }
    }
    let side = |cell: Cell| i32::from((cell.0 - from.0) * d.0 + (cell.1 - from.1) * d.1 > 0);
    nearest(sheet, (from.0 + d.0, from.1 + d.1), side, |_| true)
}

/// For a new place reached by going up or down from `from` (P§3): the nearest
/// free position to above-right (up) or below-left (down) that is not on any
/// of `rays`, the compass ways the place left has.
pub(crate) fn ladder(sheet: &Sheet, from: Cell, up: bool, rays: &[Cell]) -> Cell {
    let ideal = if up {
        (from.0 + 1, from.1 - 1)
    } else {
        (from.0 - 1, from.1 + 1)
    };
    let on_a_ray = |cell: Cell| {
        let (vx, vy) = (cell.0 - from.0, cell.1 - from.1);
        rays.iter().any(|&(dx, dy)| {
            let k = vx.abs().max(vy.abs());
            k > 0 && vx == dx * k && vy == dy * k
        })
    };
    nearest(sheet, ideal, |_| 0, |cell| !on_a_ray(cell))
}

/// For a new place reached by going up or down from a place at `parent`
/// that is not itself attached (P§3): one of the four insets at its corners.
/// Up prefers the corner above-right, then above-left; down the corner
/// below-left, then below-right; then the other two. A corner is passed over
/// if it is taken, or if the place has a compass way toward it (one of
/// `diagonals`). With no corner to be had, the nearest free inset.
pub(crate) fn corner(sheet: &Sheet, parent: Cell, up: bool, diagonals: &[Cell]) -> Cell {
    let (x, y) = parent;
    // Each corner as a diagonal step, and the inset that lies there.
    let ne = ((1, -1), (x, y - 1));
    let nw = ((-1, -1), (x - 1, y - 1));
    let sw = ((-1, 1), (x - 1, y));
    let se = ((1, 1), (x, y));
    let order = if up {
        [ne, nw, sw, se]
    } else {
        [sw, se, ne, nw]
    };
    for (way, inset) in order {
        if !diagonals.contains(&way) && !sheet.insets.contains_key(&inset) {
            return inset;
        }
    }
    let taken = Sheet {
        cells: sheet.insets.clone(),
        ..Sheet::default()
    };
    nearest(&taken, order[0].1, |_| 0, |_| true)
}

/// For a new place reached by going in or out from `from` (P§5): the nearest
/// free position.
pub(crate) fn beside(sheet: &Sheet, from: Cell) -> Cell {
    nearest(sheet, from, |_| 0, |_| true)
}

/// For a new place reached by a word from a place itself reached by a word
/// (P§6): the nearest free position to one step below.
pub(crate) fn below(sheet: &Sheet, from: Cell) -> Cell {
    nearest(sheet, (from.0, from.1 + 1), |_| 0, |_| true)
}

/// Whether `to` lies exactly along `d` from `from`.
fn exactly_along(from: Cell, d: Cell, to: Cell) -> bool {
    let (vx, vy) = (to.0 - from.0, to.1 - from.1);
    let k = vx.abs().max(vy.abs());
    k > 0 && vx == d.0 * k && vy == d.1 * k
}

/// Whether the line from `a` by the way `out` to `b`, walked back (if it was)
/// by `back`, is straight (P§8): `b` is exactly along `out` from `a`, `a`
/// exactly along `back` from `b`, and no place lies between.
pub(crate) fn straight(sheet: &Sheet, a: Cell, out: Cell, b: Cell, back: Option<Cell>) -> bool {
    if !exactly_along(a, out, b) || back.is_some_and(|back| !exactly_along(b, back, a)) {
        return false;
    }
    let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs());
    (1..steps).all(|k| {
        !sheet
            .cells
            .contains_key(&(a.0 + out.0 * k, a.1 + out.1 * k))
    })
}
