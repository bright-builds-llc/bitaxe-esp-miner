---
phase: 04
slug: stratum-v1-and-first-mining-loop
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-27
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit tests through Cargo and Bazel `rust_test` |
| **Config file** | `crates/bitaxe-stratum/BUILD.bazel`, workspace `Cargo.toml`, `MODULE.bazel` |
| **Quick run command** | `cargo test -p bitaxe-stratum --all-features` |
| **Full suite command** | `cargo test --all-features && bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //crates/bitaxe-config:tests //crates/bitaxe-core:tests` |
| **Estimated runtime** | ~60-180 seconds targeted; full Rust pre-commit checks may be longer |

---

## Sampling Rate

- **After every task commit:** Run the task's targeted `cargo test -p bitaxe-stratum ... --all-features` command.
- **After every plan wave:** Run `cargo test -p bitaxe-stratum --all-features` and touched dependency crate tests.
- **Before `/gsd-verify-work`:** Run the repo Rust pre-commit sequence from `AGENTS.md`, plus `just test`, `just parity`, and `just verify-reference` when parity/reference surfaces changed.
- **Max feedback latency:** 180 seconds for targeted Rust tests.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 04-01-01 | 01 | 1 | STR-01 | T-04-01 / T-04-02 | Malformed pool JSON is rejected before state mutation | unit/golden | `cargo test -p bitaxe-stratum stratum_v1_protocol --all-features` | no W0 | pending |
| 04-02-01 | 02 | 1 | STR-02, STR-04 | T-04-03 | Fake-pool disconnects, errors, fallback, and reconnect produce typed lifecycle states | unit/fake integration | `cargo test -p bitaxe-stratum fake_pool --all-features && cargo test -p bitaxe-stratum pool_lifecycle --all-features` | no W0 | pending |
| 04-03-01 | 03 | 2 | STR-03, STR-06 | T-04-04 | Clean jobs clear queue and valid-job tracking before stale share submission | unit/golden | `cargo test -p bitaxe-stratum mining_job --all-features && cargo test -p bitaxe-stratum mining_loop --all-features` | no W0 | pending |
| 04-04-01 | 04 | 2 | STR-05 | T-04-05 | Share counters and fallback status update only from typed accepted/rejected outcomes | unit | `cargo test -p bitaxe-stratum runtime_state --all-features` | no W0 | pending |
| 04-05-01 | 05 | 3 | STR-06, STR-07 | T-04-06 / T-04-07 | Live mining remains fail-closed without explicit ASIC/safety/evidence gates | unit/static/evidence | `rg -n "STR-00[1-7]|phase-04|mining" docs/parity docs/parity/evidence && cargo test -p bitaxe-stratum --all-features` | no W0 | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

- [ ] `crates/bitaxe-stratum/src/error.rs` — typed parse/protocol/queue/fake-pool errors.
- [ ] `crates/bitaxe-stratum/src/v1.rs` and `crates/bitaxe-stratum/src/v1/` — Stratum v1 module graph without `mod.rs`.
- [ ] `crates/bitaxe-stratum/fixtures/v1/protocol-cases.json` — provenance-rich parser/serializer fixtures.
- [ ] `crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json` — deterministic fake-pool transcripts.
- [ ] `crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json` — reference-derived mining job and coinbase fixtures.
- [ ] `crates/bitaxe-stratum/BUILD.bazel` — new source files, deps, tests, and fixture compile data.
- [ ] `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` — hardware smoke/soak criteria and run or skipped evidence conclusion.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Controlled/public pool smoke on Ultra 205 | STR-06, STR-07 | Requires connected board, safe pool target, and explicit hardware evidence approval. | Run only after fake-pool and ASIC gates pass. Record command, board, port, firmware commit, reference commit, logs, observed share result, fallback/reconnect observations when exercised, and conclusion in `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md`. |
| Long-running soak for accepted/rejected shares and reconnect | STR-04, STR-07 | Requires hardware and time-bound pool interaction; cannot be proven by unit tests. | Capture start/end time, pool target redactions, accepted/rejected counters, reconnect/fallback events, and final checklist status. |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands or Wave 0 dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all missing references.
- [x] No watch-mode flags.
- [x] Feedback latency target is under 180 seconds for targeted checks.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-27 for planning; execution must update statuses with real results.
