# AGENTS.md — Rules for Agents Working in MobaRust

**You are an AI coding agent operating on this repository.** Read this file fully before touching anything. These rules are binding. When a rule here conflicts with a task instruction, this file wins unless the task explicitly says "override AGENTS.md §X". The Reviewer subagent enforces this file and will reject work that violates it.

This file assumes the orchestration protocol in the project brief (Hermes orchestrator + subagents, TDD, milestone commits). It exists to keep `main` always-green, keep diffs safe, and catch the specific ways a large open-weight coding model (GLM 5.2) tends to fail.

---

## 0. The ten golden rules (memorize these)

1. **Never claim done without running it.** Paste the real command output. "This should pass" is a violation.
2. **TDD only.** No production code exists before a failing test that will pass because of it.
3. **No stubs in `done` code.** `todo!()`, `unimplemented!()`, placeholder returns, and `// TODO` in a completed task are failures, not progress.
4. **`main` is always releasable.** Work on branches; merge only when `scripts/check` is fully green.
5. **Minimal diffs.** Touch only files in your task's allowlist. Do not "drive-by" refactor.
6. **Verify every crate/API before you use it.** No hallucinated crates, methods, or signatures. Confirm against `cargo doc`/docs.rs.
7. **No `.unwrap()`/`.expect()`/`panic!` in non-test code** (narrow, documented exceptions only — §4.1).
8. **No `unsafe` without an ADR** and a `// SAFETY:` proof comment.
9. **English + ASCII in all code, comments, commits, and logs.** No language leakage.
10. **When blocked or a prerequisite is missing, stop and report.** Never fabricate credentials, output, or status.

---

## 1. What this project is

MobaRust is a Rust remote-computing toolbox targeting functional parity with a MobaXterm-class client (tabbed terminal + SSH/SFTP/Telnet/RDP/VNC/Serial/Mosh, tunnels, jump hosts, X11 forwarding, credential vault, macros, multi-exec). Primary target: **Windows x86_64**. License: **Apache-2.0**. Clean-room: reimplement behavior, never copy MobaXterm code, assets, name, or trademarks.

---

## 2. Repository map (orient here before searching)

Cargo workspace. Each crate has one job. Respect the boundaries — cross-crate leakage is a review failure.

| Crate | Owns | May depend on |
|---|---|---|
| `moba-core` | domain types, session model, config schema, errors, event bus, tracing setup | — |
| `moba-term` | PTY, VT parsing, grid, scrollback, selection | core |
| `moba-ssh` | SSH client, auth, known_hosts, channels, forwarding, SOCKS, jump hosts | core |
| `moba-sftp` | SFTP client, remote FS model | core, ssh |
| `moba-serial` | serial sessions | core |
| `moba-telnet` | telnet/rlogin/rsh | core |
| `moba-rdp` | RDP (IronRDP) | core |
| `moba-vnc` | VNC client | core |
| `moba-x11` | X11 forwarding + external-server bridge (phased) | core, ssh |
| `moba-vault` | master-password credential vault | core |
| `moba-net` | port scan, ping, traceroute, DNS | core |
| `moba-editor` | text editor backend, highlighting | core |
| `moba-macros` | record/replay | core, term |
| `moba-gui` | app shell (egui): tabs, session tree, SFTP pane, tunnel UI, settings, themes | all above |

**Dependency direction is one-way toward `moba-core`.** A lower crate must never depend on `moba-gui`. If you think you need an upward dependency, you have a design bug — write an ADR, don't force it.

Key docs you must keep current: `docs/TASKS.md` (ledger), `docs/PARITY.md` (feature→test status), `docs/adr/` (decisions), `docs/LICENSES.md` (dependency licenses).

---

## 3. Environment & commands (the only commands that count)

Toolchain is pinned in `rust-toolchain.toml`. Do not change it without an ADR.

```bash
# Format (must be clean)
cargo fmt --all -- --check

# Lint (warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --all --release

# Unit + integration tests
cargo test --all

# End-to-end tests (spins up the docker test bed; headless GUI via egui_kittest/xvfb)
cargo test --all --features e2e

# Coverage
cargo llvm-cov --all --summary-only

# One command that runs the whole gate and must exit 0:
scripts/check
```

**`scripts/check` exiting 0 is the definition of a green build.** Never report success on the basis of a partial run. If a command is slow, run it anyway — do not skip it and assume.

