//! The screen model for versions 5 and 8 (feature 002, T043, T051; §8.7;
//! specs/002-v5-v8/contracts/screen-model.md). The ten test programs of that
//! contract are numbered here as they are there.

mod common;

use common::*;
use zmachine_core::{Event, Key, Machine, Screen, Settings, Style};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::programs::screen_programs;
use zmachine_testkit::{Story, Story5};

const PLAIN: Style = Style {
    reverse: false,
    bold: false,
    italic: false,
    fixed_pitch: false,
};
const REVERSE: Style = Style {
    reverse: true,
    ..PLAIN
};

fn settings(columns: u16, rows: u16) -> Settings {
    Settings {
        screen_columns: columns,
        screen_rows: rows,
        ..Settings::default()
    }
}

fn window(n: u8) -> Vec<u8> {
    op::var(op::SET_WINDOW, &[S(n)])
}
fn split(n: u16) -> Vec<u8> {
    op::var(op::SPLIT_WINDOW, &[L(n)])
}
fn cursor(row: u16, column: u16) -> Vec<u8> {
    op::var(op::SET_CURSOR, &[L(row), L(column)])
}
fn style(n: u8) -> Vec<u8> {
    op::var(op::SET_TEXT_STYLE, &[S(n)])
}
fn erase(n: i16) -> Vec<u8> {
    op::var(op::ERASE_WINDOW, &[L(n as u16)])
}
fn erase_line(n: u8) -> Vec<u8> {
    op::var(op::ERASE_LINE, &[S(n)])
}

/// The canvas as text: one string per row, an unwritten cell shown as `.` and a
/// space the game printed as `_`, so the two can be told apart.
fn text_rows(screen: &Screen) -> Vec<String> {
    (1..=screen.rows())
        .map(|row| {
            (1..=screen.width())
                .map(|column| match screen.cell(row, column) {
                    None => '.',
                    Some((' ', _)) => '_',
                    Some((ch, _)) => ch,
                })
                .collect()
        })
        .collect()
}

/// The styles of the canvas: for each cell `.` if unwritten, otherwise a hex digit
/// of 1 reverse, 2 bold, 4 italic, 8 fixed pitch.
fn style_rows(screen: &Screen) -> Vec<String> {
    (1..=screen.rows())
        .map(|row| {
            (1..=screen.width())
                .map(|column| match screen.cell(row, column) {
                    None => '.',
                    Some((_, s)) => {
                        let bits = s.reverse as u32
                            | (s.bold as u32) << 1
                            | (s.italic as u32) << 2
                            | (s.fixed_pitch as u32) << 3;
                        char::from_digit(bits, 16).unwrap()
                    }
                })
                .collect()
        })
        .collect()
}

/// Runs `code` then `quit` on a canvas of the given size and returns the machine
/// at the end.
fn run_on(columns: u16, rows: u16, code: &[Vec<u8>]) -> Machine {
    let mut story = Story5::new(5);
    story.main(&cat(&[cat(code), quit()]));
    let mut m = Machine::load_with(story.build(), OPTIONS, settings(columns, rows)).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    m
}

/// A canvas of ten columns and four rows after `code`, as text rows.
fn small(code: &[Vec<u8>]) -> Vec<String> {
    text_rows(run_on(10, 4, code).screen())
}

fn program(name: &str, columns: u16, rows: u16) -> (Machine, &'static [&'static str]) {
    let program = screen_programs()
        .into_iter()
        .find(|p| p.name == name)
        .unwrap();
    (
        Machine::load_with(program.image, OPTIONS, settings(columns, rows)).unwrap(),
        program.input,
    )
}

/// Answers the event with the next line of the program's input.
fn answer(m: &mut Machine, event: &Event, line: &str) {
    match event {
        Event::LineWanted { .. } => m.provide_line(line).unwrap(),
        Event::KeyWanted => m
            .provide_key(Key::Char(line.chars().next().unwrap()))
            .unwrap(),
        other => panic!("unexpected {other:?}"),
    }
}

fn blank(columns: usize, rows: usize) -> Vec<String> {
    vec![".".repeat(columns); rows]
}

