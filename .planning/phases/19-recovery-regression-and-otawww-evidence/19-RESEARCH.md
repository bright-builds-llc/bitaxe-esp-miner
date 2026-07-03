# Phase 19: Recovery Regression And OTAWWW Evidence - Research

**Researched:** 2026-07-03 [VERIFIED: local environment current date]
**Domain:** ESP-IDF Rust recovery evidence, destructive/fault-injection gates, firmware OTA recovery, and OTAWWW/static partition parity [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md]
**Confidence:** HIGH for evidence architecture and current OTAWWW gap; MEDIUM for any future whole-`www` implementation details because Phase 19 may choose documentation rather than implementation [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

<user_constraints source=".planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md">

## User Constraints (from CONTEXT.md)

All content in this section is copied from Phase 19 context and is source-tagged here as [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md].

### Locked Decisions

## Implementation Decisions

### Recovery And Fault-Injection Gates

- **D-01:** Start Phase 19 from the current package identity. Run `just package`
  and manifest-backed release-gate validation before citing package, recovery,
  OTAWWW, or restore evidence.
- **D-02:** Before any live hardware action, run `just detect-ultra205` and
  require exactly one likely ESP32-S3 serial port plus successful
  `espflash board-info --chip esp32s3 --port <port> --non-interactive` for
  board `205`.
- **D-03:** Failed-update, large-erase or factory-restore, and interrupted OTA
  actions require explicit phase-owned allow flags. Default no-allow behavior
  must create pending evidence without running destructive or fault-injection
  commands.
- **D-04:** Every allowed action must record abort conditions, exact commands,
  detector output, board-info output, package manifest, source commit,
  reference commit, selected port, restore command, post-action safe-state
  markers, observed behavior, conclusion, and redaction status.
- **D-05:** Do not run ad hoc raw erase, rollback, write, or interrupted-upload
  commands. Use repo-owned helpers or commands whose plan names the recovery
  path and expected safe-state markers.

### Recovery Regression Evidence Shape

- **D-06:** Reuse the Phase 16 recovery-regression helper pattern rather than
  inventing a new raw-command flow, but write Phase 19 artifacts under
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`.
- **D-07:** Failed-update recovery beyond invalid rejection should prove that an
  intentionally invalid or failed upload leaves the device operable through
  HTTP/static/recovery/API checks. If `DEVICE_URL` or another prerequisite is
  absent, record the blocker and do not promote recovery behavior.
- **D-08:** Large erase evidence is destructive. It may be captured only when
  the factory image, manifest, selected port, detector gate, board-info gate,
  restore command, monitor capture, and post-restore HTTP/static or serial
  safe-state checks are all available.
- **D-09:** Interrupted OTA evidence must use a bounded interruption point,
  capture public status/body or curl failure, then prove post-interruption
  operability through HTTP/static/recovery/API or serial safe-state evidence.

### OTAWWW And Static Update Claim Boundary

- **D-10:** Keep OTAWWW separate from firmware OTA. Phase 18 valid upload and
  invalid firmware rejection do not prove whole-`www` static update behavior.
- **D-11:** Full OTAWWW parity may be claimed only if a plan implements or uses
  a documented whole-`www` partition update procedure with size checks, chunked
  erase/write behavior, recovery access, and interrupted-update
  hardware-regression evidence.
- **D-12:** If full OTAWWW parity cannot be proven safely in Phase 19, preserve
  it as an explicit REL-03 V1 parity gap with owner, blocker, operator impact,
  follow-up path, and current public route behavior. A `www.bin` package
  artifact or `Wrong API input` response alone must not verify OTAWWW.
- **D-13:** Static asset update evidence may cite Phase 17 live static serving
  and Phase 18 package identity only as supporting context. It must not convert
  static serving or package generation into whole-`www` update proof.

### Release Docs, Checklist, Redaction, And Verification

- **D-14:** Update release docs, parity checklist, requirements traceability,
  and evidence ledgers only after Phase 19 artifacts exist. Documentation should
  cite commands and artifact paths, not goals or implementation existence as
  proof.
- **D-15:** Promote checklist rows only to the exact evidence tier supported by
  Phase 19 artifacts. Failed-update recovery, large erase, interrupted update,
  OTAWWW, REL-03, REL-08, REL-07, API-09, and EVD-05 notes must distinguish
  verified behavior from blocked, pending, below-verified, and non-claim
  behavior.
- **D-16:** Redaction review is mandatory before commit. It must cover
  `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool
  credentials, worker secrets, API tokens, NVS secret values, raw terminal
  secrets, request and response bodies, serial logs, detector logs, board-info
  logs, and recovery logs.
- **D-17:** Final verification must include repo-native checks for changed
  paths, helper tests for any modified helper scripts, `just package`,
  manifest-backed release-gate validation, `just parity`,
  `just verify-reference`, lifecycle validation, and every hardware/network
  command actually used.

### the agent's Discretion

The agent may choose exact helper names, evidence JSON fields, timeout values,
plan split, redaction-review wording, and whether Phase 19 wraps the Phase 16
recovery helper or creates a phase-specific script. Those choices must preserve
explicit target input, current package identity, repo-owned ESP/esp-rs tooling,
functional core plus imperative shell, read-only reference files, no standalone
body `---` separators in parsed Markdown, and conservative evidence claims.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Full OTAWWW whole-`www` update parity remains deferred if Phase 19 cannot
  safely prove whole-partition write and interrupted-update recovery.
- Active voltage, fan, thermal, and power-control telemetry evidence belongs to
  Phase 20.
- Live mining, pool behavior, share handling, watchdog responsiveness under
  mining load, and soak behavior belong to Phase 21.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2,
  BAP, and an Angular AxeOS replacement remain outside Phase 19.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| REL-03 | OTAWWW or static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner. [VERIFIED: .planning/REQUIREMENTS.md] | Current Rust firmware returns a typed OTAWWW gap with public `Wrong API input`; full verification requires whole-`www` update plus interrupted-update hardware-regression evidence. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] [VERIFIED: tools/parity/src/main.rs] |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update need verification evidence before release parity is claimed. [VERIFIED: .planning/REQUIREMENTS.md] | Phase 16 helper already gates failed-update, large-erase, and interrupted-update with allow flags, detector, board-info, manifest, restore, and safe-state checks. [VERIFIED: scripts/phase16-recovery-regression.sh] |
