//! A story's presentation file as a draw.io diagram, and back
//! (presentations/README.md, "Laying a map out in draw.io").
//!
//! ```text
//! cargo run --release -p zmachine-corpus --example map_drawio -- export zork1
//! cargo run --release -p zmachine-corpus --example map_drawio -- import zork1
//! ```
//!
//! `export` writes `presentations/drawio/<story>.drawio`: a page for each
//! sheet of `presentations/<story>.present`, a box for each room where the
//! file puts it, a line for each way found by playing, and a last page of the
//! rooms the file does not have. `import` reads that diagram after it has been
//! changed by hand and writes the presentation file from it.
//!
//! The diagram holds the game's names for its rooms, so that a person can
//! tell the boxes apart. It is for one person's machine: the folder is
//! ignored, and nothing of a name is read back. Only a box's room number, the
//! page it is on, where it sits and the page's name reach the presentation
//! file.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use zmachine_core::{LoadOptions, Machine};
use zmachine_experience::{Seed, Session};
use zmachine_introspect::Introspector;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The size of one position of a sheet in the diagram, and of a room's box.
const STEP: (f64, f64) = (140.0, 80.0);
const BOX: (f64, f64) = (110.0, 40.0);

/// The page of rooms the file does not place. It is not a sheet.
const SPARE: &str = "Not placed";

const COMPASS: [(&str, (i32, i32)); 8] = [
    ("north", (0, -1)),
    ("northeast", (1, -1)),
    ("east", (1, 0)),
    ("southeast", (1, 1)),
    ("south", (0, 1)),
    ("southwest", (-1, 1)),
    ("west", (-1, 0)),
    ("northwest", (-1, -1)),
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["export", story] => export(story),
        ["import", story] => import(story),
        _ => Err("usage: map_drawio export|import <story>, as in `export zork1`".to_string()),
    };
    if let Err(message) = result {
        eprintln!("map_drawio: {message}");
        std::process::exit(1);
    }
}

fn path(relative: &str) -> String {
    format!("{ROOT}/{relative}")
}

fn read(relative: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path(relative)).map_err(|e| format!("{relative}: {e}"))
}

fn text(relative: &str) -> Option<String> {
    std::fs::read_to_string(path(relative)).ok()
}

// ---------------------------------------------------------------------------
// The presentation file, as this tool reads and writes it
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Layout {
    /// Each sheet's number in the file and its name.
    sheets: Vec<(u32, String)>,
    /// Room: the file's sheet, and the position.
    places: BTreeMap<u16, (u32, i32, i32)>,
}

fn layout_of(file: &str) -> Layout {
    let mut layout = Layout::default();
    for line in file.lines() {
        if let Some(rest) = line.strip_prefix("sheet: ") {
            let (number, name) = rest.split_once(' ').unwrap_or((rest, ""));
            layout
                .sheets
                .push((number.parse().unwrap_or(0), name.to_string()));
        } else if let Some(rest) = line.strip_prefix("place: ") {
            let n: Vec<i32> = rest.split(' ').filter_map(|n| n.parse().ok()).collect();
            if let [room, sheet, x, y] = n[..] {
                layout.places.insert(room as u16, (sheet as u32, x, y));
            }
        }
    }
    layout
}

fn identity(story: &[u8]) -> String {
    format!(
        "presentation: 1\nsha256: {}\nrelease: {}\nserial: {}\n",
        zmachine_introspect::pack::sha256_hex(story),
        u16::from_be_bytes([story[2], story[3]]),
        String::from_utf8_lossy(&story[0x12..0x18]),
    )
}

// ---------------------------------------------------------------------------
// What the game has: its rooms, and the ways found by playing
// ---------------------------------------------------------------------------

struct Game {
    /// Every room of the story, with the name the game prints for it.
    rooms: BTreeMap<u16, String>,
    /// The ways walked: from, the way, to.
    ways: BTreeSet<(u16, String, u16)>,
    /// The rooms reached, first reached first.
    reached: Vec<u16>,
}

fn here(session: &Session) -> Option<u16> {
    session.map().view()?.here
}

