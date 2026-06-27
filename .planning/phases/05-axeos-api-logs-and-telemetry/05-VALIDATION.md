---
phase: 05
slug: axeos-api-logs-and-telemetry
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-27
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `rust_test` plus Rust unit tests in workspace crates |
| **Config file** | `BUILD.bazel` targets and `Justfile` |
| **Quick run command** | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~10 seconds after dependencies are warm |

---

## Sampling Rate

- **After every task commit:** Run `bazel test //crates/bitaxe-api:tests //tools/parity:tests`
- **After every plan wave:** Run `bazel test //...`
- **Before `/gsd-verify-work`:** `just test` plus API comparison fixtures must be green
- **Max feedback latency:** 60 seconds for focused crate/tool checks

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 0 | API-01, API-02 | T-05-01 | Public DTOs preserve upstream field names and encodings without exposing internal structs | unit/golden | `bazel test //crates/bitaxe-api:tests` | W0 missing DTO fixtures | pending |
| 05-01-02 | 01 | 0 | API-10 | T-05-02 | API compare checks schema and captured response compatibility separately | host fixture | `bazel test //tools/parity:tests` | W0 missing compare checks | pending |
| 05-02-01 | 02 | 1 | API-03 | T-05-03 | Invalid known PATCH fields reject without writes; unknown fields are ignored | unit/integration | `bazel test //crates/bitaxe-api:tests //crates/bitaxe-config:tests` | W0 missing API wrapper tests | pending |
| 05-03-01 | 03 | 1 | API-04 | T-05-04 | ASIC/statistics/scoreboard/mining responses derive from typed runtime state and safe blocked values | unit/golden | `bazel test //crates/bitaxe-api:tests` | W0 missing mapping modules | pending |
| 05-04-01 | 04 | 2 | API-05, API-06, API-07 | T-05-05 | Logs and WebSockets are gated, bounded, and preserve upstream baseline/cadence semantics | unit/smoke | `bazel test //crates/bitaxe-api:tests` | W0 missing log/ws models | pending |
| 05-05-01 | 05 | 2 | API-08 | T-05-06 | Commands produce compatible responses before effects and cannot bypass mining gates | unit/smoke | `bazel test //crates/bitaxe-api:tests` | W0 missing command planners | pending |
| 05-06-01 | 06 | 3 | API-09, API-10 | T-05-07 | Existing AxeOS asset route usage remains administrable without Angular rewrite or Phase 7 OTA scope | fixture/static scan | `bazel test //tools/parity:tests` | W0 missing static route fixture | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

- [ ] `crates/bitaxe-api` — add Serde DTO module skeleton and first exact JSON fixture tests for upstream field names and bool/number encodings.
- [ ] `crates/bitaxe-api/fixtures/` — add provenance-labeled captured-response fixtures for system info, ASIC, statistics, scoreboard, settings PATCH, WebSocket live update, and command responses.
- [ ] `tools/parity` or `tools/api-compare` — add route/property checks for `reference/esp-miner/main/http_server/openapi.yaml` and captured response fixtures.
- [ ] `crates/bitaxe-api` — add tests for settings PATCH unknown-field ignore and invalid-known-field all-or-nothing rejection.
- [ ] `crates/bitaxe-api` — add log/WebSocket baseline tests for raw log chunks, full live update on connect, no-send unchanged state, and 500 ms cadence metadata.
- [ ] `tools/parity` or `crates/bitaxe-api/fixtures/` — add static AxeOS route usage fixture from the pinned reference asset client code.
- [ ] `firmware/bitaxe` — include an ESP-IDF HTTP/WebSocket route registration compile spike before broad handler implementation.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real firmware API smoke on Ultra 205 | API-02, API-03, API-05, API-06, API-07, API-08 | Requires connected hardware, known port, Wi-Fi/network state, and live firmware route serving | Run `just flash-monitor board=205 port=<port>`, capture boot/API logs, then use `curl` for representative HTTP routes and a repo-owned WebSocket smoke helper when available |
| Existing AxeOS assets administer live Rust firmware | API-09 | Requires browser/device interaction or a served asset fixture beyond pure route scans | Serve or load the reference AxeOS asset fixture against the Rust firmware and record which V1 admin surfaces load successfully |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
