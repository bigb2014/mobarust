//! Serial port configuration types.
//!
//! This module defines the configuration model used when opening a serial
//! port for a terminal session: baud rate, data bits, parity, stop bits, and
//! flow control. The [`SerialConfig`] struct bundles these together and
//! provides sensible defaults matching the common "9600 8N1" profile.

use serde::{Deserialize, Serialize};

/// Supported baud rates.
///
/// The enum covers the standard discrete rates used by most serial hardware.
/// Non-standard rates can be expressed via [`BaudRate::Custom`].
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaudRate {
    /// 300 baud.
    B300,
    /// 1200 baud.
    B1200,
    /// 2400 baud.
    B2400,
    /// 4800 baud.
    B4800,
    /// 9600 baud (default).
    B9600,
    /// 19200 baud.
    B19200,
    /// 38400 baud.
    B38400,
    /// 57600 baud.
    B57600,
    /// 115200 baud.
    B115200,
    /// A non-standard custom rate in bits per second.
    Custom(u32),
}

impl BaudRate {
    /// Returns the baud rate as an unsigned 32-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::BaudRate;
    /// assert_eq!(BaudRate::B9600.as_u32(), 9600);
    /// assert_eq!(BaudRate::Custom(74880).as_u32(), 74880);
    /// ```
    pub fn as_u32(&self) -> u32 {
        match self {
            BaudRate::B300 => 300,
            BaudRate::B1200 => 1200,
            BaudRate::B2400 => 2400,
            BaudRate::B4800 => 4800,
            BaudRate::B9600 => 9600,
            BaudRate::B19200 => 19200,
            BaudRate::B38400 => 38400,
            BaudRate::B57600 => 57600,
            BaudRate::B115200 => 115200,
            BaudRate::Custom(rate) => *rate,
        }
    }
}

/// Parity checking mode.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    /// No parity bit (default).
    None,
    /// Even parity.
    Even,
    /// Odd parity.
    Odd,
}

impl Parity {
    /// Returns a short string label suitable for display and "8N1"-style
    /// formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::Parity;
    /// assert_eq!(Parity::None.as_str(), "N");
    /// assert_eq!(Parity::Even.as_str(), "E");
    /// assert_eq!(Parity::Odd.as_str(), "O");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Parity::None => "N",
            Parity::Even => "E",
            Parity::Odd => "O",
        }
    }
}

/// Number of data bits per frame.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataBits {
    /// 5 data bits.
    Bits5,
    /// 6 data bits.
    Bits6,
    /// 7 data bits.
    Bits7,
    /// 8 data bits (default).
    Bits8,
}

impl DataBits {
    /// Returns the number of data bits as a `u8`.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::DataBits;
    /// assert_eq!(DataBits::Bits8.as_u8(), 8);
    /// ```
    pub fn as_u8(&self) -> u8 {
        match self {
            DataBits::Bits5 => 5,
            DataBits::Bits6 => 6,
            DataBits::Bits7 => 7,
            DataBits::Bits8 => 8,
        }
    }
}

/// Number of stop bits per frame.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopBits {
    /// 1 stop bit (default).
    Stop1,
    /// 1.5 stop bits.
    Stop1_5,
    /// 2 stop bits.
    Stop2,
}

impl StopBits {
    /// Returns a short string label suitable for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::StopBits;
    /// assert_eq!(StopBits::Stop1.as_str(), "1");
    /// assert_eq!(StopBits::Stop1_5.as_str(), "1.5");
    /// assert_eq!(StopBits::Stop2.as_str(), "2");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            StopBits::Stop1 => "1",
            StopBits::Stop1_5 => "1.5",
            StopBits::Stop2 => "2",
        }
    }
}

/// Flow control method.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowControl {
    /// No flow control (default).
    None,
    /// Hardware flow control (RTS/CTS).
    Hardware,
    /// Software flow control (XON/XOFF).
    Software,
}

impl FlowControl {
    /// Returns a short string label suitable for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::FlowControl;
    /// assert_eq!(FlowControl::None.as_str(), "none");
    /// assert_eq!(FlowControl::Hardware.as_str(), "hardware");
    /// assert_eq!(FlowControl::Software.as_str(), "software");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            FlowControl::None => "none",
            FlowControl::Hardware => "hardware",
            FlowControl::Software => "software",
        }
    }
}

/// Full configuration for a serial port connection.
///
/// A `SerialConfig` bundles the port name together with the line settings
/// (baud rate, data bits, parity, stop bits, flow control) needed to open a
/// serial terminal session. Use [`SerialConfig::new`] for the default
/// "9600 8N1, no flow control" profile.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SerialConfig {
    /// Operating-system-specific port name, e.g. `"COM3"` on Windows or
    /// `"/dev/ttyUSB0"` on Linux.
    pub port_name: String,
    /// Baud rate in bits per second.
    pub baud_rate: BaudRate,
    /// Number of data bits per frame.
    pub data_bits: DataBits,
    /// Parity mode.
    pub parity: Parity,
    /// Number of stop bits per frame.
    pub stop_bits: StopBits,
    /// Flow control method.
    pub flow_control: FlowControl,
}