| REL-07 | Build, flash, monitor, OTA, and recovery documentation must be sufficient for a developer with a connected Ultra 205 to operate safely. [VERIFIED: .planning/REQUIREMENTS.md] | `docs/release/ultra-205.md` already documents package, flash, monitor, OTA, recovery, and current non-claims; Phase 19 should update it only after artifacts exist. [VERIFIED: docs/release/ultra-205.md] |
| API-09 | Static AxeOS assets and recovery page behavior must remain compatible enough for device administration without an Angular rewrite. [VERIFIED: .planning/REQUIREMENTS.md] | Phase 17 verified live static, `/assets/app.css.gz`, missing-static redirect, `/recovery`, API coexistence, and WebSocket frames, but not whole-`www` OTAWWW update behavior. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] |
| EVD-05 | Verification layers include unit tests, golden/API comparison, hardware smoke, and hardware regression or soak where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Phase 19 planning must include shell helper tests, Rust route/parity tests, package/release-gate checks, detector-gated hardware evidence, redaction review, `just parity`, and `just verify-reference`. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md] [VERIFIED: scripts/BUILD.bazel] |

</phase_requirements>

## Summary

Phase 19 should be planned as an evidence-closure phase with a narrow optional implementation fork: either safely prove the existing recovery/fault-injection cases and document OTAWWW as a bounded REL-03 gap, or explicitly implement/verify a whole-`www` OTAWWW partition updater with hardware-regression evidence. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md]

The default recommendation is to create a Phase 19 evidence contract and phase-specific wrapper around the Phase 16 recovery-regression helper, not a new raw command flow. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase16-recovery-regression-test.sh] The wrapper should write under `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`, enforce explicit target provenance from Phase 17/18 patterns, keep no-allow behavior useful and pending-only, and only run destructive paths after documented allow flags, detector, board-info, factory image, manifest, restore, monitor, and redaction prerequisites pass. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md]

Current OTAWWW behavior is not implemented as a whole-`www` update in Rust: the pure route model returns `OtaWwwGap`, the firmware handler logs `otawww_update=gap`, and the public response is `Wrong API input`. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] Phase 17 proved only that fail-closed response live; it did not prove static partition update behavior. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

**Primary recommendation:** Plan Phase 19 in four waves: Wave 0 evidence contract/helper tests, Wave 1 package-release-detector-target identity, Wave 2 gated recovery regression and no-allow/pending evidence, Wave 3 OTAWWW gap or whole-`www` implementation decision plus docs/checklist/redaction/verification closure. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md] [VERIFIED: scripts/BUILD.bazel]

## Project Constraints (from AGENTS.md)