// Program 1: a status line drawn the way the Inform library draws one. The whole
// row is in reverse video, and it is redrawn after a move.
#[test]
fn program_1_a_status_line() {
    let (mut m, input) = program("status_line", 40, 8);
    let event = run_until_event(&mut m, BIG);
    assert_eq!(m.screen().upper_height(), 1);
    let mut expected = blank(40, 8);
    expected[0] = "_End_Of_Road_________Score:_0__Moves:_0_".to_string();
    assert_eq!(text_rows(m.screen()), expected);
    assert_eq!(style_rows(m.screen())[0], "1".repeat(40));
    assert_eq!(m.screen().cell(1, 2), Some(('E', REVERSE)));
    assert_eq!(
        m.screen().row_text(1),
        " End Of Road         Score: 0  Moves: 0 "
    );
    // The lower window got the game's text and none of the status line.
    assert_eq!(m.take_output(), "At the road.\n\n>");

    answer(&mut m, &event, input[0]);
    run_until_event(&mut m, BIG);
    expected[0] = "_Inside_Building_____Score:_5__Moves:_1_".to_string();
    assert_eq!(text_rows(m.screen()), expected);
    assert_eq!(m.take_output(), "In the building.\n\n>");
}

// Program 2: a boxed quotation drawn the way the Inform veneer draws one. The box
// lies below the one-row upper window: it is the overlay, readable when the game
// asks for input and gone once the input is given.
#[test]
fn program_2_a_boxed_quotation_is_an_overlay() {
    let (mut m, input) = program("quotation_box", 40, 8);
    let event = run_until_event(&mut m, BIG);
    assert_eq!(
        m.screen().upper_height(),
        1,
        "shrunk back to the status line"
    );
    let mut expected = blank(40, 8);
    expected[0] = "_Y2__________________Score:_0__Moves:_0_".to_string();
    expected[2] = format!(".....{}..................", "_".repeat(17));
    expected[3] = ".....__Plugh,_he_said_..................".to_string();
    expected[4] = expected[2].clone();
    assert_eq!(text_rows(m.screen()), expected);
    let styles = style_rows(m.screen());
    assert_eq!(
        styles[3],
        format!(".....{}..................", "1".repeat(17))
    );
    assert!(m.take_screen_changed());

    // The host answers; the overlay goes, the upper window stays.
    answer(&mut m, &event, input[0]);
    assert!(m.take_screen_changed(), "the overlay was emptied");
    for row in 2..=8 {
        assert_eq!(m.screen().row_text(row), " ".repeat(40), "row {row}");
    }
    assert_eq!(
        m.screen().row_text(1),
        " Y2                  Score: 0  Moves: 0 "
    );
    run_until_event(&mut m, BIG);
    let mut after = blank(40, 8);
    after[0] = "_Y2__________________Score:_0__Moves:_1_".to_string();
    assert_eq!(text_rows(m.screen()), after);
}

// Program 3: a full-screen menu. `erase_window -1`, split to the whole screen,
// draw, wait for a key; a key is an answer like any other.
#[test]
fn program_3_a_full_screen_menu() {
    let (mut m, input) = program("menu", 40, 8);
    let event = run_until_event(&mut m, BIG);
    assert_eq!(event, Event::KeyWanted);
    assert_eq!(m.screen().upper_height(), 8);
    let runs = m.take_styled_output();
    assert!(
        runs.iter().any(|r| r.clears_window),
        "erase_window -1 marks the lower window cleared"
    );
    let mut expected = blank(40, 8);
    expected[0] = format!("_Menu_{}", ".".repeat(34));
    expected[2] = format!("..>_Instructions{}", ".".repeat(24));
    expected[3] = format!("....History{}", ".".repeat(29));
    expected[7] = format!("N_=_next__Q_=_quit{}", ".".repeat(22));
    assert_eq!(text_rows(m.screen()), expected);
    assert_eq!(&style_rows(m.screen())[0][..7], "111111.");
    assert_eq!(m.screen().cursor(), (8, 19));

    // Nothing lies below an upper window that fills the screen, so a key empties
    // nothing.
    answer(&mut m, &event, input[0]);
    let event = run_until_event(&mut m, BIG);
    expected[2] = format!(".._{}", &expected[2][3..]);
    expected[3] = format!("..>.History{}", ".".repeat(29));
    assert_eq!(text_rows(m.screen()), expected);

    answer(&mut m, &event, input[1]);
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert_eq!(text_rows(m.screen()), blank(40, 8));
    assert_eq!(m.screen().upper_height(), 0, "erase_window -1 unsplits");
    assert_eq!(m.take_output(), "Back in the game.\n");
}

