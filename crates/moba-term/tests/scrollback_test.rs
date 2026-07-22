//! Integration tests for the scrollback buffer.

#[path = "../src/scrollback.rs"]
mod scrollback;

use scrollback::{Line, Scrollback, Style};

/// A new scrollback buffer is empty.
#[test]
fn new_buffer_is_empty() {
    let sb = Scrollback::new();
    assert_eq!(sb.len(), 0);
    assert!(sb.is_empty());
    assert_eq!(sb.get(0), None);
}

/// Pushing a line and retrieving it works.
#[test]
fn push_and_retrieve() {
    let mut sb = Scrollback::new();
    let line: Line = vec![('H', Style::default()), ('i', Style::default())];
    sb.push(line.clone());
    assert_eq!(sb.len(), 1);
    assert!(!sb.is_empty());
    let retrieved = sb.get(0);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &line);
}

/// Pushing N lines where N > max trims the oldest.
#[test]
fn push_more_than_max_trims_oldest() {
    let max = 5;
    let mut sb = Scrollback::with_capacity(max);
    for i in 0..10 {
        let line: Line = vec![(char::from_digit(i, 10).unwrap_or('0'), Style::default())];
        sb.push(line);
    }
    // Only the last `max` lines should remain.
    assert_eq!(sb.len(), max);
    // Index 0 = most recently pushed = digit 9
    let most_recent = sb.get(0).unwrap();
    assert_eq!(most_recent[0].0, '9');
    // Oldest surviving = digit 5 (we pushed 0..10, trimmed 0..5)
    let oldest = sb.get(max - 1).unwrap();
    assert_eq!(oldest[0].0, '5');
}

/// Line count is accurate after pushes and trims.
#[test]
fn line_count_is_accurate() {
    let mut sb = Scrollback::with_capacity(3);
    assert_eq!(sb.len(), 0);
    sb.push(vec![('a', Style::default())]);
    assert_eq!(sb.len(), 1);
    sb.push(vec![('b', Style::default())]);
    assert_eq!(sb.len(), 2);
    sb.push(vec![('c', Style::default())]);
    assert_eq!(sb.len(), 3);
    sb.push(vec![('d', Style::default())]);
    assert_eq!(sb.len(), 3); // trimmed
}

/// Total lines pushed counter is tracked (including trimmed).
#[test]
fn total_lines_pushed() {
    let mut sb = Scrollback::with_capacity(3);
    assert_eq!(sb.total_pushed(), 0);
    for _ in 0..7 {
        sb.push(vec![('x', Style::default())]);
    }
    assert_eq!(sb.total_pushed(), 7);
    assert_eq!(sb.len(), 3); // only 3 in buffer
}

/// Serialization round-trip via serde.
#[test]
fn serialization_round_trip() {
    let mut sb = Scrollback::with_capacity(5);
    sb.push(vec![('a', Style::default()), ('b', Style::default())]);
    sb.push(vec![('c', Style::default())]);

    let json = serde_json::to_string(&sb).unwrap();
    let restored: Scrollback = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.len(), 2);
    assert_eq!(restored.get(0).unwrap()[0].0, 'c');
    assert_eq!(restored.get(1).unwrap()[0].0, 'a');
}

/// Default capacity is 10000.
#[test]
fn default_capacity() {
    let sb = Scrollback::new();
    assert_eq!(sb.capacity(), 10000);
}

/// Clear empties the buffer but preserves capacity.
#[test]
fn clear_empties_buffer() {
    let mut sb = Scrollback::with_capacity(5);
    sb.push(vec![('a', Style::default())]);
    sb.push(vec![('b', Style::default())]);
    assert_eq!(sb.len(), 2);
    sb.clear();
    assert_eq!(sb.len(), 0);
    assert!(sb.is_empty());
    assert_eq!(sb.capacity(), 5);
}

/// Iterating over lines yields most-recent-first.
#[test]
fn iter_yields_most_recent_first() {
    let mut sb = Scrollback::with_capacity(10);
    sb.push(vec![('a', Style::default())]);
    sb.push(vec![('b', Style::default())]);
    sb.push(vec![('c', Style::default())]);
    let chars: Vec<char> = sb.iter().map(|l| l[0].0).collect();
    assert_eq!(chars, vec!['c', 'b', 'a']);
}
