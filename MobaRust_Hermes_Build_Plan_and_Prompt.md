# MobaRust — Master Plan & Hermes Orchestration Prompt

> **What this is:** A single, self-contained brief to paste into Hermes. It contains (1) the full build plan/spec and (2) the operative orchestration instructions that tell Hermes how to run the whole project autonomously with subagents, TDD, milestone commits, and end-to-end testing.
>
> **How to use it:** Complete the "Human prerequisites" checklist below, then paste this entire document into Hermes as the top-level task. Hermes executes Part A (the spec) by following Part B (the protocol).

---

## ⚠️ Read this first (Bear — 60-second reality check)

You are asking for feature parity with a product that has been in continuous development since 2008. That is a *genuinely large* effort — an embedded X.org-class X server and a full Cygwin-style Unix userland are each multi-month subprojects on their own. This plan does **not** pretend otherwise. Instead it is structured so that:

- **Every milestone is a working, shippable app.** You never hold a broken build. Main is always green and releasable.
- **The hardest, riskiest parity items (embedded X server, embedded Unix userland) are deferred and de-risked** by first integrating existing components, with native reimplementation flagged as long-tail/optional.
- **The agent proves each feature with automated tests before moving on**, so "parity" is measured, not asserted.

Expect this to run for a long time (weeks of agent wall-clock, many subagent cycles), not minutes. That is the nature of the target. The plan makes that time productive and safe.

---

## Human prerequisites (do these once, before pasting)

Hermes cannot conjure credentials or infrastructure. Provide these so "paste and walk away" actually works:

1. **GitHub**: Create an empty repo (e.g. `mobarust`) and authenticate the machine (`gh auth login`, or set `GH_TOKEN`/`GITHUB_TOKEN` in the environment Hermes runs in). Hermes will push, tag, and cut releases via `gh`.
2. **Rust toolchain**: Install `rustup` (Hermes can install it if missing, but pre-installing is faster). Windows is the primary target — have MSVC build tools present.
3. **Docker Desktop** (WSL2 backend): Required for the end-to-end test harness (throwaway sshd/sftp/telnet/vnc/rdp/serial servers). Make sure it starts without a prompt.
4. **Model**: Confirm Hermes is pointed at GLM ("GLM 5.2") and that your subagent spawner works. Confirm your context/token limits so budget monitoring (Part B §6) has real numbers.
5. **(X11 phase only, later)**: Have VcXsrv or WSLg available so X11-forwarding can be tested end-to-end before any native X server work begins.
6. **Time/quota budget**: Set a ceiling if you have one (max subagent spawns, max spend) so Hermes stops at a known line instead of running unbounded.

If any prerequisite is missing when Hermes reaches a step that needs it, Hermes must **pause and report** rather than fabricate or skip.

---

# PART A — MASTER PLAN (the specification to build)

## A0. Mission

Build **MobaRust**: a Rust-implemented remote-computing toolbox that reaches functional parity with MobaXterm — a tabbed terminal + multi-protocol remote client (SSH/Telnet/Rlogin/RDP/VNC/XDMCP/FTP/SFTP/Serial/Mosh), graphical SFTP browser, SSH tunneling and jump-host support, X11 server + X11-forwarding, embedded Unix tooling, credential vault, macros, multi-execution, and syntax highlighting.

**Primary target OS:** Windows (x86_64). **Secondary goal:** Linux/macOS via the same codebase where dependencies allow. Do not sacrifice a working Windows build to chase cross-platform.

## A1. Intellectual-property / clean-room rules (non-negotiable)

- This is a **clean-room functional reimplementation**. Do **not** copy, decompile, or transcribe MobaXterm's proprietary source, binaries, assets, icons, or configuration files.
- Do **not** use the name "MobaXterm", the Mobatek/MobaXterm trademarks, or their logo in the product, repo, or branding. Ship under a distinct name (default: **MobaRust**, changeable).
- Reproducing *functionality and observable behavior* is fine; reproducing *their code or protected assets* is not.
- Only depend on OSS with licenses compatible with the project license (default **Apache-2.0**). Record every dependency's license in `docs/LICENSES.md`. Do not vendor GPL/AGPL code into an Apache-licensed binary.

## A2. Technology stack (defaults; deviations require an ADR)

