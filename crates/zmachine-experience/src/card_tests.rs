//! The peek, on stories built in memory
//! (specs/014-inventory-card/contracts/card.md, L§4).

use crate::card::peek;
use crate::lookahead;
use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story, zencode};

fn print(text: &str) -> Vec<u8> {
    [op::op0(op::PRINT), zencode(text)].concat()
}

/// A story that prints `>`, reads a line, then runs `after`, which is given a
/// second read.
fn story(after: impl FnOnce(Vec<u8>) -> Vec<u8>) -> Vec<u8> {
    let mut story = Story::new();
    story.word("look");
    let mut text = vec![0u8; 64];
    text[0] = 60;
    let text = story.scratch(&text);
    let mut parse = vec![0u8; 22];
    parse[0] = 5;
    let parse = story.scratch(&parse);
    let read = op::var(op::SREAD, &[L(text), L(parse)]);
    // Round again to the first read, whatever follows.
    let body = [print(">"), read.clone(), after(read)].concat();
    let back = -(body.len() as i16 + 3) + 2;
    story.main(&[body, op::op1(op::JUMP, L(back as u16))].concat());
    story.build()
}

/// The story at its first prompt, and that prompt's address.
fn at_prompt(image: Vec<u8>) -> (Machine, u32) {
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut machine = Machine::load(image, options).unwrap();
    assert!(matches!(
        machine.run(1_000_000).unwrap(),
        Event::LineWanted { .. }
    ));
    machine.take_output();
    let address = machine.waiting_at().unwrap();
    (machine, address)
}

const LIMIT: u64 = lookahead::LIMITS.copy;

// L§4: the copy's reply, without the prompt and the white space round it.
#[test]
fn a_peek_gives_what_the_copy_printed() {
    let (machine, address) = at_prompt(story(|_| print("A brass lamp.\n\n")));
    assert_eq!(
        peek(&machine, "examine lamp", address, LIMIT).as_deref(),
        Some("A brass lamp.")
    );
    // The machine itself has run nothing: it still wants its line.
    assert_eq!(machine.waiting_at(), Some(address));
    assert_eq!(
        peek(&machine, "examine lamp", address, LIMIT).as_deref(),
        Some("A brass lamp.")
    );
}

