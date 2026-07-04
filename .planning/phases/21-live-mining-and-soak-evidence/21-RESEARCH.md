# Phase 21: Live Mining And Soak Evidence - Research

**Researched:** 2026-07-04
**Domain:** Ultra 205 BM1366 live mining, Stratum v1 evidence, soak safety gates, API/WebSocket telemetry, redaction governance
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

## Implementation Decisions

### Controlled Pool And Target Gates

- **D-01:** Begin every hardware-capable path with `just detect-ultra205`; continue only when exactly one likely ESP32-S3 port is detected and board-info passes for board `205`.
- **D-02:** Live-pool smoke is allowed only with disposable or non-secret pool configuration, no committed pool credentials, an explicit user/operator-supplied `DEVICE_URL`, safe-stop/recovery instructions, redaction review, and an allow manifest that binds board, port, package identity, source commit, reference commit, exact command, abort conditions, and post-action safe-state markers.
- **D-03:** Do not infer `DEVICE_URL` from serial logs, network scans, router state, mDNS, ARP, or redacted evidence. If no explicit target is available, record a blocked or controlled no-share artifact instead of running live HTTP/WebSocket correlation.
- **D-04:** Pool credentials, worker secrets, private endpoints, Wi-Fi credentials, API tokens, NVS secret values, and private `DEVICE_URL` values must not be read into chat output, committed, or summarized in evidence. Redacted logs may keep non-secret category labels, board `205`, port, source/reference commits, package paths, and conclusions.

### Mining Evidence Ladder

- **D-05:** Use the existing Phase 15 ladder as the starting structure: detector/package/safe boot, package-backed chip-detect or staged init, typed work/result evidence, live-pool mining smoke, bounded soak, then exact checklist promotion.
- **D-06:** A later tier must not run when an earlier required tier is missing, failed, stale, redaction-blocked, or lacks trusted wrapper/package markers. Write useful pending evidence with the exact blocker instead of bypassing the ladder.
- **D-07:** Keep `tools/parity` mining allow-manifest validation in the path. Phase 21 may extend the validator beyond Phase 15 command shapes when needed, but the extension must preserve board `205`, detector, board-info, package identity, prohibited command, required abort condition, safe-state marker, live-pool, and bounded-soak checks.
- **D-08:** Production mining evidence must not rely on raw BM1366 writes, raw pool commands, ad hoc voltage/fan controls, erase commands, rollback commands, interrupted-update commands, unbounded stress, or hidden local scripts.

### Live Mining Smoke

- **D-09:** Prefer a short live-pool micro-smoke after detector, safety, chip-detect/work-result, explicit-target, safe-stop, and redaction gates pass. The smoke should record pool connection lifecycle, subscribe/authorize behavior, notify/job flow, BM1366 work dispatch, result handling, accepted/rejected share behavior when observed, hashrate inputs, API/WebSocket status, watchdog breadcrumbs, and final safe-stop.
- **D-10:** If a live-pool run produces no shares within the bounded window, it may support only an explicit bounded no-share conclusion when the artifact proves the pool lifecycle, job/work path, duration, abort conditions, telemetry/watchdog checks, safe-stop, and redaction status. It must not be presented as accepted-share proof.
- **D-11:** Accepted share and rejected share evidence should be treated as exact observed outcomes. Do not synthesize rejected-share claims unless the controlled setup safely and explicitly produces a redaction-reviewed rejection path.
- **D-12:** If a controlled fake-pool or local harness is used, label it as controlled evidence. It can support flow, work, telemetry, watchdog, and no-share boundaries, but it cannot by itself prove live production pool behavior.

### Bounded Soak

- **D-13:** Bounded soak may run only after live smoke passes or the plan explicitly justifies an approved controlled no-share soak. The allowed duration should stay bounded and operator-readable, with `tools/parity` currently enforcing `duration_seconds` from 60 to 600 for a `bounded-soak` claim tier.
- **D-14:** Soak evidence must record duration, abort conditions, thermal/power/watchdog observations, pool lifecycle, share outcomes or bounded no-share status, periodic API/WebSocket snapshots when a target is explicit, final safe-stop/restore markers, and conclusion.
- **D-15:** Watchdog responsiveness must be proven through bounded observations during mining or soak, not merely by startup supervisor/yield breadcrumbs. Missing bounded load or watchdog recovery proof remains below verified.
- **D-16:** Any unexpected reboot, watchdog panic, unsafe temperature/power marker, detector mismatch, missing trusted wrapper marker, redaction uncertainty, lost pool control, or missing safe-state marker is a stop condition and should produce blocked evidence plus recovery notes.

### Telemetry, API, And Statistics Correlation

- **D-17:** Live API and WebSocket telemetry correlation requires an explicit `DEVICE_URL`, bounded `/api/system/info` and `/api/ws/live` captures, redaction review, and correlation to serial or runtime observations from the same run.
- **D-18:** Statistics, scoreboard, accepted/rejected counters, pool difficulty, hashrate inputs, mining activity, and work-submission state should be checked across the runtime state, API response, WebSocket frame, and evidence summary when those surfaces are available.
- **D-19:** Route presence, a no-upgrade WebSocket response, stale cached API bodies, startup-only logs, or Phase 15/20 blocked-target artifacts are supporting breadcrumbs only. They do not prove Phase 21 live telemetry freshness, cadence, or mining statistics behavior.

### Checklist, Redaction, And Verification

- **D-20:** Use exact-claim checklist promotion. `ASIC-002` through `ASIC-005`, `STR-006`, `STR-007`, `STR-008`, statistics/API rows, and `EVD-05` may move only to the evidence level supported by the final artifact.
- **D-21:** `ASIC-007` remains below verified unless Phase 21 intentionally includes a bounded frequency-transition hardware-regression artifact. Live mining or soak evidence alone must not accidentally verify frequency transition behavior.
- **D-22:** `STR-008` verified status requires mining-smoke or soak details with board, port, firmware/source commit, reference commit, redaction, conclusion, and either accepted/rejected share outcome or an approved bounded controlled no-share soak without blocker language.
- **D-23:** Final phase verification must include targeted checks for changed scripts/tools/Rust code, `just test`, `just parity`, `just verify-reference`, reference cleanliness, redaction review, lifecycle validation, and every detector/hardware/network command actually used. No wrapper-level commit/push should happen unless `21-VERIFICATION.md` reports `status: passed` and lifecycle validation succeeds for `21-2026-07-04T01-35-47`.

