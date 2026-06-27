---
phase: 05
slug: axeos-api-logs-and-telemetry
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-27
updated: 2026-06-27
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `rust_test` plus Rust unit tests in workspace crates |
| **Config file** | `BUILD.bazel` targets and `Justfile` |
| **Quick run command** | `bazel test //crates/bitaxe-api:tests --test_filter='route_shell|settings_patch_body_cap'` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~10 seconds after dependencies are warm |

---

## Wave Structure

| Wave | Plans | Reason |
|------|-------|--------|
| 1 | 05-01 | Shared API crate foundation, first wire DTOs, first fixtures |
| 2 | 05-02 | Settings module and PATCH planning, owns its `lib.rs`/Bazel wiring |
| 3 | 05-03 | System/ASIC/statistics/scoreboard/mining mappers, owns its `lib.rs`/Bazel wiring |
| 4 | 05-04 | Logs and live telemetry contracts, owns its `lib.rs`/Bazel wiring |
| 5 | 05-06 | Command planners, owns its `lib.rs`/Bazel wiring |
| 6 | 05-05 | Firmware route/WebSocket/settings/log adapters after pure contracts exist |
| 7 | 05-07 | API compare, static/recovery evidence, and checklist updates |

Wave numbers match the current plan frontmatter. Plan 05-05 intentionally executes after Plan 05-06 because firmware command routes consume pure command planners.

---

## Sampling Rate

- **After every task commit:** Run `bazel test //crates/bitaxe-api:tests //tools/parity:tests`
- **After every plan wave:** Run `bazel test //...`
- **Before `/gsd-verify-work`:** `just test` plus API comparison fixtures must be green
- **Max feedback latency:** 60 seconds for focused crate/tool checks

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists / Ownership | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------------------|--------|
| 05-01-01 | 01 | 1 | API-01, API-02, API-10 | T-05-01 | Public DTOs preserve upstream field names and encodings without exposing internal structs | unit/golden | `bazel test //crates/bitaxe-api:tests` | Owns `Cargo.toml`, `BUILD.bazel`, `lib.rs`, `snapshot.rs`, `wire.rs` only | planned |
| 05-01-02 | 01 | 1 | API-01, API-02, API-10 | T-05-02 | First fixtures use safe public defaults and no secrets | golden fixture | `bazel test //crates/bitaxe-api:tests --test_filter=wire` | Owns first system/ASIC fixtures | planned |
| 05-02-01 | 02 | 2 | API-02, API-03 | T-05-03 | Invalid known PATCH fields reject atomically; unknown fields are ignored | unit/integration | `bazel test //crates/bitaxe-api:tests //crates/bitaxe-config:tests --test_filter=settings` | Owns settings module and its `lib.rs`/Bazel wiring | planned |
| 05-02-02 | 02 | 2 | API-03 | T-05-04, T-05-05 | Persist-then-reload ordering is required before public success | unit/fake adapter | `bazel test //crates/bitaxe-api:tests --test_filter=settings` | Owns settings persistence-plan tests | planned |
| 05-03-01 | 03 | 3 | API-02, API-04 | T-05-07, T-05-10 | System, ASIC, and mining values derive from typed state and safe blocked values | unit/golden | `bazel test //crates/bitaxe-api:tests --test_filter='system|asic|mining'` | Owns system/ASIC/mining modules and wiring | planned |
| 05-03-02 | 03 | 3 | API-04 | T-05-09 | Empty statistics and scoreboard shapes stay compatible without fake history | unit/golden | `bazel test //crates/bitaxe-api:tests --test_filter='statistics|scoreboard'` | Owns statistics/scoreboard modules, fixtures, and wiring | planned |
| 05-04-01 | 04 | 4 | API-05, API-06 | T-05-11, T-05-12 | Retained logs are bounded; `/api/ws` starts at current end and streams raw chunks | unit | `bazel test //crates/bitaxe-api:tests --test_filter=logs` | Owns logs module, fixture, and wiring | planned |
| 05-04-02 | 04 | 4 | API-07 | T-05-13, T-05-14 | `/api/ws/live` sends full-on-connect, diff-only updates, and 500 ms cadence metadata | unit | `bazel test //crates/bitaxe-api:tests --test_filter=telemetry` | Owns telemetry module, fixture, and wiring | planned |
| 05-06-01 | 06 | 5 | API-08 | T-05-20, T-05-21, T-05-23 | Commands produce response JSON separately from side-effect plans | unit/golden | `bazel test //crates/bitaxe-api:tests --test_filter=commands` | Owns commands module, fixture, and wiring | planned |
| 05-06-02 | 06 | 5 | API-08 | T-05-20, T-05-23 | Pause/resume cannot force work submission ready; block dismiss is idempotent | unit | `bazel test //crates/bitaxe-api:tests --test_filter=commands` | Owns command transition tests | planned |
| 05-05-01 | 05 | 6 | API-02, API-03, API-05, API-06, API-07, API-08, API-09 | T-05-15, T-05-16 | Host route-shell tests prove denied HTTP status/body, denied WebSocket upgrade status/body, no WebSocket client registration, generic public errors, route registration, and unsupported OTA/OTAWWW are enforced | host route-shell unit + firmware build | `bazel test //crates/bitaxe-api:tests --test_filter=route_shell && cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` | Owns `crates/bitaxe-api/src/route_shell.rs` plus firmware route shell and gate compile spike | planned |
| 05-05-02 | 05 | 6 | API-02, API-03, API-04, API-05, API-06, API-07, API-08 | T-05-16, T-05-17, T-05-18, T-05-19 | Host body-cap tests prove oversized PATCH is rejected before JSON parsing and before adapter calls, with zero writes/commits/reloads and generic public error; firmware handlers call pure mappers, WebSockets are bounded, and commands respond before effects | host settings route-shell unit + firmware build | `bazel test //crates/bitaxe-api:tests --test_filter=settings_patch_body_cap && bazel build //firmware/bitaxe:firmware` | Owns `crates/bitaxe-api/src/route_shell.rs`, settings route guard, firmware adapters, and handler wiring | planned |
| 05-07-01 | 07 | 7 | API-01, API-09, API-10 | T-05-24, T-05-26, T-05-28 | API compare labels schema/captured/static/smoke evidence and separates `/recovery`/static fallback from update success | host fixture/static scan | `bazel test //tools/parity:tests` | Owns parity API compare and route/static fixtures | planned |
| 05-07-02 | 07 | 7 | API-01, API-09, API-10 | T-05-25, T-05-27 | Evidence/checklist cannot overclaim hardware, OTA, filesystem, release, static, or recovery verification | parity report | `just parity` | Owns checklist and Phase 05 evidence doc | planned |