- Use ESP-IDF Rust bindings, Bazel as canonical automation, and `just` as the human command surface. [VERIFIED: AGENTS.md]
- Keep `reference/esp-miner` pinned and read-only; run repo verification that fails on missing, dirty, or unpinned reference state. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]
- Treat `docs/parity/checklist.md` as evidence, not a task-completion scoreboard; `verified` requires explicit evidence. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
- Preserve functional core and imperative shell: pure decisions belong in crates/tools; ESP-IDF, HTTP, serial, flash, erase, OTA, and hardware effects belong in thin adapters or repo-owned scripts. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]
- Prefer ESP-IDF and esp-rs tooling for firmware build, package, flash, monitor, partitions, OTA, SPIFFS, NVS, FreeRTOS, and logging; use `.embuild/` generated ESP-IDF tools only as managed local state. [VERIFIED: AGENTS.md]
- Before autonomous Ultra 205 hardware use, run `just detect-ultra205`; proceed only with exactly one likely ESP USB serial port and successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: AGENTS.md]
- Destructive or fault-injection work is allowed only when an active phase plan documents recovery path, required evidence, allow flags, stop conditions, and safe-state checks. [VERIFIED: AGENTS.md]
- Do not read, print, summarize, or commit `wifi-credentials.json`; committed/shareable evidence must redact secrets, private endpoints, IP/MAC/SSID values, and NVS secret values unless local developer evidence is intentionally raw and uncommitted. [VERIFIED: AGENTS.md]
- Parsed Markdown must not use standalone body `---` separators after frontmatter. [VERIFIED: AGENTS.md]
- For new or changed pure/business logic, unit tests are required and should use Arrange/Act/Assert. [VERIFIED: standards/core/testing.md]
- Before commits in this Rust project, repo rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`; Phase 19 plans may use repo-native narrower checks for task execution but final commit gating should respect the Rust rule when code changes occur. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/verification.md]
- The worktree is currently on `main` ahead of `origin/main` with an unrelated modified `.planning/config.json`; Phase 19 research must not revert or normalize that file. [VERIFIED: local `git status --short --branch`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Existing Bash helpers with `set -euo pipefail` | repo-owned | Evidence orchestration for package, detector, target-lock, curl, recovery, monitor, and redaction flows. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] | Existing helpers already encode no-scan target policy, allow flags, detector gates, current-manifest checks, and redaction. [VERIFIED: scripts/phase16-recovery-regression-test.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence-test.sh] |
| `just` | 1.48.0 | Human command surface for `package`, `detect-ultra205`, `flash-monitor`, `parity`, and `verify-reference`. [VERIFIED: local `just --version`] [VERIFIED: Justfile] | Repo-local guidance requires `just` commands for normal workflows. [VERIFIED: AGENTS.md] |
| Bazel / rules_shell / rules_rust | Bazel 9.1.1, `rules_shell 0.8.0`, `rules_rust 0.70.0` | Test and automation graph for scripts, Rust crates, firmware packaging, and parity tooling. [VERIFIED: local `bazel --version`] [VERIFIED: MODULE.bazel] | Repo decisions make Bazel canonical, and existing script tests are Bazel-visible. [VERIFIED: AGENTS.md] [VERIFIED: scripts/BUILD.bazel] |
| `espflash` | 4.0.1 | USB board-info, flash, monitor, and erase command backend. [VERIFIED: local `espflash --version`] [VERIFIED: scripts/detect-ultra205.sh] | Repo guidance prefers `espflash` for flashing and monitoring where it suffices. [VERIFIED: AGENTS.md] |
| `curl` | system tool | Live HTTP upload/probe backend for `/api/system/OTA`, `/api/system/OTAWWW`, `/`, `/recovery`, and static/API checks. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] | Existing helpers wrap curl with bounded timeouts, selected headers, body snippets, and redaction. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] |
| Python 3 | 3.14.4 | Manifest parsing, target-lock JSON writing, and helper-test fixture inspection. [VERIFIED: local `python3 --version`] [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] | Existing scripts use small Python snippets for structured JSON rather than ad hoc string parsing. [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] |
| ESP-IDF Rust stack | `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2`, ESP-IDF `v5.5.4` | Firmware HTTP, OTA, SPIFFS/static, partition, logging, and platform bindings. [VERIFIED: Cargo.toml] [VERIFIED: Cargo.lock] [VERIFIED: firmware/bitaxe/Cargo.toml] | Project decisions require ESP-IDF Rust for production firmware parity. [VERIFIED: AGENTS.md] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `tools/parity` | repo-owned Rust tool | Release-gate validation, checklist overclaim guards, and `just parity`. [VERIFIED: tools/parity/src/release_gate.rs] [VERIFIED: tools/parity/src/main.rs] | Use after package evidence and after checklist/docs updates. [VERIFIED: Justfile] |
| `crates/bitaxe-api` | repo-owned Rust crate | Pure route, static, and update decisions for firmware adapters. [VERIFIED: crates/bitaxe-api/src/route_shell.rs] [VERIFIED: crates/bitaxe-api/src/update_plan.rs] | Use for any OTAWWW state change or route-claim boundary. [VERIFIED: crates/bitaxe-api/BUILD.bazel] |
| `scripts/phase13-monitor-capture.sh` | repo-owned | Bounded noninteractive serial monitor capture with optional `--no-reset`. [VERIFIED: scripts/phase13-monitor-capture.sh] | Use after large erase restore, interrupted update, or firmware OTA when post-action serial markers are required. [VERIFIED: scripts/phase16-recovery-regression.sh] |
| `scripts/phase16-http-static-smoke.sh` / `scripts/phase17-live-http-api-smoke.sh` | repo-owned | HTTP/static/recovery/API operability proof and target-lock patterns. [VERIFIED: scripts/phase16-http-static-smoke.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] | Use after failed or interrupted uploads to prove the device remains administrable. [VERIFIED: scripts/phase16-recovery-regression.sh] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Phase-specific wrapper around Phase 16 helper | Raw `espflash erase-flash`, raw `curl`, and raw `espflash monitor` commands in docs | Reject raw flow: Phase 19 context forbids ad hoc raw erase/write/interrupted upload commands, and Phase 16 helper already enforces gates and pending defaults. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md] [VERIFIED: scripts/phase16-recovery-regression.sh] |
| Current OTAWWW REL-03 gap documentation | Implement full whole-`www` partition writer in Phase 19 | Implementing full OTAWWW is only justified if the plan also adds size checks, chunked erase/write, recovery access, and interrupted-update hardware-regression evidence. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| ESP-IDF partition APIs for `www` writes | Direct flash offsets or custom binary manipulation | Reject direct offsets in firmware: ESP-IDF partition APIs enumerate/write/erase within partition boundaries and require erase-before-write semantics. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] |

**Installation:** No new dependencies should be added for Phase 19 planning; use the existing repo stack and installed tools. [VERIFIED: Cargo.toml] [VERIFIED: MODULE.bazel] [VERIFIED: local environment availability audit]

**Version verification:** Versions above were verified with `just --version`, `bazel --version`, `espflash --version`, `python3 --version`, `cargo --version`, `rustc --version`, `Cargo.toml`, `Cargo.lock`, and `MODULE.bazel`; no `npm view` check applies because Phase 19 does not require a new npm package. [VERIFIED: local environment availability audit] [VERIFIED: Cargo.lock] [VERIFIED: MODULE.bazel]

## Architecture Patterns

### Recommended Project Structure

```text
docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/
├── evidence-contract.md        # Required artifact classes and claim boundaries. [VERIFIED: phase-18 evidence-contract pattern]
├── package-release-gate/       # Copied manifest, package log, release-gate log. [VERIFIED: Phase 17/18 evidence directories]
├── serial-boot/                # Detector, board-info, flash-monitor identity. [VERIFIED: Phase 17/18 evidence directories]
├── target-lock.json            # Sanitized explicit target provenance. [VERIFIED: scripts/phase17-live-http-api-smoke.sh]
├── recovery-regression/        # failed-update, large-erase, interrupted-update logs. [VERIFIED: scripts/phase16-recovery-regression.sh]
├── otawww/                     # fail-closed gap evidence or whole-www update artifacts. [VERIFIED: crates/bitaxe-api/src/update_plan.rs]
├── summary.md                  # Final ledger based on exact artifacts. [VERIFIED: Phase 17/18 summary pattern]
└── redaction-review.md         # Final redaction gate before citation/commit. [VERIFIED: Phase 18 redaction-review pattern]

