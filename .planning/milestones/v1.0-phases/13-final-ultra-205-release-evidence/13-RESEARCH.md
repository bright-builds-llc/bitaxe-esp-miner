# Phase 13: Final Ultra 205 Release Evidence - Research

**Researched:** 2026-06-30
**Domain:** Ultra 205 release-candidate package identity, live HTTP/static/recovery/OTA evidence, destructive recovery procedures, checklist promotion, and final release gating
**Confidence:** HIGH for repo-local package, parity, release-gate, and evidence patterns; MEDIUM for live HTTP/OTA/destructive outcomes because they depend on a reachable `DEVICE_URL`, connected board state, and documented recovery paths.

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md`. [VERIFIED: .planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md]

### Locked Decisions

- Treat the current source commit as release-candidate identity only after `just package` produces a manifest with source commit, reference commit, artifacts, offsets, checksums, tool versions, and release/license/provenance paths.
- Run manifest-backed `release-gate` before trusting final hardware evidence.
- Start every live hardware attempt with `just detect-ultra205`; continue only when exactly one likely ESP32-S3 port and board-info succeed.
- Require a reachable `DEVICE_URL` for live HTTP/static/recovery/OTA probes. If unavailable, record `DEVICE_URL status: blocked` and keep affected rows below `verified`.
- Use repo-owned commands first. Direct `espflash`, `esptool.py`, erase, rollback, interrupted-update, or raw write commands are allowed only when a plan names the exact command, recovery path, stop conditions, and evidence artifacts.
- Do not mix package artifacts from one commit with serial, HTTP, OTA, rollback, or recovery evidence from another commit.
- Keep OTAWWW as the explicit REL-03 gap unless a full whole-`www` update procedure with recovery and interrupted-update hardware-regression evidence is added.
- Redaction review is required before committing logs, JSON, Markdown evidence, release summaries, or command outputs.

### Deferred Ideas (Out Of Scope)

- Non-205 boards, additional ASIC families, Stratum v2, BAP, all-board release matrices, Angular AxeOS replacement, and production mining performance tuning.
- Full OTAWWW whole-`www` update parity unless Phase 13 safely adds interrupted-update hardware-regression evidence.
- Broad runtime display/input, voltage/fan stress, long mining soak, or unbounded destructive/fault injection.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Requirement | Research Support |
| --- | --- | --- |
| FND-06 | Firmware can boot on Ultra 205 and log identity, platform status, reset reason, partition/image identity, and board/ASIC target while mining/control remain disabled. | Use `just flash-monitor board=205 port=<port> evidence-dir=<path>` after `just package` and detector gate; cite wrapper JSON/log artifacts. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] |
| API-09 | Static AxeOS assets and recovery page behavior remain compatible enough for device administration. | Live HTTP smoke must cover `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, and route coexistence. [VERIFIED: crates/bitaxe-api/src/static_plan.rs] [VERIFIED: firmware/bitaxe/src/static_files.rs] |
| REL-01 | Partition, filesystem, SPIFFS/static assets, and recovery assets support expected flash/admin flows. | Pair manifest/package evidence with live SPIFFS/static/recovery observations and post-recovery operability. [VERIFIED: tools/xtask/src/package_manifest.rs] [VERIFIED: firmware/bitaxe/src/filesystem.rs] |
| REL-02 | Firmware OTA route accepts, rejects, applies, logs, and recovers from updates with compatible behavior. | Test valid `esp-miner.bin`, invalid image rejection, reboot, post-reboot identity, and boot-validation/rollback logs. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/ota_update.rs] |
| REL-03 | OTAWWW/static-asset update is implemented or reported as a V1 gap with evidence and owner. | Keep OTAWWW fail-closed with public response `Wrong API input` unless whole-`www` interruption/recovery evidence exists. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: docs/release/ultra-205.md] |
| REL-04 | Release packaging produces named artifacts with checksums, manifests, image metadata, install notes, and source/reference commits. | `just package` and manifest-backed release gate are mandatory before flash evidence. [VERIFIED: scripts/package-firmware.sh] [VERIFIED: tools/parity/src/release_gate.rs] |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update cases have verification evidence before release parity is claimed. | Use phase-gated destructive runbooks with exact recovery path and stop conditions. Missing prerequisites produce pending evidence, not verified claims. [VERIFIED: docs/release/ultra-205.md] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke, and hardware regression/soak where appropriate. | Combine host tests, package/release-gate, wrapper serial evidence, live HTTP/OTA evidence, redaction review, checklist updates, `just parity`, and phase verifier. [VERIFIED: tools/parity/src/main.rs] |
</phase_requirements>