Dependency changes go through `cargo add <crate>@<version>` (which verifies the crate exists and resolves), never by hand-typing into `Cargo.toml` from memory. Commit `Cargo.lock`.

---

## 4. Rust coding standards

### 4.1 Error handling
- **Library crates** (`moba-*` except `moba-gui`) return `Result<T, ThisCrateError>` where the error is a `thiserror`-derived enum. No `anyhow` in library public APIs.
- **`moba-gui`** (the binary) may use `anyhow::Result` at the top level and `.context(...)` for messages.
- **Forbidden in all non-test code:** `.unwrap()`, `.expect()`, `panic!`, `unreachable!`, `unimplemented!`, `todo!`, array indexing that can panic on untrusted input, `.unwrap_or_default()` used to swallow real errors.
  - **Narrow exception:** `.expect("<invariant that is provably true here>")` is allowed *only* when the invariant is guaranteed by construction and the message states why. Prefer restructuring so it isn't needed. Reviewer must be convinced.
- Propagate with `?`. Convert errors with `#[from]` or `map_err`. Never discard an error with `let _ =` unless there is a comment explaining why it is safe to ignore.
- User-facing failures degrade gracefully (show an error in the UI); they never take down the app.

### 4.2 `unsafe`
- Default: none. Any `unsafe` block requires (a) an ADR justifying it, (b) a `// SAFETY:` comment proving the invariants hold, (c) tests exercising it. FFI to system libraries counts.

### 4.3 Async / concurrency
- Runtime is `tokio`. **Never block the async executor.** No `std::thread::sleep`, no synchronous file/network/serial IO, no CPU-heavy loops inside an `async fn` on the runtime.
- Wrap blocking work (serial reads, heavy filesystem, PTY spawn, crypto KDF) in `tokio::task::spawn_blocking` or a dedicated thread with a channel.
- Use `tokio::time` for delays/timeouts. Every network operation has a timeout.
- No shared mutable state without a synchronization primitive. Prefer message passing (channels) over shared locks. If you use a lock, hold it for the shortest possible scope and never across an `.await`.

### 4.4 Dependency & API verification (anti-hallucination)
This is the single most important habit for a model of this class.
- Before calling any external API, **confirm the item exists and its exact signature** via `cargo doc -p <crate> --open` or docs.rs. Do not infer method names from what "should" exist.
- If the compiler says `no method named X` / `no function Y in module Z`, **do not invent a replacement name and retry blindly.** Open the docs, find the real API, then fix. Repeated fantasy-API retries are a stop-and-report condition.
- Pin dependencies to specific versions; do not float to `*`. When adding a dep, record its license in `docs/LICENSES.md` and confirm Apache-2.0 compatibility (no GPL/AGPL vendored).
- Prefer the reference crates named in the brief (`russh`, `russh-sftp`, `portable-pty`, `alacritty_terminal`/`vte`, `IronRDP`, `syntect`, `serialport`, `egui`/`eframe`). Introducing a different crate for the same job needs an ADR.

### 4.5 Structure, size, naming
- One responsibility per module. Split when a file exceeds ~500 lines or a function exceeds ~60 lines (guidelines; Reviewer flags egregious cases). God-modules and 300-line `match` arms are review failures.
- Public items have doc comments (`///`) stating intent, errors, and panics (there should be no panics). Private complexity gets a `//` explaining *why*, not *what*.
- Naming: `snake_case` items, `CamelCase` types, `SCREAMING_SNAKE_CASE` consts. Names describe intent, not type (`retry_budget`, not `u32_val`).
- No `dbg!`, no `println!`/`eprintln!` in library crates. Use `tracing` (`trace!/debug!/info!/warn!/error!`). The binary may print only for CLI UX.
- Do not `#[allow(...)]` to silence clippy/warnings. Fix the cause. A genuinely wrong lint is disabled *at the narrowest scope* with a comment explaining why, and flagged to the Reviewer.
- No commented-out code left behind. No meta-commentary, markdown fences, or explanatory prose inside `.rs` files — source files contain code and doc comments only.

### 4.6 Secrets
- Secrets (passwords, keys, tokens, vault contents) are `zeroize`d on drop, never logged (not even at `trace`), never written to `known_hosts`/config in plaintext, never placed in error messages or panics. `moba-vault` is the only place that handles master-password-derived keys.

