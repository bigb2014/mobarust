//! Integration tests for the moba-serial configuration model.

use moba_serial::config::{BaudRate, DataBits, FlowControl, Parity, SerialConfig, StopBits};

/// Verifies `BaudRate::as_u32` for every variant, including `Custom`.
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

/// Verifies `Parity::as_str` for all three variants.
#[test]
fn parity_as_str() {
    assert_eq!(Parity::None.as_str(), "N");
    assert_eq!(Parity::Even.as_str(), "E");
    assert_eq!(Parity::Odd.as_str(), "O");
}

/// Verifies `DataBits::as_u8` for all four variants.
#[test]
fn data_bits_as_u8() {
    assert_eq!(DataBits::Bits5.as_u8(), 5);
    assert_eq!(DataBits::Bits6.as_u8(), 6);
    assert_eq!(DataBits::Bits7.as_u8(), 7);
    assert_eq!(DataBits::Bits8.as_u8(), 8);
}

/// Verifies `StopBits::as_str` for all three variants.
#[test]
fn stop_bits_as_str() {
    assert_eq!(StopBits::Stop1.as_str(), "1");
    assert_eq!(StopBits::Stop1_5.as_str(), "1.5");
    assert_eq!(StopBits::Stop2.as_str(), "2");
}

/// Verifies `FlowControl::as_str` for all three variants.
#[test]
fn flow_control_as_str() {
    assert_eq!(FlowControl::None.as_str(), "none");
    assert_eq!(FlowControl::Hardware.as_str(), "hardware");
    assert_eq!(FlowControl::Software.as_str(), "software");
}

/// Verifies `SerialConfig::new` produces the default 9600 8N1 profile.
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

/// Verifies `SerialConfig::display_label` formats correctly across profiles.
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

/// Verifies a `SerialConfig` survives a serde JSON round trip intact.
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