/// Plays the story: along its walkthrough if it has one, trying every way
/// offered at every turn on copies of the game, a few rooms deep.
fn play(id: &str) -> Result<Game, String> {
    let story = read(&format!("stories/{id}/{id}.z3"))?;
    let pack = text(&format!("packs/{id}.pack"));
    let walk: Vec<String> = text(&format!("stories/{id}/fidelity/walkthrough.txt"))
        .map(|walk| walk.lines().skip(2).map(str::to_string).collect())
        .unwrap_or_default();
    let depth = if walk.is_empty() { 60 } else { 3 };
    let mut session = Session::with_pack(story.clone(), Seed::Fixed(11), pack.as_deref())
        .map_err(|e| e.message)?;
    let first = here(&session).ok_or("the game does not say where the player begins")?;

    let mut game = Game {
        rooms: BTreeMap::new(),
        ways: BTreeSet::new(),
        reached: Vec::new(),
    };
    let mut tried: BTreeSet<(u16, String)> = BTreeSet::new();
    let mut search = |from: &Session, game: &mut Game| {
        let mut queue: VecDeque<(Session, u32)> = VecDeque::from([(from.clone(), 0)]);
        while let Some((at, gone)) = queue.pop_front() {
            let (Some(room), Some(view)) = (here(&at), at.map().view()) else {
                continue;
            };
            if !game.reached.contains(&room) {
                game.reached.push(room);
            }
            for way in view.ways {
                let Some(command) = way.command else { continue };
                if !tried.insert((room, way.way.clone())) {
                    continue;
                }
                let mut next = at.clone();
                if next.send_line(next.stamp(), &command).is_err() {
                    continue;
                }
                match here(&next) {
                    Some(to) if to != room => {
                        game.ways.insert((room, way.way.clone(), to));
                        if gone < depth {
                            queue.push_back((next, gone + 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    };
    search(&session, &mut game);
    for line in &walk {
        if session.send_line(session.stamp(), line).is_err() {
            break;
        }
        search(&session, &mut game);
    }

    // The rooms: every object with the parent that the first room has.
    let options = LoadOptions {
        seed: Some(11),
        entropy: 0,
    };
    let machine = Machine::load(story, options).map_err(|e| format!("the story file is {e}"))?;
    let view = machine.view();
    let introspector = match &pack {
        Some(pack) => Introspector::with_pack(&view, pack),
        None => Introspector::new(&view),
    };
    let objects = introspector
        .object_tree(&view)
        .value
        .map_err(|_| "the story's objects cannot be read")?;
    let parent = |number: u16| {
        objects
            .iter()
            .find(|object| object.number == number)
            .and_then(|object| object.parent.ok().flatten())
    };
    let rooms = parent(first).ok_or("the first room is in nothing")?;
    for object in &objects {
        if parent(object.number) == Some(rooms) {
            let name = object.printed_name.clone().unwrap_or_default();
            game.rooms.insert(object.number, name);
        }
    }
    Ok(game)
}

/// The ways between two rooms, as the map joins them: whether any lies along
/// its direction with no room of the sheet between.
fn straight(a: u16, b: u16, game: &Game, at: &BTreeMap<u16, (i32, i32)>) -> Option<bool> {
    let taken: BTreeSet<(i32, i32)> = at.values().copied().collect();
    let mut compass = false;
    for (from, way, to) in &game.ways {
        if !((*from, *to) == (a, b) || (*from, *to) == (b, a)) {
            continue;
        }
        let Some((_, (dx, dy))) = COMPASS.iter().find(|(name, _)| name == way) else {
            continue;
        };
        compass = true;
        let ((fx, fy), (tx, ty)) = (at[from], at[to]);
        let steps = (tx - fx).abs().max((ty - fy).abs());
        if steps > 0
            && (fx + dx * steps, fy + dy * steps) == (tx, ty)
            && (1..steps).all(|k| !taken.contains(&(fx + dx * k, fy + dy * k)))
        {
            return Some(true);
        }
    }
    compass.then_some(false)
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn short(way: &str) -> &str {
    match way {
        "north" => "n",
        "northeast" => "ne",
        "east" => "e",
        "southeast" => "se",
        "south" => "s",
        "southwest" => "sw",
        "west" => "w",
        "northwest" => "nw",
        "up" => "u",
        "down" => "d",
        other => other,
    }
}

fn page(name: &str, id: usize, cells: &str) -> String {
    format!(
        "  <diagram id=\"page{id}\" name=\"{}\">\n    <mxGraphModel dx=\"1200\" dy=\"800\" grid=\"1\" \
         gridSize=\"10\" guides=\"1\" tooltips=\"1\" connect=\"1\" arrows=\"0\" fold=\"1\" page=\"0\" \
         pageScale=\"1\" math=\"0\" shadow=\"0\">\n      <root>\n        <mxCell id=\"0\"/>\n        \
         <mxCell id=\"1\" parent=\"0\"/>\n{cells}      </root>\n    </mxGraphModel>\n  </diagram>\n",
        escape(name)
    )
}

fn room_box(room: u16, name: &str, x: f64, y: f64, placed: bool) -> String {
    let fill = if placed { "#FFFDF6" } else { "#EEEEEE" };
    format!(
        "        <object label=\"{}\" room=\"{room}\" id=\"room{room}\">\n          <mxCell \
         style=\"rounded=1;whiteSpace=wrap;html=1;fillColor={fill};strokeColor=#333333;fontSize=11;\" \
         vertex=\"1\" parent=\"1\">\n            <mxGeometry x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" \
         as=\"geometry\"/>\n          </mxCell>\n        </object>\n",
        escape(name),
        x - BOX.0 / 2.0,
        y - BOX.1 / 2.0,
        BOX.0,
        BOX.1
    )
}

fn export(id: &str) -> Result<(), String> {
    let game = play(id)?;
    let layout = text(&format!("presentations/{id}.present"))
        .map(|file| layout_of(&file))
        .unwrap_or_default();
    let mut pages = String::new();
    let mut edge = 0;
    for (index, (number, name)) in layout.sheets.iter().enumerate() {
        let on: BTreeMap<u16, (i32, i32)> = layout
            .places
            .iter()
            .filter(|(_, place)| place.0 == *number)
            .map(|(room, place)| (*room, (place.1, place.2)))
            .collect();
        let mut cells = String::new();
        // The lines first, so that the boxes lie over them.
        let pairs: BTreeSet<(u16, u16)> = game
            .ways
            .iter()
            .filter(|(a, _, b)| on.contains_key(a) && on.contains_key(b))
            .map(|(a, _, b)| (*a.min(b), *a.max(b)))
            .collect();
        for (a, b) in pairs {
            let ways = |from: u16, to: u16| {
                let list: Vec<&str> = game
                    .ways
                    .iter()
                    .filter(|(f, _, t)| (*f, *t) == (from, to))
                    .map(|(_, way, _)| short(way))
                    .collect();
                list.join(",")
            };
            let (out, back) = (ways(a, b), ways(b, a));
            let ladder = straight(a, b, &game, &on).is_none();
            let style = if ladder {
                "endArrow=none;html=1;dashed=1;strokeColor=#666666;fontSize=9;"
            } else {
                "endArrow=none;html=1;strokeColor=#333333;fontSize=9;"
            };
            edge += 1;
            let _ = write!(
                cells,
                "        <mxCell id=\"way{edge}\" value=\"{}\" style=\"{style}\" edge=\"1\" parent=\"1\" \
                 source=\"room{a}\" target=\"room{b}\">\n          <mxGeometry relative=\"1\" \
                 as=\"geometry\"/>\n        </mxCell>\n",
                escape(&format!("{out} / {back}"))
            );
        }
        for (room, (x, y)) in &on {
            let name = game.rooms.get(room).map(String::as_str).unwrap_or("?");
            let (x, y) = (f64::from(*x) * STEP.0, f64::from(*y) * STEP.1);
            cells.push_str(&room_box(*room, name, x, y, true));
        }
        pages.push_str(&page(name, index + 1, &cells));
    }
    // The rooms the file does not have: those reached first, in the order
    // reached, then the rest.
    let mut spare: Vec<u16> = game
        .reached
        .iter()
        .copied()
        .filter(|room| game.rooms.contains_key(room))
        .collect();
    spare.extend(
        game.rooms
            .keys()
            .filter(|room| !game.reached.contains(room)),
    );
    spare.retain(|room| !layout.places.contains_key(room));
    let mut cells = String::new();
    for (n, room) in spare.iter().enumerate() {
        let (x, y) = ((n % 8) as f64 * STEP.0, (n / 8) as f64 * STEP.1);
        cells.push_str(&room_box(*room, &game.rooms[room], x, y, false));
    }
    pages.push_str(&page(SPARE, layout.sheets.len() + 1, &cells));

    std::fs::create_dir_all(path("presentations/drawio")).map_err(|e| e.to_string())?;
    let out = format!("presentations/drawio/{id}.drawio");
    let file = format!("<mxfile host=\"map_drawio\" compressed=\"false\">\n{pages}</mxfile>\n");
    std::fs::write(path(&out), file).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{out}: {} rooms placed on {} sheets, {} not placed, {} ways",
        layout.places.len(),
        layout.sheets.len(),
        spare.len(),
        game.ways.len()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

/// One tag of the diagram: its name, with `/` before it if it closes, and
/// its attributes.
struct Tag {
    name: String,
    attributes: BTreeMap<String, String>,
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#10;", " ")
        .replace("&amp;", "&")
}

/// The tags of a diagram in order, and for each `diagram` whether any text
/// follows it before the next tag: a page draw.io has packed.
fn tags(file: &str) -> Vec<(Tag, bool)> {
    let mut found = Vec::new();
    let mut rest = file;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        // A tag ends at the first `>` that is not inside quotes.
        let mut quoted = false;
        let Some(end) = rest.find(|c| {
            if c == '"' {
                quoted = !quoted;
            }
            c == '>' && !quoted
        }) else {
            break;
        };
        let inside = rest[..end].trim_end_matches('/');
        rest = &rest[end + 1..];
        let (name, mut list) = inside
            .split_once(char::is_whitespace)
            .unwrap_or((inside, ""));
        let mut attributes = BTreeMap::new();
        while let Some((key, after)) = list.split_once("=\"") {
            let Some((value, more)) = after.split_once('"') else {
                break;
            };
            attributes.insert(key.trim().to_string(), unescape(value));
            list = more;
        }
        let packed = !rest.trim_start().starts_with('<') && !rest.trim().is_empty();
        found.push((
            Tag {
                name: name.to_string(),
                attributes,
            },
            packed,
        ));
    }
    found
}

/// A box read from the diagram: its sheet, its position, and its label for
/// messages.
type Placed = (usize, i32, i32, String);

fn import(id: &str) -> Result<(), String> {
    let source = format!("presentations/drawio/{id}.drawio");
    let file = text(&source).ok_or(format!("{source}: not found; run `export {id}` first"))?;
    let story = read(&format!("stories/{id}/{id}.z3"))?;

    let mut sheets: Vec<String> = Vec::new();
    let mut places: BTreeMap<u16, Placed> = BTreeMap::new();
    let mut faults: Vec<String> = Vec::new();
    let mut spare = false;
    let mut pending: Option<(u16, String)> = None;
    for (tag, packed) in tags(&file) {
        match tag.name.as_str() {
            "diagram" => {
                let name = tag.attributes.get("name").cloned().unwrap_or_default();
                if packed {
                    faults.push(format!(
                        "page \"{name}\" is compressed: in draw.io choose File, Properties, \
                         untick Compressed, and save again"
                    ));
                }
                spare = name.trim().eq_ignore_ascii_case(SPARE);
                if !spare {
                    if name.chars().count() > 40 {
                        faults.push(format!("page \"{name}\": a name is at most 40 characters"));
                    }
                    sheets.push(name.trim().to_string());
                }
                pending = None;
            }
            "object" | "UserObject" => {
                let label = tag.attributes.get("label").cloned().unwrap_or_default();
                pending = match tag.attributes.get("room") {
                    Some(room) => match room.trim().parse::<u16>() {
                        Ok(room) if room > 0 => Some((room, label)),
                        _ => {
                            faults
                                .push(format!("box \"{label}\": room \"{room}\" is not a number"));
                            None
                        }
                    },
                    None => None,
                };
            }
            "/object" | "/UserObject" => pending = None,
            "mxGeometry" if !spare => {
                let Some((room, label)) = pending.take() else {
                    continue;
                };
                let number = |key: &str| {
                    tag.attributes
                        .get(key)
                        .and_then(|value| value.parse::<f64>().ok())
                        .unwrap_or(0.0)
                };
                let centre = (
                    number("x") + number("width") / 2.0,
                    number("y") + number("height") / 2.0,
                );
                let cell = (
                    (centre.0 / STEP.0).round() as i32,
                    (centre.1 / STEP.1).round() as i32,
                );
                if let Some(other) = places.get(&room) {
                    faults.push(format!(
                        "room {room} has two boxes: \"{label}\" and \"{}\"",
                        other.3
                    ));
                }
                places.insert(room, (sheets.len(), cell.0, cell.1, label));
            }
            _ => {}
        }
    }
    if sheets.len() > 99 {
        faults.push("there are more than 99 pages".to_string());
    }
    // Each sheet is moved as a whole so that its positions are small numbers
    // around its middle; what matters is where places are from one another.
    let mut lines = String::new();
    for (index, name) in sheets.iter().enumerate() {
        let sheet = index + 1;
        let on: Vec<(&u16, &Placed)> = places
            .iter()
            .filter(|(_, place)| place.0 == sheet)
            .collect();
        let _ = writeln!(lines, "sheet: {sheet} {name}");
        let _ = writeln!(lines);
        if on.is_empty() {
            continue;
        }
        let middle = |pick: fn(&Placed) -> i32| {
            let all: Vec<i32> = on.iter().map(|(_, place)| pick(place)).collect();
            let (low, high) = (all.iter().min().unwrap(), all.iter().max().unwrap());
            (low + high).div_euclid(2)
        };
        let (mx, my) = (middle(|place| place.1), middle(|place| place.2));
        let mut taken: BTreeMap<(i32, i32), &str> = BTreeMap::new();
        let mut rows: Vec<(i32, i32, u16)> = Vec::new();
        for (room, (_, x, y, label)) in on {
            let cell = (x - mx, y - my);
            if cell.0.abs() > 99 || cell.1.abs() > 99 {
                faults.push(format!(
                    "box \"{label}\" on \"{name}\" is too far from the others"
                ));
            }
            if let Some(other) = taken.insert(cell, label) {
                faults.push(format!(
                    "\"{label}\" and \"{other}\" on \"{name}\" are in the same square: move one \
                     at least {} across or {} down",
                    STEP.0, STEP.1
                ));
            }
            rows.push((cell.1, cell.0, *room));
        }
        rows.sort();
        for (y, x, room) in rows {
            let _ = writeln!(lines, "place: {room} {sheet} {x} {y}");
        }
        let _ = writeln!(lines);
    }
    if !faults.is_empty() {
        return Err(format!("{source} was not used:\n  {}", faults.join("\n  ")));
    }
    let present = format!("{}\n{}", identity(&story), lines.trim_end_matches('\n')) + "\n";

    // The layer has the last word: the file must be one it will use.
    let pack = text(&format!("packs/{id}.pack"));
    let session =
        Session::with_presentation(story, Seed::Fixed(11), pack.as_deref(), Some(&present))
            .map_err(|e| e.message)?;
    if session.notice().is_some() {
        return Err(format!(
            "{source} was not used: the layer refused the file made from it. A box's room \
             number may have been changed to something that is not a room."
        ));
    }
    let out = format!("presentations/{id}.present");
    std::fs::write(path(&out), &present).map_err(|e| format!("{out}: {e}"))?;
    println!("{out}: {} rooms on {} sheets", places.len(), sheets.len());

    // What the person laying it out wants to know: which joins bend.
    let game = play(id)?;
    let layout = layout_of(&present);
    for (number, name) in &layout.sheets {
        let on: BTreeMap<u16, (i32, i32)> = layout
            .places
            .iter()
            .filter(|(_, place)| place.0 == *number)
            .map(|(room, place)| (*room, (place.1, place.2)))
            .collect();
        let pairs: BTreeSet<(u16, u16)> = game
            .ways
            .iter()
            .filter(|(a, _, b)| on.contains_key(a) && on.contains_key(b))
            .map(|(a, _, b)| (*a.min(b), *a.max(b)))
            .collect();
        let bent: Vec<String> = pairs
            .iter()
            .filter(|(a, b)| straight(*a, *b, &game, &on) == Some(false))
            .map(|(a, b)| format!("{} ({a}) and {} ({b})", game.rooms[a], game.rooms[b]))
            .collect();
        let straight = pairs
            .iter()
            .filter(|(a, b)| straight(*a, *b, &game, &on) == Some(true))
            .count();
        println!("{name}: {straight} joins straight, {} bent", bent.len());
        for pair in bent {
            println!("  bent: {pair}");
        }
    }
    Ok(())
}
