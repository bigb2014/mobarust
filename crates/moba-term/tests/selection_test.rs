//! Integration tests for the selection model.

#[path = "../src/selection.rs"]
mod selection;

use selection::{Selection, SelectionMode, TextGrid};

/// A simple test grid implementation.
struct SimpleGrid {
    rows: Vec<String>,
}

impl SimpleGrid {
    fn new(rows: &[&str]) -> Self {
        Self {
            rows: rows.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TextGrid for SimpleGrid {
    fn row_text(&self, row: usize) -> Option<&str> {
        self.rows.get(row).map(|s| s.as_str())
    }

    fn row_len(&self, row: usize) -> usize {
        self.rows.get(row).map(|s| s.len()).unwrap_or(0)
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }
}

/// Empty selection (start == end) yields empty text.
#[test]
fn empty_selection_yields_empty_text() {
    let grid = SimpleGrid::new(&["hello world"]);
    let sel = Selection::new((0, 0), (0, 0));
    assert_eq!(sel.extract_text(&grid), "");
}

/// Linear selection on a single row extracts correct text.
#[test]
fn linear_single_row() {
    let grid = SimpleGrid::new(&["hello world"]);
    let sel = Selection::new((0, 0), (0, 5));
    assert_eq!(sel.extract_text(&grid), "hello");
}

/// Linear selection spanning multiple rows extracts text with newlines.
#[test]
fn linear_multi_row() {
    let grid = SimpleGrid::new(&["abc", "def", "ghi"]);
    let sel = Selection::new((0, 1), (2, 2));
    // "bc" + newline + "def" + newline + "gh"
    assert_eq!(sel.extract_text(&grid), "bc\ndef\ngh");
}

/// Rectangular selection extracts a block of text.
#[test]
fn rectangular_selection() {
    let grid = SimpleGrid::new(&["abcdef", "ghijkl", "mnopqr"]);
    let sel = Selection::new_block((0, 1), (2, 3));
    // Cols 1..=3 (exclusive end=3 means cols 1,2)
    // Wait -- need to define semantics. End col is exclusive.
    // Rows 0..2, cols 1..3 => "bc", "hi", "no"
    assert_eq!(sel.extract_text(&grid), "bc\nhi\nno");
}

/// Reversed start/end produces the same result as normalized.
#[test]
fn reversed_start_end_normalizes() {
    let grid = SimpleGrid::new(&["hello world"]);
    let sel1 = Selection::new((0, 0), (0, 5));
    let sel2 = Selection::new((0, 5), (0, 0));
    assert_eq!(sel1.extract_text(&grid), sel2.extract_text(&grid));
    assert_eq!(sel2.extract_text(&grid), "hello");
}

/// Reversed multi-row selection normalizes correctly.
#[test]
fn reversed_multi_row_normalizes() {
    let grid = SimpleGrid::new(&["abc", "def", "ghi"]);
    let sel = Selection::new((2, 2), (0, 1));
    assert_eq!(sel.extract_text(&grid), "bc\ndef\ngh");
}

/// Serialization round-trip via serde.
#[test]
fn serialization_round_trip() {
    let sel = Selection::new((1, 2), (3, 4));
    let json = serde_json::to_string(&sel).unwrap();
    let restored: Selection = serde_json::from_str(&json).unwrap();
    assert_eq!(sel, restored);
}

/// Block mode serialization round-trip.
#[test]
fn block_mode_serialization() {
    let sel = Selection::new_block((0, 1), (2, 3));
    let json = serde_json::to_string(&sel).unwrap();
    let restored: Selection = serde_json::from_str(&json).unwrap();
    assert_eq!(sel, restored);
    assert_eq!(restored.mode(), SelectionMode::Block);
}

/// Selection mode is correctly reported.
#[test]
fn selection_mode() {
    let linear = Selection::new((0, 0), (0, 5));
    assert_eq!(linear.mode(), SelectionMode::Linear);
    let block = Selection::new_block((0, 0), (2, 3));
    assert_eq!(block.mode(), SelectionMode::Block);
}

/// Empty selection is reported correctly.
#[test]
fn is_empty_check() {
    let empty = Selection::new((0, 0), (0, 0));
    assert!(empty.is_empty());
    let nonempty = Selection::new((0, 0), (0, 1));
    assert!(!nonempty.is_empty());
}

/// Block selection on single row.
#[test]
fn block_single_row() {
    let grid = SimpleGrid::new(&["abcdef"]);
    let sel = Selection::new_block((0, 1), (0, 4));
    assert_eq!(sel.extract_text(&grid), "bcd");
}

/// Block selection with reversed corners normalizes.
#[test]
fn block_reversed_normalizes() {
    let grid = SimpleGrid::new(&["abcdef", "ghijkl"]);
    let sel1 = Selection::new_block((0, 1), (1, 3));
    let sel2 = Selection::new_block((1, 3), (0, 1));
    assert_eq!(sel1.extract_text(&grid), sel2.extract_text(&grid));
    assert_eq!(sel2.extract_text(&grid), "bc\nhi");
}

/// Normalized start/end positions.
#[test]
fn normalized_positions() {
    let sel = Selection::new((5, 10), (2, 3));
    let (start, end) = sel.normalized();
    assert_eq!(start, (2, 3));
    assert_eq!(end, (5, 10));
}
