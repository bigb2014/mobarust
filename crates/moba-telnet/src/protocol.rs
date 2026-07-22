//! Telnet protocol implementation.
//!
//! Handles Telnet option negotiation (IAC/DO/DONT/WILL/WONT),
//! line-mode buffering, and basic connectivity.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// Telnet IAC (Interpret As Command) byte.
const IAC: u8 = 0xFF;
/// Telnet DO command.
const DO: u8 = 0xFD;
/// Telnet DONT command.
const DONT: u8 = 0xFE;
/// Telnet WILL command.
const WILL: u8 = 0xFB;
/// Telnet WONT command.
const WONT: u8 = 0xFC;
/// Telnet SB (subnegotiation begin).
const SB: u8 = 0xFA;
/// Telnet SE (subnegotiation end).
const SE: u8 = 0xF0;

/// Telnet command options.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelnetCommand {
    /// IAC DO option.
    Do(u8),
    /// IAC DONT option.
    Dont(u8),
    /// IAC WILL option.
    Will(u8),
    /// IAC WONT option.
    Wont(u8),
    /// IAC SB ... SE subnegotiation.
    Subnegotiation(u8, Vec<u8>),
}

/// A Telnet protocol parser that processes a byte stream and separates
/// text data from Telnet commands.
pub struct TelnetParser {
    /// Pending data (text) to be read.
    data: VecDeque<u8>,
    /// Buffered commands for inspection.
    commands: Vec<TelnetCommand>,
    /// Parser state machine.
    state: ParseState,
    /// Current subnegotiation option.
    sb_option: u8,
    /// Current subnegotiation data buffer.
    sb_data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ParseState {
    /// Normal data.
    Data,
    /// Got IAC, waiting for command byte.
    Iac,
    /// Got IAC DO, waiting for option.
    DoOpt,
    /// Got IAC DONT, waiting for option.
    DontOpt,
    /// Got IAC WILL, waiting for option.
    WillOpt,
    /// Got IAC WONT, waiting for option.
    WontOpt,
    /// In subnegotiation, waiting for IAC or SE.
    Sb,
    /// In subnegotiation, got IAC, waiting for SE or data.
    SbIac,
}

impl TelnetParser {
    /// Creates a new Telnet parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
            commands: Vec::new(),
            state: ParseState::Data,
            sb_option: 0,
            sb_data: Vec::new(),
        }
    }

    /// Feeds raw bytes into the parser.
    pub fn feed(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            match self.state {
                ParseState::Data => {
                    if byte == IAC {
                        self.state = ParseState::Iac;
                    } else {
                        self.data.push_back(byte);
                    }
                }
                ParseState::Iac => {
                    match byte {
                        DO => self.state = ParseState::DoOpt,
                        DONT => self.state = ParseState::DontOpt,
                        WILL => self.state = ParseState::WillOpt,
                        WONT => self.state = ParseState::WontOpt,
                        SB => {
                            self.state = ParseState::Sb;
                            self.sb_data.clear();
                        }
                        IAC => {
                            // Escaped IAC byte in data stream.
                            self.data.push_back(IAC);
                            self.state = ParseState::Data;
                        }
                        _ => {
                            // Unknown command, ignore.
                            self.state = ParseState::Data;
                        }
                    }
                }
                ParseState::DoOpt => {
                    self.commands.push(TelnetCommand::Do(byte));
                    self.state = ParseState::Data;
                }
                ParseState::DontOpt => {
                    self.commands.push(TelnetCommand::Dont(byte));
                    self.state = ParseState::Data;
                }
                ParseState::WillOpt => {
                    self.commands.push(TelnetCommand::Will(byte));
                    self.state = ParseState::Data;
                }
                ParseState::WontOpt => {
                    self.commands.push(TelnetCommand::Wont(byte));
                    self.state = ParseState::Data;
                }
                ParseState::Sb => {
                    if byte == IAC {
                        self.state = ParseState::SbIac;
                    } else {
                        if self.sb_data.is_empty() {
                            self.sb_option = byte;
                        }
                        self.sb_data.push(byte);
                    }
                }
                ParseState::SbIac => {
                    if byte == SE {
                        // End of subnegotiation. sb_data[0] is the option.
                        let opt = self.sb_data.remove(0);
                        self.commands
                            .push(TelnetCommand::Subnegotiation(opt, self.sb_data.clone()));
                        self.sb_data.clear();
                        self.state = ParseState::Data;
                    } else {
                        // IAC inside SB data — add it to the buffer.
                        self.sb_data.push(byte);
                        self.state = ParseState::Sb;
                    }
                }
            }
        }
    }

    /// Returns available data bytes as a Vec (copies).
    #[must_use]
    pub fn data_vec(&self) -> Vec<u8> {
        self.data.iter().copied().collect()
    }

    /// Drains and returns available data bytes.
    pub fn drain_data(&mut self) -> Vec<u8> {
        self.data.drain(..).collect()
    }

    /// Returns true if data is available.
    #[must_use]
    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }

    /// Drains and returns parsed commands.
    pub fn drain_commands(&mut self) -> Vec<TelnetCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Returns true if commands are available.
    #[must_use]
    pub fn has_commands(&self) -> bool {
        !self.commands.is_empty()
    }

    /// Resets the parser state.
    pub fn reset(&mut self) {
        self.data.clear();
        self.commands.clear();
        self.state = ParseState::Data;
        self.sb_data.clear();
    }
}