scripts/
├── phase19-recovery-otawww-evidence.sh       # Optional wrapper over Phase 16 + OTAWWW gap/update checks. [VERIFIED: 19-CONTEXT.md]
└── phase19-recovery-otawww-evidence-test.sh  # Required if wrapper is added. [VERIFIED: scripts/BUILD.bazel pattern]
```

### Pattern 1: Evidence Contract First

**What:** Create `evidence-contract.md` before live evidence is cited, with artifact classes initialized as `pending` or `absent - not cited`. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/evidence-contract.md]

**When to use:** Use at Wave 0 so later plans cannot accidentally cite missing bodies, logs, or target artifacts. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/evidence-contract.md]

**Example:**

```markdown
| Artifact | Initial status | Claim boundary |
| --- | --- | --- |
| `recovery-regression/failed-update.log` | pending | Failed-update recovery evidence only after post-failure HTTP/static/recovery/API operability is captured. |
| `otawww/otawww-gap.log` | pending | REL-03 gap evidence only; not whole-`www` update proof. |
```

Source: Phase 18 contract pattern. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/evidence-contract.md]

### Pattern 2: Current Package Identity Before Hardware

**What:** Run `just package`, copy `bitaxe-ultra205-package.json`, run manifest-backed release-gate, and keep source/reference commits aligned with later flash/evidence artifacts. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

**When to use:** Use before failed-update, large-erase, interrupted-update, OTAWWW, restore, docs, checklist, or release claims. [VERIFIED: 19-CONTEXT.md]

**Example:**

```bash
just package
bazel run //tools/parity:report -- release-gate \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
```

Source: Release-gate and package command pattern. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/release_gate.rs]

### Pattern 3: Explicit Target Lock, No Scan

**What:** Accept `DEVICE_URL` only from an explicit origin-only input or trusted board `205` flash-monitor evidence, write only redacted target provenance, and record `network_scan: disabled`. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh]

**When to use:** Use for failed-update, interrupted-update, post-restore HTTP/static checks, and OTAWWW gap/update probes. [VERIFIED: 19-CONTEXT.md]

**Example:**

```bash
scripts/phase18-firmware-ota-evidence.sh \
  --device-url-from-flash-evidence target/phase19-recovery-regression-and-otawww-evidence-dev-raw/flash-command-evidence.json \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port <detected-port> \
  --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence \
  --target-lock-out docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json
```

Source: Phase 18 wrapper pattern; update arguments if Phase 19 creates its own wrapper. [VERIFIED: scripts/phase18-firmware-ota-evidence.sh]

### Pattern 4: Allow Flags Are Execution Gates, Not Documentation

**What:** No allow flag must produce pending evidence without calling curl for failed/interrupted uploads or `espflash erase-flash` for large erase. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase16-recovery-regression-test.sh]

**When to use:** Use for failed-update, large-erase/factory-restore, and interrupted OTA. [VERIFIED: 19-CONTEXT.md]

**Example:**

```bash
scripts/phase16-recovery-regression.sh \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port <detected-port> \
  --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/recovery-regression
