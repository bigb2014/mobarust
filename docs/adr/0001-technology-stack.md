# ADR-0001: Technology Stack Selection

**Date:** 2026-07-22
**Status:** Accepted

## Context

MobaRust needs a technology stack that supports building a multi-protocol remote-computing client with a tabbed terminal GUI, targeting Windows x86_64 as the primary platform. The stack must be:

- Memory-safe (handling untrusted network input from SSH/Telnet/VNC/RDP)
- Capable of producing a single static binary for portable distribution
- Testable headlessly for autonomous CI/CD
- Backed by mature ecosystem crates for each protocol

## Decision

### Language: Rust (stable, pinned via rust-toolchain.toml)

Rust provides memory safety without GC, excellent FFI for system libraries, a single-binary output, and a mature async ecosystem. The MSRV is pinned to 1.97.0.

### Async runtime: tokio

The de facto standard async runtime for Rust. All network I/O, SSH, SFTP, and tunneling will be async. Blocking work (serial, PTY, crypto KDF) will use `spawn_blocking`.

### GUI framework: egui / eframe

Chosen for:
- Pure Rust, no web view dependency
- Single static binary output
- Headless testing via `egui_kittest` (essential for autonomous E2E verification)
- Strong Windows story
- Immediate-mode simplicity for rapid development

Alternative considered: Tauri (Rust + web UI + xterm.js). Rejected as default because it requires WebDriver/tauri-driver for E2E testing and adds web-view complexity. Permitted as a switch if justified by a future ADR.

### SSH: russh / russh-sftp

Apache-2.0 licensed, pure Rust SSH implementation. Mature, actively maintained, and integrates with tokio.

### Terminal: vte / alacritty_terminal

Apache-2.0 VT engine from Alacritty. `portable-pty` from WezTerm for PTY abstraction.

### RDP: IronRDP

MIT/Apache-2.0, from Devolutions. The only mature Rust RDP implementation.

### Vault: argon2id + AES-GCM

`argon2` for KDF, `aes-gcm` for AEAD encryption, `zeroize` for secret hygiene.

### Config: serde + toml

Standard Rust serialization stack. TOML for human-editable config files.

### Logging: tracing + tracing-subscriber

Structured, async-aware logging. No `println!` in library crates.

## Consequences

- All 13 crates are in a single Cargo workspace with one-way dependency direction toward `moba-core`.
- Cross-platform support is secondary; Windows is primary and must never regress.
- `egui_kittest` provides the headless UI testing path required by the autonomous workflow.
- The `e2e` feature flag gates Docker-based integration tests that require real protocol servers.