*Status: planned · green · red · flaky*

---

## Former Wave 0 Gaps

There is no standalone Wave 0 plan after revision. The earlier Wave 0 gaps are fully assigned to executable tasks:

- `serde`, `serde_json`, `thiserror`, and API crate build setup: Plan 05-01 Task 1.
- First DTO/fixture tests: Plan 05-01 Task 1 and Task 2.
- Captured-response fixtures: Plan 05-01 Task 2 and Plan 05-07 Task 1.
- API compare route/property checks: Plan 05-07 Task 1.
- Settings PATCH unknown-field, invalid-known-field, no-write, and oversized-body checks: Plan 05-02 Task 1 and Plan 05-05 Task 2. Plan 05-05 explicitly owns the route-shell body-cap tests that prove oversized bodies are rejected before JSON parsing and before writes/commits/reloads.
- HTTP/WebSocket access denial and no-registration checks: Plan 05-05 Task 1 through `//crates/bitaxe-api:tests --test_filter=route_shell`.
- Log/WebSocket baseline and cadence tests: Plan 05-04 Task 1 and Task 2.
- Static AxeOS route usage, `/recovery`, and static fallback fixtures: Plan 05-07 Task 1.
- ESP-IDF HTTP/WebSocket compile spike: Plan 05-05 Task 1.

`wave_0_complete: true` means no pre-execution scaffold remains unmapped; it does not mean implementation artifacts already exist.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real firmware API smoke on Ultra 205 | API-02, API-03, API-05, API-06, API-07, API-08 | Requires connected hardware, known port, Wi-Fi/network state, and live firmware route serving | Run `just flash-monitor board=205 port=<port>`, capture boot/API logs, then use `curl` for representative HTTP routes and a repo-owned WebSocket smoke helper when available |
| Existing AxeOS assets administer live Rust firmware | API-09 | Requires browser/device interaction or a served asset fixture beyond pure route scans | Serve or load the reference AxeOS asset fixture against the Rust firmware and record which V1 admin surfaces load successfully |

Manual checks are evidence upgrades only. The executable plan package remains Nyquist-compliant because every task has an automated verification command.

---

## Validation Sign-Off

- [x] All 14 actual tasks have `<automated>` verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Former Wave 0 gaps are assigned to concrete executable tasks
- [x] Plan waves match frontmatter and same-wave file ownership has no known overlap
- [x] No watch-mode flags
- [x] Feedback latency target is < 60s for focused checks
- [x] `nyquist_compliant: true` set in frontmatter
- [x] `wave_0_complete: true` set truthfully: no standalone pre-execution Wave 0 remains

**Approval:** ready for execution
