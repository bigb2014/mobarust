//! Terminal macros: record and replay terminal input sequences.
//!
//! A macro is a sequence of bytes (keystrokes) that can be recorded
//! and replayed to a terminal's PTY.

use serde::{Deserialize, Serialize};

/// A recorded macro: a named sequence of bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Macro {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The recorded byte sequence.
    pub data: Vec<u8>,
    /// Whether this macro should loop (repeat).
    pub loop_: bool,
}

impl Macro {
    /// Creates a new empty macro.
    #[must_use]
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data: Vec::new(),
            loop_: false,
        }
    }

    /// Appends bytes to the macro.
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Clears the macro data.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the byte count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the macro has no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the macro data as a byte slice.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// The macro recorder: records bytes and manages macro storage.
pub struct MacroRecorder {
    /// The macro being recorded (if any).
    recording: Option<Macro>,
    /// All saved macros.
    macros: Vec<Macro>,
}

impl MacroRecorder {
    /// Creates a new empty recorder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            recording: None,
            macros: Vec::new(),
        }
    }

    /// Starts recording a new macro.
    pub fn start(&mut self, id: &str, name: &str) {
        self.recording = Some(Macro::new(id, name));
    }

    /// Returns true if currently recording.
    #[must_use]
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Appends bytes to the current recording.
    pub fn record(&mut self, bytes: &[u8]) {
        if let Some(ref mut m) = self.recording {
            m.append(bytes);
        }
    }

    /// Stops recording and saves the macro. Returns the macro id if a recording was active.
    pub fn stop(&mut self) -> Option<String> {
        if let Some(m) = self.recording.take() {
            let id = m.id.clone();
            self.macros.push(m);
            Some(id)
        } else {
            None
        }
    }

    /// Cancels the current recording without saving.
    pub fn cancel(&mut self) {
        self.recording = None;
    }

    /// Returns all saved macros.
    #[must_use]
    pub fn macros(&self) -> &[Macro] {
        &self.macros
    }

    /// Gets a macro by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.id == id)
    }

    /// Removes a macro by id.
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.macros.len();
        self.macros.retain(|m| m.id != id);
        self.macros.len() < len
    }

    /// Replays a macro, returning its byte data.
    #[must_use]
    pub fn replay(&self, id: &str) -> Option<&[u8]> {
        self.get(id).map(|m| m.data())
    }

    /// Saves all macros to JSON.
    ///
    /// # Errors
    /// Returns a serde error if serialization fails.
    pub fn save_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.macros)
    }

    /// Loads macros from JSON.
    ///
    /// # Errors
    /// Returns a serde error if deserialization fails.
    pub fn load_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.macros = serde_json::from_str(json)?;
        Ok(())
    }
}

impl Default for MacroRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_new_is_empty() {
        let m = Macro::new("m1", "test");
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.id, "m1");
        assert_eq!(m.name, "test");
    }

    #[test]
    fn macro_append_and_data() {
        let mut m = Macro::new("m1", "test");
        m.append(b"hello");
        m.append(b" world");
        assert_eq!(m.len(), 11);
        assert_eq!(m.data(), b"hello world");
    }

    #[test]
    fn macro_clear() {
        let mut m = Macro::new("m1", "test");
        m.append(b"data");
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn macro_serde_round_trip() {
        let mut m = Macro::new("m1", "test");
        m.append(b"hello");
        let json = serde_json::to_string(&m).unwrap();
        let back: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn recorder_start_stop() {
        let mut rec = MacroRecorder::new();
        assert!(!rec.is_recording());
        rec.start("m1", "login");
        assert!(rec.is_recording());
        rec.record(b"ssh user@host\r");
        rec.record(b"password\r");
        let id = rec.stop();
        assert_eq!(id, Some("m1".to_string()));
        assert!(!rec.is_recording());
        assert_eq!(rec.macros().len(), 1);
    }

    #[test]
    fn recorder_cancel() {
        let mut rec = MacroRecorder::new();
        rec.start("m1", "test");
        rec.record(b"data");
        rec.cancel();
        assert!(!rec.is_recording());
        assert_eq!(rec.macros().len(), 0);
    }

    #[test]
    fn recorder_get_and_replay() {
        let mut rec = MacroRecorder::new();
        rec.start("m1", "login");
        rec.record(b"ssh\r");
        rec.stop();
        assert!(rec.get("m1").is_some());
        assert_eq!(rec.replay("m1"), Some(b"ssh\r".as_slice()));
        assert!(rec.get("nonexistent").is_none());
    }

    #[test]
    fn recorder_remove() {
        let mut rec = MacroRecorder::new();
        rec.start("m1", "a");
        rec.stop();
        rec.start("m2", "b");
        rec.stop();
        assert!(rec.remove("m1"));
        assert_eq!(rec.macros().len(), 1);
        assert!(!rec.remove("nonexistent"));
    }

    #[test]
    fn recorder_save_load_json() {
        let mut rec = MacroRecorder::new();
        rec.start("m1", "login");
        rec.record(b"ssh user@host\r");
        rec.stop();
        let json = rec.save_json().unwrap();
        let mut rec2 = MacroRecorder::new();
        rec2.load_json(&json).unwrap();
        assert_eq!(rec2.macros().len(), 1);
        assert_eq!(rec2.replay("m1"), Some(b"ssh user@host\r".as_slice()));
    }
}