```

Source: Default pending behavior test. [VERIFIED: scripts/phase16-recovery-regression-test.sh]

### Anti-Patterns to Avoid

- **Promoting `www.bin` packaging to OTAWWW verification:** `www.bin` is a package artifact, while OTAWWW verification requires a live whole-`www` update procedure and interrupted-update hardware-regression evidence. [VERIFIED: scripts/package-firmware.sh] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: 19-CONTEXT.md]
- **Treating `Wrong API input` as whole-`www` proof:** Phase 17 observed only the fail-closed gap response, and the pure Rust model labels OTAWWW as `OtaWwwGap`. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] [VERIFIED: crates/bitaxe-api/src/update_plan.rs]
- **Using invalid firmware rejection as rollback proof:** Phase 18 explicitly records invalid rejection as separate from rollback and boot-validation proof. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md]
- **Running raw destructive commands:** Phase 19 context and AGENTS forbid ad hoc erase, rollback, write, or interrupted-upload commands outside documented phase gates. [VERIFIED: 19-CONTEXT.md] [VERIFIED: AGENTS.md]
- **Leaking target or credential values in committed evidence:** Phase 17/18 patterns commit sanitized target locks and redacted bodies/logs only. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| USB serial detection and board-info gate | Custom port scanner or mDNS/router inference | `just detect-ultra205` plus `espflash board-info --chip esp32s3 --port <port> --non-interactive` | Repo policy requires exactly one likely ESP USB port and board-info before hardware use. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Package/release identity | Manual checksum list | `just package` and `tools/parity release-gate --manifest` | Release-gate enforces required artifacts including `esp-miner.bin`, `www.bin`, factory image, partition table, and `otadata-initial.bin`. [VERIFIED: tools/parity/src/release_gate.rs] |
| Failed-update, large-erase, interrupted-update command flow | Raw curl/espflash commands in docs | Phase 16 helper pattern or a Phase 19 wrapper around it | Existing helper enforces allow flags, detector/board-info gates, current manifest source commit, restore path, and safe-state markers. [VERIFIED: scripts/phase16-recovery-regression.sh] |
| Target provenance | Network scan, ARP scan, mDNS lookup, serial-log guessing | Explicit origin-only `DEVICE_URL` or trusted flash-monitor evidence converted to `target-lock.json` | Phase 17/18 helpers intentionally record `network_scan: disabled` and commit only redacted target provenance. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] |
| OTAWWW partition writes if implemented | Direct absolute flash offsets or custom flash image manipulation in firmware | ESP-IDF `esp_partition_find_first`, `esp_partition_erase_range`, and `esp_partition_write` via `esp-idf-sys` | ESP-IDF partition APIs operate inside partition boundaries and require erase-before-write. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] |
| Redaction review | Unreviewed grep output or raw committed bodies/logs | Phase 18 redaction-review matrix and allowlisted match classes | Phase 18 review scans artifacts and classifies allowed labels/placeholders while excluding raw targets, IP/MAC/SSID, credentials, tokens, and secrets. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md] |

**Key insight:** Phase 19 failures are more likely to be overclaiming, stale identity, missing redaction, or unsafe command execution than missing low-level primitives; plan around evidence gates first. [VERIFIED: .planning/STATE.md] [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md]

## Common Pitfalls

### Pitfall 1: Current Package Drift

**What goes wrong:** A helper runs against a package manifest whose `source_commit` no longer equals current `HEAD` or the flashed wrapper JSON. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

**Why it happens:** Evidence/docs commits can advance repository `HEAD` after package capture. [VERIFIED: .planning/STATE.md]

**How to avoid:** Refresh package/release-gate evidence after helper/script changes and verify manifest, flash JSON, and current commit alignment before live actions. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

**Warning signs:** Logs say `manifest source_commit does not match current HEAD` or flash JSON `firmware_commit` differs from copied manifest `source_commit`. [VERIFIED: scripts/phase16-recovery-regression-test.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

### Pitfall 2: OTAWWW Evidence Overclaim

**What goes wrong:** The plan marks OTAWWW verified because `www.bin` exists or because `POST /api/system/OTAWWW` returns `Wrong API input`. [VERIFIED: scripts/package-firmware.sh] [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

**Why it happens:** Packaging, static serving, route presence, and whole-`www` update are separate evidence classes. [VERIFIED: 19-CONTEXT.md]

**How to avoid:** Keep `OTA-002` deferred unless a whole-`www` procedure writes the `www` SPIFFS partition and survives interrupted-update hardware regression. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**Warning signs:** Checklist row `OTA-002` says `verified` without `hardware-regression` and `interrupted-update`. [VERIFIED: tools/parity/src/main.rs]

### Pitfall 3: Destructive Action Without Restore Proof

**What goes wrong:** Large erase runs, but no factory reflash, monitor capture, or post-restore safe-state markers are recorded. [VERIFIED: scripts/phase16-recovery-regression.sh]

**Why it happens:** A raw erase command bypasses the helper's restore and marker checks. [VERIFIED: scripts/phase16-recovery-regression-test.sh]

**How to avoid:** Large erase may run only through the gated helper or Phase 19 wrapper with factory image, restore command, monitor capture, and safe-state marker checks. [VERIFIED: 19-CONTEXT.md] [VERIFIED: scripts/phase16-recovery-regression.sh]

**Warning signs:** Evidence contains `espflash erase-flash` but no `factory reflash command`, no `large-erase-post-restore-monitor.log`, or missing `safe_state: mining=disabled` / `spiffs_mount=available`. [VERIFIED: scripts/phase16-recovery-regression.sh]

### Pitfall 4: Interrupted Upload Completes

**What goes wrong:** The intended interrupted OTA returns HTTP 200 and therefore is not an interruption test. [VERIFIED: scripts/phase16-recovery-regression.sh]

**Why it happens:** `curl --max-time 1 --limit-rate 1024` may still complete if the payload or network conditions make upload too fast. [VERIFIED: scripts/phase16-recovery-regression.sh]

**How to avoid:** Treat HTTP 200 as blocked for interrupted-update evidence and preserve post-interruption operability as the required proof. [VERIFIED: scripts/phase16-recovery-regression.sh]

**Warning signs:** `interrupted-update public status: 200` or `curl_status` is not `28`. [VERIFIED: scripts/phase16-recovery-regression.sh]

### Pitfall 5: HTTP Upload Error Handling Is Misread

**What goes wrong:** A connection failure is reported as passed recovery because curl returned a body or status field. [VERIFIED: scripts/phase16-recovery-regression.sh]

**Why it happens:** ESP-IDF `httpd_req_recv` may return timeout/interrupted or failure codes, and helper curl failures must be classified separately from public HTTP rejection. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html]

**How to avoid:** Require `curl_status`, public status/body, and post-action HTTP/static/recovery/API or serial safe-state proof before promoting. [VERIFIED: scripts/phase16-recovery-regression.sh]

**Warning signs:** Logs have nonzero curl status but no blocked conclusion. [VERIFIED: scripts/phase16-recovery-regression.sh]

### Pitfall 6: GPL Expression Drift

**What goes wrong:** Phase 19 copies upstream OTAWWW C implementation into MIT-only Rust. [VERIFIED: PROVENANCE.md] [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md]

**Why it happens:** The reference OTAWWW flow is concise and tempting to port line-by-line. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**How to avoid:** Use reference breadcrumbs and independent Rust design; if source expression is intentionally ported, mark it GPL-3.0-compatible and isolate it. [VERIFIED: PROVENANCE.md]

**Warning signs:** Rust code mirrors upstream control flow/comment wording around chunk counters, erase loop, or status strings without provenance/licensing review. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: PROVENANCE.md]

## Code Examples

### No-Allow Recovery Evidence

```bash
scripts/phase16-recovery-regression.sh \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port <detected-port> \
  --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/recovery-regression
```

Source: pending-by-default behavior. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase16-recovery-regression-test.sh]

### Allowed Failed Update With Operability Proof

```bash
scripts/phase16-recovery-regression.sh \
  --device-url "${DEVICE_URL}" \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port <detected-port> \
  --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/recovery-regression \
  --allow-failed-update
```

Source: helper requires detector/board-info/current-manifest gate and then post-failure HTTP/static smoke. [VERIFIED: scripts/phase16-recovery-regression.sh]

### OTAWWW Gap Probe

```bash
curl --silent --show-error --max-time 10 \
  --dump-header otawww.headers.raw \
  --output otawww.body.raw \
  --write-out "%{http_code}" \
  --request POST \
  --data-binary "" \
  "${DEVICE_URL%/}/api/system/OTAWWW"
