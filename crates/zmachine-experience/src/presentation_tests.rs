//! The presentation file, read and refused
//! (specs/017-authored-map/contracts/presentation.md, part F, cited F§).

use crate::presentation::Presentation;

const HEAD: &str = "presentation: 1\nsha256: 3f2a000000000000000000000000000000000000000000000000000000000000\nrelease: 119\nserial: 880429\n";

fn with(lines: &str) -> String {
    format!("{HEAD}{lines}")
}

// F§1 to F§3: the identity, sheets with and without a name, and places.
#[test]
fn a_file_of_the_contracts_form_is_read() {
    let text = with(
        "# a comment\n\nsheet: 1 Above Ground\nsheet: 2\nplace: 180 1 0 0\nplace: 81 1 0 -1\r\nplace: 9 2 -3 4\n",
    );
    let file = Presentation::parse(&text).unwrap();
    assert_eq!(file.sha256, format!("3f2a{}", "0".repeat(60)));
    assert_eq!((file.release, &file.serial), (119, b"880429"));
    assert_eq!(file.sheets.get(&1), Some(&Some("Above Ground".to_string())));
    assert_eq!(file.sheets.get(&2), Some(&None));
    assert_eq!(file.places.get(&180), Some(&(1, (0, 0))));
    assert_eq!(file.places.get(&81), Some(&(1, (0, -1))));
    assert_eq!(file.places.get(&9), Some(&(2, (-3, 4))));
    assert_eq!(file.places.len(), 3);
}

// F§1 to F§5: every departure is refused, with the line it is on.
#[test]
fn a_file_out_of_form_is_refused_whole() {
    let good = "sheet: 1 Above Ground\nplace: 180 1 0 0\n";
    for (text, line) in [
        // The identity lines, in their order.
        (good.to_string(), 1),
        (HEAD.replace("presentation: 1", "presentation: 2") + good, 1),
        (HEAD.replace("release: 119\n", "") + good, 3),
        (HEAD.replace("serial: 880429", "serial: 88042") + good, 4),
        (HEAD.replace("sha256: 3f2a", "sha256: 3F2A") + good, 2),
        // A sheet given twice, out of range, or with too long a name.
        (with("sheet: 1\nsheet: 1 Again\n"), 6),
        (with("sheet: 0\n"), 5),
        (with("sheet: 100\n"), 5),
        (with(&format!("sheet: 1 {}\n", "x".repeat(41))), 5),
        // A place on a sheet not declared, given twice, out of range, or
        // not numbers.
        (with("place: 180 1 0 0\n"), 5),
        (with("sheet: 1\nplace: 180 1 0 0\nplace: 180 1 1 0\n"), 7),
        (with("sheet: 1\nplace: 180 1 100 0\n"), 6),
        (with("sheet: 1\nplace: 180 1 0\n"), 6),
        (with("sheet: 1\nplace: kitchen 1 0 0\n"), 6),
        (with("sheet: 1\nplace: 0 1 0 0\n"), 6),
        // Two places in one position of one sheet.
        (with("sheet: 1\nplace: 180 1 2 2\nplace: 81 1 2 2\n"), 7),
        // A key the form does not have, and a line that is no key at all.
        (with("sheet: 1\nlabel: 180 West of House\n"), 6),
        (with("sheet: 1\nplace 180 1 0 0\n"), 6),
    ] {
        match Presentation::parse(&text) {
            Ok(file) => panic!("read as {file:?}:\n{text}"),
            Err(error) => assert_eq!(error.line, line, "{}:\n{text}", error.what),
        }
    }
    // The same position on two sheets is two positions.
    assert!(
        Presentation::parse(&with(
            "sheet: 1\nsheet: 2\nplace: 180 1 2 2\nplace: 81 2 2 2\n"
        ))
        .is_ok()
    );
    // A file may have no places at all.
    assert!(Presentation::parse(HEAD).is_ok());
}