// Program 4: §8.7.2.1, §15 split_window: enlarging and shrinking the upper window
// keeps what is on the screen (only version 3 clears), and a cursor left below the
// window goes to the top left.
#[test]
fn program_4_changing_the_split_keeps_contents() {
    let m = run_on(
        10,
        4,
        &[
            split(2),
            window(1),
            cursor(2, 1),
            print("two"),
            split(1),
            print("one"),
            split(3),
            cursor(3, 1),
            print("three"),
            split(0),
        ],
    );
    assert_eq!(
        text_rows(m.screen()),
        ["one.......", "two.......", "three.....", ".........."]
    );
    assert_eq!(m.screen().upper_height(), 0);
    assert_eq!(
        m.screen().cursor(),
        (1, 1),
        "below the window, so back to the top left"
    );
}

/// Fills all four rows of a ten-column canvas through a four-row upper window,
/// then leaves a two-row split with the upper window selected.
fn filled() -> Vec<Vec<u8>> {
    vec![
        split(4),
        window(1),
        print("aaaaaaaaaa"),
        cursor(2, 1),
        print("bbbbbbbbbb"),
        cursor(3, 1),
        print("cccccccccc"),
        cursor(4, 1),
        print("dddddddddd"),
        split(2),
    ]
}

// Program 5: each `erase_window` operand and `erase_line` (§8.7.3.2.1, §8.7.3.3,
// §8.7.3.4, §15).
#[test]
fn program_5_the_erases() {
    let (a, b, c, d) = ("aaaaaaaaaa", "bbbbbbbbbb", "cccccccccc", "dddddddddd");
    let empty = "..........";
    let with = |extra: Vec<Vec<u8>>| small(&[filled(), extra].concat());
    assert_eq!(with(vec![]), [a, b, c, d]);
    // 1: the upper window's rows, and its cursor to the top left.
    assert_eq!(
        with(vec![erase(1), print("x")]),
        ["x.........", empty, c, d]
    );
    // 0: the lower window, which is everything below the split.
    assert_eq!(with(vec![erase(0)]), [a, b, empty, empty]);
    // -2: everything, without unsplitting.
    let m = run_on(10, 4, &[filled(), vec![erase(-2)]].concat());
    assert_eq!(text_rows(m.screen()), [empty; 4]);
    assert_eq!(m.screen().upper_height(), 2);
    // -1: everything, unsplit, and the lower window selected: what is printed
    // next is ordinary text.
    let mut m = run_on(10, 4, &[filled(), vec![erase(-1), print("lower")]].concat());
    assert_eq!(text_rows(m.screen()), [empty; 4]);
    assert_eq!(m.screen().upper_height(), 0);
    assert_eq!(m.take_output(), "lower\n");
    // An operand that names no window does nothing.
    assert_eq!(with(vec![erase(2), erase(-3)]), [a, b, c, d]);

    // erase_line 1: from the cursor to the end of its row. Any other value: nothing.
    assert_eq!(
        with(vec![cursor(2, 4), erase_line(1)]),
        [a, "bbb.......", c, d]
    );
    assert_eq!(
        with(vec![cursor(2, 4), erase_line(0), erase_line(2)]),
        [a, b, c, d]
    );
    // In the lower window erase_line does nothing to the canvas.
    assert_eq!(with(vec![window(0), erase_line(1)]), [a, b, c, d]);
}

