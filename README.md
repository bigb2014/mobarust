# MobaRust

A Rust-implemented remote-computing toolbox targeting functional parity with MobaXterm: tabbed terminal + multi-protocol remote client (SSH/SFTP/Telnet/RDP/VNC/Serial/Mosh), graphical SFTP browser, SSH tunneling and jump-host support, X11 forwarding, credential vault, macros, multi-execution, and syntax highlighting.

**Primary target OS:** Windows (x86_64). **Secondary:** Linux/macOS where dependencies allow.

## License

Apache-2.0. See [LICENSE](LICENSE).

## Clean-room notice

MobaRust is a clean-room functional reimplementation. It does not copy, decompile, or transcribe MobaXterm's proprietary source, binaries, assets, icons, or configuration files. Only functionality and observable behavior are reproduced.

## Building

```bash
# Build all crates
cargo build --all --release

# Run the quality gate (fmt + clippy + build + test)
scripts/check
```

## Development container

A Docker-based development environment is provided for reproducible builds:

```bash
docker compose -f docker-compose.dev.yml up -d
docker exec -it mobarust-dev bash
```

## Workspace layout

| Crate | Responsibility |
|---|---|
| `moba-core` | Domain types, session model, config schema, errors, event bus, tracing |
| `moba-term` | PTY, VT parsing, grid, scrollback, selection |
| `moba-ssh` | SSH client, auth, known_hosts, channels, forwarding, SOCKS, jump hosts |
| `moba-sftp` | SFTP client, remote FS model |
| `moba-serial` | Serial sessions |
| `moba-telnet` | Telnet/Rlogin/Rsh |
| `moba-rdp` | RDP (IronRDP) |
| `moba-vnc` | VNC client |
| `moba-x11` | X11 forwarding + external-server bridge |
| `moba-vault` | Master-password credential vault |
| `moba-net` | Port scan, ping, traceroute, DNS |
| `moba-editor` | Text editor backend, highlighting |
| `moba-macros` | Record/replay macros |
| `moba-gui` | App shell (egui): tabs, session tree, SFTP pane, tunnel UI, settings |

## Documentation

- [AGENTS.md](AGENTS.md) - Rules for AI agents working in this repository
- [docs/TASKS.md](docs/TASKS.md) - Task ledger
- [docs/PARITY.md](docs/PARITY.md) - Feature-parity matrix
- [docs/adr/](docs/adr/) - Architecture Decision Records
- [docs/LICENSES.md](docs/LICENSES.md) - Dependency licenses