//! The snapshot as JSON (specs/004-first-demo/contracts/experience-api.md,
//! "Snapshot JSON"). One line; every field always present, `null` where it has
//! no value.

use crate::map::MapView;
use crate::{CompassPlace, Extra, Session, Status, StripItem, Turn};

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
    key(out, "dark_ahead");
    out.push_str(if place.dark_ahead { "true" } else { "false" });
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
    key(out, "dark_ahead");
    out.push_str(if extra.dark_ahead { "true" } else { "false" });
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
    out.push('}');
    let strip = session.inventory();
    key(&mut out, "inventory");
    out.push('{');
    key(&mut out, "known");
    out.push_str(if strip.known { "true" } else { "false" });
    key(&mut out, "note");
    optional_text(&mut out, strip.note);
    key(&mut out, "spoken");
    text(&mut out, &strip.spoken);
    key(&mut out, "items");
    list(&mut out, &strip.items, item);
    out.push('}');
    let light = session.light();
    key(&mut out, "light");
    out.push('{');
    key(&mut out, "dark");
    out.push_str(match light.dark {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    });
    key(&mut out, "note");
    optional_text(&mut out, light.note);
    out.push('}');
    // What a screen reader is to speak for the newest turn (feature 010).
    key(&mut out, "announcement");
    optional_text(&mut out, session.announcement());
    // Words to show once, and the question before starting over (feature 011).
    key(&mut out, "notice");
    optional_text(&mut out, session.notice());
    key(&mut out, "start_over");
    optional_text(&mut out, session.start_over());
    // The sheet of the map the player is on (feature 012).
    key(&mut out, "map");
    match session.map().view() {
        Some(view) => map(&mut out, &view),
        None => out.push_str("null"),
    }
    out.push('}');
    out
}

fn item(out: &mut String, item: &StripItem) {
    out.push('{');
    key(out, "name");
    text(out, &item.name);
    key(out, "state");
    text(out, item.state.as_str());
    key(out, "contents");
    list(out, &item.contents, self::item);
    out.push('}');
}

fn flag(out: &mut String, value: bool) {
    out.push_str(if value { "true" } else { "false" });
}

/// The map's sheet (specs/012-map-first-slice/contracts/map.md, part S).
pub(crate) fn map(out: &mut String, view: &MapView) {
    out.push('{');
    key(out, "note");
    optional_text(out, view.note);
    key(out, "spoken");
    text(out, &view.spoken);
    key(out, "sheet");
    out.push_str(&view.sheet.to_string());
    key(out, "sheets");
    out.push_str(&view.sheets.to_string());
    key(out, "here");
    optional_number(out, view.here);
    key(out, "lamp");
    text(out, view.lamp.as_str());
    key(out, "places");
    list(out, &view.places, |out, place| {
        out.push('{');
        key(out, "id");
        out.push_str(&place.id.to_string());
        key(out, "x");
        out.push_str(&place.x.to_string());
        key(out, "y");
        out.push_str(&place.y.to_string());
        key(out, "inset");
        flag(out, place.inset);
        key(out, "name");
        optional_text(out, place.name.as_deref());
        key(out, "attached_to");
        optional_number(out, place.attached_to);
        out.push('}');
    });
    key(out, "lines");
    list(out, &view.lines, |out, line| {
        out.push('{');
        key(out, "a");
        out.push_str(&line.a.to_string());
        key(out, "b");
        out.push_str(&line.b.to_string());
        key(out, "out");
        text(out, &line.out);
        key(out, "back");
        optional_text(out, line.back.as_deref());
        key(out, "kind");
        text(out, line.kind.as_str());
        key(out, "label");
        optional_text(out, line.label.as_deref());
        out.push('}');
    });
    key(out, "leads");
    list(out, &view.leads, |out, lead| {
        out.push('{');
        key(out, "from");
        out.push_str(&lead.from.to_string());
        key(out, "way");
        text(out, &lead.way);
        key(out, "label");
        text(out, &lead.label);
        key(out, "to");
        optional_text(out, lead.to.as_deref());
        key(out, "sheet");
        out.push_str(&lead.sheet.to_string());
        out.push('}');
    });
    key(out, "stubs");
    list(out, &view.stubs, |out, stub| {
        out.push('{');
        key(out, "of");
        out.push_str(&stub.of.to_string());
        key(out, "way");
        text(out, &stub.way);
        key(out, "state");
        text(out, &stub.state);
        key(out, "dark_ahead");
        flag(out, stub.dark_ahead);
        out.push('}');
    });
    key(out, "ways");
    list(out, &view.ways, |out, way| {
        out.push('{');
        key(out, "way");
        text(out, &way.way);
        key(out, "label");
        text(out, &way.label);
        key(out, "state");
        text(out, &way.state);
        key(out, "command");
        optional_text(out, way.command.as_deref());
        key(out, "leads_to");
        optional_text(out, way.leads_to.as_deref());
        key(out, "dark_ahead");
        flag(out, way.dark_ahead);
        key(out, "refused");
        flag(out, way.refused);
        key(out, "varies");
        flag(out, way.varies);
        key(out, "returns");
        flag(out, way.returns);
        key(out, "spoken");
        text(out, &way.spoken);
        out.push('}');
    });
    out.push('}');
}
