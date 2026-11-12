//! The JSON rendering of answers (docs/introspection.md section 2).
//!
//! Each rendering is one line with no trailing new-line, with fields in the
//! specification's order. A value that can be unknown is either the value or
//! `{"unknown":"<reason>"}`.

use crate::{
    Answer, Carried, ContainerState, DoorState, Exactness, Exit, ExitKind, Exits, Family, Light,
    Location, Object, PackState, PartOfSpeech, Reach, Scope, ScopeEntry, Source, Tri, Unknown,
    UnknownDirection, Visibility, Word,
};

/// Something the specification gives a JSON form.
pub trait Json {
    /// Appends the JSON form to `out`.
    fn write_json(&self, out: &mut String);
}

/// The JSON form of an answer or of any part of one.
pub fn to_json<T: Json + ?Sized>(value: &T) -> String {
    let mut out = String::new();
    value.write_json(&mut out);
    out
}

/// Writes `{"key":value,…}` from the fields given, in order.
fn object(out: &mut String, fields: &[(&str, &dyn Json)]) {
    out.push('{');
    for (i, (key, value)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        key.write_json(out);
        out.push(':');
        value.write_json(out);
    }
    out.push('}');
}

impl Json for str {
    fn write_json(&self, out: &mut String) {
        out.push('"');
        for ch in self.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                control if (control as u32) < 0x20 => {
                    out.push_str(&format!("\\u{:04x}", control as u32))
                }
                other => out.push(other),
            }
        }
        out.push('"');
    }
}

impl Json for &str {
    fn write_json(&self, out: &mut String) {
        (**self).write_json(out);
    }
}

impl Json for String {
    fn write_json(&self, out: &mut String) {
        self.as_str().write_json(out);
    }
}

impl Json for u8 {
    fn write_json(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Json for u16 {
    fn write_json(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Json for usize {
    fn write_json(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

/// `None` is `null`.
impl<T: Json> Json for Option<T> {
    fn write_json(&self, out: &mut String) {
        match self {
            Some(value) => value.write_json(out),
            None => out.push_str("null"),
        }
    }
}

impl<T: Json> Json for [T] {
    fn write_json(&self, out: &mut String) {
        out.push('[');
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            item.write_json(out);
        }
        out.push(']');
    }
}

impl<T: Json> Json for Vec<T> {
    fn write_json(&self, out: &mut String) {
        self.as_slice().write_json(out);
    }
}

impl Json for Unknown {
    fn write_json(&self, out: &mut String) {
        object(out, &[("unknown", &self.as_str())]);
    }
}

/// A part that can be unknown: the value, or `{"unknown":"<reason>"}`.
impl<T: Json> Json for Result<T, Unknown> {
    fn write_json(&self, out: &mut String) {
        match self {
            Ok(value) => value.write_json(out),
            Err(reason) => reason.write_json(out),
        }
    }
}

/// Words the specification writes as themselves.
macro_rules! as_word {
    ($($type:ty),*) => {$(
        impl Json for $type {
            fn write_json(&self, out: &mut String) {
                self.as_str().write_json(out);
            }
        }
    )*};
}
as_word!(
    Exactness,
    Tri,
    Family,
    PartOfSpeech,
    ExitKind,
    DoorState,
    ContainerState,
    Source,
    PackState,
    Light,
    Reach,
    Visibility
);

impl<T: Json> Json for Answer<T> {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[("exactness", &self.exactness), ("value", &self.value)],
        );
    }
}

impl Json for Object {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("number", &self.number),
                ("name", &self.name),
                ("printed_name", &self.printed_name),
                ("parent", &self.parent),
                ("children", &self.children),
                ("attributes", &self.attributes),
                ("properties", &self.properties),
            ],
        );
    }
}

impl Json for Word {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("word", &self.word),
                ("parts", &self.parts),
                ("direction", &self.direction),
            ],
        );
    }
}

impl Json for Location {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("room", &self.room),
                ("name", &self.name),
                ("holder", &self.holder),
                ("lit", &self.lit),
            ],
        );
    }
}

impl Json for Carried {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("object", &self.object),
                ("name", &self.name),
                ("worn", &self.worn),
                ("state", &self.state),
                ("seen", &self.seen),
                ("source", &self.source),
                ("contents", &self.contents),
            ],
        );
    }
}

impl Json for Exit {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("direction", &self.direction),
                ("words", &self.words),
                ("kind", &self.kind),
                ("destination", &self.destination),
                ("destination_name", &self.destination_name),
                ("door", &self.door),
                ("state", &self.state),
                ("source", &self.source),
            ],
        );
    }
}

impl Json for UnknownDirection {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("direction", &self.direction),
                ("words", &self.words),
                ("reason", &self.reason.as_str()),
            ],
        );
    }
}

impl Json for Exits {
    fn write_json(&self, out: &mut String) {
        object(out, &[("exits", &self.exits), ("unknown", &self.unknown)]);
    }
}

impl Json for ScopeEntry {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("object", &self.object),
                ("name", &self.name),
                ("reach", &self.reach),
                ("within", &self.within),
                ("visibility", &self.visibility),
                ("scenery", &self.scenery),
                ("concealed", &self.concealed),
            ],
        );
    }
}

/// Whether the player was found: `"known"`, or the reason not.
struct Player<'a>(&'a Result<(), Unknown>);

impl Json for Player<'_> {
    fn write_json(&self, out: &mut String) {
        match self.0 {
            Ok(()) => "known".write_json(out),
            Err(reason) => reason.write_json(out),
        }
    }
}

impl Json for Scope {
    fn write_json(&self, out: &mut String) {
        object(
            out,
            &[
                ("light", &self.light),
                ("player", &Player(&self.player)),
                ("objects", &self.objects),
            ],
        );
    }
}

/// One line for a stop of the game, as a host that watches a game writes it:
/// the specification's version, the stop's number, what the game wants (`line`
/// or `key`), the family, and the answers to `location`, `exits`, `inventory`
/// and `scope` (specification section 2.4).
pub fn stop_line(
    introspector: &crate::Introspector,
    view: &zmachine_core::View<'_>,
    stop: usize,
    wants: &str,
) -> String {
    let mut out = String::new();
    object(
        &mut out,
        &[
            ("spec", &introspector.spec_version()),
            ("stop", &stop),
            ("wants", &wants),
            ("family", &introspector.family()),
            ("pack", &introspector.pack()),
            ("location", &introspector.location(view)),
            ("exits", &introspector.exits(view)),
            ("inventory", &introspector.inventory(view)),
            ("scope", &introspector.scope(view)),
        ],
    );
    out
}

/// The object tree, the dictionary, or both, as one object with the
/// specification's version and the family (specification section 2.4).
pub fn tables(
    introspector: &crate::Introspector,
    view: &zmachine_core::View<'_>,
    tree: bool,
    dictionary: bool,
) -> String {
    let (objects, words) = (
        introspector.object_tree(view),
        introspector.dictionary(view),
    );
    let version = introspector.spec_version();
    let family = introspector.family();
    let mut fields: Vec<(&str, &dyn Json)> = vec![("spec", &version), ("family", &family)];
    if tree {
        fields.push(("object_tree", &objects));
    }
    if dictionary {
        fields.push(("dictionary", &words));
    }
    let mut out = String::new();
    object(&mut out, &fields);
    out
}