### the agent's Discretion

The agent may choose the exact plan count, evidence pack names, helper filenames, whether to extend Phase 15 scripts or introduce Phase 21-named wrappers, JSON field names, WebSocket capture helper reuse, and exact checklist note wording. Those choices must preserve ESP-IDF/esp-rs tooling preference, functional-core/imperative-shell structure, read-only reference policy, secret redaction, detector gates, package identity, safe-stop markers, and conservative evidence semantics.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Non-205 board mining or soak evidence remains future board-specific work.
- Stratum v2, BAP accessory behavior, all-board release matrix, and Angular AxeOS rewrite remain out of Phase 21.
- Active voltage/fan/fault/self-test/load hardware regression remains outside Phase 21 unless a plan explicitly adds a prerequisite-only safety artifact without claiming mining proof.
- OTA, OTAWWW, rollback, erase, failed-update recovery, interrupted-update, and release-recovery flows remain outside Phase 21.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ASIC-07 | BM1366 initialization, work-send, and result-receive behavior have hardware-smoke evidence before release parity is claimed. | Phase 15 already proves diagnostic chip-detect/work-result only; Phase 21 must add package-backed live mining or bounded no-share evidence before broader ASIC mining claims can move. [VERIFIED: `.planning/REQUIREMENTS.md`; `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md`] |
| STR-06 | First Ultra 205 mining loop connects config, Stratum v1, BM1366 work dispatch, result parsing, and global state without bypassing safety gates. | Current firmware still publishes blocked mining status by default, while pure Stratum and mining-loop state exist in `crates/bitaxe-stratum`; planner must include an explicit controlled live-mining firmware path or record a blocked evidence boundary. [VERIFIED: `firmware/bitaxe/src/main.rs`; `crates/bitaxe-stratum/src/v1/mining_loop.rs`] |
| STR-07 | Mining parity has hardware-smoke and soak criteria with command, board, port, commits, logs, observed result, and conclusion. | `tools/parity/src/mining_allow.rs` already validates board `205`, detector, board-info, package identity, abort conditions, safe-state markers, live-pool, and bounded-soak contracts; Phase 21 should extend it only with tests. [VERIFIED: `tools/parity/src/mining_allow.rs`] |
| SAFE-09 | Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve observable responsiveness under load. | Phase 20 records only startup/yield watchdog breadcrumbs; Phase 21 must prove bounded mining/soak responsiveness or keep watchdog recovery below verified. [VERIFIED: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md`] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. | Nyquist validation is enabled, Bazel exposes existing script/crate tests, and final verification must include targeted tests, hardware commands used, redaction, parity, reference cleanliness, and lifecycle validation. [VERIFIED: `.planning/config.json`; `bazel query 'kind(.*test, //...)'`; `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`] |
</phase_requirements>

## Summary

Phase 21 should be planned as a gated evidence-and-enablement phase, not as a broad mining refactor. The current codebase has strong pure BM1366, Stratum v1, runtime-state, API, WebSocket, flash, and parity validation surfaces, but the firmware still defaults to safe-state mining disabled and publishes `mining_loop_status=blocked`; Phase 15 evidence proves diagnostic chip-detect/work-result and controlled no-share governance only. [VERIFIED: `firmware/bitaxe/src/main.rs`; `firmware/bitaxe/src/asic_adapter/status.rs`; `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md`]

The planner should reuse the Phase 15 ladder and Phase 20 redaction style, then add only the missing Phase 21 surfaces: controlled live-pool package mode if needed, pool lifecycle capture, share/no-share classification, bounded soak with 60-600 second validator constraints, fresh `/api/system/info` and `/api/ws/live` frame capture from an explicit `DEVICE_URL`, watchdog responsiveness observations under bounded load, redaction review, and exact checklist promotion. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `tools/parity/src/mining_allow.rs`; `scripts/phase17-websocket-capture.mjs`]

**Primary recommendation:** Plan Phase 21 as a tiered evidence ladder: detector/package/safe boot, current-code live-mining capability audit, controlled firmware enablement if missing, package-backed live micro-smoke, bounded soak, telemetry correlation, redaction, conservative checklist updates, and final verification. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `firmware/bitaxe/src/main.rs`; `tools/parity/src/main.rs`]

## Project Constraints (from AGENTS.md)

- Use ESP-IDF/esp-rs tooling first for firmware build, package, flash, monitor, OTA, SPIFFS, NVS, FreeRTOS, logging, and image workflows; document any alternate tool path before adding it. [VERIFIED: `AGENTS.md`]
- Treat `.embuild/` as local generated ESP-IDF/esp-rs state; do not commit or hand-edit it. [VERIFIED: `AGENTS.md`]
- Before autonomous hardware use, run `just detect-ultra205`; proceed only when exactly one ESP32-S3 USB candidate is found and board-info succeeds. [VERIFIED: `AGENTS.md`; `scripts/detect-ultra205.sh`]
- If `wifi-credentials.json` exists, repo commands may receive it as an argument, but agents must not read, print, summarize, or commit its contents. [VERIFIED: `AGENTS.md`; local probe found `wifi-credentials.json` present]
- Evidence must record board `205`, port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, conclusion, and no committed secrets. [VERIFIED: `AGENTS.md`]
- Destructive or fault-injection verification is allowed only when the active phase plan documents recovery path and required evidence; Phase 21 should not run erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside documented gates. [VERIFIED: `AGENTS.md`; `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]
- Follow functional core and imperative shell: pure decisions in Rust crates/tools, hardware/network/filesystem effects in thin firmware/script adapters. [VERIFIED: `AGENTS.md`; `standards/core/architecture.md`; `standards/languages/rust.md`]
- Unit-test pure and business logic with focused Arrange/Act/Assert tests; use repo-owned verification before commits. [VERIFIED: `standards/core/testing.md`; `standards/core/verification.md`; `standards/languages/rust.md`]
- Rust commits require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before commit. [VERIFIED: `AGENTS.md`]
- GSD/parsed Markdown must not use standalone body `---` separators after frontmatter; use headings or `***` instead. [VERIFIED: `AGENTS.md`]
- Keep the upstream reference read-only and avoid copying GPL source expression into MIT-only Rust files; use breadcrumbs and isolate GPL-derived expression if intentionally ported. [VERIFIED: `PROVENANCE.md`; `docs/adr/0013-mit-first-with-gpl-guardrails.md`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| ESP-IDF Rust firmware stack | ESP-IDF `v5.5.4`, `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2` | Firmware services, HTTP/WebSocket, Wi-Fi, NVS, OTA, logging, and ESP-IDF bindings. | Project decision requires ESP-IDF Rust for production firmware parity; `esp-idf-svc` documents support for ESP-IDF services including Wi-Fi, HTTP server, WebSocket, NVS, OTA, timers, and event loop. [VERIFIED: `Cargo.toml`; CITED: https://docs.rs/crate/esp-idf-svc/latest/source/README.md] |
| Bazel + rules_rust | Bazel `9.1.1`, `rules_rust 0.70.0`, `rules_shell 0.8.0` | Canonical automation graph for tests, package, flash-shaped workflows, and script targets. | Project already pins Bazel and Bzlmod rules; `just test` routes through `bazel test //...`. [VERIFIED: `.bazelversion`; `MODULE.bazel`; `Justfile`] |
| `just` command surface | `just 1.48.0` | Human entrypoint for detector, package, flash-monitor, parity, reference verification, and tests. | Repo-local guidance requires `just` for phase hardware commands; local version probe succeeded. [VERIFIED: `Justfile`; local `just --version`] |
| `tools/parity` mining allow validator | workspace crate `bitaxe-parity 0.1.0` | Validates mining allow manifests, prohibited tokens, detector/package identity, live-pool gates, and bounded-soak constraints. | Phase 21 context requires keeping this validator in the path; current code already validates the Phase 15 mining ladder. [VERIFIED: `tools/parity/src/mining_allow.rs`; `Cargo.toml`] |
| `tools/flash` evidence wrapper | workspace crate `bitaxe-flash 0.1.0` with `espflash 4.0.1` backend | Flash/monitor evidence, manifest resolution, trusted output classification, redaction modes, and JSON evidence. | Existing wrapper records source/reference markers and supports `--redact-evidence` for commit-ready artifacts. [VERIFIED: `tools/flash/src/main.rs`; local `espflash --version`] |
| Phase 15 mining scripts | repo-owned shell and Node helpers | Diagnostic packages, controlled mining smoke, bounded-soak placeholder, redaction, and WebSocket probes. | Phase 21 context says to use Phase 15 ladder as the starting structure. [VERIFIED: `scripts/phase15-bm1366-diagnostic-package.sh`; `scripts/phase15-controlled-mining.sh`; `scripts/phase15-websocket-capture.mjs`] |
| Phase 17 WebSocket capture helper | repo-owned Node helper; Node `v24.13.0` with global `WebSocket` available | Bounded `/api/ws` and `/api/ws/live` frame capture from explicit origin-only `DEVICE_URL`. | This helper captures real frames and validates origin-only targets; local Node exposes `globalThis.WebSocket` as a function. [VERIFIED: `scripts/phase17-websocket-capture.mjs`; local Node probe] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Typed JSON manifests, API snapshots, allow manifests, and evidence parsing. | Use for new Rust validators and JSON evidence contracts instead of shell string parsing. [VERIFIED: `Cargo.toml`; `cargo metadata --no-deps`] |
| `clap` | `4.6.1` | CLI parsing for Rust tools. | Use if adding or extending `tools/parity` commands. [VERIFIED: `Cargo.toml`; `tools/parity/Cargo.toml`] |
| `camino` | `1.2.3` | UTF-8 path handling in tools. | Use for new path fields in Rust validators. [VERIFIED: `Cargo.toml`; `tools/parity/src/mining_allow.rs`] |
| `anyhow` / `thiserror` | `anyhow 1.0.102`, `thiserror 2.0.18` | CLI error context and library errors. | Use `anyhow` in CLI/tools; use `thiserror` in library crates. [VERIFIED: `Cargo.toml`; `standards/languages/rust.md`] |
| `curl` | `8.7.1` | Bounded `/api/system/info` capture from explicit `DEVICE_URL`. | Use only through repo-owned helpers with redaction. [VERIFIED: local `curl --version`; `scripts/phase14-live-telemetry.sh`; `scripts/phase15-controlled-mining.sh`] |
| `rg` | `15.1.0` | Redaction and policy-term scans. | Use for scoped final evidence-tree scans, then manually review expected matches. [VERIFIED: local `rg --version`; Phase 15/20 redaction reviews] |
| `jq` | `1.7.1` | Optional JSON assertion in verification scripts. | Use for simple local validation when present; do not make it the only parser for core Rust logic. [VERIFIED: local `jq --version`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Reusing/extending Phase 15 wrappers | Brand-new Phase 21 scripts | Phase 21-named scripts are acceptable only if `tools/parity/src/mining_allow.rs` is extended with tests to accept their exact command shape; current validator only accepts approved Phase 15 wrapper shapes. [VERIFIED: `tools/parity/src/mining_allow.rs`] |
| Phase 17 WebSocket frame capture | Phase 14 route-status live telemetry helper | Phase 14 helper records route status and explicitly says maintained WebSocket frame capture is unavailable, so it is insufficient for Phase 21 frame/cadence proof. [VERIFIED: `scripts/phase14-live-telemetry.sh`; `scripts/phase17-websocket-capture.mjs`] |
| Disposable live pool | Local fake-pool harness only | Fake-pool evidence supports controlled flow/work/no-share boundaries, but the Phase 21 context says it cannot by itself prove live production pool behavior. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `crates/bitaxe-stratum/src/v1/fake_pool.rs`] |
| `std`/ESP-IDF service path | Bare-metal/no_std stack | Project decision requires ESP-IDF Rust; `esp-idf-svc` also warns its ESP-IDF Rust crates are community-maintained and may lack HIL tests, so hardware evidence remains mandatory. [VERIFIED: `AGENTS.md`; CITED: https://docs.rs/crate/esp-idf-svc/latest/source/README.md] |

**Installation:**

No new packages should be added for Phase 21 unless a code audit proves the live mining adapter cannot be implemented with existing workspace crates and ESP-IDF/esp-rs tooling. [VERIFIED: `Cargo.toml`; `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Version verification:** Versions above were verified from `Cargo.toml`, `.bazelversion`, `MODULE.bazel`, local CLI probes, `cargo metadata --no-deps`, and `bazel query`. No `npm view` check applies because Phase 21 should not add npm packages. [VERIFIED: local commands]

## Architecture Patterns

### Recommended Project Structure

```text
docs/parity/evidence/phase-21-live-mining-and-soak-evidence/
  evidence-contract.md          # pack schema, stop conditions, recovery, citation rules
  preflight/                    # detector, board-info, package, safe baseline
  bm1366-work-result/           # reused or refreshed package-backed diagnostic evidence
  live-mining-smoke/            # live or blocked controlled pool smoke artifacts
  bounded-soak/                 # 60-600 second soak or approved controlled no-share soak
  live-api-websocket-telemetry/ # /api/system/info and /api/ws/live captures
  redaction-review.md           # final artifact-specific review
  summary.md                    # exact claim matrix and non-claims
scripts/
  phase21-*.sh                  # only if Phase 15 wrappers cannot express Phase 21 semantics
tools/parity/src/
  mining_allow.rs               # extend with tests before accepting new command shapes
```

This structure mirrors the Phase 15/20 evidence-pack pattern and keeps generated evidence under the parity evidence tree. [VERIFIED: `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md`; `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md`]

### Pattern 1: Evidence Ladder Before Live Effects

**What:** Plan hard gates in this order: `just detect-ultra205`, package identity, safe baseline, diagnostic BM1366 proof, explicit target/pool inputs, live smoke, bounded soak, telemetry correlation, redaction, checklist promotion. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**When to use:** Use this for every hardware-capable Phase 21 task, including no-share and blocked paths. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Example:**

```bash
just detect-ultra205
bazel run //tools/parity:report -- mining-allow --manifest <allow.json> --surface mining-smoke --allowed-command "<exact wrapper command>"
```

Source: `scripts/detect-ultra205.sh` and `tools/parity/src/mining_allow.rs`. [VERIFIED: source files]

### Pattern 2: Validator-First Wrapper Changes

**What:** If Phase 21 introduces `scripts/phase21-*`, first update `ALLOWED_SURFACES`, command-shape checks, claim-tier tests, and negative tests in `tools/parity/src/mining_allow.rs`. [VERIFIED: `tools/parity/src/mining_allow.rs`]

**When to use:** Use this before any new hardware command shape or claim tier is cited in evidence. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Example:**

```rust
// Source: tools/parity/src/mining_allow.rs
// Add tests that reject unsafe commands and accept only the exact Phase 21 wrapper shape.
```

### Pattern 3: Explicit Origin Target, Never Discovery

**What:** Accept `DEVICE_URL` only as an explicit user/operator input or trusted origin-only artifact; do not infer it from logs, scans, router state, mDNS, ARP, or redacted evidence. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `scripts/phase17-websocket-capture.mjs`]

**When to use:** Use for `/api/system/info`, `/api/ws/live`, and any live HTTP/WebSocket correlation. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Example:**

```bash
node scripts/phase17-websocket-capture.mjs \
  --device-url "$DEVICE_URL" \
  --path /api/ws/live \
  --out docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-ws-live.txt \
  --duration-ms 10000 \
  --max-frames 5
```

Source: `scripts/phase17-websocket-capture.mjs`. [VERIFIED: source file]

### Pattern 4: Functional Core, Imperative Shell

**What:** Keep share classification, allow-manifest rules, API/runtime mapping, and checklist promotion in Rust pure/tool code; keep USB, curl, WebSocket, pool credentials, flashing, and hardware timing inside wrappers/firmware adapters. [VERIFIED: `standards/core/architecture.md`; `crates/bitaxe-stratum/src/v1/state.rs`; `tools/parity/src/mining_allow.rs`; `scripts/phase15-controlled-mining.sh`]

**When to use:** Use when adding share outcome classification, soak status, or manifest validation logic. [VERIFIED: `standards/languages/rust.md`; `standards/core/testing.md`]

### Anti-Patterns to Avoid

- **Live proof from route presence:** `/api/ws/live` route status or no-upgrade response is not live frame/cadence proof. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `scripts/phase14-live-telemetry.sh`]
- **Verified status with blocker language:** `tools/parity` rejects live ASIC/mining verified rows containing terms such as missing live prerequisites, blocked, pending, below verified, no reachable `DEVICE_URL`, or unverified. [VERIFIED: `tools/parity/src/main.rs`]
- **Raw mining or hardware commands:** Raw BM1366 writes, raw pool commands, voltage/fan controls, erase/rollback/interrupted-update commands, and hidden scripts are prohibited for Phase 21 evidence. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `tools/parity/src/mining_allow.rs`]
- **Credential-bearing evidence:** Pool credentials, worker secrets, private endpoints, Wi-Fi credentials, API tokens, NVS secret values, and private `DEVICE_URL` values must not be committed or summarized. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `AGENTS.md`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Ultra 205 hardware detection | A custom serial-port scanner or board probe | `just detect-ultra205` | It already enforces exactly one likely ESP USB serial port plus ESP32-S3 board-info. [VERIFIED: `scripts/detect-ultra205.sh`] |
| Package identity and trusted serial capture | Direct `espflash` evidence parsing in new scripts | `tools/flash` / `just flash-monitor` evidence JSON and logs | The wrapper records source/reference commits, trusted output, manifest paths, capture status, and redaction mode. [VERIFIED: `tools/flash/src/main.rs`] |
| Mining allow gates | Ad hoc JSON/string checks in shell | `tools/parity` `mining-allow` | Existing Rust validator enforces board, detector, package identity, abort conditions, safe-state markers, live-pool inputs, bounded-soak duration, and prohibited tokens. [VERIFIED: `tools/parity/src/mining_allow.rs`] |
| Stratum message/state semantics | New parser or share counter logic in shell | `crates/bitaxe-stratum` | Existing pure Rust code covers Stratum v1 messages, fake-pool, mining loop, queue, share counters, difficulty, lifecycle, and hashrate inputs. [VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`; `crates/bitaxe-stratum/src/v1/state.rs`; `crates/bitaxe-stratum/src/v1/mining_loop.rs`] |
| `/api/ws/live` frame capture | Route-only curl checks or custom WebSocket client | `scripts/phase17-websocket-capture.mjs` | It validates explicit origin-only targets, bounds duration/frames, uses Node global WebSocket, and redacts frames. [VERIFIED: `scripts/phase17-websocket-capture.mjs`; local Node probe] |
| Redaction scans | One-off grep patterns without review status | Phase 15/20 redaction review pattern plus `rg` scans | Prior reviews distinguish expected policy labels from secrets and clear only specific artifacts. [VERIFIED: Phase 15/20 redaction review files] |
| Checklist promotion rules | Manual verified-row reasoning only | `just parity` with `--fail-on-invalid-verified` | The validator already rejects live ASIC/mining overclaims and specific `STR-008`/`ASIC-007` invalid promotions. [VERIFIED: `Justfile`; `tools/parity/src/main.rs`] |

**Key insight:** Phase 21 complexity is in evidence trust boundaries, not parsing or transport primitives; custom shell shortcuts are likely to create false parity claims or secret leaks. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `tools/parity/src/main.rs`; Phase 15/20 redaction reviews]

## Common Pitfalls

### Pitfall 1: Planning Live Evidence When Firmware Still Fails Closed

**What goes wrong:** The phase tries to run live mining smoke using scripts that only record pending/control-boundary status. [VERIFIED: `scripts/phase15-controlled-mining.sh`]

**Why it happens:** Current firmware startup still logs safe-state mining disabled and publishes blocked mining-loop status by default; Phase 15 scripts do not themselves enable production mining. [VERIFIED: `firmware/bitaxe/src/main.rs`; `firmware/bitaxe/src/asic_adapter/status.rs`; `scripts/phase15-controlled-mining.sh`]

**How to avoid:** Add a first plan task that audits or creates a controlled live-mining firmware mode/package path before live-pool claims are attempted. [VERIFIED: `firmware/bitaxe/src/main.rs`; `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Warning signs:** Evidence says `conclusion: pending`, `controlled-no-share`, `work_submission=disabled`, or `hardware_evidence_ack_missing`. [VERIFIED: `scripts/phase15-controlled-mining.sh`; `firmware/bitaxe/src/asic_adapter/status.rs`]

### Pitfall 2: Treating Bounded No-Share As Accepted-Share Proof

**What goes wrong:** A no-share live run is cited as production share proof. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Why it happens:** A pool can subscribe/authorize and send jobs without producing an accepted share in a short window. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**How to avoid:** Label no-share artifacts as bounded no-share, record lifecycle/job/work/duration/telemetry/watchdog/safe-stop, and keep accepted-share claims absent unless observed. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`; `tools/parity/src/main.rs`]

**Warning signs:** Notes mention `controlled no-share` and `missing live prerequisites`, or no `accepted share`/`rejected share` text appears for `STR-008`. [VERIFIED: `tools/parity/src/main.rs`]

### Pitfall 3: New Phase 21 Wrapper Rejected By Parity

**What goes wrong:** A new script generates evidence but `mining-allow` rejects its command shape. [VERIFIED: `tools/parity/src/mining_allow.rs`]

**Why it happens:** `is_expected_phase15_command` currently accepts Phase 15 wrapper names and `rg` path under Phase 15 redaction only. [VERIFIED: `tools/parity/src/mining_allow.rs`]

**How to avoid:** Either reuse Phase 15 wrapper names or update `mining_allow.rs` with Phase 21 command-shape tests before running hardware. [VERIFIED: `tools/parity/src/mining_allow.rs`; `scripts/phase15-controlled-mining-test.sh`]

**Warning signs:** `mining_allow_status: failed` or error text naming "approved Phase 15 wrapper". [VERIFIED: `tools/parity/src/mining_allow.rs`]

### Pitfall 4: Committing Developer-Raw Network Evidence

**What goes wrong:** Raw IP, MAC, SSID, private `DEVICE_URL`, pool URL, or credentials are committed. [VERIFIED: `AGENTS.md`; `tools/flash/src/main.rs`]

**Why it happens:** `tools/flash` has developer-raw and commit-redacted evidence modes, and live mining requires sensitive operational inputs. [VERIFIED: `tools/flash/src/main.rs`; `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**How to avoid:** Use `--redact-evidence` for commit-targeted artifacts, keep raw artifacts under gitignored `target/`, and run artifact-specific redaction review before citations. [VERIFIED: `tools/flash/src/main.rs`; Phase 20 redaction review]

**Warning signs:** Evidence contains raw `http://`, private IPs, MAC addresses, SSIDs, `stratumUser`, `stratumPassword`, `poolUrl`, or `DEVICE_URL` values outside placeholders. [VERIFIED: `scripts/phase15-controlled-mining.sh`; `scripts/phase17-websocket-capture.mjs`; `tools/flash/src/main.rs`]

### Pitfall 5: Watchdog Breadcrumb Overclaim

**What goes wrong:** Startup watchdog/yield markers are treated as proof of watchdog responsiveness during mining or soak. [VERIFIED: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md`]

**Why it happens:** Phase 20 records watchdog startup/yield breadcrumbs only. [VERIFIED: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md`]

**How to avoid:** Require bounded observations during live mining or soak, with no unexpected reboot/panic and recorded API/WebSocket/serial responsiveness. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

**Warning signs:** Evidence lacks duration, periodic observations, and post-run safe-state markers. [VERIFIED: `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md`]

## Code Examples

### Mining Allow Manifest Skeleton

```json
{
  "board": "205",
  "detector_command": "just detect-ultra205",
  "board_info_status": "passed",
  "surface": "mining-smoke",
  "claim_tier": "live-pool-smoke",
  "evidence_class": "hardware-smoke",
  "allowed_inputs": {
    "pool_config": "disposable-or-non-secret",
    "device_url": "explicit"
  },
  "abort_conditions": [
    "detector_mismatch",
    "board_info_failure",
    "missing_trusted_wrapper_markers",
    "redaction_uncertainty",
    "unsafe_temperature_or_power",
    "watchdog_unresponsive"
  ],
  "post_action_safe_state_markers": [
    "safe_state: mining=disabled",
    "hardware_control=disabled",
    "work_submission=disabled"
  ]
}
```

Source: `tools/parity/src/mining_allow.rs`. [VERIFIED: source file]

### Bounded Soak Validator Constraint

```text
bounded-soak requires allowed_inputs.duration_seconds between 60 and 600
```

Source: `tools/parity/src/mining_allow.rs`. [VERIFIED: source file]

### Redaction Scan Pattern

```bash
rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-21-live-mining-and-soak-evidence
```

Source pattern adapted from Phase 15 redaction review. [VERIFIED: `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`]

### WebSocket Frame Capture

```bash
node scripts/phase17-websocket-capture.mjs \
  --device-url "$DEVICE_URL" \
  --path /api/ws/live \
  --duration-ms 10000 \
  --max-frames 5 \
  --out docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-ws-live.txt
```

Source: `scripts/phase17-websocket-capture.mjs`. [VERIFIED: source file]

### Checklist Promotion Wording Pattern

```text
Board 205 port <redacted-or-usb-port> firmware commit <source> reference commit <reference>
accepted share observed redaction passed conclusion recorded.
```

Source: `tools/parity/src/main.rs` tests for `STR-008`. [VERIFIED: source file]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Diagnostic work/result and controlled no-share only | Live-pool smoke or approved bounded controlled no-share soak with validator-backed metadata | Phase 21 context, 2026-07-04 | Planner must not reuse Phase 15 no-share evidence as live production proof. [VERIFIED: `21-CONTEXT.md`; Phase 15 ledger] |
| WebSocket route/no-upgrade response as support | Bounded real `/api/ws/live` frame capture from explicit `DEVICE_URL` | Phase 17 helper and Phase 21 decisions | Planner should capture frames when explicit target exists; route presence is only a breadcrumb. [VERIFIED: `scripts/phase17-websocket-capture.mjs`; `21-CONTEXT.md`] |
| Startup watchdog breadcrumb | Bounded watchdog responsiveness during mining or soak | Phase 21 decisions | SAFE-09 stays below verified without bounded load/mining observations. [VERIFIED: Phase 20 summary; `21-CONTEXT.md`] |
| Checklist updates by evidence labels alone | Exact-claim validator rejects blocker language and missing metadata | Current `tools/parity` | Planner must update notes and evidence classes together and then run `just parity`. [VERIFIED: `tools/parity/src/main.rs`] |

**Deprecated/outdated:**

- Network target inference from serial logs, redacted evidence, router state, mDNS, ARP, or scans is out of scope for Phase 21. [VERIFIED: `21-CONTEXT.md`]
- Raw BM1366 writes, raw pool commands, ad hoc voltage/fan controls, erase, rollback, interrupted-update, unbounded stress, and hidden local scripts are prohibited evidence paths. [VERIFIED: `21-CONTEXT.md`; `tools/parity/src/mining_allow.rs`]
- `ASIC-007` frequency transition cannot be verified by live mining or soak alone. [VERIFIED: `21-CONTEXT.md`; `tools/parity/src/main.rs`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

All claims in this research were verified against local project files, local command probes, or cited official docs. No assumed claims are intentionally present.

## Open Questions

1. **Will execution have an explicit origin-only `DEVICE_URL` and disposable/non-secret live pool configuration?**
   - What we know: Context requires both for live HTTP/WebSocket and live-pool smoke. [VERIFIED: `21-CONTEXT.md`]
   - What's unclear: The value cannot be inferred and should not be committed or summarized. [VERIFIED: `21-CONTEXT.md`; `AGENTS.md`]
   - Recommendation: Plan a live path plus a blocked/approved bounded no-share fallback. [VERIFIED: `21-CONTEXT.md`]

2. **Does the current firmware need a controlled live-mining enablement path before evidence can run?**
   - What we know: Current firmware main logs safe-state mining disabled and publishes blocked mining-loop status; pure mining-loop and state models exist. [VERIFIED: `firmware/bitaxe/src/main.rs`; `crates/bitaxe-stratum/src/v1/mining_loop.rs`]
   - What's unclear: Whether an uninspected later mechanism enables live mining outside the searched firmware paths. [VERIFIED: `rg` search across `firmware/bitaxe/src` and `crates/bitaxe-stratum/src`]
   - Recommendation: Make Wave 0 audit live-mining firmware readiness and add a controlled package mode only if the audit confirms it is missing. [VERIFIED: `21-CONTEXT.md`; `tools/parity/src/mining_allow.rs`]

3. **Will accepted-share proof be required, or is approved bounded no-share soak sufficient for closure?**
   - What we know: `STR-008` verified can accept live share metadata or approved bounded controlled no-share soak without blocker language. [VERIFIED: `tools/parity/src/main.rs`]
   - What's unclear: Pool difficulty and smoke duration may not produce shares in the bounded window. [VERIFIED: `21-CONTEXT.md`]
   - Recommendation: Plan exact outcomes: accepted/rejected if observed, otherwise bounded no-share only with explicit non-claim language. [VERIFIED: `21-CONTEXT.md`; `tools/parity/src/main.rs`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Ultra 205 detector gate | Hardware smoke and soak | yes | `just detect-ultra205` passed, port `/dev/cu.usbmodem1101`, ESP32-S3 board-info passed; MAC omitted here | If unavailable, record blocked evidence and do not run hardware. [VERIFIED: local `just detect-ultra205`; `AGENTS.md`] |
| `wifi-credentials.json` | Optional flash/bring-up Wi-Fi seed | yes | local ignored file present; contents not read | If absent, require already configured device networking or block explicit-target telemetry. [VERIFIED: local `test -f`; `AGENTS.md`] |
| `.embuild/espressif` | ESP-IDF managed tooling | yes | directory present | If absent, run repo bootstrap/build path before firmware package work. [VERIFIED: local `test -d`; `AGENTS.md`] |
| `just` | Human command surface | yes | `1.48.0` | No fallback; install just or use repo commands only for diagnostics. [VERIFIED: local probe] |
| Bazel | Build/test/package | yes | `9.1.1` | No fallback for canonical graph. [VERIFIED: local probe; `.bazelversion`] |
| Cargo/Rust | Rust tests/tools/firmware | yes | `cargo 1.88.0-nightly`, `rustc 1.88.0-nightly` | No fallback for Rust changes. [VERIFIED: local probes] |
| `espflash` | USB board-info, flash, monitor | yes | `4.0.1` | No fallback for Phase 21 hardware evidence. [VERIFIED: local probe] |
| `espup` | ESP Rust toolchain setup | yes | `0.15.1` | Already installed; `just bootstrap-esp` is repo opt-in installer if stale. [VERIFIED: local probe; `AGENTS.md`] |
| `ldproxy` | ESP-IDF Rust linker | yes | `0.3.4` from `cargo install --list`; `ldproxy --version` panics without linker args | Treat as available; do not use `ldproxy --version` as health check. [VERIFIED: local probes] |
| Node | WebSocket capture helpers | yes | `v24.13.0`, `globalThis.WebSocket` is `function` | If missing, WebSocket frame proof blocks or requires maintained helper alternative. [VERIFIED: local probes; `scripts/phase17-websocket-capture.mjs`] |
| `curl` | API route capture | yes | `8.7.1` | If missing, API telemetry capture blocks. [VERIFIED: local probe] |
| `rg` | Redaction scans | yes | `15.1.0` | If missing, use another grep-compatible scan and document reduced confidence. [VERIFIED: local probe] |
| `jq` | Optional JSON spot checks | yes | `1.7.1` | Use Rust/Node parsers instead. [VERIFIED: local probe] |

**Missing dependencies with no fallback:** None found during research. [VERIFIED: local probes]

**Missing dependencies with fallback:** None found during research. [VERIFIED: local probes]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `9.1.1` wrapping Rust and shell tests; Cargo/Rust workspace tests. [VERIFIED: `.bazelversion`; `bazel query 'kind(.*test, //...)'`] |
| Config file | `BUILD.bazel`, per-package `BUILD.bazel`, `Cargo.toml`, `MODULE.bazel`. [VERIFIED: repo files] |
| Quick run command | `bazel test //tools/parity:tests //scripts:phase15_controlled_mining_test //scripts:phase15_bm1366_diagnostic_package_test` before Phase 21-specific targets exist. [VERIFIED: `bazel query`; `scripts/BUILD.bazel`] |
| Full suite command | `just test` plus Rust pre-commit checks when committing. [VERIFIED: `Justfile`; `AGENTS.md`] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| ASIC-07 | BM1366 diagnostic chip-detect/work-result and any new controlled live-mining ASIC gate | unit + script + hardware smoke | `cargo test -p bitaxe-asic --all-features adapter_gate work result transcript`; `bazel test //scripts:phase15_bm1366_diagnostic_package_test`; hardware `just detect-ultra205` then gated flash/monitor command | yes for existing diagnostics; Phase 21 live package test likely Wave 0 if added. [VERIFIED: Phase 15 verification; `bazel query`] |
| STR-06 | Stratum v1 mining-loop state, pool lifecycle, work dispatch, share submission, and safe gates | unit + script + hardware smoke | `cargo test -p bitaxe-stratum --all-features mining_loop fake_pool queue`; add Phase 21 wrapper tests if new live mode is added | yes for pure crates; live firmware wrapper test likely Wave 0. [VERIFIED: Phase 15 verification; source files] |
| STR-07 | Mining smoke/soak criteria and allow manifest metadata | unit + script | `cargo test -p bitaxe-parity --all-features mining_allow`; `bazel test //tools/parity:tests`; new Phase 21 script test target if command shape changes | yes for existing validator; Phase 21 extensions likely Wave 0. [VERIFIED: `tools/parity/src/mining_allow.rs`; `bazel query`] |
| SAFE-09 | Watchdog responsiveness and safe-stop under bounded mining/soak load | script + hardware smoke/soak | Existing: `bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase20_failure_paths_test`; Phase 21 must add evidence assertions for bounded mining/soak observations if code changes | partial; Phase 21 load/soak assertion test is a Wave 0 gap. [VERIFIED: Phase 20 summary; `bazel query`] |
| EVD-05 | Layered verification, redaction, parity, reference cleanliness, lifecycle | workflow | `just test`; `just parity`; `just verify-reference`; `git diff -- reference/esp-miner --exit-code`; GSD lifecycle validation for phase `21` | yes for existing commands; `21-VERIFICATION.md` not yet present. [VERIFIED: `Justfile`; init output] |

### Sampling Rate

- **Per task commit:** Run scoped Rust/script tests for changed paths; if committing in this Rust repo, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: `AGENTS.md`]
- **Per wave merge:** Run `just test`, `just parity`, and `just verify-reference`. [VERIFIED: `Justfile`; `21-CONTEXT.md`]
- **Phase gate:** Full suite plus detector/hardware commands actually used, redaction review, reference cleanliness, and lifecycle validation before `21-VERIFICATION.md` reports `passed`. [VERIFIED: `21-CONTEXT.md`]

### Wave 0 Gaps

- [ ] Add tests for any `tools/parity/src/mining_allow.rs` Phase 21 command-shape extension before new hardware wrapper use. [VERIFIED: current validator accepts Phase 15 shapes only]
- [ ] Add a Phase 21 wrapper test target under `scripts/BUILD.bazel` if a new `scripts/phase21-*.sh` wrapper is introduced. [VERIFIED: current Phase 15/20 scripts have Bazel test targets]
- [ ] Add a live-mining readiness audit artifact before planning live smoke, because current firmware startup remains fail-closed by default. [VERIFIED: `firmware/bitaxe/src/main.rs`; `firmware/bitaxe/src/asic_adapter/status.rs`]
- [ ] Add a redaction-review scaffold for Phase 21 before final checklist citations. [VERIFIED: Phase 15/20 redaction reviews]

## Security Domain

### Applicable ASVS Categories

OWASP lists ASVS as a standard for web application technical security controls and identifies latest stable ASVS 5.0.0 on the project page/GitHub; this phase maps only relevant categories to local controls. [CITED: https://owasp.org/www-project-application-security-verification-standard/; CITED: https://github.com/OWASP/ASVS]

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no for new app auth; yes for Stratum pool authorization handling | Pool credentials stay operator-supplied, disposable/non-secret where possible, and absent from committed evidence. [VERIFIED: `21-CONTEXT.md`; `scripts/phase15-controlled-mining.sh`] |
| V3 Session Management | no for browser sessions | Phase 21 does not add user sessions; WebSocket state remains existing route client tracking. [VERIFIED: `firmware/bitaxe/src/websocket_api.rs`] |
| V4 Access Control | yes for local HTTP/WebSocket target gating | Use explicit origin-only `DEVICE_URL`, no scanning/inference, and existing route access gates. [VERIFIED: `21-CONTEXT.md`; `scripts/phase17-websocket-capture.mjs`; `firmware/bitaxe/src/http_api.rs`] |
| V5 Input Validation | yes | Parse allow manifests and URLs through Rust/Node structured parsers; validate duration, surface, claim tier, command shape, and explicit origin target. [VERIFIED: `tools/parity/src/mining_allow.rs`; `scripts/phase17-websocket-capture.mjs`] |
| V6 Cryptography | no new crypto implementation | Do not hand-roll cryptography; existing mining hash behavior remains in pure Stratum/ASIC crates with tests and reference breadcrumbs. [VERIFIED: `crates/bitaxe-stratum/src/v1/mining.rs`; `PROVENANCE.md`] |

### Known Threat Patterns for Phase 21

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Credential disclosure in committed evidence | Information Disclosure | Use disposable/non-secret pool config, do not read credential files into chat, use redaction helpers, scoped `rg` scans, and artifact-specific redaction review. [VERIFIED: `21-CONTEXT.md`; `AGENTS.md`; Phase 15/20 redaction reviews] |
| Unsafe hardware operation from unvalidated command | Tampering / Denial of Service | Route through `mining-allow`, prohibit raw BM1366/pool/erase/voltage/fan commands, require abort conditions and safe-state markers. [VERIFIED: `tools/parity/src/mining_allow.rs`; `21-CONTEXT.md`] |
| Target spoofing or accidental network probing | Spoofing / Information Disclosure | Accept only explicit origin-only `DEVICE_URL`; keep network scanning disabled. [VERIFIED: `21-CONTEXT.md`; `scripts/phase17-websocket-capture.mjs`] |
| Overclaimed release parity | Repudiation / Tampering | Use exact-claim checklist promotion and `just parity` verified-row guards. [VERIFIED: `tools/parity/src/main.rs`; `docs/adr/0012-parity-verification-evidence.md`] |
| Watchdog starvation during soak | Denial of Service | Bound duration, define stop conditions, capture periodic responsiveness, and require final safe-stop markers. [VERIFIED: `21-CONTEXT.md`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md` - locked decisions, scope, evidence ladder, redaction, deferred items. [VERIFIED: local read]
- `.planning/REQUIREMENTS.md` - Phase 21 requirement IDs and traceability. [VERIFIED: local read]
- `.planning/STATE.md` and `.planning/v1.0-MILESTONE-AUDIT.md` - prior phase outcomes and live mining/soak gap. [VERIFIED: local read/search]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/*`, `PROVENANCE.md`, ADRs 0001/0003/0005/0006/0012/0013/0014 - repo constraints, evidence policy, licensing, reference rules. [VERIFIED: local read/search]
- `tools/parity/src/mining_allow.rs` and `tools/parity/src/main.rs` - allow-manifest and checklist promotion rules. [VERIFIED: local read]
- `scripts/phase15-controlled-mining.sh`, `scripts/phase15-bm1366-diagnostic-package.sh`, `scripts/phase17-websocket-capture.mjs`, `scripts/detect-ultra205.sh` - reusable evidence wrappers and explicit-target behavior. [VERIFIED: local read]
- `firmware/bitaxe/src/main.rs`, `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/websocket_api.rs`, `firmware/bitaxe/src/runtime_snapshot.rs` - current firmware safe state, blocked mining-loop status, HTTP/WebSocket and runtime snapshot integration points. [VERIFIED: local read/search]
- `crates/bitaxe-stratum`, `crates/bitaxe-api`, `crates/bitaxe-asic` - pure mining, state, API, and BM1366 logic. [VERIFIED: local read/search]
- Local environment probes: `just detect-ultra205`, CLI versions, Node `globalThis.WebSocket`, Bazel query, Cargo metadata. [VERIFIED: local commands]

### Secondary (MEDIUM confidence)

- `https://docs.rs/crate/esp-idf-svc/latest/source/README.md` - `esp-idf-svc` service coverage and caveat that esp-idf Rust crates are community-maintained and may lack HIL tests. [CITED: docs.rs]
- `https://owasp.org/www-project-application-security-verification-standard/` and `https://github.com/OWASP/ASVS` - ASVS purpose and current stable version information. [CITED: OWASP]

### Tertiary (LOW confidence)

- None used.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - pinned in repo files and verified with local CLI probes. [VERIFIED: `Cargo.toml`; `.bazelversion`; `MODULE.bazel`; local commands]
- Architecture: HIGH - constrained by Phase 21 locked decisions plus existing Phase 15/20 implementation patterns. [VERIFIED: `21-CONTEXT.md`; Phase 15/20 ledgers]
- Pitfalls: HIGH - derived from current validators, current firmware safe-state behavior, and prior blocked evidence. [VERIFIED: `tools/parity/src/main.rs`; `firmware/bitaxe/src/main.rs`; Phase 15/20 ledgers]
- Live production mining capability: MEDIUM - current searched firmware paths appear fail-closed, but the planner should confirm with a Wave 0 audit before implementation. [VERIFIED: `rg` search; `firmware/bitaxe/src/main.rs`]

**Research date:** 2026-07-04
**Valid until:** 2026-07-11 for live hardware/tool availability; 2026-08-03 for local architecture and evidence rules unless Phase 21 implementation changes them.
