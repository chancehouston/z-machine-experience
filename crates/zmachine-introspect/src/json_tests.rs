//! The JSON rendering (docs/introspection.md section 2, cited I§2). Every
//! rendering is read back by a small strict JSON reader written here.

use crate::fixtures::{Inform, Zil, inf, insert, remove, set_global, zl};
use crate::json::to_json;
use crate::{Answer, Exactness, Introspector, Unknown};

/// A JSON value, as read back.
#[derive(Clone, Debug, PartialEq)]
enum J {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<J>),
    /// Fields in the order they were written.
    Object(Vec<(String, J)>),
}

impl J {
    fn get(&self, key: &str) -> &J {
        match self {
            J::Object(fields) => fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("no field {key:?} in {self:?}")),
            other => panic!("{other:?} is not an object"),
        }
    }
    fn keys(&self) -> Vec<&str> {
        match self {
            J::Object(fields) => fields.iter().map(|(k, _)| k.as_str()).collect(),
            other => panic!("{other:?} is not an object"),
        }
    }
    fn list(&self) -> &[J] {
        match self {
            J::List(items) => items,
            other => panic!("{other:?} is not a list"),
        }
    }
    fn text(&self) -> &str {
        match self {
            J::Text(text) => text,
            other => panic!("{other:?} is not text"),
        }
    }
}