// Program 6: §8.7.3.1: printing in the last column is legal; what follows on that
// row is discarded; a new-line goes to the next row, and on the last row nowhere.
#[test]
fn program_6_the_right_edge_and_the_last_row() {
    let m = run_on(
        10,
        4,
        &[
            split(4),
            window(1),
            cursor(1, 8),
            print("abcdef"),
            print("\nnext"),
            cursor(4, 9),
            print("yz!!\nmore"),
        ],
    );
    assert_eq!(
        text_rows(m.screen()),
        [".......abc", "next......", "..........", "........yz"]
    );
    assert_eq!(
        m.screen().cursor(),
        (4, 11),
        "past the edge of the last row"
    );
}

// Program 7: §8.7.2.3, §15 set_cursor: row 0, column 0 and beyond the screen are brought to
// the nearest cell; in the lower window it does nothing; before any split the
// upper window has no rows but the canvas can still be drawn on.
#[test]
fn program_7_set_cursor_at_and_beyond_the_edges() {
    assert_eq!(
        small(&[
            split(4),
            window(1),
            cursor(0, 0),
            print("a"),
            cursor(99, 99),
            print("z"),
            cursor(2, 99),
            print("y"),
            cursor(99, 2),
            print("b")
        ]),
        ["a.........", ".........y", "..........", ".b.......z"]
    );
    // While the lower window is selected the cursor does not move and nothing is
    // drawn.
    let mut m = run_on(10, 4, &[split(2), cursor(2, 5), print("text")]);
    assert_eq!(text_rows(m.screen()), blank(10, 4));
    assert_eq!(m.screen().cursor(), (1, 1));
    assert_eq!(m.take_output(), "text\n");
    // Before any split: the same as an overlay with an upper window of no rows.
    let m = run_on(10, 4, &[window(1), cursor(3, 2), print("hi")]);
    assert_eq!(text_rows(m.screen())[2], ".hi.......");
    assert_eq!(m.screen().upper_height(), 0);
}

// Program 8: every style, alone and combined, in both windows; and Flags 2 bit 1,
// which forces fixed pitch (§8.1, §8.7.1).
#[test]
fn program_8_styles_in_both_windows() {
    let mut code = vec![split(1), window(1)];
    for bits in 0..16u8 {
        code.extend([style(0), style(bits), print("x")]);
    }
    code.extend([
        style(0),
        window(0),
        style(2),
        print("B"),
        style(4),
        print("I"),
        style(0),
        print("p"),
    ]);
    // Flags 2 bit 1 set: fixed pitch in both windows whatever the style.
    code.extend([
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        print("f"),
        window(1),
        print("F"),
    ]);
    let mut m = run_on(20, 2, &code);
    // Selecting the upper window again put its cursor at the top left (§8.7.2), so
    // the last character replaced the first.
    assert_eq!(style_rows(m.screen())[0], "8123456789abcdef....");
    assert_eq!(text_rows(m.screen())[0], "Fxxxxxxxxxxxxxxx....");
    let runs: Vec<(String, Style)> = m
        .take_styled_output()
        .into_iter()
        .map(|r| (r.text, r.style))
        .collect();
    assert_eq!(
        runs,
        vec![
            (
                "B".to_string(),
                Style {
                    bold: true,
                    ..PLAIN
                }
            ),
            (
                "I".to_string(),
                Style {
                    bold: true,
                    italic: true,
                    ..PLAIN
                }
            ),
            ("p".to_string(), PLAIN),
            // The program ends with the upper window selected, so no line feed
            // is added to the main text.
            (
                "f".to_string(),
                Style {
                    fixed_pitch: true,
                    ..PLAIN
                }
            ),
        ]
    );
}

// Program 9: §8.7.2.1: a split taller than the screen is the whole screen.
#[test]
fn program_9_a_split_larger_than_the_screen() {
    for asked in [4u16, 5, 24, 255, 0x7FFF, 0xFFFF] {
        let m = run_on(10, 4, &[split(asked), window(1), cursor(4, 1), print("ok")]);
        assert_eq!(m.screen().upper_height(), 4, "split_window {asked}");
        assert_eq!(text_rows(m.screen())[3], "ok........");
    }
}