**Language/runtime:** Rust (stable, pinned via `rust-toolchain.toml`), async on `tokio`.

**Workspace = Cargo workspace of focused crates:**

| Crate | Responsibility |
|---|---|
| `moba-core` | Shared domain types, session model, config schema, event bus, error types (`thiserror`/`anyhow`), `tracing` setup |
| `moba-term` | Terminal engine: PTY (`portable-pty`), VT parsing + grid (`vte` and/or `alacritty_terminal`), scrollback, selection, colors |
| `moba-ssh` | SSH client (`russh`), auth (password/key/agent), known_hosts, keepalive, channels, port-forwarding, SOCKS, jump hosts |
| `moba-sftp` | SFTP client (`russh-sftp`), remote FS model for the browser |
| `moba-serial` | Serial sessions (`serialport`) |
| `moba-telnet` | Telnet/Rlogin/Rsh |
| `moba-rdp` | RDP via `IronRDP` |
| `moba-vnc` | VNC client (`vnc-rs` or equivalent) |
| `moba-x11` | X11 integration (see A5 — phased; starts as forwarding + external server bridge) |
| `moba-vault` | Master-password credential vault (`argon2id` KDF + AES-GCM or `age`), zeroize on drop |
| `moba-net` | Network tools: port scan, ping, traceroute, DNS |
| `moba-editor` | Text-editor backend (syntax highlighting via `syntect`) |
| `moba-macros` | Record/replay terminal macros |
| `moba-gui` | The application shell: tabs, session tree sidebar, docked SFTP pane, split/multi-view, multi-exec, tunnel manager UI, settings, themes |

**GUI framework — default: `egui`/`eframe`.** Rationale: single static binary, pure-Rust, strong Windows story, and testable headlessly via `egui_kittest`, which is essential for the agent to self-verify UI end-to-end. The terminal grid renders through `egui` using the `moba-term` engine.

- **Permitted alternative:** Tauri (Rust backend + web UI + `xterm.js`) *only if* the agent writes an ADR justifying it and stands up `tauri-driver` + WebDriver for E2E. Do not switch frameworks mid-stream without an ADR and a green test suite on both sides of the change.

**Cross-cutting libraries:** `serde`(+`serde_json`/`toml`) config, `tracing`+`tracing-subscriber` logging, `clap` for any CLI surface, `directories` for config paths, `zeroize` for secrets.

## A3. Prior art to leverage (do not reinvent)

The agent should study and, where license-compatible, depend on:

- **WezTerm** (MIT) — closest Rust analog (GPU terminal + SSH + multiplexing). Primary architectural reference; source of `portable-pty`.
- **Alacritty** (`alacritty_terminal`, Apache-2.0) — proven VT engine.
- **russh / russh-sftp** (Apache-2.0) — SSH/SFTP.
- **IronRDP** (Devolutions, MIT/Apache) — RDP.
- **RustDesk** (AGPL — reference only, do **not** vendor) — remote-desktop UX patterns.

Reuse via dependency where the license fits; otherwise learn the approach and implement independently.

## A4. Feature-parity matrix (MobaXterm feature → MobaRust milestone)

Every row must end the project with passing automated tests. "MX" = MobaXterm's documented feature.

| MX feature | MobaRust milestone |
|---|---|
| Tabbed terminal, antialiased fonts | M1 |
| Local Unix-command shell in terminal | M1 (basic) / M9 (embedded userland) |
| Session manager (SSH/Telnet/Rlogin/RDP/VNC/XDMCP/FTP/SFTP/Serial), auto-saved, left sidebar | M2 + M8 |
| SSH client (auth, known_hosts, keepalive) | M3 |
| Graphical SFTP browser (auto-popup, drag-drop) | M4 |
| SSH tunnels / port forwarding (graphical, local/remote/dynamic-SOCKS) | M5 |
| SSH gateway / jump host (for SSH/Telnet/RDP/VNC) | M5 |
| Password management + Master password | M6 |
| Multi-execution (send to many servers) | M7 |
| Macros (record/replay) | M7 |
| Split screen / multi-view | M7 |
| Telnet / Rlogin / Rsh | M8 |
| Serial | M8 |
| Mosh | M8 |
| RDP (with config settings) | M8 |
| VNC | M8 |
| Embedded Unix commands (bash, ls, grep, awk, sed, rsync, wget, scp, ssh…) | M9 |
| Package manager (MobApt-style) | M9 |
| Embedded servers/daemons (e.g., local sshd/ftp/tftp for remote access) | M9 |
| X11 server + easy DISPLAY export + X11-forwarding | M10 |
| X extensions (OpenGL/Composite/Randr), XDMCP remote Unix desktop | M10 (long-tail) |
| Network tools (port scanner, etc.) | M11 |
| Text editor (edit remote files on double-click) + syntax highlighting | M11 |
| Terminal syntax highlighting | M11 |
| Session logging, themes/color schemes, keyboard shortcuts, settings UI | M11 |
| Packaging (portable exe + installer), plugins/add-ons, auto-update | M12 |

