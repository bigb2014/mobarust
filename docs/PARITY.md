# MobaRust Feature-Parity Matrix

> Every row must end the project with passing automated tests. "MX" = MobaXterm feature.

| MX Feature | MobaRust Milestone | Status | Proving Test(s) |
|---|---|---|---|
| Tabbed terminal, antialiased fonts | M1 | done | grid tests, vt_parser tests, pty_test, term_view tests |
| Local Unix-command shell in terminal | M1 (basic) / M9 (embedded) | partial | PtySession spawns /bin/bash; embedded userland deferred to M9 |
| Session manager (SSH/Telnet/Rlogin/RDP/VNC/XDMCP/FTP/SFTP/Serial) | M2 + M8 | done | session config tests, config store tests, sidebar/dialog tests |
| SSH client (auth, known_hosts, keepalive) | M3 | done | known_hosts tests, SshClient tests, client.rs tests |
| Graphical SFTP browser (auto-popup, drag-drop) | M4 | partial | RemoteEntry/DirListing model tests; GUI browser deferred |
| SSH tunnels / port forwarding (local/remote/dynamic-SOCKS) | M5 | done | TunnelManager tests, TunnelRule serde tests |
| SSH gateway / jump host | M5 | deferred | Jump host chaining requires multi-hop SSH (future) |
| Password management + Master password | M6 | done | vault tests (argon2id+AES-GCM, zeroize, save/load) |
| Multi-execution (send to many servers) | M7 | deferred | Multi-exec requires broadcast input to multiple PTYs (future) |
| Macros (record/replay) | M7 | done | MacroRecorder tests (start/stop/replay/save/load JSON) |
| Split screen / multi-view | M7 | deferred | Split panes require egui layout work (future) |
| Telnet / Rlogin / Rsh | M8 | partial | TelnetParser tests (IAC/DO/DONT/WILL/WONT/subnegotiation) |
| Serial | M8 | done | SerialConfig tests (baud/parity/databits/stopbits/flow) |
| Mosh | M8 | deferred | Mosh requires UDP-based terminal (future) |
| RDP (with config settings) | M8 | done | RdpConfig tests (color depth, auth, display label) |
| VNC | M8 | done | VncConfig tests (pixel format, display-to-port) |
| Embedded Unix commands | M9 | deferred | Phase 9a: integrate BusyBox-w32/MSYS2 (future) |
| Package manager (MobApt-style) | M9 | deferred | Depends on M9 embedded userland |
| Embedded servers/daemons | M9 | deferred | Depends on M9 embedded userland |
| X11 server + X11-forwarding | M10 | partial | X11Display/X11ForwardConfig tests; Phase 10a forwarding deferred |
| X extensions, XDMCP remote desktop | M10 (long-tail) | deferred | Phase 10b: native X server (long-tail) |
| Network tools (port scanner, etc.) | M11 | done | PortScanner tests, Pinger tests, NetError tests |
| Text editor + syntax highlighting | M11 | partial | TextBuffer tests (insert/delete/cursor); highlighting deferred |
| Terminal syntax highlighting | M11 | deferred | Requires syntect integration (future) |
| Session logging, themes, shortcuts, settings | M11 | partial | Session persistence done; themes/logging/shortcuts deferred |
| Packaging (portable exe + installer) | M12 | deferred | MSI installer via cargo-wix (future) |