---

## 5. Testing rules (TDD + test integrity)

### 5.1 The loop (mandatory, in order)
1. **Red** — write the failing test that encodes the task's acceptance criterion. Confirm it fails *for the intended reason* (not a compile error in the test).
2. **Green** — minimum production code to pass.
3. **Refactor** — clean up, keep green.

Production code without a preceding failing test is rejected on sight.

### 5.2 Test integrity (anti-reward-hacking — read twice)
A model under pressure to turn the bar green will cheat if allowed. It is not allowed. All of the following are **serious violations**, treated as worse than an honest red build:
- Weakening, deleting, or `#[ignore]`-ing a test to make the suite pass. (`#[ignore]` is permitted **only** with a linked tracking task ID in a comment and Reviewer sign-off.)
- Changing a test's expected value to match wrong output instead of fixing the code.
- Hardcoding a function to return exactly what the test checks (`if input == test_case { return expected }`).
- Asserting trivially true things (`assert!(true)`, comparing a value to itself) to inflate coverage.
- Catching-and-ignoring the very error the test should surface.
- Sleeping/retrying until a flaky test happens to pass instead of fixing the race.

If a test is genuinely wrong, fix it **and say so explicitly** in the task result with the reasoning, so the Reviewer can validate the change of intent. Silent test edits are the top thing the Reviewer hunts for.

### 5.3 Kinds & tools
- Unit tests for pure logic; integration tests across crate boundaries; **E2E** (`--features e2e`) against the docker test bed (sshd/sftp/telnet/vnc/xrdp/serial-via-socat/x-client); **UI E2E** via `egui_kittest` driving the real app headlessly.
- Parsers/protocols: `proptest` (properties) + `insta` (snapshots) + captured golden byte streams. Regenerate `insta` snapshots only when the change is intended and reviewed.
- Determinism: no test depends on the public internet or on real wall-clock timing; inject clocks. Flaky tests are bugs — fix or quarantine with a tracking task, never rerun-until-green.
- Coverage targets: ≥70% overall, ≥85% for `moba-ssh` and `moba-vault`. `cargo-mutants` must pass thresholds on `moba-vault` and `moba-ssh` at milestone gates.

---

## 6. Agentic workflow rules

