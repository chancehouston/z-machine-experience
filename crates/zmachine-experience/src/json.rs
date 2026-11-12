//! The snapshot as JSON (specs/004-first-demo/contracts/experience-api.md,
//! "Snapshot JSON"). One line; every field always present, `null` where it has
//! no value.

use crate::{CompassPlace, Extra, Session, Status, Turn};

fn text(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
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

fn optional_text(out: &mut String, value: Option<&str>) {
    match value {
        Some(value) => text(out, value),
        None => out.push_str("null"),
    }
}

fn optional_number<T: ToString>(out: &mut String, value: Option<T>) {
    match value {
        Some(value) => out.push_str(&value.to_string()),
        None => out.push_str("null"),
    }
}

/// Writes `"key":` with a comma before it unless it is the first of its object.
fn key(out: &mut String, name: &str) {
    if !out.ends_with('{') {
        out.push(',');
    }
    text(out, name);
    out.push(':');
}

fn list<T>(out: &mut String, items: &[T], write: impl Fn(&mut String, &T)) {
    out.push('[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write(out, item);
    }
    out.push(']');
}

fn status(out: &mut String, status: &Status) {
    out.push('{');
    key(out, "place");
    text(out, &status.place);
    key(out, "score");
    optional_number(out, status.score);
    key(out, "moves");
    optional_number(out, status.moves);
    key(out, "hours");
    optional_number(out, status.hours);
    key(out, "minutes");
    optional_number(out, status.minutes);
    out.push('}');
}

fn turn(out: &mut String, turn: &Turn) {
    out.push('{');
    key(out, "number");
    out.push_str(&turn.number.to_string());
    key(out, "command");
    optional_text(out, turn.command.as_deref());
    key(out, "source");
    optional_text(out, turn.source.map(|s| s.as_str()));
    key(out, "text");
    text(out, &turn.text);
    out.push('}');
}

fn place(out: &mut String, place: &CompassPlace) {
    out.push('{');
    key(out, "place");
    text(out, place.place.id());
    key(out, "label");
    text(out, place.label);
    key(out, "state");
    text(out, place.state.as_str());
    key(out, "command");
    optional_text(out, place.command.as_deref());
    key(out, "leads_to");
    optional_text(out, place.leads_to.as_deref());
    key(out, "spoken");
    text(out, &place.spoken);
    out.push('}');
}

fn extra(out: &mut String, extra: &Extra) {
    out.push('{');
    key(out, "label");
    text(out, &extra.label);
    key(out, "state");
    text(out, extra.state.as_str());
    key(out, "command");
    text(out, &extra.command);
    key(out, "leads_to");
    optional_text(out, extra.leads_to.as_deref());
    key(out, "spoken");
    text(out, &extra.spoken);
    out.push('}');
}

pub(crate) fn snapshot(session: &Session, since: usize) -> String {
    let stamp = session.stamp();
    let turns = session.turns();
    let compass = session.compass();
    let mut out = String::from("{");
    key(&mut out, "generation");
    out.push_str(&stamp.generation.to_string());
    key(&mut out, "turn_count");
    out.push_str(&stamp.turn_count.to_string());
    key(&mut out, "waiting");
    text(&mut out, session.waiting().as_str());
    key(&mut out, "message");
    optional_text(&mut out, session.message());
    key(&mut out, "status");
    match session.status() {
        Some(line) => status(&mut out, line),
        None => out.push_str("null"),
    }
    key(&mut out, "turns");
    list(&mut out, turns.get(since..).unwrap_or(&[]), turn);
    key(&mut out, "compass");
    out.push('{');
    key(&mut out, "offered");
    out.push_str(if compass.offered { "true" } else { "false" });
    key(&mut out, "known");
    out.push_str(if compass.known { "true" } else { "false" });
    key(&mut out, "note");
    optional_text(&mut out, compass.note);
    key(&mut out, "places");
    list(&mut out, &compass.places, place);
    key(&mut out, "extras");
    list(&mut out, &compass.extras, extra);
    out.push_str("}}");
    out
}
