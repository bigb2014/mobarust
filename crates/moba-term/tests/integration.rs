//! Integration tests for the moba-term terminal engine.
//!
//! These tests exercise the full parser -> grid pipeline, including
//! proptest fuzzing and insta snapshot tests.

use moba_term::{Color, Cursor, Terminal};

// --- proptest: random byte sequences don't panic ---

proptest::proptest! {
    #[test]
    fn random_bytes_dont_panic(bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..256)) {
        let mut term = Terminal::new(24, 80);
        term.process(&bytes);
        // Just reaching here without panicking is a pass.
    }

    #[test]
    fn random_printable_bytes_dont_panic(bytes in proptest::collection::vec(0x20u8..0x7fu8, 0..256)) {
        let mut term = Terminal::new(24, 80);
        term.process(&bytes);
    }
}

// --- insta snapshot tests ---

#[test]
fn snapshot_plain_text() {
    let mut term = Terminal::new(5, 20);
    term.process_str("Hello World");
    insta::assert_snapshot!(term.grid.to_text(), @"Hello World");
}

#[test]
fn snapshot_multi_line_with_colors() {
    let mut term = Terminal::new(5, 30);
    term.process_str("\x1b[31mRed\x1b[0m\r\n\x1b[32mGreen\x1b[0m\r\n\x1b[34mBlue\x1b[0m");
    insta::assert_snapshot!(term.grid.to_text(), @r"
Red
Green
Blue
");
}

#[test]
fn snapshot_cursor_movement_and_text() {
    let mut term = Terminal::new(5, 20);
    // Write at row 0, then move to row 2 col 5 and write.
    term.process_str("Top");
    term.process_str("\x1b[3;6H");
    term.process_str("X");
    insta::assert_snapshot!(term.grid.to_text(), @r"
Top

     X
");
}

#[test]
fn snapshot_clear_screen_and_rewrite() {
    let mut term = Terminal::new(3, 10);
    term.process_str("ABC\r\nDEF\r\nGHI");
    term.process_str("\x1b[2J");
    term.process_str("\x1b[1;1H");
    term.process_str("Clean");
    insta::assert_snapshot!(term.grid.to_text(), @"Clean");
}

#[test]
fn snapshot_scroll_content() {
    let mut term = Terminal::new(3, 10);
    term.process_str("L1\r\nL2\r\nL3\r\nL4\r\nL5");
    insta::assert_snapshot!(term.grid.to_text(), @r"
L3
L4
L5
");
}

// --- Combined attribute verification ---

#[test]
fn mixed_text_and_colors_preserve_attributes() {
    let mut term = Terminal::new(1, 20);
    term.process_str("\x1b[1;31mError\x1b[0m: \x1b[32mOK\x1b[0m");

    // "E" should be bold + red.
    let e = term.grid.cell(0, 0).unwrap();
    assert_eq!(e.ch, 'E');
    assert!(e.attrs.bold);
    assert_eq!(e.attrs.fg, Color::RED);

    // ":" should be normal.
    let colon = term.grid.cell(0, 5).unwrap();
    assert_eq!(colon.ch, ':');
    assert!(!colon.attrs.bold);

    // "O" should be green.
    let o = term.grid.cell(0, 7).unwrap();
    assert_eq!(o.ch, 'O');
    assert_eq!(o.attrs.fg, Color::GREEN);
}

#[test]
fn cursor_position_after_complex_sequence() {
    let mut term = Terminal::new(10, 20);
    term.process_str("\x1b[5;10HABC\x1b[2D");
    // After "ABC" at (4,9): cursor at (4,12). 2D moves to (4,10).
    assert_eq!(term.grid.cursor(), Cursor { row: 4, col: 10 });
}