## Summary

Phase 13 is a release-evidence closure phase, not a broad feature phase. Most implementation surfaces already exist: package manifest v2, factory image generation, wrapper-owned flash-monitor evidence, route manifest policy, static/recovery serving, firmware OTA, boot validation, parity guards, release docs, and prior safety/ASIC evidence ledgers. The missing value is final source-commit-to-hardware identity and live evidence for the release-sensitive paths that Phase 8 left blocked by no reachable `DEVICE_URL`. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-summary.md]

**Primary recommendation:** plan Phase 13 as an ordered evidence ladder: package and release-gate identity, detector-gated factory flash and serial boot, explicit `DEVICE_URL` resolution, live static/recovery smoke, valid/invalid firmware OTA plus boot-validation/rollback observation, destructive/recovery procedures only where safe, checklist/docs update, redaction review, and final verification. [VERIFIED: .planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md]

## Project Constraints

- Keep `reference/esp-miner` read-only. Use it as behavioral evidence and provenance reference only. [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]
- Use ESP-IDF and esp-rs tooling through existing repo wrappers before adding alternate tool paths. [VERIFIED: AGENTS.md]
- Run `just detect-ultra205` before autonomous hardware use and stop on zero ports, multiple ports, board-info failure, non-205 target, or missing recovery/evidence instructions. [VERIFIED: AGENTS.md]
- Destructive or fault-injection verification is allowed only when the active phase plan documents recovery path and required evidence. [VERIFIED: AGENTS.md]
- Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, API tokens, NVS secret values, or raw terminal secrets. [VERIFIED: AGENTS.md]
- Preserve functional core plus imperative shell: pure route/update/static decisions stay in crates; ESP-IDF HTTP, SPIFFS, OTA, reboot, rollback, serial, and flash effects stay in firmware/tools. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]
- For Rust code changes, satisfy the repo pre-commit sequence before committing: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: AGENTS.md]

## Existing Integration Points

| Surface | Existing Asset | Phase 13 Use |
| --- | --- | --- |
| Package workflow | `Justfile`, `scripts/package-firmware.sh`, `tools/xtask/src/package_manifest.rs` | Build final manifest and artifacts, record checksums and commits. |
| Release gate | `tools/parity/src/release_gate.rs`, `tools/parity/src/main.rs` | Validate release docs plus manifest-backed artifact requirements before hardware claims. |
| USB evidence wrapper | `tools/flash/src/main.rs` | Produce trusted serial boot evidence tied to manifest, source commit, reference commit, board, and port. |
| Detector gate | `scripts/detect-ultra205.sh` | Required first command before hardware use; records board-info output. |
| Route manifest | `crates/bitaxe-api/src/route_shell.rs` | Supplies required live routes: firmware OTA, OTAWWW gap, `/recovery`, and static wildcard. |
| Static/recovery decisions | `crates/bitaxe-api/src/static_plan.rs`, `firmware/bitaxe/src/static_files.rs`, `firmware/bitaxe/static/www/`, `firmware/bitaxe/static/recovery_page.html` | Define live static/recovery behavior to probe and evidence. |
| OTA decisions/effects | `crates/bitaxe-api/src/update_plan.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/ota_update.rs`, `firmware/bitaxe/src/boot_validation.rs` | Provide firmware OTA accept/reject/status, ESP-IDF streaming, reboot, and boot-validation behavior. |
| Evidence ledger | `docs/parity/checklist.md`, `docs/parity/evidence/` | Final release evidence citations and conservative row promotion. |
| Operator docs | `docs/release/ultra-205.md` | Existing destructive/recovery procedure source and final docs update target. |

## Architecture Patterns

### Pattern 1: Package-To-Hardware Identity Chain

**What:** Record `just package`, package manifest path, artifact checksums, source commit, reference commit, release-gate result, detector output, flash-monitor JSON/log, `DEVICE_URL`, and HTTP/OTA observations in one Phase 13 ledger.

**Why:** Phase 13 specifically closes the missing final-commit package-to-hardware evidence identity. Without a single chain, later reviewers cannot prove the live device ran the cited release artifacts.

### Pattern 2: Device URL As A Hard Gate

**What:** Treat `DEVICE_URL` as a first-class evidence input. The helper or runbook should record how it was provided, sanitize it if private, prove reachability, and stop if it is missing or points at the wrong target.

