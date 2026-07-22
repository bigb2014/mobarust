//! Integration tests for the [`TextBuffer`] editor backend.

use moba_editor::TextBuffer;

#[test]
fn new_is_empty() {
    let buf = TextBuffer::new();
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.to_text(), "");
    assert_eq!(buf.cursor(), (0, 0));
    assert_eq!(buf.get_line(0), Some(""));
}

#[test]
fn from_text_to_text_roundtrip() {
    let input = "hello\nworld\nfoo";
    let buf = TextBuffer::from_text(input);
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.to_text(), input);
}

#[test]
fn from_text_empty() {
    let buf = TextBuffer::from_text("");
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.to_text(), "");
}

#[test]
fn from_text_trailing_newline() {
    // A trailing newline should not create an extra empty line.
    let buf = TextBuffer::from_text("a\nb\n");
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.to_text(), "a\nb");
}

#[test]
fn insert_char() {
    let mut buf = TextBuffer::new();
    buf.insert_char('H');
    buf.insert_char('i');
    assert_eq!(buf.to_text(), "Hi");
    assert_eq!(buf.cursor(), (0, 2));
}

#[test]
fn insert_char_newline() {
    let mut buf = TextBuffer::from_text("abc");
    buf.move_cursor(0, 1);
    buf.insert_char('\n');
    assert_eq!(buf.to_text(), "a\nbc");
    assert_eq!(buf.cursor(), (1, 0));
}

#[test]
fn insert_text_single_line() {
    let mut buf = TextBuffer::from_text("HelloWorld");
    buf.move_cursor(0, 5);
    buf.insert_text(" there ");
    assert_eq!(buf.to_text(), "Hello there World");
    assert_eq!(buf.cursor(), (0, 12));
}

#[test]
fn insert_text_multi_line() {
    let mut buf = TextBuffer::from_text("AB");
    buf.move_cursor(0, 1);
    buf.insert_text("X\nY");
    // "A" + "X" on line 0, "Y" + "B" (remainder) on line 1.
    assert_eq!(buf.to_text(), "AX\nYB");
    assert_eq!(buf.cursor(), (1, 1));
}

#[test]
fn insert_text_multi_line_three() {
    let mut buf = TextBuffer::from_text("ab");
    buf.move_cursor(0, 1);
    buf.insert_text("X\nY\nZ");
    // "a" + "X" on line 0, "Y" on line 1, "Z" + "b" (remainder) on line 2.
    assert_eq!(buf.to_text(), "aX\nY\nZb");
    assert_eq!(buf.cursor(), (2, 1));
}

#[test]
fn backspace_same_line() {
    let mut buf = TextBuffer::from_text("hello");
    buf.move_cursor(0, 3);
    buf.backspace();
    assert_eq!(buf.to_text(), "helo");
    assert_eq!(buf.cursor(), (0, 2));
}

#[test]
fn backspace_at_line_start_merges() {
    let mut buf = TextBuffer::from_text("abc\ndef");
    buf.move_cursor(1, 0);
    buf.backspace();
    assert_eq!(buf.to_text(), "abcdef");
    assert_eq!(buf.cursor(), (0, 3));
}

#[test]
fn backspace_at_origin_noop() {
    let mut buf = TextBuffer::from_text("abc");
    buf.move_cursor(0, 0);
    buf.backspace();
    assert_eq!(buf.to_text(), "abc");
    assert_eq!(buf.cursor(), (0, 0));
}

#[test]
fn delete_same_line() {
    let mut buf = TextBuffer::from_text("hello");
    buf.move_cursor(0, 1);
    buf.delete();
    assert_eq!(buf.to_text(), "hllo");
    assert_eq!(buf.cursor(), (0, 1));
}

#[test]
fn delete_at_line_end_merges() {
    let mut buf = TextBuffer::from_text("abc\ndef");
    buf.move_cursor(0, 3);
    buf.delete();
    assert_eq!(buf.to_text(), "abcdef");
    assert_eq!(buf.cursor(), (0, 3));
}

#[test]
fn delete_at_buffer_end_noop() {
    let mut buf = TextBuffer::from_text("abc");
    buf.move_cursor(0, 3);
    buf.delete();
    assert_eq!(buf.to_text(), "abc");
}

#[test]
fn cursor_movement_left() {
    let mut buf = TextBuffer::from_text("abc\ndef");
    buf.move_cursor(1, 2);
    buf.move_left();
    assert_eq!(buf.cursor(), (1, 1));
    buf.move_left();
    assert_eq!(buf.cursor(), (1, 0));
    // Wrap to previous line end.
    buf.move_left();
    assert_eq!(buf.cursor(), (0, 3));
    // No further wrapping past origin.
    buf.move_left();
    assert_eq!(buf.cursor(), (0, 2));
}