## A5. X11 & embedded-userland strategy (the two hard ones — read carefully)

These are the parity items most likely to sink a naive attempt. Mandatory phased approach:

- **X11 (M10):**
  1. **Phase 10a (required):** Make X11-forwarding work end-to-end over `moba-ssh` by bridging to an **existing** X server (WSLg or VcXsrv). Auto-set `DISPLAY`, tunnel X traffic over the SSH channel, and verify a remote GUI app (e.g., `xclock`/`xeyes`) renders. This delivers the *user-visible* parity feature.
  2. **Phase 10b (optional / long-tail):** Native Rust X server. Treat as a research spike with its own mini-roadmap; only start after 10a ships and only if budget allows. Not required for a "usable parity" release.
- **Embedded userland (M9):**
  1. **Phase 9a (required):** Bundle/integrate an existing portable userland (e.g., BusyBox-w32 and/or a curated MSYS2 subset) and wire the local terminal + a package fetch mechanism to it. Provide `ssh/scp/rsync/grep/awk/sed/...` availability.
  2. **Phase 9b (optional):** Deeper native reimplementation of specific tools only where it materially helps; otherwise integration is parity enough.

Both phases must be gated behind feature flags so an incomplete 10b/9b never breaks main.

## A6. Milestone roadmap (each milestone = branch → green E2E → merge → annotated tag → GitHub release)

For **every** milestone: features implemented **TDD-first**, all quality gates green (A7), docs/ADRs updated, `docs/TASKS` ledger current, then commit + `vX.Y.0` tag + release notes.

- **M0 — Scaffolding & guardrails.** Cargo workspace with all empty crates compiling; `rust-toolchain.toml`; `.gitignore`; `LICENSE` (Apache-2.0); `README`; `AGENTS.md`; `docs/adr/` + ADR-0001 (stack); `docs/TASKS.md` (ledger); local quality-gate script (`scripts/check.*`); GitHub Actions CI mirroring gates on Windows+Linux; `docker-compose.test.yml` skeleton; `tracing` + config loading. **DoD:** `build/test/clippy -D warnings/fmt --check` all green in CI; tag `v0.0.1`.
- **M1 — Local terminal MVP.** One tab, local PTY shell, VT parsing, rendered grid, scrollback, copy/paste, resize/reflow, font rendering. **Tests:** VTE golden/snapshot (`insta`), `proptest` on parser, PTY echo E2E. Tag `v0.1.0`.
- **M2 — Tabs + session manager + persistence.** Multiple tabs; left sidebar session tree; create/save/edit/delete sessions; config persisted (`serde`); reopen on launch. **Tests:** round-trip serialization snapshots; UI E2E via `egui_kittest` (create→save→reload). Tag `v0.2.0`.
- **M3 — SSH client.** `russh`; password + public-key + agent auth; `known_hosts` verification (TOFU + mismatch refusal); keepalive; terminal over SSH; resize propagation. **E2E:** dockerized `sshd`. **Tests:** host-key mismatch is rejected; auth matrix. Tag `v0.3.0`.
- **M4 — Graphical SFTP browser.** Auto-open remote-file pane on SSH connect; browse/upload/download; drag-drop; rename/mkdir/chmod/delete; progress. **E2E:** against sshd/sftp; upload→list→download→checksum-match. Tag `v0.4.0`.
- **M5 — Tunnels + jump hosts.** Graphical tunnel manager: local, remote, and dynamic (SOCKS) forwards; SSH gateway/jump-host chaining for SSH/Telnet/RDP/VNC. **E2E:** forward a port to a docker service and assert reachability; multi-hop chain. Tag `v0.5.0`.
- **M6 — Credential vault + master password.** Encrypted store (`argon2id`+AEAD), unlock flow, per-session secret binding, `zeroize`. **Tests:** wrong master password fails; at-rest ciphertext has no plaintext; no secret hits logs. Tag `v0.6.0`.
- **M7 — Multi-exec, macros, split/multi-view.** Broadcast input to selected tabs; record/replay macros; split panes and grid multi-view. **Tests:** broadcast reaches N ptys; macro replay reproduces byte stream; layout snapshots. Tag `v0.7.0`.
- **M8 — More protocols.** Telnet/Rlogin/Rsh; Serial (loopback E2E via `socat` PTY pair); Mosh; RDP (`IronRDP`, config settings) with xrdp E2E; VNC with a docker VNC server E2E. Tag `v0.8.0`.
- **M9 — Embedded userland + package manager + embedded servers.** Per A5 Phase 9a. **Tests:** `ssh/scp/rsync/grep/awk` invocable from local terminal; package fetch installs a tool; a local daemon accepts a loopback connection. Tag `v0.9.0`.
- **M10 — X11.** Per A5 Phase 10a (10b optional). **E2E:** remote `xclock`/`xeyes` renders via forwarding; XDMCP session reaches a login where feasible. Tag `v0.10.0`.
- **M11 — Tools, editor, highlighting, logging, themes, settings.** Port scanner/ping/traceroute; `MobaTextEditor`-equivalent with `syntect` highlighting and edit-remote-on-double-click; terminal syntax highlighting; session logging; themes/color schemes; global shortcuts; settings UI. Tag `v0.11.0`.
- **M12 — Packaging & polish.** Portable `.exe` + installer (`cargo-wix`/MSI); optional auto-update; plugin/add-on interface; performance pass; accessibility; docs/site. Tag `v1.0.0`.