impl Default for TelnetParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_passes_through() {
        let mut p = TelnetParser::new();
        p.feed(b"Hello World");
        assert_eq!(p.drain_data(), b"Hello World");
    }

    #[test]
    fn iac_do_parsed() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, DO, 3]); // DO Suppress Go Ahead
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], TelnetCommand::Do(3));
    }

    #[test]
    fn iac_will_parsed() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, WILL, 1]); // WILL Echo
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], TelnetCommand::Will(1));
    }

    #[test]
    fn iac_dont_parsed() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, DONT, 24]); // DONT Terminal Type
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], TelnetCommand::Dont(24));
    }

    #[test]
    fn iac_wont_parsed() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, WONT, 31]); // WONT Window Size
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], TelnetCommand::Wont(31));
    }

    #[test]
    fn escaped_iac_in_data() {
        let mut p = TelnetParser::new();
        p.feed(&[b'A', IAC, IAC, b'B']);
        assert_eq!(p.drain_data(), vec![b'A', IAC, b'B']);
    }

    #[test]
    fn subnegotiation_parsed() {
        let mut p = TelnetParser::new();
        // IAC SB 24 1 IAC SE -- Terminal Type, SEND
        p.feed(&[IAC, SB, 24, 1, IAC, SE]);
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            TelnetCommand::Subnegotiation(opt, data) => {
                assert_eq!(*opt, 24);
                assert_eq!(data, &[1]);
            }
            _ => panic!("expected subnegotiation"),
        }
    }

    #[test]
    fn mixed_text_and_commands() {
        let mut p = TelnetParser::new();
        p.feed(b"Hello ");
        p.feed(&[IAC, WILL, 1]);
        p.feed(b" World");
        assert_eq!(p.drain_data(), b"Hello  World");
        let cmds = p.drain_commands();
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn reset_clears_state() {
        let mut p = TelnetParser::new();
        p.feed(b"data");
        p.feed(&[IAC, DO, 1]);
        p.reset();
        assert!(!p.has_data());
        assert!(!p.has_commands());
    }

    #[test]
    fn has_data_and_commands_flags() {
        let mut p = TelnetParser::new();
        assert!(!p.has_data());
        assert!(!p.has_commands());
        p.feed(b"text");
        p.feed(&[IAC, WILL, 1]);
        assert!(p.has_data());
        assert!(p.has_commands());
    }
}