**Why:** Phase 8 left release evidence blocked by no reachable `DEVICE_URL`. The fix is not to infer success from route registration; live HTTP claims need live HTTP responses.

### Pattern 3: Small Repo-Owned Probe Helper

**What:** Add a focused HTTP/OTA evidence helper only if it reduces ad hoc `curl` usage. It should read `DEVICE_URL`, package manifest, and artifact paths; issue bounded requests; record status codes, selected headers, response snippets, artifact checksums, and conclusions; and avoid secret-bearing output.

**Why:** Manual `curl` transcripts are easy to under-record or over-record. A small helper can make evidence repeatable while still leaving destructive actions gated by plan/runbook instructions.

### Pattern 4: Destructive Runbooks Before Destructive Commands

**What:** Large erase, rollback, failed update recovery, and interrupted-update evidence must have exact command, expected failure/interruption point, stop condition, factory image recovery path, post-recovery checks, and redaction review before execution.

**Why:** Release parity claims are high risk. The repo-local hardware guidance permits destructive checks only with documented recovery procedure and required evidence.

### Pattern 5: Checklist Promotion Last

**What:** Update `docs/parity/checklist.md` only after evidence exists and the row can state the exact evidence class. Then run `just parity`.

**Why:** Existing parity guards reject release/OTA/static verified rows with missing live terms or blocker language. Checklist changes should be the conclusion, not the driver.

## Common Pitfalls

### Pitfall 1: Treating Serial Route Registration As Live HTTP Proof

Serial logs showing `/recovery`, `/api/system/OTA`, and `/api/system/OTAWWW` registered are useful, but they do not prove `/`, `/assets/app.css.gz`, redirects, recovery HTML, OTA upload, or OTAWWW response over HTTP.

### Pitfall 2: Treating Valid OTA Upload As Rollback Proof

Successful `/api/system/OTA` response text does not prove rollback or boot validation. Evidence must capture post-reboot identity and boot-validation/partition state.

### Pitfall 3: Verifying OTAWWW From `www.bin`

`www.bin` package generation proves an artifact exists. It does not prove whole-`www` update behavior, recovery after interruption, or static update parity.

### Pitfall 4: Running Destructive Commands Without A Recovery Path

Erase, rollback, interrupted update, and raw flash writes can leave the board unusable for the phase. If recovery is not documented with the current factory image, record pending evidence instead.

### Pitfall 5: Leaking Network Or Credential Data

`DEVICE_URL`, HTTP headers, upload logs, NVS dumps, pool config, Wi-Fi state, and terminal environment can include private data. Redaction review must happen before staging evidence.

### Pitfall 6: Allowing The Auto-Chain To Commit Failed Evidence

This wrapper should push only after verification status is `passed` and lifecycle validation passes. If live evidence is blocked, verification may still pass only if the phase explicitly records pending evidence and keeps release rows below verified; otherwise commit/push must stop.

## Planning Guidance

Recommended plan split:

1. **Package identity and release-gate baseline:** Run package/release-gate, capture manifest identity, and establish evidence ledger/redaction structure.
2. **Detector, flash, and serial boot evidence:** Use `just detect-ultra205` and `just flash-monitor` to bind the final package to a live Ultra 205.
3. **HTTP/static/recovery probe path:** Establish `DEVICE_URL`, add or run repo-owned live probes for static/recovery/route coexistence, and record blocked evidence if unavailable.
4. **Firmware OTA, invalid image, and boot validation:** Exercise valid/invalid OTA only after recovery is documented; capture reboot/partition/identity logs.
5. **Destructive recovery evidence or explicit pending records:** Run rollback, large erase, failed-update, and interrupted-update checks only with documented recovery and stop conditions.
6. **Checklist, docs, parity, and verification closure:** Update checklist/docs conservatively, run redaction review, `just parity`, release gate, and final verification.

Plans may merge or split these steps, but every plan should map to FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-08, or EVD-05.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust unit tests through Cargo and Bazel, repo shell scripts, package/release-gate validation, and detector-gated hardware/network evidence. |
| Config file | Workspace `Cargo.toml`, `MODULE.bazel`, `Justfile`, `tools/parity/BUILD.bazel`, `tools/flash/BUILD.bazel`, and firmware/tool `BUILD.bazel` files. |
| Quick run command | `just parity` after checklist/evidence changes; targeted `cargo test -p bitaxe-api -p bitaxe-parity --all-features` after route/update/parity logic changes. |
| Full suite command | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features && just package && just parity` |
| Release gate command | `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| Hardware gate command | `just detect-ultra205` |
| Hardware evidence command | `just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence` |
| Network evidence input | `DEVICE_URL=<reachable Ultra 205 URL>` or equivalent plan-defined argument. |
| Estimated runtime | Host checks: minutes. Hardware/network/destructive evidence: depends on package build, USB port, device network reachability, OTA reboot time, and recovery prerequisites. |