```

Source: Phase 17 route probe shape; Phase 19 wrapper should redact raw outputs before commit. [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

### Whole-`www` Implementation Shape If Phase 19 Chooses It

```rust
// Source: ESP-IDF partition API plus pinned reference behavior.
// Keep this in a firmware adapter, behind pure route decisions and explicit evidence gates.
let partition = unsafe {
        sys::esp_partition_find_first(
        sys::esp_partition_type_t_ESP_PARTITION_TYPE_DATA,
        sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_SPIFFS,
        b"www\0".as_ptr().cast(),
    )
};
```

Source: ESP-IDF partition APIs operate inside partition boundaries and upstream looks up the `www` SPIFFS partition before erase/write. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Raw monitor fallback evidence | Wrapper-owned flash-monitor evidence with trusted marker classification | Phase 9 [VERIFIED: .planning/ROADMAP.md] | Phase 19 should use wrapper-owned serial evidence and not raw `espflash monitor` as trusted proof. [VERIFIED: docs/release/ultra-205.md] |
| Package-only/static route evidence for release rows | Checklist guards require live hardware evidence and reject blocker language for verified release/OTA/static rows | Phase 10 and later [VERIFIED: tools/parity/src/main.rs] | Phase 19 docs/checklist updates must use exact artifact paths and conservative tiers. [VERIFIED: tools/parity/src/main.rs] |
| OTAWWW treated as route presence | OTAWWW is an explicit typed REL-03 gap in Rust, with fail-closed `Wrong API input` public behavior | Phase 7/10/17 [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] | Phase 19 must not treat route presence or public gap response as whole-`www` proof. [VERIFIED: 19-CONTEXT.md] |
| Firmware OTA response considered enough | Valid OTA remains below verified without post-OTA identity and `ota_boot_validation=` markers | Phase 18 [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md] | Recovery regression claims must require post-action operability or serial markers, not only upload status. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md] |
| Upstream OTAWWW writes `www` partition | Rust firmware currently returns OTAWWW gap; full parity would need independent ESP-IDF partition erase/write implementation and hardware-regression evidence | Current state [VERIFIED: firmware/bitaxe/src/http_api.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] | Planner should choose gap documentation unless it can schedule safe implementation plus interrupted-update proof. [VERIFIED: 19-CONTEXT.md] |

**Deprecated/outdated:**

- Treating `Wrong API input` as OTAWWW verification is outdated for Phase 19; it is only fail-closed gap evidence. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]
- Treating invalid image rejection as rollback is outdated for Phase 19; Phase 18 explicitly separated those claims. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md]
- Using raw destructive commands outside repo-owned helpers is forbidden by current Phase 19 context and repo guidance. [VERIFIED: 19-CONTEXT.md] [VERIFIED: AGENTS.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

No `[ASSUMED]` claims are used; all research claims are sourced from repo files, local command output, pinned reference code, or official documentation. [VERIFIED: pre-submission review]

## Open Questions (RESOLVED)

1. **Will Phase 19 implement whole-`www` OTAWWW or document the REL-03 gap?**
   - What we know: Current Rust behavior is a typed OTAWWW gap with `Wrong API input`, while upstream supports `POST /api/system/OTAWWW` with `www.bin` and whole-partition erase/write. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c]
   - RESOLVED: The generated Phase 19 plans take the REL-03 gap path by default. Whole-`www` OTAWWW parity may be claimed only if execution adds documented whole-partition update proof plus interrupted-update hardware-regression evidence; otherwise Plan 19-04 records owner, blocker, operator impact, follow-up path, and current public route behavior. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-04-PLAN.md] [VERIFIED: 19-CONTEXT.md]

2. **Is a reachable explicit `DEVICE_URL` available during execution?**
   - What we know: Phase 17/18 used trusted USB flash-monitor evidence to derive sanitized target provenance, and Phase 19 must not scan or infer network targets. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh]
   - RESOLVED: Target availability is execution-gated. Plan 19-02 must record a sanitized target lock with `network_scan: disabled` when trusted board `205` flash-monitor evidence yields an origin-only URL, or explicit blocked/pending evidence when no target is available. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-02-PLAN.md] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

3. **Which destructive allow flags will be permitted?**
   - What we know: Failed-update, large erase, and interrupted OTA require explicit phase-owned allow flags. [VERIFIED: 19-CONTEXT.md]
   - RESOLVED: Allow flags are independent and opt-in during execution. Plan 19-03 runs a safe no-allow path by default, and only adds `--allow-failed-update`, `--allow-large-erase`, or `--allow-interrupted-ota` when the matching `PHASE19_ALLOW_*` environment gate is explicitly set and prerequisites pass. [VERIFIED: .planning/phases/19-recovery-regression-and-otawww-evidence/19-03-PLAN.md] [VERIFIED: scripts/phase16-recovery-regression.sh]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `git` | source/reference identity | yes [VERIFIED: local `command -v git`] | `/opt/homebrew/bin/git` [VERIFIED: local `command -v git`] | none |
| `just` | package, detect, flash, parity, reference commands | yes [VERIFIED: local `command -v just`] | 1.48.0 [VERIFIED: local `just --version`] | Direct Bazel/scripts only for diagnostics; plans should prefer `just`. [VERIFIED: Justfile] |
| `bazel` | tests, package, release-gate, parity | yes [VERIFIED: local `command -v bazel`] | 9.1.1 [VERIFIED: local `bazel --version`] | none |
| `cargo` / `rustc` | Rust tests and firmware/tool builds | yes [VERIFIED: local `command -v cargo`; local `command -v rustc`] | cargo/rustc 1.88.0 nightly [VERIFIED: local `cargo --version`; local `rustc --version`] | none |
| `espflash` | board-info, erase, flash, monitor | yes [VERIFIED: local `command -v espflash`] | 4.0.1 [VERIFIED: local `espflash --version`] | managed `.embuild` ESP-IDF tools only when repo guidance allows and `espflash` cannot cover the workflow. [VERIFIED: AGENTS.md] |
| `python3` | JSON parsing/writing in helpers | yes [VERIFIED: local `command -v python3`] | 3.14.4 [VERIFIED: local `python3 --version`] | none |
| `node` | existing WebSocket helpers if needed for ancillary checks | yes [VERIFIED: local `command -v node`] | v24.13.0 [VERIFIED: local `node --version`] | not required for core Phase 19 unless a WebSocket check is added. [VERIFIED: scripts/BUILD.bazel] |
| `shasum` | shell checksum fallback | yes [VERIFIED: local `command -v shasum`] | system tool [VERIFIED: local `command -v shasum`] | `sha256sum` fallback exists in helper. [VERIFIED: scripts/phase16-recovery-regression.sh] |
| Connected Ultra 205 | live hardware and destructive/fault-injection evidence | not probed during research [VERIFIED: research action log] | unknown | Execution plan must run `just detect-ultra205` and block/pending on zero, multiple, or board-info failure. [VERIFIED: AGENTS.md] |
| Reachable `DEVICE_URL` | failed-update, interrupted-update, HTTP/static/recovery/API, OTAWWW probe | not probed during research [VERIFIED: research action log] | unknown | Use explicit origin-only URL or trusted flash-monitor target extraction; otherwise write blocked/pending evidence. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] |

**Missing dependencies with no fallback:**

- A connected Ultra 205 and reachable target are execution-time prerequisites for live evidence; absence should produce pending/blocked artifacts, not failed research. [VERIFIED: AGENTS.md] [VERIFIED: 19-CONTEXT.md]

**Missing dependencies with fallback:**

- No missing local CLI dependency was found for research-time planning; hardware and `DEVICE_URL` are intentionally deferred to gated execution. [VERIFIED: local environment availability audit]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel 9.1.1 with `rules_shell 0.8.0` shell tests and `rules_rust 0.70.0` Rust tests. [VERIFIED: local `bazel --version`] [VERIFIED: MODULE.bazel] [VERIFIED: scripts/BUILD.bazel] |
| Config file | `MODULE.bazel`, `scripts/BUILD.bazel`, `crates/bitaxe-api/BUILD.bazel`, `tools/parity/BUILD.bazel`. [VERIFIED: MODULE.bazel] [VERIFIED: scripts/BUILD.bazel] [VERIFIED: crates/bitaxe-api/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] |
| Quick run command | `bazel test //scripts:phase16_recovery_regression_test //scripts:phase18_firmware_ota_evidence_test //crates/bitaxe-api:tests //tools/parity:tests` [VERIFIED: scripts/BUILD.bazel] [VERIFIED: crates/bitaxe-api/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] |
| Full suite command | `just test` [VERIFIED: Justfile] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| REL-03 | OTAWWW remains an explicit gap unless whole-`www` update and interrupted-update hardware-regression evidence exist. [VERIFIED: .planning/REQUIREMENTS.md] | unit + parity guard | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` | yes [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: tools/parity/src/main.rs] |
| REL-08 | Failed-update, large erase, and interrupted-update require allow gates and post-action proof. [VERIFIED: .planning/REQUIREMENTS.md] | shell unit + hardware execution evidence | `bazel test //scripts:phase16_recovery_regression_test` plus gated execution commands only if allowed | yes for Phase 16 helper; Phase 19 wrapper missing [VERIFIED: scripts/phase16-recovery-regression-test.sh] |
| REL-07 | Operator docs cite exact package, recovery, OTAWWW, restore, and non-claim artifacts. [VERIFIED: .planning/REQUIREMENTS.md] | docs/static check + parity | `rg -n "phase-19-recovery-regression-and-otawww-evidence|failed-update|large erase|interrupted|OTAWWW" docs/release/ultra-205.md docs/parity/checklist.md .planning/REQUIREMENTS.md && just parity` | docs exist; Phase 19 edits not yet present [VERIFIED: docs/release/ultra-205.md] [VERIFIED: docs/parity/checklist.md] |
| API-09 | Static/recovery administration remains usable after failed/interrupted update or remains below verified. [VERIFIED: .planning/REQUIREMENTS.md] | live HTTP/static smoke + shell test | `bazel test //scripts:phase16_recovery_regression_test` and gated helper run with `--allow-failed-update` or `--allow-interrupted-ota` when prerequisites exist | yes for helper tests [VERIFIED: scripts/phase16-recovery-regression-test.sh] |
| EVD-05 | Evidence stack includes unit/helper tests, release-gate, hardware evidence or pending docs, redaction, parity, reference, lifecycle validation. [VERIFIED: .planning/REQUIREMENTS.md] | aggregate verification | `just package && bazel run //tools/parity:report -- release-gate --manifest <phase19-manifest> && just parity && just verify-reference && node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 19 --expect-id 19-2026-07-03T17-34-52 --expect-mode yolo --raw` | commands exist [VERIFIED: Justfile] [VERIFIED: local lifecycle validation output] |

