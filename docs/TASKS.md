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
| M0-T08 | Verify scripts/check green, tag v0.0.1 | in_progress | orchestrator | `scripts/check` exits 0, tag pushed |

## M1 — Local Terminal MVP

Not yet decomposed.

## M2 — Tabs + Session Manager + Persistence

Not yet decomposed.

## M3 — SSH Client

Not yet decomposed.

## Milestones M4-M12

See `docs/PARITY.md` for the full feature-parity matrix.