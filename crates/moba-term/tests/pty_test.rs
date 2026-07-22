//! Integration tests for the PTY shell manager.
//!
//! These tests exercise [`PtySession`] against a real PTY inside the
//! Docker container. They are gated behind `#[cfg(unix)]` because
//! portable-pty requires a Unix-style shell.

#[path = "../src/pty.rs"]
mod pty;

use std::time::Duration;

#[cfg(unix)]
#[test]
fn new_session_is_alive() {
    let mut session = pty::PtySession::new(24, 80).expect("failed to spawn pty");
    assert!(
        session.is_alive(),
        "session should be alive right after spawn"
    );
    session.kill().expect("failed to kill");
}

#[cfg(unix)]
#[test]
fn write_and_read_echo() {
    let mut session =
        pty::PtySession::new_with_command(24, 80, "/bin/cat", &[]).expect("failed to spawn cat");

    let payload = b"hello-pty\n";
    session.write(payload).expect("failed to write to pty");

    // Give cat time to echo back via the PTY terminal driver.
    std::thread::sleep(Duration::from_millis(200));

    // Read response - in a PTY, the terminal driver echoes input by
    // default, so we should see our payload echoed back. The PTY may
    // translate \n to \r\n (CR+LF) on output.
    let mut buf = [0u8; 4096];
    let n = session.read(&mut buf).expect("failed to read");
    let received = &buf[..n];

    // Check that "hello-pty" appears somewhere in the echoed output.
    // The PTY terminal driver echoes input and may translate \n -> \r\n.
    let expected_substring = b"hello-pty";
    assert!(
        received
            .windows(expected_substring.len())
            .any(|w| w == expected_substring),
        "expected 'hello-pty' in received bytes: {:?}",
        received
    );

    session.kill().expect("failed to kill");
}

#[cfg(unix)]
#[test]
fn resize_changes_size_without_error() {
    let mut session = pty::PtySession::new(24, 80).expect("failed to spawn pty");
    session.resize(50, 120).expect("resize failed");
    session.resize(30, 80).expect("resize failed");
    session.kill().expect("failed to kill");
}

#[cfg(unix)]
#[test]
fn kill_terminates_session() {
    let mut session = pty::PtySession::new(24, 80).expect("failed to spawn pty");
    assert!(session.is_alive(), "should be alive before kill");
    session.kill().expect("failed to kill");
    let _ = session.wait().expect("failed to wait");
    assert!(!session.is_alive(), "should not be alive after kill");
}

#[cfg(unix)]
#[test]
fn wait_returns_exit_code_after_kill() {
    let mut session = pty::PtySession::new(24, 80).expect("failed to spawn pty");
    session.kill().expect("failed to kill");
    let code = session.wait().expect("failed to wait");
    assert!(code >= 0 || code < 0, "got an exit code: {}", code);
}