// Program 10: `get_cursor` reports the upper window's cursor when it is selected,
// and row 1, column 1 for the lower window; `set_font` (§15).
#[test]
fn program_10_get_cursor_and_set_font() {
    let mut story = Story5::new(5);
    let table = story.reserve(4);
    let report = || {
        cat(&[
            op::var(op::GET_CURSOR, &[L(table)]),
            window(0),
            to(op::op2(op::LOADW, L(table), S(0)), g(0)),
            show(0),
            to(op::op2(op::LOADW, L(table), S(1)), g(0)),
            show(0),
        ])
    };
    let font = |n: u8| cat(&[to(op::ext(op::SET_FONT, &[S(n)]), g(0)), show(0)]);
    story.main(&cat(&[
        split(3),
        window(1),
        cursor(2, 7),
        report(),
        window(1),
        print("abc"),
        report(),
        print("lower text "),
        report(),
        font(0),
        font(1),
        font(3),
        font(4),
        quit(),
    ]));
    let mut m = Machine::load_with(story.build(), OPTIONS, settings(10, 4)).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert_eq!(m.take_output(), "2 7 1 4 lower text 1 1 1 1 0 0 \n");
}

// contracts/screen-model.md "What the host is told": `take_screen_changed` is true
// once after anything alters the canvas, the split or the cursor.
#[test]
fn the_host_is_told_once_of_each_change() {
    let mut story = Story5::new(5);
    story.main(&cat(&[
        read_char(g(0)),
        split(1),
        read_char(g(0)),
        window(1),
        cursor(1, 3),
        read_char(g(0)),
        print("x"),
        read_char(g(0)),
        window(0),
        print("lower"),
        read_char(g(0)),
        quit(),
    ]));
    let mut m = Machine::load_with(story.build(), OPTIONS, settings(10, 4)).unwrap();
    let mut changes = vec![];
    while run_until_event(&mut m, BIG) == Event::KeyWanted {
        changes.push(m.take_screen_changed());
        assert!(!m.take_screen_changed(), "true only once");
        m.provide_key(Key::Return).unwrap();
    }
    // Nothing yet; the split; the cursor; a character; text in the lower window.
    assert_eq!(changes, [false, true, true, true, false]);
}

// Guarantee 6 and contracts/core-api.md: a version 3 story never writes to the
// canvas, whatever it does with its windows.
#[test]
fn a_version_3_story_leaves_the_canvas_alone() {
    let mut story = Story::new();
    story.main(&cat(&[
        split(1),
        window(1),
        print("status"),
        window(0),
        print("text"),
        quit(),
    ]));
    let mut m = load(&mut story);
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert!(!m.take_screen_changed());
    assert_eq!(m.screen().upper_height(), 0);
    assert!((1..=m.screen().rows()).all(|row| m.screen().row_text(row).trim().is_empty()));
    assert_eq!((m.screen().width(), m.screen().rows()), (80, 24));
}

// §8.7.3.3 on restart: the screen is blank and unsplit again, and styles are off.
#[test]
fn restart_leaves_the_screen_blank_and_unsplit() {
    let mut story = Story5::new(5);
    // A global survives nothing, so the second pass is told apart by a key.
    story.main(&cat(&[
        read_char(g(0)),
        skip_if(op::op2(op::JE, V(g(0)), S(b'q')), false, &quit()),
        split(2),
        window(1),
        style(1),
        print("drawn"),
        op::op0(op::RESTART),
    ]));
    let mut m = Machine::load_with(story.build(), OPTIONS, settings(10, 4)).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::KeyWanted);
    m.provide_key(Key::Char('r')).unwrap();
    assert_eq!(
        run_until_event(&mut m, BIG),
        Event::KeyWanted,
        "restarted, waiting again"
    );
    assert_eq!(text_rows(m.screen()), blank(10, 4));
    assert_eq!(m.screen().upper_height(), 0);
    assert_eq!(m.screen().cursor(), (1, 1));
    assert!(m.take_screen_changed());
    m.provide_key(Key::Char('q')).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    // Text after the restart is in the lower window and plain.
    assert!(m.take_styled_output().iter().all(|run| run.style == PLAIN));
}

