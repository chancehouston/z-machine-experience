//! Tests of look-ahead on stories built in memory
//! (specs/009-compass-lookahead/contracts/lookahead.md, cited L§).

use crate::lookahead::{COPIES, Finding, LIMITS, Limits, LookAhead, Outcome, chance};
use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_introspect::Introspector;
use zmachine_testkit::toy::{ASK, COIN, DOWN, EAST, NORTH, QUIT, SOUTH, SPIN, STAY, Toy, UP, WEST};

/// A toy whose first room has six ways the table calls conditional: north is
/// allowed, south refused, east decided by an even draw, west ends the game,
/// up asks a question and down never stops.
pub(crate) fn toy() -> Toy {
    let mut toy = Toy::new(3);
    let there = Toy::room(1) as u8;
    toy.uncertain(0, NORTH, there)
        .uncertain(0, SOUTH, STAY)
        .uncertain(0, EAST, COIN)
        .uncertain(0, WEST, QUIT)
        .uncertain(0, UP, ASK)
        .uncertain(0, DOWN, SPIN);
    // A way back, and a plain way on, so that the player can be walked about.
    toy.way(1, SOUTH, 0).way(1, NORTH, 2).way(2, SOUTH, 1);
    toy
}

/// The toy at its first prompt, with what look-ahead needs.
pub(crate) fn at_prompt(image: Vec<u8>, seed: u32) -> (Machine, Introspector, u32) {
    let options = LoadOptions {
        seed: Some(seed),
        entropy: 0,
    };
    let mut machine = Machine::load(image, options).unwrap();
    assert!(matches!(
        machine.run(1_000_000).unwrap(),
        Event::LineWanted { .. }
    ));
    machine.take_output();
    let introspector = Introspector::new(&machine.view());
    let address = machine.waiting_at().unwrap();
    (machine, introspector, address)
}

fn step(machine: &mut Machine, line: &str) {
    machine.provide_line(line).unwrap();
    assert!(matches!(
        machine.run(1_000_000).unwrap(),
        Event::LineWanted { .. }
    ));
    machine.take_output();
}

// The fixture is what the tests need: the interface recognises it and reports
// each of the six ways as conditional (tasks.md T003).
#[test]
fn the_fixture_has_six_conditional_ways() {
    use zmachine_introspect::ExitKind;
    let (machine, introspector, _) = at_prompt(toy().build(), 1);
    let exits = introspector.exits(&machine.view()).value.unwrap();
    assert_eq!(exits.exits.len(), 6);
    let kinds: Vec<_> = exits
        .exits
        .iter()
        .map(|e| (e.direction, e.kind, e.destination))
        .collect();
    assert!(
        exits.exits.iter().all(|e| e.kind == ExitKind::Conditional),
        "{kinds:?}"
    );
}

// L§4, 6: a copy given the allowed way's word walked to that room; given the
// refused way's word, stayed.
#[test]
fn one_copy_walks_or_stays() {
    let (machine, introspector, address) = at_prompt(toy().build(), 1);
    let mut look = LookAhead::new(&machine, &introspector, address, 0, LIMITS);
    assert_eq!(look.copy("north", None).0, Outcome::Walked(Toy::room(1)));
    assert_eq!(look.copy("south", None).0, Outcome::Stayed);
}

// L§7: three copies that agree make a finding.
#[test]
fn finding_is_what_all_three_copies_did() {
    let (machine, introspector, address) = at_prompt(toy().build(), 1);
    let mut look = LookAhead::new(&machine, &introspector, address, 0, LIMITS);
    assert_eq!(
        look.finding("north"),
        Finding::Walked {
            room: Toy::room(1),
            name: Some("Room 1".to_string()),
            dark: false,
        }
    );
    assert_eq!(look.finding("south"), Finding::Refused);
}

// L§5: the values given to copies 1 and 2 differ, depend only on the count of
// commands and the copy's number, and are not the small values a game uses for
// its predictable mode.
#[test]
fn finding_gives_the_copies_other_chance() {
    assert_eq!(COPIES, 3);
    for sent in 0..500u64 {
        let (one, two) = (chance(sent, 1), chance(sent, 2));
        assert_ne!(one, two, "{sent}");
        assert_eq!((one, two), (chance(sent, 1), chance(sent, 2)));
        assert!(one >= 1_000 && two >= 1_000 && one <= 0x7FFF_FFFF && two <= 0x7FFF_FFFF);
    }
    let distinct: std::collections::BTreeSet<u32> = (0..500).map(|s| chance(s, 1)).collect();
    assert!(distinct.len() > 490, "{}", distinct.len());
}

