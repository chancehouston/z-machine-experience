//! The snapshot as JSON (specs/004-first-demo/contracts/experience-api.md
//! "Snapshot JSON"; data-model.md "Snapshot" and its validation rules). Every
//! snapshot is read back by a small strict JSON reader written here.

use crate::{Seed, Session};
use zmachine_testkit::op;
use zmachine_testkit::{Story, zencode};

/// A JSON value, as read back.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum J {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<J>),
    Object(Vec<(String, J)>),
}

impl J {
    pub fn get(&self, key: &str) -> &J {
        match self {
            J::Object(fields) => fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("no field {key:?}")),
            other => panic!("{other:?} is not an object"),
        }
    }
    pub fn keys(&self) -> Vec<&str> {
        match self {
            J::Object(fields) => fields.iter().map(|(k, _)| k.as_str()).collect(),
            other => panic!("{other:?} is not an object"),
        }
    }
    pub fn list(&self) -> &[J] {
        match self {
            J::List(items) => items,
            other => panic!("{other:?} is not a list"),
        }
    }
    pub fn text(&self) -> &str {
        match self {
            J::Text(text) => text,
            other => panic!("{other:?} is not text"),
        }
    }
    pub fn number(&self) -> f64 {
        match self {
            J::Number(n) => *n,
            other => panic!("{other:?} is not a number"),
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&self) -> u8 {
        *self.bytes.get(self.at).expect("unexpected end")
    }
    fn expect(&mut self, byte: u8) {
        assert_eq!(self.peek() as char, byte as char, "at {}", self.at);
        self.at += 1;
    }
    fn literal(&mut self, word: &str, value: J) -> J {
        assert!(
            self.bytes[self.at..].starts_with(word.as_bytes()),
            "at {}",
            self.at
        );
        self.at += word.len();
        value
    }
    fn value(&mut self) -> J {
        match self.peek() {
            b'n' => self.literal("null", J::Null),
            b't' => self.literal("true", J::Bool(true)),
            b'f' => self.literal("false", J::Bool(false)),
            b'"' => J::Text(self.string()),
            b'[' => {
                self.at += 1;
                let mut items = Vec::new();
                if self.peek() == b']' {
                    self.at += 1;
                    return J::List(items);
                }
                loop {
                    items.push(self.value());
                    if self.peek() == b']' {
                        self.at += 1;
                        return J::List(items);
                    }
                    self.expect(b',');
                }
            }
            b'{' => {
                self.at += 1;
                let mut fields: Vec<(String, J)> = Vec::new();
                if self.peek() == b'}' {
                    self.at += 1;
                    return J::Object(fields);
                }
                loop {
                    let key = self.string();
                    assert!(!fields.iter().any(|(k, _)| *k == key), "{key:?} twice");
                    self.expect(b':');
                    fields.push((key, self.value()));
                    if self.peek() == b'}' {
                        self.at += 1;
                        return J::Object(fields);
                    }
                    self.expect(b',');
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = self.at;
                while self.at < self.bytes.len()
                    && matches!(self.bytes[self.at], b'-' | b'0'..=b'9')
                {
                    self.at += 1;
                }
                J::Number(
                    std::str::from_utf8(&self.bytes[start..self.at])
                        .unwrap()
                        .parse()
                        .unwrap(),
                )
            }
            other => panic!("unexpected {:?} at {}", other as char, self.at),
        }
    }
    fn string(&mut self) -> String {
        self.expect(b'"');
        let mut out = Vec::new();
        loop {
            let byte = self.peek();
            self.at += 1;
            match byte {
                b'"' => return String::from_utf8(out).expect("valid UTF-8"),
                b'\\' => {
                    let escape = self.peek();
                    self.at += 1;
                    match escape {
                        b'"' | b'\\' | b'/' => out.push(escape),
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'u' => {
                            let hex =
                                std::str::from_utf8(&self.bytes[self.at..self.at + 4]).unwrap();
                            self.at += 4;
                            let mut buffer = [0; 4];
                            out.extend(
                                char::from_u32(u32::from_str_radix(hex, 16).unwrap())
                                    .unwrap()
                                    .encode_utf8(&mut buffer)
                                    .as_bytes(),
                            );
                        }
                        other => panic!("bad escape \\{}", other as char),
                    }
                }
                0..=0x1F => panic!("a raw control character in a string"),
                _ => out.push(byte),
            }
        }
    }
}

pub(crate) fn read(json: &str) -> J {
    assert!(!json.contains('\n'), "one line");
    let mut reader = Reader {
        bytes: json.as_bytes(),
        at: 0,
    };
    let value = reader.value();
    assert_eq!(reader.at, json.len(), "nothing follows the value");
    value
}

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn game() -> Session {
    let story = std::fs::read(format!("{ROOT}/stories/zork1/zork1.z3")).unwrap();
    Session::new(story, Seed::Fixed(11)).unwrap()
}

fn say(session: &mut Session, line: &str) {
    session.send_line(session.stamp(), line).unwrap();
}

// Contract "Snapshot JSON": the fields, in order, each always present.
#[test]
fn the_snapshot_has_every_field_in_order() {
    let s = game();
    let snapshot = read(&s.snapshot_json(0));
    assert_eq!(
        snapshot.keys(),
        [
            "generation",
            "turn_count",
            "waiting",
            "message",
            "status",
            "turns",
            "compass"
        ]
    );
    assert_eq!(snapshot.get("generation").number(), 1.0);
    assert_eq!(snapshot.get("turn_count").number(), 1.0);
    assert_eq!(snapshot.get("waiting").text(), "command");
    assert_eq!(
        snapshot.get("message"),
        &J::Null,
        "no value is null, never left out"
    );
    let status = snapshot.get("status");
    assert_eq!(
        status.keys(),
        ["place", "score", "moves", "hours", "minutes"]
    );
    assert_eq!(status.get("place").text(), "West of House");
    assert_eq!(
        (status.get("score").number(), status.get("moves").number()),
        (0.0, 0.0)
    );
    assert_eq!(
        (status.get("hours"), status.get("minutes")),
        (&J::Null, &J::Null)
    );
    let turn = &snapshot.get("turns").list()[0];
    assert_eq!(turn.keys(), ["number", "command", "source", "text"]);
    assert_eq!(
        (
            turn.get("number").number(),
            turn.get("command"),
            turn.get("source")
        ),
        (0.0, &J::Null, &J::Null)
    );
    assert!(
        turn.get("text")
            .text()
            .ends_with("There is a small mailbox here.\n\n>")
    );
}

// Contract, data-model "Compass": the twelve places, always, in order, with
// every field; the overall fields; extras.
#[test]
fn the_compass_in_a_snapshot() {
    let s = game();
    let snapshot = read(&s.snapshot_json(0));
    let compass = snapshot.get("compass");
    assert_eq!(
        compass.keys(),
        ["offered", "known", "note", "places", "extras"]
    );
    assert_eq!(
        (
            compass.get("offered"),
            compass.get("known"),
            compass.get("note")
        ),
        (&J::Bool(true), &J::Bool(true), &J::Null)
    );
    let places = compass.get("places").list();
    let ids: Vec<&str> = places.iter().map(|p| p.get("place").text()).collect();
    assert_eq!(
        ids,
        [
            "north",
            "northeast",
            "east",
            "southeast",
            "south",
            "southwest",
            "west",
            "northwest",
            "up",
            "down",
            "in",
            "out"
        ]
    );
    for place in places {
        assert_eq!(
            place.keys(),
            ["place", "label", "state", "command", "leads_to", "spoken"]
        );
    }
    let north = &places[0];
    assert_eq!(north.get("label").text(), "North");
    assert_eq!(north.get("state").text(), "open");
    assert_eq!(north.get("command").text(), "north");
    assert_eq!(north.get("leads_to").text(), "North of House");
    assert_eq!(north.get("spoken").text(), "North, to North of House");
    let east = &places[2];
    assert_eq!(
        (
            east.get("state").text(),
            east.get("command"),
            east.get("leads_to")
        ),
        ("closed", &J::Null, &J::Null)
    );
    assert_eq!(east.get("spoken").text(), "East, no exit");
    assert_eq!(compass.get("extras"), &J::List(vec![]));
}

// Contract: `turns` holds the turns from `since` onward, and `turn_count` the
// total, so that the app is sent only what is new.
#[test]
fn the_snapshot_carries_the_turns_since_the_one_asked_for() {
    let mut s = game();
    say(&mut s, "open mailbox");
    s.tap(s.stamp(), crate::Place::North).unwrap();
    let all = read(&s.snapshot_json(0));
    assert_eq!(all.get("turn_count").number(), 3.0);
    assert_eq!(all.get("turns").list().len(), 3);
    let latest = read(&s.snapshot_json(2));
    let turns = latest.get("turns").list();
    assert_eq!(turns.len(), 1);
    assert_eq!(
        (
            turns[0].get("number").number(),
            turns[0].get("command").text(),
            turns[0].get("source").text()
        ),
        (2.0, "north", "tap")
    );
    assert_eq!(all.get("turns").list()[1].get("source").text(), "typed");
    assert_eq!(latest.get("turn_count").number(), 3.0);
    assert_eq!(
        read(&s.snapshot_json(3)).get("turns"),
        &J::List(vec![]),
        "nothing new"
    );
    assert_eq!(
        read(&s.snapshot_json(usize::MAX)).get("turns"),
        &J::List(vec![])
    );
    // Everything but the turns is the same whatever `since` is.
    assert_eq!(all.get("compass"), latest.get("compass"));
    assert_eq!(all.get("status"), latest.get("status"));
}

// Contract: the waiting states, a fault's message, and the idle compass.
#[test]
fn the_waiting_states_in_a_snapshot() {
    let mut s = game();
    say(&mut s, "quit");
    let snapshot = read(&s.snapshot_json(0));
    assert_eq!(snapshot.get("waiting").text(), "answer");
    let compass = snapshot.get("compass");
    assert_eq!(compass.get("offered"), &J::Bool(false));
    assert!(
        compass
            .get("places")
            .list()
            .iter()
            .all(|p| p.get("state").text() == "idle" && p.get("command") == &J::Null)
    );
    assert_eq!(
        compass.get("places").list()[0].get("spoken").text(),
        "North, not available now"
    );
    say(&mut s, "y");
    assert_eq!(read(&s.snapshot_json(0)).get("waiting").text(), "ended");

    // A story that divides by zero at once.
    let mut story = Story::new();
    story.main(
        &[
            op::op0(op::PRINT),
            zencode("Oh no. "),
            op::op2(op::DIV, op::Arg::S(1), op::Arg::S(0)),
            op::store(16),
        ]
        .concat(),
    );
    let faulted = Session::new(story.build(), Seed::Fixed(1)).unwrap();
    let snapshot = read(&faulted.snapshot_json(0));
    assert_eq!(snapshot.get("waiting").text(), "fault");
    assert!(snapshot.get("message").text().contains("division by zero"));
    assert_eq!(snapshot.get("status"), &J::Null, "no status line was drawn");
    assert_eq!(
        snapshot.get("turns").list()[0].get("text").text(),
        "Oh no. "
    );
}

// Contract: text is escaped as JSON requires and reads back exactly.
#[test]
fn text_reads_back_exactly() {
    let mut s = game();
    for line in ["say \"hello\" \\ there", "caf\u{e9}", "\ttab"] {
        say(&mut s, line);
    }
    let snapshot = read(&s.snapshot_json(0));
    for (turn, read_back) in s.turns().iter().zip(snapshot.get("turns").list()) {
        assert_eq!(read_back.get("text").text(), turn.text);
        match &turn.command {
            Some(command) => assert_eq!(read_back.get("command").text(), command),
            None => assert_eq!(read_back.get("command"), &J::Null),
        }
    }
    assert_eq!(
        snapshot.get("turns").list()[1].get("command").text(),
        "say \"hello\" \\ there"
    );
}

// Data-model "Validation rules", on the real game at many stops.
#[test]
fn snapshots_keep_the_validation_rules() {
    let script =
        std::fs::read_to_string(format!("{ROOT}/stories/zork1/fidelity/walkthrough.txt")).unwrap();
    let mut s = game();
    let mut checked = 0;
    for (n, line) in script.lines().skip(2).enumerate() {
        if n % 16 == 0 {
            let snapshot = read(&s.snapshot_json(n));
            let turns = snapshot.get("turns").list();
            assert_eq!(
                snapshot.get("turn_count").number() as usize,
                turns.last().unwrap().get("number").number() as usize + 1
            );
            assert!(
                turns
                    .windows(2)
                    .all(|pair| pair[0].get("number").number() + 1.0
                        == pair[1].get("number").number())
            );
            let compass = snapshot.get("compass");
            let offered = compass.get("offered") == &J::Bool(true);
            let mut commands = Vec::new();
            for entry in compass
                .get("places")
                .list()
                .iter()
                .chain(compass.get("extras").list())
            {
                let state = entry.get("state").text();
                let lit = state == "open" || state == "maybe";
                assert_eq!(
                    entry.get("command") != &J::Null,
                    lit,
                    "a command exactly when open or maybe"
                );
                assert!(entry.get("leads_to") == &J::Null || state == "open");
                assert_eq!(state == "idle", !offered);
                if lit {
                    commands.push(entry.get("command").text().to_string());
                }
            }
            let count = commands.len();
            commands.sort();
            commands.dedup();
            assert_eq!(commands.len(), count, "no two controls send the same line");
            checked += 1;
        }
        say(&mut s, line);
    }
    assert!(checked >= 20, "{checked}");
}