#[test]
fn cursor_movement_right() {
    let mut buf = TextBuffer::from_text("ab\ncd");
    buf.move_cursor(0, 0);
    buf.move_right();
    assert_eq!(buf.cursor(), (0, 1));
    buf.move_right();
    assert_eq!(buf.cursor(), (0, 2));
    // Wrap to next line start.
    buf.move_right();
    assert_eq!(buf.cursor(), (1, 0));
    buf.move_right();
    assert_eq!(buf.cursor(), (1, 1));
    buf.move_right();
    assert_eq!(buf.cursor(), (1, 2));
    // No further movement past buffer end.
    buf.move_right();
    assert_eq!(buf.cursor(), (1, 2));
}

#[test]
fn cursor_movement_up() {
    let mut buf = TextBuffer::from_text("longer\nab");
    buf.move_cursor(1, 2);
    buf.move_up();
    assert_eq!(buf.cursor(), (0, 2));
    // Moving up again clamps to row 0.
    buf.move_up();
    assert_eq!(buf.cursor(), (0, 2));
}

#[test]
fn cursor_movement_up_clamps_col() {
    let mut buf = TextBuffer::from_text("ab\nlonger");
    buf.move_cursor(1, 6);
    buf.move_up();
    assert_eq!(buf.cursor(), (0, 2)); // clamped to "ab".len()
}

#[test]
fn cursor_movement_down() {
    let mut buf = TextBuffer::from_text("ab\nlonger");
    buf.move_cursor(0, 2);
    buf.move_down();
    assert_eq!(buf.cursor(), (1, 2));
    buf.move_down();
    assert_eq!(buf.cursor(), (1, 2)); // already at last row
}

#[test]
fn cursor_movement_down_clamps_col() {
    let mut buf = TextBuffer::from_text("longer\nab");
    buf.move_cursor(0, 6);
    buf.move_down();
    assert_eq!(buf.cursor(), (1, 2)); // clamped to "ab".len()
}

#[test]
fn move_cursor_clamps() {
    let mut buf = TextBuffer::from_text("ab\ncd");
    buf.move_cursor(100, 100);
    assert_eq!(buf.cursor(), (1, 2));
}

#[test]
fn line_count() {
    let buf = TextBuffer::from_text("a\nb\nc\nd");
    assert_eq!(buf.line_count(), 4);
}

#[test]
fn get_line() {
    let buf = TextBuffer::from_text("alpha\nbeta\ngamma");
    assert_eq!(buf.get_line(0), Some("alpha"));
    assert_eq!(buf.get_line(1), Some("beta"));
    assert_eq!(buf.get_line(2), Some("gamma"));
    assert_eq!(buf.get_line(3), None);
}

#[test]
fn line_returns_owned() {
    let buf = TextBuffer::from_text("hello\nworld");
    assert_eq!(buf.line(0), "hello");
    assert_eq!(buf.line(1), "world");
    assert_eq!(buf.line(5), "");
}

#[test]
fn clear() {
    let mut buf = TextBuffer::from_text("some\ntext\nhere");
    buf.clear();
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.to_text(), "");
    assert_eq!(buf.cursor(), (0, 0));
}

#[test]
fn cursor_clamps_at_bounds() {
    let mut buf = TextBuffer::from_text("hi");
    // Moving cursor beyond last col clamps to line end.
    buf.move_cursor(0, 100);
    assert_eq!(buf.cursor(), (0, 2));
    // Moving cursor beyond last row clamps to last row.
    buf.move_cursor(5, 0);
    assert_eq!(buf.cursor(), (0, 0));
}

#[test]
fn insert_text_empty_noop() {
    let mut buf = TextBuffer::from_text("abc");
    buf.move_cursor(0, 1);
    buf.insert_text("");
    assert_eq!(buf.to_text(), "abc");
    assert_eq!(buf.cursor(), (0, 1));
}

#[test]
fn cursor_struct_copy_clone() {
    use moba_editor::Cursor;

    let c1 = Cursor { row: 2, col: 3 };
    let c2 = c1; // Copy
    assert_eq!(c1, c2); // PartialEq

    let c3 = c1;
    assert_eq!(c1, c3);

    assert_eq!(c1, Cursor { row: 2, col: 3 });
    assert_ne!(c1, Cursor { row: 0, col: 0 });
}

#[test]
fn editor_error_variants() {
    use moba_editor::EditorError;

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let err = EditorError::IoError(io_err);
    assert!(matches!(err, EditorError::IoError(_)));

    let err2 = EditorError::InvalidPosition;
    assert!(matches!(err2, EditorError::InvalidPosition));
}