### Sampling Rate

- **Per task commit:** Run the narrow changed-path tests: Phase 19 wrapper test if added, `//scripts:phase16_recovery_regression_test`, `//scripts:phase18_firmware_ota_evidence_test`, `//crates/bitaxe-api:tests`, or `//tools/parity:tests` based on touched files. [VERIFIED: scripts/BUILD.bazel] [VERIFIED: crates/bitaxe-api/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel]
- **Per wave merge:** Run `just parity`, `just verify-reference`, and affected Bazel tests; run hardware commands only when the plan's allow gates and prerequisites are satisfied. [VERIFIED: Justfile] [VERIFIED: 19-CONTEXT.md]
- **Phase gate:** Run `just package`, release-gate on the copied Phase 19 manifest, `just parity`, `just verify-reference`, lifecycle validation, redaction scan/review, and all hardware/network command verifications actually used. [VERIFIED: 19-CONTEXT.md] [VERIFIED: Justfile]

### Wave 0 Gaps

- [ ] `scripts/phase19-recovery-otawww-evidence.sh` and `scripts/phase19-recovery-otawww-evidence-test.sh` if Phase 19 wraps Phase 16 or adds OTAWWW gap/update orchestration. [VERIFIED: 19-CONTEXT.md] [VERIFIED: scripts/BUILD.bazel]
- [ ] `scripts/BUILD.bazel` targets `phase19_recovery_otawww_evidence` and `phase19_recovery_otawww_evidence_test` if new scripts are added. [VERIFIED: scripts/BUILD.bazel]
- [ ] `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/evidence-contract.md` before live artifacts are cited. [VERIFIED: Phase 18 evidence-contract pattern]
- [ ] Final redaction review artifact with absent-not-cited rows for missing body/header/log classes. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md]

## Security Domain

### Applicable ASVS Categories