// §15 save_undo, restore_undo; data-model.md "Undo position": undo and restore
// return memory and the stack,
// not the screen. What was drawn after the saved position stays.
#[test]
fn undo_leaves_the_screen_as_it_is() {
    let mut story = Story5::new(5);
    story.main(&cat(&[
        to(op::ext(op::SAVE_UNDO, &[]), g(0)),
        skip_if(
            op::op2(op::JE, V(g(0)), S(2)),
            false,
            &cat(&[print("undone"), quit()]),
        ),
        split(1),
        window(1),
        print("kept"),
        window(0),
        to(op::ext(op::RESTORE_UNDO, &[]), g(1)),
        quit(),
    ]));
    let mut m = Machine::load_with(story.build(), OPTIONS, settings(10, 4)).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert_eq!(text_rows(m.screen())[0], "kept......");
    assert_eq!(m.screen().upper_height(), 1);
    assert_eq!(m.take_output(), "undone\n");
}

// ---- Cross-check against the reference interpreter (002 FR-033, T051) ----

/// The screens dfrotz printed for a program, one per time it waited for input:
/// 24 rows of text each, without dfrotz's row numbers.
fn reference_screens(name: &str) -> Vec<Vec<String>> {
    let path = format!(
        "{}/tests/fixtures/screen/{name}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let lines: Vec<&str> = text.lines().collect();
    let mut screens = vec![];
    let mut at = 0;
    while at < lines.len() {
        // Row 1 follows the echo of the previous prompt on the same line.
        let start = ["01  ", "01] "]
            .iter()
            .filter_map(|m| lines[at].rfind(m))
            .max();
        let Some(start) = start.filter(|_| at + 24 <= lines.len()) else {
            at += 1;
            continue;
        };
        let mut rows = vec![lines[at][start + 4..].to_string()];
        for (n, line) in lines[at + 1..at + 24].iter().enumerate() {
            assert!(
                line.starts_with(&format!("{:02}", n + 2)),
                "{name}: row {} of a screen: {line:?}",
                n + 2
            );
            rows.push(line.get(4..).unwrap_or("").to_string());
        }
        screens.push(rows);
        at += 24;
    }
    screens
}

// For programs 1 to 3 on dfrotz's 80 by 24 screen, the rows of the upper window
// read the same in our canvas as on dfrotz's screen each time the program waits.
// Rows below the upper window are compared only at the first wait of program 2,
// where they hold the quotation box: after that this model empties the overlay
// and Frotz's screen keeps it until it scrolls (a choice the contract marks).
// dfrotz trims trailing spaces; styles are not compared.
#[test]
fn the_upper_window_reads_as_it_does_on_the_reference_screen() {
    for program in screen_programs() {
        let screens = reference_screens(program.name);
        // dfrotz shows its screen each time the program waits, and once more at the
        // end if the program printed after its last input.
        assert!(
            screens.len() >= program.input.len(),
            "{}: one screen for each wait",
            program.name
        );
        let mut m = Machine::load_with(program.image, OPTIONS, settings(80, 24)).unwrap();
        for (n, (reference, line)) in screens.iter().zip(program.input).enumerate() {
            let event = run_until_event(&mut m, BIG);
            let screen = m.screen();
            let mut compared = 0;
            for row in 1..=screen.rows() {
                let ours = screen.row_text(row);
                let in_upper = row <= screen.upper_height();
                let overlay = program.name == "quotation_box" && n == 0 && !ours.trim().is_empty();
                if in_upper || overlay {
                    assert_eq!(
                        ours.trim_end(),
                        reference[row as usize - 1].trim_end(),
                        "{}: wait {}, row {row}",
                        program.name,
                        n + 1
                    );
                    compared += 1;
                }
            }
            assert!(compared >= 1, "{}: wait {}", program.name, n + 1);
            answer(&mut m, &event, line);
        }
    }
}