impl SerialConfig {
    /// Creates a new `SerialConfig` for the given port with default line
    /// settings: 9600 baud, 8 data bits, no parity, 1 stop bit, no flow
    /// control (the classic "9600 8N1" profile).
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::{SerialConfig, BaudRate, DataBits, Parity, StopBits, FlowControl};
    ///
    /// let cfg = SerialConfig::new("COM3");
    /// assert_eq!(cfg.port_name, "COM3");
    /// assert_eq!(cfg.baud_rate, BaudRate::B9600);
    /// assert_eq!(cfg.data_bits, DataBits::Bits8);
    /// assert_eq!(cfg.parity, Parity::None);
    /// assert_eq!(cfg.stop_bits, StopBits::Stop1);
    /// assert_eq!(cfg.flow_control, FlowControl::None);
    /// ```
    pub fn new(port: &str) -> Self {
        Self {
            port_name: port.to_string(),
            baud_rate: BaudRate::B9600,
            data_bits: DataBits::Bits8,
            parity: Parity::None,
            stop_bits: StopBits::Stop1,
            flow_control: FlowControl::None,
        }
    }

    /// Returns a compact human-readable label for this configuration, e.g.
    /// `"COM3 115200 8N1"`.
    ///
    /// The format is `<port> <baud> <data><parity><stop>`, matching the
    /// conventional shorthand used by terminal applications.
    ///
    /// # Examples
    ///
    /// ```
    /// use moba_serial::config::{SerialConfig, BaudRate};
    ///
    /// let mut cfg = SerialConfig::new("COM3");
    /// cfg.baud_rate = BaudRate::B115200;
    /// assert_eq!(cfg.display_label(), "COM3 115200 8N1");
    /// ```
    pub fn display_label(&self) -> String {
        format!(
            "{} {} {}{}{}",
            self.port_name,
            self.baud_rate.as_u32(),
            self.data_bits.as_u8(),
            self.parity.as_str(),
            self.stop_bits.as_str(),
        )
    }
}

impl Default for SerialConfig {
    /// Returns the default serial configuration.
    ///
    /// This is equivalent to [`SerialConfig::new`] with an empty port name,
    /// useful when a default is required by context (e.g. deserialization
    /// fallbacks or `#[derive(Default)]` consumers).
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baud_rate_as_u32() {
        assert_eq!(BaudRate::B300.as_u32(), 300);
        assert_eq!(BaudRate::B1200.as_u32(), 1200);
        assert_eq!(BaudRate::B2400.as_u32(), 2400);
        assert_eq!(BaudRate::B4800.as_u32(), 4800);
        assert_eq!(BaudRate::B9600.as_u32(), 9600);
        assert_eq!(BaudRate::B19200.as_u32(), 19200);
        assert_eq!(BaudRate::B38400.as_u32(), 38400);
        assert_eq!(BaudRate::B57600.as_u32(), 57600);
        assert_eq!(BaudRate::B115200.as_u32(), 115200);
        assert_eq!(BaudRate::Custom(74880).as_u32(), 74880);
    }

    #[test]
    fn parity_as_str() {
        assert_eq!(Parity::None.as_str(), "N");
        assert_eq!(Parity::Even.as_str(), "E");
        assert_eq!(Parity::Odd.as_str(), "O");
    }

    #[test]
    fn data_bits_as_u8() {
        assert_eq!(DataBits::Bits5.as_u8(), 5);
        assert_eq!(DataBits::Bits6.as_u8(), 6);
        assert_eq!(DataBits::Bits7.as_u8(), 7);
        assert_eq!(DataBits::Bits8.as_u8(), 8);
    }

    #[test]
    fn stop_bits_as_str() {
        assert_eq!(StopBits::Stop1.as_str(), "1");
        assert_eq!(StopBits::Stop1_5.as_str(), "1.5");
        assert_eq!(StopBits::Stop2.as_str(), "2");
    }

    #[test]
    fn flow_control_as_str() {
        assert_eq!(FlowControl::None.as_str(), "none");
        assert_eq!(FlowControl::Hardware.as_str(), "hardware");
        assert_eq!(FlowControl::Software.as_str(), "software");
    }

    #[test]
    fn serial_config_defaults() {
        let cfg = SerialConfig::new("COM3");
        assert_eq!(cfg.port_name, "COM3");
        assert_eq!(cfg.baud_rate, BaudRate::B9600);
        assert_eq!(cfg.data_bits, DataBits::Bits8);
        assert_eq!(cfg.parity, Parity::None);
        assert_eq!(cfg.stop_bits, StopBits::Stop1);
        assert_eq!(cfg.flow_control, FlowControl::None);
    }

    #[test]
    fn display_label_format() {
        let mut cfg = SerialConfig::new("COM3");
        cfg.baud_rate = BaudRate::B115200;
        assert_eq!(cfg.display_label(), "COM3 115200 8N1");

        let mut cfg2 = SerialConfig::new("/dev/ttyUSB0");
        cfg2.baud_rate = BaudRate::B38400;
        cfg2.data_bits = DataBits::Bits7;
        cfg2.parity = Parity::Even;
        cfg2.stop_bits = StopBits::Stop2;
        assert_eq!(cfg2.display_label(), "/dev/ttyUSB0 38400 7E2");
    }

    #[test]
    fn serde_round_trip() {
        let mut cfg = SerialConfig::new("COM5");
        cfg.baud_rate = BaudRate::Custom(74880);
        cfg.parity = Parity::Odd;
        cfg.stop_bits = StopBits::Stop1_5;
        cfg.flow_control = FlowControl::Hardware;
        let json = serde_json::to_string(&cfg).expect("serialize");
        let restored: SerialConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, restored);
    }
}