## A7. Quality gates (every task must pass before it is "done")

A single script `scripts/check` runs and **must exit 0**:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all --release`
4. `cargo test --all` (unit + integration)
5. `cargo test --all --features e2e` (E2E; brings up docker test bed; runs headless GUI via `xvfb`/`egui_kittest`)
6. Coverage checked with `cargo llvm-cov` (target ≥70% overall, ≥85% for `moba-ssh`/`moba-vault`); mutation testing with `cargo-mutants` on `moba-vault` and `moba-ssh` at milestone gates.

CI (GitHub Actions) runs the same gates on Windows + Linux and is the backstop that keeps main green.

## A8. Testing strategy (TDD + autonomous E2E)

- **TDD everywhere:** red (write the failing test that encodes the acceptance criterion) → green (minimal code) → refactor (stay green). No production code without a preceding failing test.
- **Layers:** unit (pure logic); integration (crate boundaries); **E2E** (real protocols against throwaway servers) + **UI E2E** (`egui_kittest` driving the actual app headlessly).
- **Parsers/protocols:** `proptest` property tests + `insta` snapshots + captured golden byte streams.
- **The E2E test bed** (`docker-compose.test.yml`) provides, on demand: OpenSSH `sshd` (+SFTP), a Telnet server, TigerVNC, `xrdp`, a serial PTY pair (`socat`), and a minimal X client image (`xclock`). Tests spin it up, assert, tear it down. This is the "local copy to click through" — the agent stands it up itself; you don't.
- **Determinism:** no test may depend on the public internet or on wall-clock timing without a controllable clock; flaky tests are bugs and must be fixed or quarantined with a tracking task, never ignored.

## A9. Repository & GitHub conventions

- **Branches:** short-lived `feat/<milestone>-<task-id>`; merge to `main` only when green; `main` is always releasable.
- **Commits:** Conventional Commits (`feat:`, `fix:`, `test:`, `refactor:`, `chore:`, `docs:`). Commit at **every** green atomic task, not just milestones.
- **Milestones:** annotated tag `vX.Y.0` + `gh release create` with notes summarizing parity items delivered and their proving tests.
- **Docs:** `README` (build/run), `AGENTS.md` (rules for any agent touching the repo — this file's Part B distilled), `docs/adr/*` (every non-trivial decision), `docs/TASKS.md` (live ledger), `docs/LICENSES.md`, `docs/PARITY.md` (the A4 matrix with live status).
- **Never commit secrets.** `known_hosts`, vault files, tokens are git-ignored. CI secrets come from GitHub Actions secrets only.

---

# PART B — HERMES ORCHESTRATION PROMPT (how to execute Part A)

**You are the orchestrator.** Your job is to deliver Part A by decomposing it to atomic tasks, assigning them to subagents, verifying their work, keeping main always-green, committing at every green task, tagging/releasing at every milestone, and never letting a subagent silently run out of context ("gas"). You do not write large amounts of code yourself; you plan, dispatch, verify, integrate, and record.

## B1. Prime directives

1. **Never hand back a broken build.** Main must always pass `scripts/check`. Work on branches; merge only when green.
2. **TDD is mandatory.** Reject any subagent result that added production code without a preceding failing test. Send it back.
3. **Independent verification.** The agent that writes a feature does **not** get the final say that it works. A separate Tester/Reviewer subagent re-runs the full gate and checks the result against the task's acceptance criteria before you mark it done.
4. **Persist all state to files, not to your context.** The ledger (`docs/TASKS.md`) and ADRs are the source of truth so that any agent (including a fresh you) can resume after a restart.
5. **Respect the context budget.** Assume your model (GLM 5.2) has a finite window. Prefer file-based handoffs and terse structured summaries over long transcripts. Compact aggressively.
6. **When blocked or a prerequisite is missing, stop and report — never fabricate** credentials, test results, API behavior, or "done" status.

## B2. The task ledger (canonical state)

Maintain `docs/TASKS.md` (or `.hermes/tasks.json`). Every task record has:

```
id            e.g. M3-T07
title         one line
milestone     M0..M12
status        todo | doing | blocked | in_review | done
owner         subagent id, or "-"
depends_on    [task ids]
acceptance    the testable Definition of Done (what test proves it)
files         allowlist of paths this task may touch
budget        {max_turns, max_tokens} assigned to the subagent
spent         {turns, tokens} last reported
checkpoints   short dated progress notes
handoff       resumable "state + next step" if paused/blocked
result        summary + paths + test names that now pass
```

Update the ledger at task creation, at every subagent checkpoint, and at completion. This file is committed. It is how work survives restarts.

## B3. Atomic decomposition rule

Before assigning anything, decompose to **atomic** tasks. A task is atomic iff **all** hold:

- It has exactly one clear, testable acceptance criterion (a specific test that will pass).
- One subagent can plausibly finish it within its assigned budget (B6).
- It touches a small, enumerable file allowlist.
- It has no hidden sub-decisions that warrant their own ADR.

If any fails, **decompose further before dispatching.** Do a pre-flight size estimate; if you're unsure it fits the budget, split it. Prefer many small tasks over few large ones. Record dependencies so parallelizable tasks can run concurrently and dependent ones wait.

## B4. Subagent roles

Spawn role-scoped subagents; give each only the context it needs:

- **Planner** — expands a milestone into atomic tasks in the ledger with acceptance criteria and file allowlists. Does not write code.
- **Test-author** — writes the failing test(s) first (the "red").
- **Implementer** — writes minimal code to pass, then refactors (the "green/refactor"). Reads only its file allowlist + named references.
- **Reviewer/Tester** — independent; re-runs `scripts/check`, confirms acceptance test passes for the stated reason, checks for secret leakage and clippy/fmt, verifies ledger + docs updated. Only the Reviewer can move a task to `done`.
- **Integrator** — merges the green branch to main, commits (Conventional Commit), and at milestone boundaries creates the tag + `gh release`.
- **Fixer** — spawned on failure with the failing output as its primary context; bounded retries (B7).

## B5. TDD execution loop (per atomic task)

1. **Red:** Test-author writes the failing test encoding `acceptance`. Confirm it fails for the right reason.
2. **Green:** Implementer writes the minimum code to pass. Only the allowlisted files.
3. **Refactor:** Implementer cleans up while keeping green.
4. **Gate:** run `scripts/check`. Must be fully green.
5. **Review:** Reviewer independently re-runs the gate and validates acceptance; checks docs/ledger/ADR updates.
6. **Integrate:** Integrator merges to main and commits. Ledger → `done`.

Never skip 1. Never let 5 be done by the same subagent that did 2–3.

## B6. Budget / "don't run out of gas" protocol

This is the mechanism that keeps subagents from dying mid-task with lost work.

- **Assign a budget** (`max_turns`, `max_tokens`) to every subagent at dispatch, sized to the atomic task. If you can't size it, the task isn't atomic — split it (B3).
- **Mandatory checkpoints:** the subagent must, every few steps, write a one-paragraph structured checkpoint to its ledger record: what's done, what's next, % of budget spent, current file(s). Checkpoints go to the **ledger/files**, not back into your context as raw transcript.
- **Soft stop at 70% budget:** if not yet green at 70% spent, the subagent must stop cleanly, write a `handoff` (exact state + next concrete step), and return control. You then either (a) resume it with a fresh budget, or (b) if it stalled, **split the remaining work into smaller tasks** and reassign. Splitting on stall is the default remedy — a task that couldn't finish in budget was too big.
- **Stall detection:** if two consecutive checkpoints show no meaningful progress, treat as stalled: halt the subagent, capture its handoff, decompose, reassign.
- **Context hygiene (prevents gas waste):** subagents read only their file allowlist; use `rg`/`grep` to locate code instead of loading whole files; never paste full files back to you — return paths + a terse summary + the names of tests now passing. Decisions get written to ADRs so the reasoning can be dropped from context and re-loaded only if needed.
- **Never let a subagent "finish" by truncating scope silently.** Incomplete = `blocked`/`handoff`, with an honest note — not `done`.

## B7. Self-healing loop (fix-as-you-go, bounded)

- On any gate failure (build/test/clippy/fmt/E2E), spawn a **Fixer** with the exact failing output as primary context and a **retry cap of 3**.
- Each retry must change strategy, not just rerun. If still failing after 3, mark the task `blocked` with a written diagnosis (root-cause hypothesis + what was tried) and either decompose the fix into smaller tasks or escalate to you for a decision.
- **Flaky tests are bugs:** if a test passes/fails nondeterministically, open a tracking task and fix or quarantine it with a note — never paper over it by rerunning until green.
- Regressions: if a change breaks a previously-passing test, the change is reverted or fixed before merge. Main never regresses.

## B8. Milestone gate & release protocol

Before tagging any `vX.Y.0`:

1. All milestone tasks `done` in the ledger.
2. Full `scripts/check` green **including** `--features e2e` (docker test bed up).
3. `docs/PARITY.md` updated: each delivered feature row links to the test(s) that prove it.
4. Mutation tests pass thresholds on `moba-vault`/`moba-ssh`.
5. Integrator: merge, annotated tag, `gh release create` with notes listing parity items delivered + proving tests.
6. Commit the updated ledger and docs.

Only then start the next milestone. Report a one-screen milestone summary to Bear.

## B9. Escalation / stop conditions (pause and report to Bear)

Stop and surface — do not guess or fabricate — when:

- A **human prerequisite** is missing (no GitHub auth, Docker won't start, no X server for M10, model/limits unknown).
- A dependency is **license-incompatible** with Apache-2.0 and no compliant alternative is obvious.
- A parity feature appears to require **copying MobaXterm's protected code/assets** to match (it should not — reimplement; but if you think it does, ask).
- You hit the **budget/quota ceiling** Bear set.
- A milestone's core acceptance **cannot be met** after decomposition + bounded fixing (e.g., a chosen crate can't do the protocol). Write the ADR options and ask.

Reports are short and structured: what's blocked, why, options, your recommendation.

## B10. Kickoff sequence (do these in order, now)

1. **Verify prerequisites** (GitHub auth, Rust, Docker, model/limits). Missing → B9 report and stop.
2. Spawn **Planner** to produce M0's atomic tasks in `docs/TASKS.md`.
3. Execute **M0** end-to-end via the loops above (scaffold, gates, CI, docker test skeleton, ADR-0001, AGENTS.md distilled from Part B, PARITY.md from A4). Commit each green task; tag `v0.0.1`; report M0 summary.
4. Have Planner decompose **M1**; proceed milestone by milestone through **M12**, honoring every gate.
5. Keep the ledger, ADRs, and PARITY.md current at all times — they are how the project survives any restart of you or your subagents.

**Begin with step 1.**