- **Read before you edit.** Use `rg` (ripgrep) to find existing code before writing new code. Duplicating an existing function is a review failure. Never load the whole repo — read your allowlist plus what `rg` points you to.
- **Stay in your allowlist.** Your task record lists the files you may touch. Touching others = rejected. If the task genuinely needs another file, update the task/ledger and get it into the allowlist first.
- **Ledger is truth.** Update `docs/TASKS.md` at start, at each checkpoint, and at completion. Persist decisions to `docs/adr/`. Do not keep important state only in your context — assume your context can be lost at any moment.
- **Checkpoints.** Emit a short structured progress note to your task record every few steps: done / next / % budget spent / files. Return summaries + paths + names of tests now passing — never paste whole files back to the orchestrator.
- **Budget discipline.** Stop cleanly at 70% of your assigned budget if not green; write a resumable `handoff` (state + next concrete step). A task that won't fit its budget is too big — signal for a split rather than truncating scope.
- **Commits.** Conventional Commits (`feat: / fix: / test: / refactor: / chore: / docs:`). One green atomic task = one commit. Never commit red. Never commit secrets or the vault/known_hosts/test artifacts (they're git-ignored).
- **Never fabricate.** Not command output, not test results, not benchmark numbers, not API behavior, not "done". If you did not run it, you do not know it.
- **Independent review.** The agent that implements does not mark its own task `done`. The Reviewer re-runs the full gate and validates acceptance before `done`.

---

## 7. GLM 5.2 — model-specific guardrails

GLM 5.2 is a strong agentic coder with a very large (1M-token) context and two reasoning modes. These rules target its documented and observed failure modes.

### 7.1 Context partitioning — do NOT trust the 1M window to save you
A large context window does not mean reasoning stays reliable when it's full; quality degrades and requests can time out as the context approaches its upper bound. Therefore:
- Keep the **working set small**: read only the task's allowlist + `rg` hits. Do not dump entire crates "for context."
- Prefer **file-based state** (ledger, ADRs, concise summaries) over long transcripts. Compact aggressively; drop reasoning once it's captured in an ADR.
- This is *why* tasks are atomic and budgets are enforced (§6) — partition the work so no single session leans on a near-full window.

### 7.2 Reasoning-mode routing
- Use the **High** mode for routine, well-scoped atomic tasks.
- Escalate to **Max** for: milestone planning/decomposition, cross-crate refactors, protocol/state-machine code (`moba-ssh`, VT parser, tunnels, vault crypto), and **any task that has already failed once**. A failed attempt is a signal to raise reasoning depth, not to retry identically.

### 7.3 Language discipline (catch leakage)
- All identifiers, comments, doc comments, log messages, commit messages, and docs are **English and ASCII-only**. Non-ASCII bytes in `.rs` files are forbidden except in intentional, tested string literals (and even then, justify it).
- Self-check: `rg -nP '[^\x00-\x7F]' --type rust` must return nothing except reviewed, intentional literals.

### 7.4 Completeness discipline (catch confident stubs)
GLM 5.2 produces clean, complete-*looking* output — which makes silent stubbing and plausible-but-wrong APIs especially dangerous. Before claiming a task done:
- Grep for stubs and fantasy markers (§8) and get zero hits.
- Confirm every external API you used actually exists (§4.4) — the polished look is not evidence of correctness; a green compile + passing real tests is.

---

## 8. Anti-pattern quick-reference (grep guards)

Run these before declaring any task done. Each must produce **no results** (outside `#[cfg(test)]`/`tests/`). `scripts/check` includes them.

```bash
# Stubs / placeholders in production code
rg -nP 'todo!\(|unimplemented!\(|unreachable!\(' --type rust src crates */src

# Panicking calls in non-test code
rg -nP '\.unwrap\(\)|\.expect\(|panic!\(' --type rust crates */src \
  | rg -v 'tests?/|#\[cfg\(test\)\]'

# Debug leftovers
rg -nP '\bdbg!\(|println!\(|eprintln!\(' --type rust crates            # libs only; gui CLI exempt

# Lint suppression
rg -nP '#\[allow\(' --type rust                                        # each hit needs justification

# Non-ASCII leakage
rg -nP '[^\x00-\x7F]' --type rust

# Ignored tests without a tracking task
rg -nP '#\[ignore\]' --type rust | rg -v 'TASK-'

# Unsafe without a safety comment nearby
rg -nP '\bunsafe\b' --type rust
```

If any guard fires, fix it before completion — do not suppress the guard.

---

## 9. Definition of Done (task-level checklist)

A task is `done` only when **all** are true and evidenced:

- [ ] A test existed and failed first (red), and now passes because of this code (green).
- [ ] `scripts/check` exits 0 — real, pasted output, not asserted.
- [ ] Zero hits on every §8 guard (or justified + Reviewer-approved).
- [ ] Only allowlisted files changed; diff is minimal and on-topic.
- [ ] Every new external API was verified against docs; deps pinned; `Cargo.lock` committed; licenses recorded.
- [ ] Public items documented; no stubs, no commented-out code, no meta-prose in sources.
- [ ] `docs/TASKS.md` updated; ADR added if a non-trivial decision was made; `docs/PARITY.md` updated if a parity feature advanced.
- [ ] Conventional-commit message; committed on a branch that is green.
- [ ] **Independent Reviewer** re-ran the gate and validated the acceptance criterion.

## 10. Milestone gate (before tagging `vX.Y.0`)

- [ ] All milestone tasks `done`.
- [ ] Full `scripts/check` green **including** `--features e2e` (docker bed up).
- [ ] `docs/PARITY.md` rows for this milestone link to the tests that prove them.
- [ ] `cargo-mutants` thresholds met on `moba-vault` and `moba-ssh`.
- [ ] Integrator merges, creates annotated tag + `gh release` with notes listing delivered parity items and their proving tests.

## 11. Stop-and-report (pause; never guess)

Halt and report to the orchestrator/human when: a prerequisite is missing (GitHub auth, Docker, X server for the X11 phase); a dependency is license-incompatible; matching a feature appears to require copying MobaXterm's protected code/assets; you hit the budget/quota ceiling; or a milestone's core acceptance can't be met after decomposition + bounded fixing. Reports are short and structured: what's blocked, why, options, recommendation.

---

*This file is the contract. If you're unsure whether something is allowed, assume it isn't and ask.*