// L§4: a copy that stops anywhere but the command prompt gives nothing.
#[test]
fn a_peek_that_does_not_come_back_gives_nothing() {
    // A question, from another read.
    let (machine, address) = at_prompt(story(|read| [print("Which? >"), read].concat()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // The end of the game.
    let (machine, address) = at_prompt(story(|_| [print("Bye."), op::op0(op::QUIT)].concat()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A fault.
    let (machine, address) = at_prompt(story(|_| {
        [print("Oh. "), op::op2(op::DIV, S(1), S(0)), op::store(16)].concat()
    }));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A copy that never stops: the limit.
    let (machine, address) = at_prompt(story(|_| {
        let nop = op::op0(op::NOP);
        let back = -(nop.len() as i16 + 3) + 2;
        [nop, op::op1(op::JUMP, L(back as u16))].concat()
    }));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A reply of nothing at all describes nothing.
    let (machine, address) = at_prompt(story(|_| Vec::new()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
}

// ---------------------------------------------------------------------------
// Stage 2: what an item is for
// (specs/014-inventory-card/contracts/card.md, L§10 to L§14)
// ---------------------------------------------------------------------------

mod actions {
    use crate::card;
    use crate::strip::{Kind, Strip, StripItem};
    use crate::verbs::Acts;
    use zmachine_introspect::Tri::{self, No, Unknown as Unk, Yes};
    use zmachine_introspect::{Answer, Carried, Source, Uses};

    const NOTHING: Uses = Uses {
        container: No,
        opens: No,
        open: No,
        switch: No,
        on: No,
        read: No,
        eat: No,
        drink: No,
        wear: No,
        light: No,
        lamp: No,
    };

    fn carried(name: &str, uses: Uses, worn: Tri, contents: Vec<Carried>) -> Carried {
        Carried {
            object: 1,
            name: Ok(name.to_string()),
            worn,
            state: None,
            seen: Yes,
            source: Source::File,
            words: Ok(vec![name.to_string()]),
            uses,
            contents,
        }
    }

    fn item(name: &str, uses: Uses) -> StripItem {
        Strip::with(&Answer::exact(vec![carried(name, uses, No, vec![])]), 9)
            .offering(true)
            .items
            .remove(0)
    }

    /// A dictionary with every word an action needs, but for `without`.
    fn acts(without: &[&str]) -> Acts {
        let words: Vec<String> = [
            "drop", "open", "close", "turn", "on", "off", "read", "wear", "remove", "eat", "drink",
        ]
        .iter()
        .filter(|word| !without.contains(word))
        .map(|word| word.to_string())
        .collect();
        Acts::of(&words, 9)
    }

    fn labels(item: &StripItem, acts: &Acts) -> Vec<String> {
        card::actions(item, acts)
            .into_iter()
            .map(|action| format!("{}={}", action.label, action.command))
            .collect()
    }

    // L§10: each pair by its state, and Drop for anything.
    #[test]
    fn an_action_follows_the_items_state() {
        let all = acts(&[]);
        assert_eq!(labels(&item("rock", NOTHING), &all), ["Drop=drop rock"]);
        let closed = Uses {
            opens: Yes,
            container: Yes,
            ..NOTHING
        };
        assert_eq!(
            labels(&item("sack", closed), &all),
            ["Open=open sack", "Drop=drop sack"]
        );
        let open = Uses {
            open: Yes,
            ..closed
        };
        assert_eq!(
            labels(&item("sack", open), &all),
            ["Close=close sack", "Drop=drop sack"]
        );
        let off = Uses {
            switch: Yes,
            lamp: Yes,
            ..NOTHING
        };
        assert_eq!(
            labels(&item("lamp", off), &all),
            ["Turn on=turn on lamp", "Drop=drop lamp"]
        );
        let on = Uses {
            on: Yes,
            light: Yes,
            ..off
        };
        assert_eq!(
            labels(&item("lamp", on), &all),
            ["Turn off=turn off lamp", "Drop=drop lamp"]
        );
    }

    // L§10: the order: open or close, turn on or off, read, wear or take off;
    // then Drop, eat, drink. The safest are nearest the close control.
    #[test]
    fn the_actions_are_in_the_owners_order() {
        let everything = Uses {
            container: Yes,
            opens: Yes,
            open: No,
            switch: Yes,
            on: No,
            read: Yes,
            eat: Yes,
            drink: Yes,
            wear: Yes,
            light: No,
            lamp: Yes,
        };
        let labels: Vec<String> = card::actions(&item("thing", everything), &acts(&[]))
            .into_iter()
            .map(|action| action.label)
            .collect();
        assert_eq!(
            labels,
            ["Open", "Turn on", "Read", "Wear", "Drop", "Eat", "Drink"]
        );
        let spoken: Vec<String> = card::actions(&item("thing", everything), &acts(&[]))
            .into_iter()
            .map(|action| action.spoken)
            .collect();
        assert_eq!(spoken[0], "Open thing");
        assert_eq!(spoken[6], "Drink thing");
        // Worn, it is taken off.
        let strip = Strip::with(
            &Answer::exact(vec![carried(
                "hat",
                Uses {
                    wear: Yes,
                    ..NOTHING
                },
                Yes,
                vec![],
            )]),
            9,
        )
        .offering(true);
        assert_eq!(
            labels_of(&strip.items[0], &acts(&[])),
            ["Take off=remove hat", "Drop=drop hat"]
        );
    }

    fn labels_of(item: &StripItem, acts: &Acts) -> Vec<String> {
        labels(item, acts)
    }

    // L§11: where a part is not known nothing is offered for it; where the
    // state of a pair is not known, neither of the pair is.
    #[test]
    fn what_is_not_known_is_not_offered() {
        let all = acts(&[]);
        assert_eq!(
            labels(&item("thing", Uses::UNKNOWN), &all),
            ["Drop=drop thing"]
        );
        let opens = Uses {
            opens: Yes,
            open: Unk,
            ..NOTHING
        };
        assert_eq!(labels(&item("box", opens), &all), ["Drop=drop box"]);
        let switch = Uses {
            switch: Yes,
            on: Unk,
            ..NOTHING
        };
        assert_eq!(labels(&item("lamp", switch), &all), ["Drop=drop lamp"]);
    }

    // L§10: an action is offered only where every word of its command is in
    // the story's dictionary.
    #[test]
    fn an_action_needs_every_word_of_its_command() {
        let lamp = item(
            "lamp",
            Uses {
                switch: Yes,
                lamp: Yes,
                ..NOTHING
            },
        );
        assert_eq!(labels(&lamp, &acts(&["on"])), ["Drop=drop lamp"]);
        assert_eq!(labels(&lamp, &acts(&["turn"])), ["Drop=drop lamp"]);
        assert_eq!(labels(&lamp, &acts(&["drop"])), ["Turn on=turn on lamp"]);
        let page = item(
            "page",
            Uses {
                read: Yes,
                ..NOTHING
            },
        );
        assert_eq!(labels(&page, &acts(&["read"])), ["Drop=drop page"]);
    }

    // L§14: an item inside another has no action.
    #[test]
    fn an_item_inside_another_has_no_action() {
        let page = carried(
            "page",
            Uses {
                read: Yes,
                ..NOTHING
            },
            No,
            vec![],
        );
        let mut bag = carried(
            "bag",
            Uses {
                container: Yes,
                opens: Yes,
                open: Yes,
                ..NOTHING
            },
            No,
            vec![page],
        );
        bag.state = Some(zmachine_introspect::ContainerState::Open);
        let strip = Strip::with(&Answer::exact(vec![bag]), 9).offering(true);
        let inside = &strip.items[0].contents[0];
        assert!(inside.card);
        assert!(card::actions(inside, &acts(&[])).is_empty());
    }

    // L§13: the kind is the first that applies of lamp, container, page, food,
    // clothing, and else plain; a lamp says whether it is lit.
    #[test]
    fn an_items_kind() {
        let kind = |uses: Uses| {
            let item = item("thing", uses);
            (item.kind, item.lit)
        };
        assert_eq!(kind(NOTHING), (Kind::Plain, None));
        assert_eq!(kind(Uses::UNKNOWN), (Kind::Plain, None));
        assert_eq!(
            kind(Uses {
                lamp: Yes,
                switch: Yes,
                ..NOTHING
            }),
            (Kind::Lamp, Some(false))
        );
        assert_eq!(
            kind(Uses {
                lamp: Yes,
                on: Yes,
                ..NOTHING
            }),
            (Kind::Lamp, Some(true))
        );
        assert_eq!(
            kind(Uses {
                lamp: Yes,
                light: Yes,
                on: Unk,
                ..NOTHING
            }),
            (Kind::Lamp, Some(true))
        );
        assert_eq!(
            kind(Uses {
                lamp: Yes,
                on: Unk,
                light: Unk,
                ..NOTHING
            }),
            (Kind::Lamp, None)
        );
        assert_eq!(
            kind(Uses {
                container: Yes,
                read: Yes,
                ..NOTHING
            }),
            (Kind::Container, None)
        );
        assert_eq!(
            kind(Uses {
                read: Yes,
                eat: Yes,
                ..NOTHING
            }),
            (Kind::Page, None)
        );
        assert_eq!(
            kind(Uses {
                eat: Yes,
                wear: Yes,
                ..NOTHING
            }),
            (Kind::Food, None)
        );
        assert_eq!(
            kind(Uses {
                drink: Yes,
                ..NOTHING
            }),
            (Kind::Food, None)
        );
        assert_eq!(
            kind(Uses {
                wear: Yes,
                ..NOTHING
            }),
            (Kind::Clothing, None)
        );
        assert_eq!(Kind::Lamp.as_str(), "lamp");
        assert_eq!(Kind::Clothing.as_str(), "clothing");
    }
}