OWASP ASVS latest stable is 5.0.0 as of the researched date, and it is a verification standard for web application technical controls. [CITED: https://owasp.org/www-project-application-security-verification-standard/] The table below uses the GSD security headings while mapping them to Phase 19's firmware HTTP/update surface. [VERIFIED: 19-CONTEXT.md]

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| Authentication | no | Phase 19 does not add user authentication; it preserves the existing private-network/origin gate and documents no auth expansion. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] |
| Session Management | no | Phase 19 does not create sessions or cookies; WebSocket/session behavior is not changed by the recovery/OTAWWW evidence plan. [VERIFIED: 19-CONTEXT.md] |
| Access Control | yes | Use existing private-network/AP-mode update access decisions and do not bypass access gates in OTA/OTAWWW handlers. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| Input Validation | yes | Validate origin-only `DEVICE_URL`, manifest fields, board `205`, current source commit, artifact presence, response status/body markers, and upload size if OTAWWW is implemented. [VERIFIED: scripts/phase18-firmware-ota-evidence.sh] [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| Cryptography | limited | Use manifest SHA-256 checksums and existing package/release-gate validation; do not add custom crypto. [VERIFIED: tools/xtask/src/package_manifest.rs] [VERIFIED: tools/parity/src/release_gate.rs] |
| File/Upload Handling | yes | Use bounded upload reads, content-length/size checks, ESP-IDF partition bounds, selected response artifacts, and post-action operability checks. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] |
| Logging/Privacy | yes | Redact target URLs, IP/MAC/SSID, credentials, NVS secrets, body snippets, curl errors, detector logs, board-info logs, and serial/recovery logs before commit. [VERIFIED: 19-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md] |

### Known Threat Patterns For Phase 19

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Unauthorized update or recovery request | Elevation of privilege / Tampering | Preserve access gate decisions and AP-mode update rejection; evidence must not bypass firmware route gates. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| Stale package/flash identity | Tampering / Repudiation | Require manifest-backed release gate and commit alignment before hardware action. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: tools/parity/src/release_gate.rs] |
| Partial or interrupted upload leaves device unusable | Denial of service | Require bounded interruption evidence plus post-interruption HTTP/static/recovery/API or serial safe-state proof. [VERIFIED: 19-CONTEXT.md] [VERIFIED: scripts/phase16-recovery-regression.sh] |
| Whole-`www` partition overwrite corrupts static UI | Denial of service / Tampering | Keep OTAWWW as gap unless size checks, chunked erase/write, recovery access, and interrupted-update hardware-regression are implemented and proven. [VERIFIED: 19-CONTEXT.md] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| Evidence leaks private network data or credentials | Information disclosure | Commit only redacted target locks/logs/bodies and keep raw developer artifacts under `target/`. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md] |
| Raw destructive command bypasses recovery path | Denial of service / Repudiation | Use allow flags and repo-owned helper commands; record exact commands, abort conditions, restore command, and safe-state markers. [VERIFIED: 19-CONTEXT.md] [VERIFIED: scripts/phase16-recovery-regression.sh] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md` - locked Phase 19 decisions, discretion, deferred scope, code context, and evidence path. [VERIFIED: local file read]
- `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`, `.planning/PROJECT.md` - requirement mapping, project state after Phase 18, Phase 19 success criteria, and project constraints. [VERIFIED: local file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - repo-local hardware, GSD, evidence, testing, verification, and Rust constraints. [VERIFIED: local file read]
- `scripts/phase16-recovery-regression.sh` and `scripts/phase16-recovery-regression-test.sh` - existing recovery/fault-injection helper pattern and tests. [VERIFIED: local file read]
- `scripts/phase17-live-http-api-smoke.sh`, `scripts/phase18-firmware-ota-evidence.sh`, and their tests - target-lock, no-scan, OTA evidence, and redaction patterns. [VERIFIED: local file read]
- `crates/bitaxe-api/src/update_plan.rs`, `crates/bitaxe-api/src/route_shell.rs`, `firmware/bitaxe/src/http_api.rs`, `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs` - current OTAWWW gap, route, firmware handler, checklist guard, and release-gate implementation. [VERIFIED: local file read]
- `reference/esp-miner/main/http_server/http_server.c`, `reference/esp-miner/readme.md`, `reference/esp-miner/partitions.csv`, `firmware/bitaxe/partitions-ultra205.csv` - pinned upstream OTAWWW/OTA behavior and matching partition layout. [VERIFIED: local file read]
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md`, `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md`, and `redaction-review.md` - latest evidence boundaries and redaction pattern. [VERIFIED: local file read]
- Espressif ESP-IDF HTTP server, partition, and OTA docs - upload read semantics, partition erase/write constraints, and rollback behavior. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html]
- OWASP ASVS project page - current ASVS verification-standard source and latest stable 5.0.0 note. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Secondary (MEDIUM confidence)

- Local command availability audit for `just`, `bazel`, `cargo`, `rustc`, `espflash`, `python3`, `node`, `shasum`, and lifecycle validation. [VERIFIED: local command output]

### Tertiary (LOW confidence)

- None. [VERIFIED: pre-submission review]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - no new dependencies are needed and versions/tools were verified locally or in lock/module files. [VERIFIED: local environment availability audit] [VERIFIED: Cargo.lock] [VERIFIED: MODULE.bazel]
- Architecture: HIGH - existing Phase 16/17/18 helpers and evidence ledgers provide direct patterns. [VERIFIED: scripts/phase16-recovery-regression.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase18-firmware-ota-evidence.sh]
- OTAWWW current behavior: HIGH - pure Rust model, firmware handler, Phase 17 evidence, and parity guards all agree that current OTAWWW is a gap. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] [VERIFIED: tools/parity/src/main.rs]
- Full whole-`www` implementation plan: MEDIUM - upstream and ESP-IDF APIs are clear, but Phase 19 context does not decide to implement it and hardware evidence would still be required. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/partition.html] [VERIFIED: 19-CONTEXT.md]
- Pitfalls: HIGH - current residual gaps and overclaim risks are documented in Phase 17/18 evidence and parity guard tests. [VERIFIED: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md] [VERIFIED: tools/parity/src/main.rs]

**Research date:** 2026-07-03 [VERIFIED: local environment current date]
**Valid until:** 2026-07-10 for external docs/tool versions; repo-local evidence findings remain valid until Phase 19 implementation changes these files. [VERIFIED: research freshness policy]