### Phase Requirements To Verification Map

| Requirement | Automated Verification | Hardware / Network Verification | Evidence Artifact |
| --- | --- | --- | --- |
| FND-06 | Package manifest and flash wrapper tests remain green if touched. | Flash-monitor log records identity, platform, reset reason, partition/image state, board/ASIC, and safe startup. | Phase 13 ledger plus `flash-command-evidence.json` and `flash-monitor.log`. |
| API-09 | Static/recovery route unit tests and API compare stay green. | `DEVICE_URL` responses for `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, and route coexistence. | HTTP probe JSON/log plus ledger summary. |
| REL-01 | Manifest/release-gate proves artifact layout and docs. | Live SPIFFS/static/recovery behavior after factory flash and any recovery action. | Package manifest, release-gate output, HTTP evidence. |
| REL-02 | Update-plan tests and firmware build stay green. | Valid OTA, invalid image rejection, reboot, post-reboot identity, boot-validation/rollback logs. | OTA evidence pack and serial/monitor logs. |
| REL-03 | OTAWWW gap tests stay green unless intentionally expanded. | Public `Wrong API input` response, or full whole-`www` interrupted-update hardware-regression evidence if implemented. | OTAWWW section in ledger and checklist. |
| REL-04 | Package manifest/release-gate checks pass. | Package artifacts used for live flash and OTA match the manifest. | Manifest, checksums, commands, evidence ledger. |
| REL-08 | Destructive runbook files and evidence schemas are reviewed. | Rollback, large erase, failed-update, and interrupted-update evidence only when recovery path exists. | Recovery evidence sections, generated logs, redaction review. |
| EVD-05 | `just parity`, targeted tests, and lifecycle validation pass. | Evidence layers are present or pending with exact blocker language and no overclaims. | Checklist, verification report, release summary. |

### Sampling Strategy

- After Markdown evidence/docs changes: check for only top-level frontmatter `---` delimiters where frontmatter is present; run `just parity` if checklist/evidence rows changed.
- After helper script changes: run syntax checks and any script tests; do not hide failures with `|| true`.
- After Rust tool/crate changes: run targeted Cargo tests for affected packages before full pre-commit checks.
- Before live hardware: run `just detect-ultra205`.
- Before live HTTP/OTA: prove `DEVICE_URL` reaches the just-flashed device or record blocker.
- After live evidence: run redaction review before staging generated logs/JSON.
- Before phase completion: run release gate, `just parity`, relevant Rust checks, and phase verifier.

### Manual-Only Verifications

| Behavior | Requirement | Why Manual | Instructions |
| --- | --- | --- | --- |
| Connected Ultra 205 detector gate | FND-06, EVD-05 | Requires physical USB board. | Run `just detect-ultra205`; proceed only with exactly one successful `port=<path>`. |
| Live `DEVICE_URL` reachability | API-09, REL-01, REL-02 | Requires device network setup. | Provide a reachable URL for the just-flashed board; record sanitized target and reachability result. |
| Firmware OTA and reboot observation | REL-02, REL-08 | Requires live upload, reboot, and post-reboot logs. | Use manifest `esp-miner.bin`, capture response, monitor reboot, and record boot-validation state. |
| Rollback, large erase, failed update, interrupted update | REL-08 | Destructive or fault-injection behavior needs recovery control. | Run only plan-approved commands with current factory image recovery path and stop conditions. |
| Redaction review | EVD-05 | Humans must confirm generated evidence is secret-free. | Inspect all generated artifacts before commit; record result in ledger or `redaction-review.md`. |

### Validation Risks

- `DEVICE_URL` may be unavailable. Correct output is explicit pending evidence and no verified release rows.
- Valid OTA may reboot the device and disrupt the current network address. Plans should include how to re-establish `DEVICE_URL` or record that as a blocker.
- Large erase and interrupted update can require USB recovery. Do not run unless the current factory image recovery path is proven.
- Full pre-commit checks are mandatory before final code commits in this Rust repo when code changed.