/// The next twenty draws of a machine's generator, read off a copy by sending
/// it the refused way twenty times and reading the toy's own draw each turn
/// (it stores `random 100` in a global at every command).
fn next_draws(machine: &Machine) -> Vec<u16> {
    let mut copy = machine.clone();
    (0..20)
        .map(|_| {
            step(&mut copy, "south");
            let view = copy.view();
            let globals = view.word(0x0C).unwrap() as usize;
            view.word(globals + 2 * 23).unwrap()
        })
        .collect()
}

// L§8, 9: after findings for every way, the player's machine is what it was,
// to its next draws of chance.
#[test]
fn untouched_by_any_number_of_findings() {
    let (mut machine, introspector, address) = at_prompt(toy().build(), 5);
    step(&mut machine, "south");
    let before = format!("{machine:?}");
    let draws = next_draws(&machine);
    let mut look = LookAhead::new(&machine, &introspector, address, 1, LIMITS);
    for way in ["north", "south", "east", "west", "up", "down"] {
        look.finding(way);
    }
    assert!(look.spent() > 0);
    assert_eq!(format!("{machine:?}"), before);
    assert_eq!(next_draws(&machine), draws);
    assert_eq!(machine.clone().take_output(), "");
}

// L§7, FR-008; research R2: a way decided by an even draw is seldom settled,
// and when it is, the finding is what the game's own draw gives; a way that is
// always allowed is always settled.
#[test]
fn chance_keeps_its_question_mark() {
    let (mut machine, introspector, address) = at_prompt(toy().build(), 9);
    let (mut settled, mut sure) = (0, 0);
    for sent in 0..200u64 {
        let mut look = LookAhead::new(&machine, &introspector, address, sent, LIMITS);
        let own = look.copy("east", None).0;
        match look.finding("east") {
            Finding::None => {}
            Finding::Refused => {
                settled += 1;
                assert_eq!(own, Outcome::Stayed, "{sent}");
            }
            Finding::Walked { room, .. } => {
                settled += 1;
                assert_eq!(own, Outcome::Walked(room), "{sent}");
            }
        }
        if look.finding("north") != Finding::None {
            sure += 1;
        }
        // The refused way moves the game on by one command, and one draw.
        step(&mut machine, "south");
    }
    assert_eq!(sure, 200);
    assert!(
        settled < 67,
        "an even draw was settled {settled} times of 200"
    );
    assert!(
        settled > 20,
        "an even draw was settled {settled} times of 200"
    );
}

// L§6, 18; FR-007, FR-013: a way that ends the game, one that asks, and one
// that never stops have no finding.
#[test]
fn harm_has_no_finding() {
    let (machine, introspector, address) = at_prompt(toy().build(), 1);
    let limits = Limits {
        copy: 50_000,
        total: u64::MAX,
    };
    let mut look = LookAhead::new(&machine, &introspector, address, 0, limits);
    for way in ["west", "up", "down"] {
        assert_eq!(look.copy(way, None).0, Outcome::Nothing, "{way}");
        assert_eq!(look.finding(way), Finding::None, "{way}");
    }
    // The copy that never stops was cut off at its limit, and no later.
    assert_eq!(look.most(), 50_000);
}

// L§19: with the total spent, a way not yet begun has no finding.
#[test]
fn limit_on_the_total_leaves_later_ways() {
    let (machine, introspector, address) = at_prompt(toy().build(), 1);
    let limits = Limits {
        copy: 50_000,
        total: 4_000,
    };
    let mut look = LookAhead::new(&machine, &introspector, address, 0, limits);
    assert_ne!(look.finding("north"), Finding::None);
    assert!(look.spent() < 4_000, "{}", look.spent());
    assert_ne!(look.finding("south"), Finding::None);
    assert!(look.spent() >= 4_000, "{}", look.spent());
    assert_eq!(look.finding("north"), Finding::None);
}
