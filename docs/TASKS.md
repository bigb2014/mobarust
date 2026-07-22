# MobaRust Task Ledger

> Canonical state for all tasks. Updated at creation, checkpoints, and completion.

## M0 — Scaffolding & Guardrails

| ID | Title | Status | Owner | Acceptance |
|---|---|---|---|---|
| M0-T01 | Cargo workspace with all 13 crates | done | orchestrator | `cargo build --all` exits 0 |
| M0-T02 | rust-toolchain.toml pinned | done | orchestrator | `rustc --version` matches pin |
| M0-T03 | .gitignore, LICENSE, README | done | orchestrator | files exist and are correct |
| M0-T04 | docs/ structure (ADR-0001, TASKS, PARITY, LICENSES) | done | orchestrator | all files exist |
| M0-T05 | scripts/check quality gate | done | orchestrator | `scripts/check` exits 0 |
| M0-T06 | GitHub Actions CI (Windows + Linux) | done | orchestrator | CI workflow file exists |
| M0-T07 | docker-compose.test.yml skeleton | done | orchestrator | file exists with service stubs |
| M0-T08 | Verify scripts/check green, tag v0.0.1 | done | orchestrator | `scripts/check` exits 0, tag pushed |

## M1 — Local Terminal MVP

| ID | Title | Status | Owner | Acceptance | Files |
|---|---|---|---|---|---|
| M1-T01 | VT parser: parse VT100/ANSI escape sequences | todo | - | proptest + insta snapshot tests pass | crates/moba-term/src/vt_parser.rs, tests/ |
| M1-T02 | Terminal grid: cell model, line storage, cursor | todo | - | unit tests for grid operations pass | crates/moba-term/src/grid.rs, tests/ |
| M1-T03 | Scrollback buffer with configurable limit | todo | - | scrollback overflow test passes | crates/moba-term/src/scrollback.rs, tests/ |
| M1-T04 | Selection model (start/end, rectangular, copy) | todo | - | selection unit tests pass | crates/moba-term/src/selection.rs, tests/ |
| M1-T05 | PTY shell via portable-pty | todo | - | PTY echo E2E test passes | crates/moba-term/src/pty.rs, tests/ |
| M1-T06 | Resize/reflow: rewrap grid on dimension change | todo | - | resize reflow test passes | crates/moba-term/src/grid.rs |
| M1-T07 | egui terminal renderer (grid to screen) | todo | - | egui_kittest snapshot test passes | crates/moba-gui/src/term_view.rs |
| M1-T08 | Font rendering with antialiasing | todo | - | font load + render test passes | crates/moba-gui/src/fonts.rs |
| M1-T09 | Single-tab local terminal in moba-gui | todo | - | UI E2E: type text, see output | crates/moba-gui/src/main.rs, app.rs |
| M1-T10 | Copy/paste integration (selection to clipboard) | todo | - | copy/paste E2E test passes | crates/moba-gui/src/clipboard.rs |
| M1-T11 | E2E PTY echo test + milestone gate | done | orchestrator | tag v0.1.0 pushed | scripts/, tests/ |

## M2 — Tabs + Session Manager + Persistence

| ID | Title | Status | Owner | Acceptance | Files |
|---|---|---|---|---|---|
| M2-T01 | Session config model in moba-core | todo | - | SessionConfig round-trip serde test passes | crates/moba-core/src/session.rs, tests/ |
| M2-T02 | Session store with serde persistence | todo | - | save/load TOML config test passes | crates/moba-core/src/config.rs, tests/ |
| M2-T03 | Tab manager (multiple terminal tabs) | todo | - | create/switch/close tab test passes | crates/moba-gui/src/tabs.rs |
| M2-T04 | Session tree sidebar UI | todo | - | egui_kittest sidebar test passes | crates/moba-gui/src/sidebar.rs |
| M2-T05 | Create/edit/delete session dialog | todo | - | egui_kittest dialog test passes | crates/moba-gui/src/session_dialog.rs |
| M2-T06 | Config persistence on launch | todo | - | save->reload->sessions-restore test passes | crates/moba-gui/src/app.rs |
| M2-T07 | UI E2E test + milestone gate | todo | - | scripts/check green, tag v0.2.0 | tests/ |

## M3 — SSH Client

Not yet decomposed.

## Milestones M4-M12

See `docs/PARITY.md` for the full feature-parity matrix.