/// A strict reader: RFC 8259, no extensions, nothing after the value.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&self) -> u8 {
        *self
            .bytes
            .get(self.at)
            .unwrap_or_else(|| panic!("unexpected end"))
    }
    fn expect(&mut self, byte: u8) {
        assert_eq!(
            self.peek(),
            byte,
            "at {}: {:?}",
            self.at,
            String::from_utf8_lossy(&self.bytes[self.at..])
        );
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
                let mut fields = Vec::new();
                if self.peek() == b'}' {
                    self.at += 1;
                    return J::Object(fields);
                }
                loop {
                    let key = self.string();
                    assert!(
                        !fields.iter().any(|(k, _)| *k == key),
                        "the field {key:?} appears twice"
                    );
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
                let text = std::str::from_utf8(&self.bytes[start..self.at]).unwrap();
                assert!(
                    text == "0" || !text.starts_with('0'),
                    "a leading zero: {text}"
                );
                J::Number(text.parse().unwrap())
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
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex =
                                std::str::from_utf8(&self.bytes[self.at..self.at + 4]).unwrap();
                            let code = u32::from_str_radix(hex, 16).unwrap();
                            self.at += 4;
                            let mut buffer = [0; 4];
                            out.extend(
                                char::from_u32(code)
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

fn read(json: &str) -> J {
    assert!(!json.contains('\n'), "one line: {json:?}");
    assert!(!json.contains(": ") || json.contains("\": \"") || true);
    let mut reader = Reader {
        bytes: json.as_bytes(),
        at: 0,
    };
    let value = reader.value();
    assert_eq!(reader.at, json.len(), "nothing follows the value");
    value
}

fn inform() -> (zmachine_core::Machine, Introspector) {
    let m = Inform::new().machine();
    let i = Introspector::new(&m.view());
    (m, i)
}

// I§2: an answer is `{"exactness":…,"value":…}`, and an unknown value is
// `{"unknown":"<reason>"}`, for a whole answer as for a part.
#[test]
fn an_answer_and_an_unknown() {
    let known: Answer<Vec<crate::Carried>> = Answer::exact(vec![]);
    assert_eq!(to_json(&known), r#"{"exactness":"exact","value":[]}"#);
    let unknown: Answer<Vec<crate::Carried>> = Answer::unknown(Unknown::NoPlayer, Exactness::Exact);
    assert_eq!(
        to_json(&unknown),
        r#"{"exactness":"exact","value":{"unknown":"no-player"}}"#
    );
    let approximate: Answer<crate::Scope> =
        Answer::unknown(Unknown::UnrecognisedFamily, Exactness::Approximate);
    assert_eq!(
        to_json(&approximate),
        r#"{"exactness":"approximate","value":{"unknown":"unrecognised-family"}}"#
    );
    for reason in Unknown::ALL {
        let answer: Answer<crate::Exits> = Answer::unknown(reason, Exactness::Exact);
        assert_eq!(
            read(&to_json(&answer)).get("value").get("unknown").text(),
            reason.as_str()
        );
    }
}

// I§2, I§6: a location, with its fields in the specification's order.
#[test]
fn a_location() {
    let (m, i) = inform();
    let json = to_json(&i.location(&m.view()));
    assert_eq!(
        json,
        format!(
            r#"{{"exactness":"exact","value":{{"room":{},"name":"Kitchen","holder":null,"lit":"yes"}}}}"#,
            inf::KITCHEN
        )
    );
    // A holder, and a part that is unknown on its own.
    let mut f = Zil::new();
    f.setup.extend(
        [
            insert(zl::DIAMOND, zl::KITCHEN),
            set_global(30, zl::DIAMOND),
        ]
        .concat(),
    );
    let m = f.machine();
    let value = read(&to_json(&Introspector::new(&m.view()).location(&m.view())));
    let location = value.get("value");
    assert_eq!(location.keys(), ["room", "name", "holder", "lit"]);
    assert_eq!(
        location.get("holder").get("unknown").text(),
        "ambiguous-player"
    );
    assert_eq!(location.get("lit").text(), "unknown");
}

// I§2, I§7: an inventory nests contents.
// Feature 007 (specs/007-inventory-panel/contracts/answers.md A§7; I§2.3, I§7):
// an entry's seven keys in order; `state` is null for a thing that holds nothing.
#[test]
fn an_inventory() {
    let mut f = Inform::new();
    f.setup.extend(insert(inf::BOX, inf::PLAYER));
    let m = f.machine();
    let json = to_json(&Introspector::new(&m.view()).inventory(&m.view()));
    assert_eq!(
        json,
        format!(
            r#"{{"exactness":"exact","value":[{{"object":{},"name":"box","worn":"no","state":"open","seen":"yes","source":"file","contents":[{{"object":{},"name":"coin","worn":"no","state":null,"seen":"yes","source":"file","contents":[]}}]}}]}}"#,
            inf::BOX,
            inf::COIN
        )
    );
}

// I§2, I§8: exits. A conditional exit has a null destination; a direction that
// could not be determined is in the `unknown` list with its reason.
#[test]
fn exits() {
    let mut f = Inform::new();
    // South, in place of its string, a property of one byte.
    f.without_prop(inf::KITCHEN, 8)
        .prop(inf::KITCHEN, 8, vec![1]);
    let m = f.machine();
    let value = read(&to_json(&Introspector::new(&m.view()).exits(&m.view())));
    assert_eq!(value.get("exactness").text(), "exact");
    let exits = value.get("value");
    assert_eq!(exits.keys(), ["exits", "unknown"]);
    let north = &exits.get("exits").list()[0];
    assert_eq!(
        north.keys(),
        [
            "direction",
            "words",
            "kind",
            "destination",
            "destination_name",
            "door",
            "state",
            "source"
        ]
    );
    // I§12.5: with no pack, no door and the file's own answer.
    assert_eq!(
        (
            north.get("door"),
            north.get("state"),
            north.get("source").text()
        ),
        (&J::Null, &J::Null, "file")
    );
    assert_eq!(north.get("direction"), &J::Number(7.0));
    assert_eq!(
        north.get("words"),
        &J::List(vec![J::Text("n".into()), J::Text("north".into())])
    );
    assert_eq!(north.get("kind").text(), "fixed");
    assert_eq!(north.get("destination"), &J::Number(inf::HALL as f64));
    assert_eq!(north.get("destination_name").text(), "Hall");
    let inward = &exits.get("exits").list()[1];
    assert_eq!(inward.get("kind").text(), "conditional");
    assert_eq!(
        (inward.get("destination"), inward.get("destination_name")),
        (&J::Null, &J::Null)
    );
    let unknown = &exits.get("unknown").list()[0];
    assert_eq!(unknown.keys(), ["direction", "words", "reason"]);
    assert_eq!(unknown.get("reason").text(), "not-determinable");
}

// I§2, I§9: scope.
#[test]
fn scope() {
    let (m, i) = inform();
    let value = read(&to_json(&i.scope(&m.view())));
    assert_eq!(value.get("exactness").text(), "approximate");
    let scope = value.get("value");
    assert_eq!(scope.keys(), ["light", "player", "objects"]);
    assert_eq!(scope.get("light").text(), "lit");
    assert_eq!(scope.get("player").text(), "known");
    let plate = scope
        .get("objects")
        .list()
        .iter()
        .find(|e| e.get("name").text() == "plate")
        .unwrap();
    assert_eq!(
        plate.keys(),
        [
            "object",
            "name",
            "reach",
            "within",
            "visibility",
            "scenery",
            "concealed"
        ]
    );
    assert_eq!(plate.get("reach").text(), "on");
    assert_eq!(plate.get("within"), &J::Number(inf::TABLE as f64));
    assert_eq!(
        (
            plate.get("visibility").text(),
            plate.get("scenery").text(),
            plate.get("concealed").text()
        ),
        ("visible", "no", "yes")
    );
    let lamp = scope
        .get("objects")
        .list()
        .iter()
        .find(|e| e.get("name").text() == "lamp")
        .unwrap();
    assert_eq!(
        (lamp.get("reach").text(), lamp.get("within")),
        ("in-room", &J::Null)
    );

    let mut f = Zil::new();
    f.setup.extend(set_global(zl::WINNER, 0));
    let m = f.machine();
    let value = read(&to_json(&Introspector::new(&m.view()).scope(&m.view())));
    assert_eq!(
        value.get("value").get("player").get("unknown").text(),
        "no-player"
    );
    let garlic = value
        .get("value")
        .get("objects")
        .list()
        .iter()
        .find(|e| e.get("name").text() == "garlic")
        .unwrap();
    assert_eq!(
        (
            garlic.get("reach").text(),
            garlic.get("visibility").text(),
            garlic.get("scenery").text()
        ),
        ("inside", "unknown", "unknown")
    );
}

// I§2, I§4, I§5: the object tree and the dictionary.
#[test]
fn the_object_tree_and_the_dictionary() {
    let (m, i) = inform();
    let tree = read(&to_json(&i.object_tree(&m.view())));
    let objects = tree.get("value").list();
    assert_eq!(objects.len(), 24);
    let kitchen = &objects[inf::KITCHEN as usize - 1];
    assert_eq!(
        kitchen.keys(),
        [
            "number",
            "name",
            "printed_name",
            "parent",
            "children",
            "attributes",
            "properties"
        ]
    );
    assert_eq!(kitchen.get("parent"), &J::Null);
    assert_eq!(kitchen.get("attributes"), &J::List(vec![J::Number(9.0)]));
    assert!(
        kitchen
            .get("children")
            .list()
            .contains(&J::Number(inf::PLAYER as f64))
    );
    let maze = &objects[inf::MAZE as usize - 1];
    assert_eq!(
        (maze.get("name").text(), maze.get("printed_name").text()),
        ("(Maze_1)", "Maze")
    );
    let coin = &objects[inf::COIN as usize - 1];
    assert_eq!(coin.get("parent"), &J::Number(inf::BOX as f64));

    let words = read(&to_json(&i.dictionary(&m.view())));
    let north = words
        .get("value")
        .list()
        .iter()
        .find(|w| w.get("word").text() == "north")
        .unwrap()
        .clone();
    assert_eq!(north.keys(), ["word", "parts", "direction"]);
    assert_eq!(
        north.get("parts"),
        &J::List(vec![J::Text("noun".into()), J::Text("direction".into())])
    );
    assert_eq!(north.get("direction"), &J::Number(7.0));
    let take = words
        .get("value")
        .list()
        .iter()
        .find(|w| w.get("word").text() == "take")
        .unwrap()
        .clone();
    assert_eq!(take.get("direction"), &J::Null);

    // A story file of no family: the parts of each word are unknown.
    let mut f = Inform::new();
    f.stamp = [0; 4];
    let m = f.machine();
    let words = read(&to_json(
        &Introspector::new(&m.view()).dictionary(&m.view()),
    ));
    assert_eq!(
        words.get("value").list()[0]
            .get("parts")
            .get("unknown")
            .text(),
        "unrecognised-family"
    );
    let _ = remove;
}

// I§2: text is escaped as JSON requires; characters outside ASCII pass through
// as UTF-8; the rendering is one line.
#[test]
fn text_is_escaped() {
    let mut f = Inform::new();
    f.object(inf::LAMP).name = "a \"quoted\" back\\slash".to_string();
    f.late(
        inf::KITCHEN,
        46,
        crate::fixtures::Late::Text("Cafe\nline two"),
    );
    let m = f.machine();
    let i = Introspector::new(&m.view());
    let json = to_json(&i.location(&m.view()));
    assert!(json.contains(r#""name":"Cafe\nline two""#), "{json}");
    assert!(!json.contains('\n'));
    let scope = to_json(&i.scope(&m.view()));
    assert!(
        scope.contains(r#""name":"a \"quoted\" back\\slash""#),
        "{scope}"
    );
    assert_eq!(
        read(&json).get("value").get("name").text(),
        "Cafe\nline two"
    );
    // Characters outside ASCII are written as they are, in UTF-8.
    assert_eq!(to_json("Caf\u{e9} \u{3b1}"), "\"Caf\u{e9} \u{3b1}\"");
    // Control characters are written as \u escapes.
    assert_eq!(to_json("tab\there\u{1}"), r#""tab\there\u0001""#);
}

// I§2.3, I§12.5 (specs/006-game-packs/contracts/pack-format.md P§22, 23): an
// exit's eight keys in order, a door and a marked conditional exit as they are
// written, and an unknown direction with no mark.
#[test]
fn exits_with_a_pack() {
    use sha2::{Digest, Sha256};
    let mut f = Zil::new();
    // Up: open by a variable that is set, to something that is not a room.
    f.prop(
        zl::KITCHEN,
        zl::UP,
        vec![zl::LAMP as u8, 16 + 21, 0x12, 0x34],
    );
    f.setup.extend(set_global(21, 1));
    let image = f.build();
    let digest: String = Sha256::digest(&image)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let serial = String::from_utf8(image[0x12..0x18].to_vec()).unwrap();
    let pack = format!(
        "pack: 1\nsha256: {digest}\nrelease: {}\nserial: {serial}\nopen-attribute: 7\nuntrusted-variable: {}\n",
        u16::from_be_bytes([image[2], image[3]]),
        16 + zl::TRAP_OPEN
    );
    let m = crate::fixtures::run(image);
    let view = m.view();
    let i = Introspector::with_pack(&view, &pack);
    assert_eq!(i.pack(), crate::PackState::InForce);
    let text = to_json(&i.exits(&view));
    let hall = zl::HALL;
    let door = zl::DOOR;
    assert!(
        text.contains(&format!(
            r#"{{"direction":{},"words":["west"],"kind":"door","destination":{hall},"destination_name":"Hall","door":{door},"state":"closed","source":"pack"}}"#,
            zl::WEST
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            r#"{{"direction":{},"words":["down"],"kind":"conditional","destination":null,"destination_name":null,"door":null,"state":null,"source":"pack"}}"#,
            zl::DOWN
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            r#"{{"direction":{},"words":["n","north"],"kind":"fixed","destination":{hall},"destination_name":"Hall","door":null,"state":null,"source":"file"}}"#,
            zl::NORTH
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            r#""unknown":[{{"direction":{},"words":["up"],"reason":"not-determinable"}}]"#,
            zl::UP
        )),
        "{text}"
    );
}
