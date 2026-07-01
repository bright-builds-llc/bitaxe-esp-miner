# Phase 16: Current Commit Release Evidence Completion - Research

**Researched:** 2026-07-01  
**Domain:** Ultra 205 release evidence orchestration, current-commit package identity, live HTTP/OTA/recovery evidence, and gated recovery regression  
**Confidence:** HIGH for local tooling and evidence surfaces; MEDIUM for hardware/network availability because this research intentionally did not run detector, flash, erase, or `DEVICE_URL` probes. [VERIFIED: user prompt] [VERIFIED: local file .planning/config.json] [VERIFIED: local commands]

<user_constraints>
## User Constraints (from CONTEXT.md)

The following locked decisions, discretion areas, and deferred ideas are copied from `.planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md`. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

### Locked Decisions

## Implementation Decisions

### Same-Commit Release Identity

- **D-01:** Treat the current source commit as the only release-candidate identity for this phase. Run `just package` after the phase starts, record the current source commit, reference commit, package manifest, artifact paths, checksums, install notes, and release-gate result, then flash and monitor artifacts from that same manifest.
- **D-02:** Do not reuse Phase 13 package, serial, HTTP, OTA, or recovery evidence as current release proof. Phase 13 source commit `190849539700b8f9a7909fd2b6ebd84142557968` is historical evidence and must remain below current-commit release parity unless the current commit matches it.
- **D-03:** Package-to-hardware identity is a hard gate. If `flash-monitor` evidence, HTTP probes, OTA runs, or recovery runs point at a different source commit, unknown manifest, stale factory image, wrong board, or untrusted wrapper output, record the mismatch and keep affected rows below `verified`.

### Live Device URL And HTTP/OTA Evidence

- **D-04:** Require an explicit reachable `DEVICE_URL` for live HTTP/static/recovery/OTA evidence. The phase must not infer the target through network scanning; it may accept `DEVICE_URL` from environment, argument, or a documented file-backed source if the artifact records how the target was identified.
- **D-05:** Live HTTP evidence must cover `/`, `/assets/app.css.gz`, a missing static path redirect/body, `/recovery`, API route coexistence, `/api/ws`, `/api/ws/live`, `/api/system/OTA`, and `/api/system/OTAWWW`. Each probe should record method, path, sanitized target, status, selected headers, sanitized response snippet or marker, expected result, actual result, and conclusion.
- **D-06:** Firmware OTA evidence must cover a valid `esp-miner.bin` upload, invalid image rejection, AP/APSTA rejection when applicable, public response/status, selected next partition or explicit unavailable state, reboot scheduling, post-reboot firmware identity, and boot-validation or rollback state.
- **D-07:** OTAWWW remains the REL-03 static-update gap unless this phase captures whole-`www` partition update behavior, recovery access, and interrupted-update hardware-regression evidence. A `www.bin` package artifact or `Wrong API input` response alone must not verify OTAWWW parity.

### Rollback, Erase, Failed-Update, And Interrupted-Update Gates

- **D-08:** Destructive and fault-injection evidence may run only through documented, phase-owned helpers or repo-owned commands that name exact commands, prerequisites, allow flags, abort conditions, recovery steps, expected safe-state markers, and evidence output paths.
- **D-09:** Before any erase, rollback, failed-update, interrupted-update, or raw recovery operation, rerun `just detect-ultra205`, require exactly one selected port, require successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`, require board `205`, require the current package manifest and factory image, and record the detector output.
- **D-10:** Large erase evidence must record the erase command, tool version, factory reflash command, monitor command, post-erase boot identity, safe state, SPIFFS/static/recovery/API reachability, and final restore conclusion. If any prerequisite is missing, write pending evidence and do not run erase.
- **D-11:** Failed-update and interrupted-update evidence must record route, artifact name and checksum, failure or interruption point, public status/body, post-failure partition/static/API state, recovery steps, final safe state, and redaction review. Invalid upload rejection is not rollback proof by itself.

### Checklist, Release Docs, Redaction, And Final Verification

- **D-12:** Promote checklist rows only to the exact evidence tier supported by the current Phase 16 artifacts. Static, recovery, firmware OTA, rollback, erase, failed-update, interrupted-update, and release rows remain below `verified` when live evidence is blocked, partial, stale, or redaction-uncleared.
- **D-13:** Update release documentation, parity checklist, requirements traceability, milestone audit, and release-summary material only after evidence artifacts exist. Documentation should cite commands and artifacts, not restate goals as proof.
- **D-14:** Every generated log, JSON, Markdown evidence file, API response, WebSocket capture, and pasted terminal snippet must receive redaction review before commit. The review must cover pool credentials, worker secrets, Wi-Fi credentials, private endpoints, explicit `DEVICE_URL`, API tokens, NVS secret values, local terminal secrets, and recovery logs.
- **D-15:** Final verification must run the relevant repo-native checks for changed paths, `just package`, manifest-backed release gate, `just parity`, `just verify-reference`, redaction review, lifecycle validation, and any hardware/network/recovery commands the phase actually used. No final wrapper commit or push should happen unless `16-VERIFICATION.md` has `status: passed` and lifecycle validation passes for `16-2026-07-01T12-36-46`.

### the agent's Discretion

The agent may choose the exact plan split, evidence helper names, evidence
directory layout, JSON field names, sanitized HTTP output shape, checklist
wording, and whether current-commit release evidence helpers live in shell,
Rust host tools, or existing parity/flash tooling. Those choices must preserve
repo-owned ESP/esp-rs tooling, keep `reference/esp-miner` read-only, prefer
typed parsing for manifests and probe results where practical, avoid ad hoc
destructive commands, preserve functional core plus imperative shell, and avoid
standalone body `---` separators in parsed Markdown artifacts.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, production mining optimization, active voltage/fan stress, broad runtime display/input parity, and long stress/soak runs remain out of Phase 16.
- Full OTAWWW whole-`www` update parity remains deferred unless Phase 16 captures exact whole-partition write, recovery, and interrupted-update hardware-regression evidence.
- Live pool evidence and accepted/rejected share behavior remain outside Phase 16 unless needed only as context and already covered by safe, redaction-cleared prerequisites.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| FND-06 | The ESP-IDF Rust firmware app can boot on Ultra 205 and log firmware identity, platform status, reset reason, partition/image identity, and selected board/ASIC target while mining and hardware control remain disabled. [VERIFIED: local file .planning/REQUIREMENTS.md] | Use `just detect-ultra205` and `just flash-monitor board=205 port=<path> manifest=<current manifest> evidence-dir=<phase16>/serial-boot`; `tools/flash` trusted-output checks require commit, reference, reset, SPIFFS, route shell, ESP-IDF, and safe-state markers. [VERIFIED: local files Justfile, scripts/detect-ultra205.sh, tools/flash/src/main.rs] |
| API-09 | Static AxeOS assets and recovery page behavior remain compatible enough for device administration without requiring an Angular rewrite in V1. [VERIFIED: local file .planning/REQUIREMENTS.md] | Reuse or extend `scripts/phase13-http-static-smoke.sh` for `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, API coexistence, and WebSocket coexistence with explicit `DEVICE_URL`; add `/api/system/OTA` route coverage for D-05. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, crates/bitaxe-api/src/static_plan.rs, firmware/bitaxe/src/static_files.rs] |
| REL-01 | Partition layout, filesystem layout, SPIFFS/static assets, and recovery assets support the same user-facing flash and administration flows expected from upstream. [VERIFIED: local file .planning/REQUIREMENTS.md] | Use `just package` manifest artifacts, flash-monitor SPIFFS markers, HTTP/static/recovery probes, and post-restore static/API smoke after gated recovery operations. [VERIFIED: local files scripts/package-firmware.sh, tools/xtask/src/package_manifest.rs, scripts/phase13-recovery-regression.sh] |
| REL-02 | Firmware OTA route behavior accepts, rejects, applies, logs, and recovers from updates with upstream-compatible observable behavior. [VERIFIED: local file .planning/REQUIREMENTS.md] | Use `scripts/phase13-firmware-ota-smoke.sh` pattern for valid `esp-miner.bin`, invalid rejection, manifest checksum match, post-reboot identity, and `ota_boot_validation=` markers. [VERIFIED: local files scripts/phase13-firmware-ota-smoke.sh, crates/bitaxe-api/src/update_plan.rs, firmware/bitaxe/src/ota_update.rs, firmware/bitaxe/src/boot_validation.rs] |
| REL-03 | OTAWWW or static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner. [VERIFIED: local file .planning/REQUIREMENTS.md] | Keep OTAWWW deferred unless whole-`www` partition update, recovery access, and interrupted-update hardware-regression are actually captured; the current code documents fail-closed `Wrong API input` behavior. [VERIFIED: local files .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md, docs/parity/checklist.md, crates/bitaxe-api/src/update_plan.rs] |
| REL-04 | Release packaging produces named artifacts with checksums, manifests, image metadata, installation notes, and source/reference commit identifiers. [VERIFIED: local file .planning/REQUIREMENTS.md] | `just package` and `tools/parity release-gate --manifest` already validate manifest v2 artifacts, checksums, install notes, license inventory, and provenance. [VERIFIED: local files Justfile, scripts/package-firmware.sh, tools/xtask/src/package_manifest.rs, tools/parity/src/release_gate.rs] |
| REL-07 | Build, flash, monitor, OTA, and recovery documentation is sufficient for a developer with a connected Ultra 205 to operate the firmware safely. [VERIFIED: local file .planning/REQUIREMENTS.md] | Update `docs/release/ultra-205.md`, Phase 16 evidence ledger, redaction review, and checklist only after evidence exists; cite commands and artifact paths. [VERIFIED: local files docs/release/ultra-205.md, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update cases have verification evidence before release parity is claimed. [VERIFIED: local file .planning/REQUIREMENTS.md] | Existing recovery helper has allow-flag gates and large-erase detector recheck, but Phase 16 should strengthen all erase/rollback/failed/interrupted actions with a current package and detector allow contract before execution. [VERIFIED: local files scripts/phase13-recovery-regression.sh, tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: local file .planning/REQUIREMENTS.md] | Reuse Bazel shell tests, Cargo unit tests, release gate, parity validator, reference guard, redaction review, hardware smoke, and hardware-regression ledgers; Nyquist validation is enabled for this repo. [VERIFIED: local files .planning/config.json, scripts/BUILD.bazel, tools/parity/src/main.rs, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md] |
</phase_requirements>

## Summary

Phase 16 is an evidence-completion phase, not a feature expansion phase. The plan should package the current source commit, prove that the exact manifest is flashed, then run live HTTP/static/recovery/OTA and gated recovery-regression evidence only when the explicit device, detector, package identity, and redaction prerequisites are satisfied. [VERIFIED: local files .planning/ROADMAP.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

The most important implementation gap is not firmware code; it is orchestration rigor. Phase 13 already has package, flash-monitor, HTTP/static, firmware OTA, and recovery helpers, but its final evidence is stale for the current commit and blocked on `DEVICE_URL` plus allow flags. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md] Phase 16 should reuse those helpers where they satisfy the contract and add current-commit identity plus stricter gates where they do not. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh, scripts/phase13-recovery-regression.sh]

**Primary recommendation:** Build a Phase 16 evidence wrapper layer around current `just package`, release gate, detector, flash-monitor, Phase 13 HTTP/OTA/recovery helpers, and redaction/checklist updates; add a typed Phase 16 allow/identity gate if destructive and fault-injection evidence will run. [VERIFIED: local files Justfile, tools/flash/src/main.rs, tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs]

## Project Constraints (from AGENTS.md)

- Repo work must follow repo-local `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant `standards/` pages before planning or implementation. [VERIFIED: local files AGENTS.md, AGENTS.bright-builds.md, standards/index.md]
- `reference/esp-miner` is read-only behavioral evidence, not a workspace for edits. [VERIFIED: local files AGENTS.md, docs/adr/0005-read-only-reference-implementation.md]
- Use repo-owned ESP/esp-rs tooling first: pinned `esp-idf-sys`, checked-in Cargo config, `espup`, `ldproxy`, `embuild`, managed `.embuild` tools, and `espflash` before custom CMake, PlatformIO, or manually managed ESP-IDF installs. [VERIFIED: local file AGENTS.md]
- Treat `.embuild/` as local, gitignored, generated ESP-IDF/esp-rs tool state; do not commit or hand-edit it. [VERIFIED: local file AGENTS.md]
- Before autonomous Ultra 205 hardware use, run `just detect-ultra205`; success requires exactly one likely ESP USB serial port and successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: local files AGENTS.md, scripts/detect-ultra205.sh]
- Stop or record evidence pending when there are zero likely ports, multiple likely ports, board-info fails, the target is not board `205`, or required recovery/evidence instructions are missing. [VERIFIED: local file AGENTS.md]
- Phase-gated destructive or fault-injection verification is allowed only when the active phase plan documents recovery path and required evidence. [VERIFIED: local file AGENTS.md]
- Every hardware run must record board `205`, port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, logs, observed behavior, and conclusion. [VERIFIED: local file AGENTS.md]
- Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence. [VERIFIED: local file AGENTS.md]
- GSD and frontmatter-parsed Markdown must not use standalone body `---` separators after YAML frontmatter. [VERIFIED: local files AGENTS.md, standards/core/verification.md]
- Bright Builds standards prefer functional core plus imperative shell, typed parse boundaries, illegal-state-reducing structures, early returns, repo-native verification, and focused Arrange/Act/Assert tests for changed pure logic. [VERIFIED: local files AGENTS.bright-builds.md, standards/core/architecture.md, standards/core/code-shape.md, standards/core/testing.md, standards/languages/rust.md]

## Standard Stack

### Core

| Tool / Library | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `just` | 1.48.0 [VERIFIED: local command `just --version`] | Human command surface for package, flash-monitor, detector, parity, test, and reference guard. | `Justfile` defines canonical phase-relevant commands. [VERIFIED: local file Justfile] |
| Bazel / Bazelisk | 9.1.1 [VERIFIED: local command `bazel --version`] | Canonical automation graph for package, tests, parity, flash tools, and script wrappers. | Repo stack pins Bazel 9.1.1 and routes `just package`, `just test`, `just flash-monitor`, and `just parity` through Bazel where practical. [VERIFIED: local files .bazelversion, Justfile, AGENTS.md] |
| Rust/Cargo | rustc 1.88.0-nightly, cargo 1.88.0-nightly [VERIFIED: local commands `rustc --version`, `cargo --version`] | Host tooling, parity validators, flash wrapper, xtask packaging, and firmware crate build path. | Existing tools are Rust crates with Bazel targets and Cargo tests. [VERIFIED: local files Cargo.toml, tools/parity/BUILD.bazel, tools/flash/BUILD.bazel, tools/xtask/BUILD.bazel] |
| ESP-IDF Rust stack | ESP-IDF `v5.5.4` in release metadata [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, tools/parity/src/release_gate.rs] | Firmware baseline and release identity. | Project stack mandates ESP-IDF Rust for production firmware parity. [VERIFIED: local file AGENTS.md] |
| `espflash` | 4.0.1 [VERIFIED: local command `espflash --version`] | Board-info, flash, monitor, factory image write, and OTA package image creation. | Repo flash wrapper and package script use `espflash`; AGENTS prefers it where it suffices. [VERIFIED: local files tools/flash/src/main.rs, scripts/package-firmware.sh, AGENTS.md] |
| `tools/flash` | repo-owned [VERIFIED: local file tools/flash/src/main.rs] | Flash, monitor, flash-monitor, manifest resolution, evidence JSON/log capture, trusted marker classification. | It already checks source/reference identity and safe boot markers for evidence. [VERIFIED: local file tools/flash/src/main.rs] |
| `tools/parity` | repo-owned [VERIFIED: local files tools/parity/src/main.rs, tools/parity/src/release_gate.rs] | Release gate, checklist validation, API compare, safety/mining allow validators. | It enforces release-sensitive evidence classes and manifest-backed release gates. [VERIFIED: local file tools/parity/src/main.rs] |

### Supporting

| Tool / Library | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| Bash | GNU bash 3.2.57 [VERIFIED: local command `bash --version`] | Existing shell helpers and tests. | Use for Phase 16 orchestration wrappers that mostly call existing repo commands and record evidence. [VERIFIED: local files scripts/phase13-*.sh, scripts/BUILD.bazel] |
| Python 3 | 3.14.4 [VERIFIED: local command `python3 --version`] | Existing shell helper JSON extraction. | Use only for lightweight manifest reads in shell helpers when a Rust validator is unnecessary. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh] |
| Node.js | v24.13.0 [VERIFIED: local command `node --version`] | GSD lifecycle tooling and Phase 15 WebSocket helper precedent. | Use for lifecycle validation and only for WebSocket capture if a live WebSocket check becomes part of evidence. [VERIFIED: local files .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md, scripts/phase15-websocket-capture.mjs] |
| `curl` | 8.7.1 [VERIFIED: local command `curl --version`] | Explicit `DEVICE_URL` HTTP/static/OTA probes. | Use only with an explicit supplied URL; no scanning or target inference. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh] |
| `espup` | 0.15.1 [VERIFIED: local command `espup --version`] | ESP Rust toolchain installation. | Available for local toolchain support; `just bootstrap-esp` is the explicit installer path. [VERIFIED: local file AGENTS.md] |
| `ldproxy` | installed but no usable `--version` output [VERIFIED: local command `ldproxy --version` returned linker-argument panic] | ESP-IDF Rust linker proxy. | Treat as present but not version-probe friendly; do not rely on `ldproxy --version` in evidence. [VERIFIED: local command] |
| Managed `.embuild` `esptool.py` | generated by pinned ESP-IDF workflow, not globally on PATH [VERIFIED: local command `esptool.py version`; VERIFIED: local file AGENTS.md] | Factory image merge and ESP-IDF support scripts. | Use via `scripts/package-firmware.sh` after the pinned workflow generates it; do not require global `esptool.py`. [VERIFIED: local files scripts/package-firmware.sh, AGENTS.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Existing `tools/flash` wrapper | Direct `espflash` commands | Direct commands lose manifest resolution and trusted evidence JSON; use direct `espflash` only where an existing helper already documents the exact destructive command, such as erase. [VERIFIED: local files tools/flash/src/main.rs, scripts/phase13-recovery-regression.sh] |
| Existing Phase 13 HTTP/OTA/recovery helpers | Fresh one-off curl scripts | One-off scripts risk missing redaction, blocked-state, and no-scan behavior already tested in Phase 13 helpers. [VERIFIED: local files scripts/phase13-*-test.sh] |
| Typed Rust allow validator | Shell-only allow flags | Shell flags are already present, but typed validators in Phase 14/15 catch board, detector, package identity, command, and redaction mismatches before live action. [VERIFIED: local files tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs] |
| Network discovery | Explicit `DEVICE_URL` | Discovery is forbidden for this phase; use explicit URL or record blocked evidence. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |

**Installation:** No new dependencies are recommended for planning Phase 16. If prerequisites are missing during execution, use existing `just doctor` for read-only diagnosis and `just bootstrap-esp` for explicit ESP tooling setup. [VERIFIED: local files AGENTS.md, Justfile]

```bash
just doctor
just bootstrap-esp
```

**Version verification:** For this research, package registry checks are not applicable because no new npm/Cargo package dependency is recommended. Versions above were verified with local CLI commands. [VERIFIED: local commands]

## Architecture Patterns

### Recommended Project Structure

```text
docs/parity/evidence/phase-16-current-commit-release-evidence-completion/
├── package-release-gate.md
├── current-package/
│   └── bitaxe-ultra205-package.json
├── serial-boot/
│   ├── flash-command-evidence.json
│   └── flash-monitor.log
├── http-static-recovery/
│   └── http-static-smoke.log
├── firmware-ota/
│   ├── firmware-ota-smoke.log
│   └── post-ota-monitor.log
├── recovery-regression/
│   ├── recovery-regression.log
│   ├── large-erase.log
│   └── interrupted-ota.log
├── redaction-review.md
└── summary.md
```

Use component subdirectories when generated artifacts reduce manual transcription risk; the preferred evidence root is already named in Phase 16 context. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

### Pattern 1: Package Identity First

**What:** Run `just package`, copy or cite the exact manifest, run manifest-backed release gate, and use the same manifest path for flash, HTTP, OTA, and recovery evidence. [VERIFIED: local files Justfile, tools/xtask/src/package_manifest.rs, tools/parity/src/release_gate.rs]

**When to use:** Always at the start of Phase 16 execution, and again if any source commit changes before live hardware or network evidence. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Example:**

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

### Pattern 2: Detector-Gated Flash-Monitor

**What:** Use `just detect-ultra205` to select the port, then pass that port and the current manifest to `flash-monitor` so the wrapper records trusted serial evidence. [VERIFIED: local files AGENTS.md, scripts/detect-ultra205.sh, tools/flash/src/main.rs]

**When to use:** Before any Phase 16 live hardware claim and after current package identity is recorded. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Example:**

```bash
just detect-ultra205
just flash-monitor board=205 port=<detected-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35
```

### Pattern 3: Explicit-URL Live HTTP Probes

**What:** Probe live HTTP/static/recovery/API/WebSocket/OTA routes only through an explicit `DEVICE_URL`, record sanitized target data, and block without curl if the URL is missing or invalid. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**When to use:** After serial boot proves the current package identity on board `205`. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, tools/flash/src/main.rs]

**Example:**

```bash
scripts/phase13-http-static-smoke.sh --device-url "$DEVICE_URL" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery
```

Phase 16 must extend or wrap this helper because the current Phase 13 helper probes `/api/system/OTAWWW` but not a plain `/api/system/OTA` route response in the HTTP/static matrix. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]

### Pattern 4: Gated Recovery Regression

**What:** Run failed-update, interrupted-update, and erase recovery only through helpers or repo commands that document allow flags, abort conditions, detector recheck, current manifest/factory image, restore, safe-state markers, and output paths. [VERIFIED: local files scripts/phase13-recovery-regression.sh, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**When to use:** Only after package identity, detector, explicit URL, current factory image, and recovery plan are available. [VERIFIED: local files AGENTS.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Planning note:** The Phase 13 recovery helper re-runs the destructive detector gate immediately before large erase, but failed-update and interrupted-update currently rely on allow flags and URL/image checks without the full D-09 detector recheck. Phase 16 should tighten this before those operations are allowed. [VERIFIED: local file scripts/phase13-recovery-regression.sh]

### Anti-Patterns To Avoid

- **Reusing Phase 13 as current proof:** Phase 13 live hardware evidence is tied to source commit `190849539700b8f9a7909fd2b6ebd84142557968`, while later package checks were not live-flashed. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]
- **Network scanning for the device:** Phase 16 requires explicit `DEVICE_URL`; helpers should block without scanning. [VERIFIED: local files .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md, scripts/phase13-http-static-smoke.sh]
- **Treating invalid rejection as rollback proof:** Existing release docs and evidence state that rejected uploads are not rollback proof. [VERIFIED: local files docs/release/ultra-205.md, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]
- **Promoting rows before redaction review:** The phase context requires redaction review for generated logs, JSON, Markdown, API responses, WebSocket captures, and terminal snippets before commit. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Firmware package and factory image creation | Custom binary merge or ad hoc image copier | `just package` via `scripts/package-firmware.sh` | The package wrapper already creates manifest v2, `esp-miner.bin`, `www.bin`, factory image, otadata, checksums, install notes, and reference cleanliness. [VERIFIED: local files scripts/package-firmware.sh, tools/xtask/src/package_manifest.rs] |
| Flash and serial evidence | Raw `espflash flash` plus manual log notes | `just flash-monitor` / `tools/flash` | The wrapper writes evidence JSON/logs and validates trusted boot markers. [VERIFIED: local file tools/flash/src/main.rs] |
| Device discovery | Network scans or serial inference | Explicit `DEVICE_URL` and `just detect-ultra205` | Phase 16 forbids network inference and requires detector-gated hardware identity. [VERIFIED: local files .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md, AGENTS.md] |
| HTTP/static/OTA probes | One-off curl snippets | Phase 13 helpers or Phase 16 wrappers with tests | Existing helpers sanitize responses, block on missing URL, and have Bazel tests. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh, scripts/BUILD.bazel] |
| Destructive/fault gates | Manual erase/write/failure commands | Phase-owned helper plus typed allow manifest or strengthened tested shell gate | AGENTS and Phase 16 decisions require documented prereqs, aborts, recovery, detector, and safe-state markers. [VERIFIED: local files AGENTS.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |
| Checklist promotion | Manual status edits from intent | `just parity` and cited artifact evidence | Parity validator guards release-sensitive, OTA, OTAWWW, and release-image verified rows. [VERIFIED: local file tools/parity/src/main.rs] |
| Redaction | Human memory after docs are committed | Explicit redaction review artifact before checklist/docs promotion | Phase 13/14/15 evidence ledgers all use redaction review artifacts. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md] |

**Key insight:** The dangerous part of Phase 16 is not writing curl commands; it is proving that each claim belongs to the same current package, same board, same port, same explicit device URL, and redaction-cleared artifact set. [VERIFIED: local files .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

## Common Pitfalls

### Pitfall 1: Stale Current-Commit Evidence

**What goes wrong:** A package or checklist update after live flash gets cited as if the later commit was flashed. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

**Why it happens:** Phase 13 ended with live hardware evidence for one source commit and later package/release checks for another source commit. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

**How to avoid:** Treat source commit, manifest path, flash-monitor JSON, HTTP/OTA logs, and recovery logs as one identity chain; if source changes, repackage and rerun affected evidence. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Warning signs:** Manifest source commit differs from observed serial `firmware_commit=`, flash-monitor output is untrusted, or docs cite old Phase 13 paths for current parity. [VERIFIED: local files tools/flash/src/main.rs, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

### Pitfall 2: Treating `DEVICE_URL` As Optional

**What goes wrong:** Package, serial, or route-registration evidence is promoted into live HTTP/static/recovery/OTA evidence without actual HTTP responses. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

**Why it happens:** Serial logs prove route registration, not live network reachability. [VERIFIED: local files tools/flash/src/main.rs, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

**How to avoid:** Keep live evidence blocked unless explicit `DEVICE_URL` is provided and probe artifacts exist. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]

**Warning signs:** Evidence says `DEVICE_URL status: blocked`, no `curl_status`, or no selected headers/body snippets exist. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]

### Pitfall 3: OTAWWW Overclaim

**What goes wrong:** `www.bin` packaging or `Wrong API input` response is treated as static update parity. [VERIFIED: local files docs/parity/checklist.md, docs/release/ultra-205.md]

**Why it happens:** OTAWWW is a release gap unless whole-`www` partition update and interrupted-update recovery are captured. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**How to avoid:** Keep OTAWWW deferred unless Phase 16 intentionally captures the full hardware-regression path. [VERIFIED: local files docs/parity/checklist.md, tools/parity/src/main.rs]

**Warning signs:** `OTA-002` marked verified without hardware-regression and interrupted-update wording; `REL-03` promoted from package-only evidence. [VERIFIED: local file tools/parity/src/main.rs]

### Pitfall 4: Destructive Actions Without Current Recovery Gate

**What goes wrong:** Erase, failed-update, interrupted-update, rollback, or raw recovery runs with stale factory image, wrong port, missing URL, no recovery path, or no safe-state markers. [VERIFIED: local files AGENTS.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Why it happens:** The existing Phase 13 recovery helper predates Phase 16 D-09 and only applies the full detector recheck immediately before large erase. [VERIFIED: local file scripts/phase13-recovery-regression.sh]

**How to avoid:** Add Phase 16 allow/identity gate or strengthen the recovery helper so every rollback/failed/interrupted/erase operation rechecks detector, board-info, current manifest, current factory image, aborts, recovery steps, and safe-state expectations. [VERIFIED: local files tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs]

**Warning signs:** Allow flag is present but detector output is absent, factory image path is not from current manifest, or post-failure HTTP/static smoke is missing. [VERIFIED: local file scripts/phase13-recovery-regression.sh]

### Pitfall 5: Redaction After Promotion

**What goes wrong:** Checklist, release guide, or requirements get updated from raw logs before secret review. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

**Why it happens:** Live HTTP, WebSocket, OTA, recovery, and terminal logs can contain private network details or credentials. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh]

**How to avoid:** Make `redaction-review.md` a gate before docs/checklist updates; explicitly state absent artifacts are not cited. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md]

**Warning signs:** Raw `DEVICE_URL`, Wi-Fi fields, pool fields, tokens, or private endpoint snippets appear in evidence. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-firmware-ota-smoke.sh]

## Code Examples

### Current Package And Release Gate

```bash
# Source: Justfile and tools/parity/src/release_gate.rs
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

This records the package workflow and validates manifest-backed release docs, but it does not prove live hardware until paired with detector-gated flash-monitor evidence. [VERIFIED: local files Justfile, tools/parity/src/release_gate.rs, docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md]

### Detector-Gated Serial Boot

```bash
# Source: AGENTS.md, scripts/detect-ultra205.sh, tools/flash/src/main.rs
just detect-ultra205
just flash-monitor board=205 port=<port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35
```

The serial output should contain trusted markers for firmware commit, reference commit, reset reason, ESP-IDF version, SPIFFS, route shell, boot validation, and safe state. [VERIFIED: local file tools/flash/src/main.rs]

### Explicit Live HTTP Probe

```bash
# Source: scripts/phase13-http-static-smoke.sh
scripts/phase13-http-static-smoke.sh --device-url "$DEVICE_URL" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery
```

If `DEVICE_URL` is missing or invalid, the helper writes blocked evidence and exits before curl. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]

### Firmware OTA Probe

```bash
# Source: scripts/phase13-firmware-ota-smoke.sh
scripts/phase13-firmware-ota-smoke.sh --device-url "$DEVICE_URL" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port <port> --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota --monitor-seconds 45
```

The helper validates the manifest `firmware_ota_image` path and checksum before upload, rejects invalid-image acceptance, and requires post-OTA monitor markers. [VERIFIED: local file scripts/phase13-firmware-ota-smoke.sh]

### Recovery Regression Shape

```bash
# Source: scripts/phase13-recovery-regression.sh
scripts/phase13-recovery-regression.sh --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port <port> --device-url "$DEVICE_URL" --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/recovery-regression --allow-failed-update --allow-large-erase --allow-interrupted-ota
```

Do not run this command until the active plan documents the recovery path, allow flags, detector gate, current package identity, and redaction requirements for each operation. [VERIFIED: local files AGENTS.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

## State Of The Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Package-only release confidence | Manifest v2 plus release gate plus live wrapper evidence | Existing by Phase 13 [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md] | Release parity planning must separate package, serial, HTTP, OTA, recovery, and redaction evidence. [VERIFIED: local file docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md] |
| Manual flash logs | `tools/flash` evidence JSON and trusted marker classifier | Existing by Phase 13 [VERIFIED: local file tools/flash/src/main.rs] | Phase 16 should rely on wrapper-owned evidence, not hand-written serial conclusions. [VERIFIED: local file tools/flash/src/main.rs] |
| Broad safety/mining claims from one run | Surface-specific allow manifests and conservative ledgers | Established in Phases 14 and 15 [VERIFIED: local files .planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md] | Phase 16 destructive/fault evidence should use a similar allow/identity pattern if it runs. [VERIFIED: local files tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs] |
| Checklist edits as assertions | `just parity` validates evidence-class guardrails | Existing in `tools/parity` [VERIFIED: local file tools/parity/src/main.rs] | Planner must include parity after docs/checklist edits and before final verification. [VERIFIED: local files Justfile, tools/parity/src/main.rs] |

**Deprecated/outdated:**

- Using Phase 13 live hardware evidence as current release proof is outdated for Phase 16 unless the current commit equals `190849539700b8f9a7909fd2b6ebd84142557968`. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
- Treating `www.bin` generation as OTAWWW parity is explicitly invalid for REL-03. [VERIFIED: local files docs/parity/checklist.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
- Treating invalid-image rejection as rollback proof is explicitly invalid in release docs and Phase 16 decisions. [VERIFIED: local files docs/release/ultra-205.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

## Assumptions Log

All claims in this research were verified from local files or local commands, except future runtime availability questions listed in Open Questions. No `[ASSUMED]` claims are used as planning facts. [VERIFIED: local research pass]

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| None | No assumed factual claims recorded. | N/A | N/A |

## Open Questions (RESOLVED)

1. **Will an explicit reachable `DEVICE_URL` be available during execution?**
   - What we know: Phase 13 and Phase 15 both recorded missing `DEVICE_URL` blockers; Phase 16 requires explicit `DEVICE_URL`. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
   - What's unclear: This research did not curl or probe any device URL by user instruction. [VERIFIED: user prompt]
   - **RESOLVED:** Treat `DEVICE_URL` availability as an execution-time prerequisite, not a planning uncertainty. Plans must require an explicit reachable `DEVICE_URL` before live HTTP/static/recovery/OTA probes and must write blocked evidence with `network_scan: disabled` when it is absent or unreachable.
   - Recommendation: Plan blocked/pending branches for all live HTTP/OTA evidence and require an explicit `DEVICE_URL` input before live probes. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]

2. **Will exactly one Ultra 205 serial port pass detector and board-info during execution?**
   - What we know: `just detect-ultra205` implements the required detector/board-info gate. [VERIFIED: local file scripts/detect-ultra205.sh]
   - What's unclear: This research did not run `just detect-ultra205` by user instruction. [VERIFIED: user prompt]
   - **RESOLVED:** Treat detector success as an execution-time hardware gate. Plans must run `just detect-ultra205`, continue only when exactly one Ultra 205 port passes board-info, and otherwise record pending or blocked evidence without running flash, OTA, erase, rollback, failed-update, interrupted-update, or raw recovery commands.
   - Recommendation: Make detector output the first hardware artifact and stop/pending on zero, multiple, mismatched, or board-info-failed ports. [VERIFIED: local file AGENTS.md]

3. **Will destructive/fault operations be authorized with adequate recovery gates?**
   - What we know: Phase 16 decisions require allow flags, current manifest/factory image, detector gate, recovery steps, abort conditions, and safe-state markers before erase/rollback/failed/interrupted operations. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
   - What's unclear: No Phase 16 evidence or allow manifests exist yet. [VERIFIED: local commands `find docs/parity/evidence -maxdepth 2 -path '*phase-16*' -print`, `find .planning/phases/16-current-commit-release-evidence-completion -maxdepth 1 -type f -print`]
   - **RESOLVED:** Destructive and fault-injection authorization is only granted when the Phase 16 plan-owned gates exist and pass during execution: current package manifest, current factory image, detector/board-info transcript, explicit allow flags, abort conditions, recovery steps, safe-state checks, and redaction review. If any gate is missing, plans must write pending evidence and must not run the operation.
   - Recommendation: Add a Wave 0 gate task for a Phase 16 allow/identity manifest or equivalent tested shell preflight before any destructive action. [VERIFIED: local files tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | Human command surface | yes [VERIFIED: local command] | 1.48.0 | None needed |
| Bazel | Package, tests, parity, flash wrappers | yes [VERIFIED: local command] | 9.1.1 | None recommended |
| Cargo | Rust host tests/tools | yes [VERIFIED: local command] | 1.88.0-nightly | None recommended |
| Rustc | Rust host tools and firmware crates | yes [VERIFIED: local command] | 1.88.0-nightly | None recommended |
| `espflash` | Board-info, flash, monitor, erase, image save | yes [VERIFIED: local command] | 4.0.1 | None recommended |
| `curl` | Explicit `DEVICE_URL` probes | yes [VERIFIED: local command] | 8.7.1 | None recommended |
| `python3` | Existing shell helper JSON parsing | yes [VERIFIED: local command] | 3.14.4 | Prefer Rust validator for new complex parsing |
| Node.js | GSD lifecycle and optional WebSocket capture precedent | yes [VERIFIED: local command] | v24.13.0 | Avoid unless WebSocket capture is needed |
| `espup` | ESP toolchain setup | yes [VERIFIED: local command] | 0.15.1 | `just bootstrap-esp` if setup is stale [VERIFIED: local file AGENTS.md] |
| `ldproxy` | ESP-IDF Rust linker path | yes, but version probe panics without linker args [VERIFIED: local command] | unavailable | Do not require `ldproxy --version` in evidence |
| Global `esptool.py` | ESP-IDF merge/utility fallback | no on PATH [VERIFIED: local command] | unavailable | Use managed `.embuild` ESP-IDF tool from `scripts/package-firmware.sh`. [VERIFIED: local files AGENTS.md, scripts/package-firmware.sh] |
| Ultra 205 hardware | Live evidence | not checked [VERIFIED: user prompt] | unavailable | Run `just detect-ultra205` during execution; stop/pending on failure. [VERIFIED: local file AGENTS.md] |
| `DEVICE_URL` | Live HTTP/static/recovery/OTA | not checked [VERIFIED: user prompt] | unavailable | Record blocked evidence when absent. [VERIFIED: local file scripts/phase13-http-static-smoke.sh] |

**Missing dependencies with no fallback:**

- A reachable explicit `DEVICE_URL` is required for live HTTP/static/recovery/OTA evidence; without it, the plan must produce blocked evidence and avoid checklist promotion. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
- A detector-approved Ultra 205 port is required for hardware evidence; without it, hardware and destructive work must stop or remain pending. [VERIFIED: local file AGENTS.md]

**Missing dependencies with fallback:**

- Global `esptool.py` is not on PATH, but the repo uses managed `.embuild` ESP-IDF tools through the package workflow. [VERIFIED: local files AGENTS.md, scripts/package-firmware.sh]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `sh_test` for shell helpers; Cargo/Rust unit tests for host tools; `just test` aggregates `bazel test //...`. [VERIFIED: local files scripts/BUILD.bazel, Cargo.toml, Justfile] |
| Config file | `MODULE.bazel`, `BUILD.bazel`, per-crate `BUILD.bazel`, and `Cargo.toml`. [VERIFIED: local command `find . -maxdepth 3 ...`] |
| Quick run command | `bazel test //scripts:phase13_http_static_smoke_test //scripts:phase13_firmware_ota_smoke_test //scripts:phase13_recovery_regression_test` [VERIFIED: local file scripts/BUILD.bazel] |
| Full suite command | `just test` and `cargo test --all-features` for Rust host tools after code changes. [VERIFIED: local files Justfile, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| FND-06 | Current package flash-monitor records trusted boot identity and safe-state markers. | unit plus hardware smoke | `cargo test -p bitaxe-flash --all-features flash_monitor` if crate filter is available; otherwise `cargo test -p bitaxe-flash --all-features` plus live `just flash-monitor ...` during execution. [VERIFIED: local file tools/flash/src/main.rs] | yes, `tools/flash/src/main.rs` tests exist [VERIFIED: local file tools/flash/src/main.rs] |
| API-09 | Live HTTP/static/recovery probes block on missing URL and sanitize output. | shell unit plus live smoke | `bazel test //scripts:phase13_http_static_smoke_test` [VERIFIED: local file scripts/BUILD.bazel] | yes |
| REL-01 | SPIFFS/static/recovery package and post-restore reachability. | shell unit plus live smoke | `bazel test //scripts:phase13_http_static_smoke_test //scripts:phase13_recovery_regression_test` [VERIFIED: local file scripts/BUILD.bazel] | yes |
| REL-02 | Firmware OTA valid upload, invalid rejection, post-reboot identity and boot-validation marker. | shell unit plus live OTA | `bazel test //scripts:phase13_firmware_ota_smoke_test` [VERIFIED: local file scripts/BUILD.bazel] | yes |
| REL-03 | OTAWWW remains explicit gap unless whole-`www` regression evidence exists. | parity validation plus live gap probe | `just parity`; add/extend HTTP helper test if `/api/system/OTAWWW` evidence shape changes. [VERIFIED: local files Justfile, scripts/phase13-http-static-smoke.sh, tools/parity/src/main.rs] | yes for current gap behavior |
| REL-04 | Manifest v2 release package and release gate pass for current commit. | Rust unit plus workflow | `cargo test -p xtask --all-features package_manifest`; `cargo test -p bitaxe-parity --all-features release_gate`; `just package`; `bazel run //tools/parity:report -- release-gate --manifest ...` [VERIFIED: local files tools/xtask/src/package_manifest.rs, tools/parity/src/release_gate.rs] | yes |
| REL-07 | Release guide and evidence docs cite real artifacts and preserve safety boundaries. | docs/parity check | `just parity`; `just verify-reference`; lifecycle validation. [VERIFIED: local files Justfile, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] | yes for existing docs |
| REL-08 | Rollback, erase, failed-update, interrupted-update are gated and evidenced before claims. | shell unit plus hardware regression | `bazel test //scripts:phase13_recovery_regression_test`; add Phase 16 gate tests if helper is tightened. [VERIFIED: local files scripts/BUILD.bazel, scripts/phase13-recovery-regression.sh] | yes for Phase 13 helper, gap for Phase 16 D-09 all-operation gate |
| EVD-05 | Evidence layers and validation gates exist. | aggregate | `just test`; `just parity`; `just verify-reference`; lifecycle validation. [VERIFIED: local files Justfile, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md] | yes |

### Sampling Rate

- **Per task commit:** Run targeted Bazel shell tests and Cargo filters for touched tools/helpers. [VERIFIED: local files scripts/BUILD.bazel, tools/parity/src/release_gate.rs]
- **Per wave merge:** Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, and `just test` when Rust or shell behavior changes. [VERIFIED: local files AGENTS.md, .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md]
- **Phase gate:** Run `just package`, manifest-backed release gate, applicable live hardware/network/recovery commands, redaction review, `just parity`, `just verify-reference`, reference diff, and lifecycle validation for `16-2026-07-01T12-36-46`. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

### Wave 0 Gaps

- [ ] Add or specify a Phase 16 current-commit identity gate that compares current git HEAD, package manifest `source_commit`, flash-monitor observed firmware commit, and evidence artifact paths before docs/checklist promotion. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, tools/flash/src/main.rs]
- [ ] Add `/api/system/OTA` route probe coverage to the HTTP/static evidence path or create a Phase 16 HTTP wrapper with a test, because the current Phase 13 HTTP helper covers OTAWWW but not plain OTA route probing. [VERIFIED: local file scripts/phase13-http-static-smoke.sh]
- [ ] Tighten failed-update and interrupted-update gating so D-09 detector/current-manifest/factory-image requirements apply before every rollback/failed/interrupted/erase action, not only large erase. [VERIFIED: local file scripts/phase13-recovery-regression.sh]
- [ ] Create a Phase 16 redaction review template that explicitly covers API bodies, WebSocket frames, recovery logs, destructive logs, terminal snippets, private `DEVICE_URL`, pool/Wi-Fi credentials, tokens, NVS secrets, and absent artifacts. [VERIFIED: local file .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no new auth surface [VERIFIED: local phase scope] | Do not add authentication changes in this evidence phase; preserve existing access gate behavior. [VERIFIED: local file firmware/bitaxe/src/http_api.rs] |
| V3 Session Management | no [VERIFIED: local phase scope] | No session state should be introduced by evidence helpers. [VERIFIED: local file .planning/ROADMAP.md] |
| V4 Access Control | yes [VERIFIED: local files firmware/bitaxe/src/http_api.rs, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] | Use explicit `DEVICE_URL`, access gate behavior, detector-approved board/port, and allow manifests for risky operations. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, tools/parity/src/safety_allow.rs] |
| V5 Input Validation | yes [VERIFIED: local files scripts/phase13-*.sh, tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs] | Validate manifest JSON, route expectations, URL scheme, artifact names/checksums, command shape, board/port, and evidence class before action. [VERIFIED: local files scripts/phase13-firmware-ota-smoke.sh, tools/parity/src/safety_allow.rs] |
| V6 Cryptography | yes for checksums only [VERIFIED: local files tools/xtask/src/package_manifest.rs, scripts/phase13-firmware-ota-smoke.sh] | Use existing SHA-256 manifest/checksum functions; do not hand-roll crypto. [VERIFIED: local files tools/xtask/src/package_manifest.rs, scripts/phase13-firmware-ota-smoke.sh] |
| V7 Error Handling and Logging | yes [VERIFIED: local files scripts/phase13-http-static-smoke.sh, scripts/phase13-recovery-regression.sh] | Log blocked/pending states explicitly, sanitize bodies, and fail fast on mismatches. [VERIFIED: local files scripts/phase13-*.sh] |
| V12 File and Resources | yes [VERIFIED: local files scripts/package-firmware.sh, tools/xtask/src/package_manifest.rs] | Use manifest-named artifacts and current factory image; avoid arbitrary raw writes. [VERIFIED: local files scripts/package-firmware.sh, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |

### Known Threat Patterns For This Phase

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Private `DEVICE_URL`, Wi-Fi, pool, token, or NVS secret leak in evidence | Information Disclosure | Redacted snippets plus explicit redaction review before docs/checklist promotion. [VERIFIED: local files scripts/phase13-http-static-smoke.sh, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |
| Wrong-device destructive command | Tampering / Denial of Service | `just detect-ultra205`, exactly one port, board-info success, board `205`, current manifest/factory image, abort on mismatch. [VERIFIED: local files AGENTS.md, scripts/detect-ultra205.sh] |
| Stale artifact used for release proof | Tampering / Repudiation | Package-to-hardware identity chain and current-commit gate. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md] |
| Network scan or unintended target probing | Information Disclosure / Tampering | Explicit `DEVICE_URL` only; helpers block without URL and log scanning disabled. [VERIFIED: local file scripts/phase13-http-static-smoke.sh] |
| Invalid OTA accepted or partial OTA leaves device unusable | Tampering / Denial of Service | Invalid rejection test, valid OTA post-reboot monitor, interrupted-update recovery smoke, factory restore path. [VERIFIED: local files scripts/phase13-firmware-ota-smoke.sh, scripts/phase13-recovery-regression.sh] |
| Checklist overclaim | Repudiation | `just parity` release-sensitive row validation and evidence-tier guardrails. [VERIFIED: local file tools/parity/src/main.rs] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md` - locked decisions, discretion, deferred scope, canonical references, preferred command order. [VERIFIED: local file]
- `.planning/ROADMAP.md` - Phase 16 goal, requirements, success criteria, and verification expectations. [VERIFIED: local file]
- `.planning/REQUIREMENTS.md` - FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-07, REL-08, EVD-05 descriptions and traceability. [VERIFIED: local file]
- `.planning/STATE.md` - milestone state, Phase 13/15 blockers, historical evidence status. [VERIFIED: local file]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/` - repo and Bright Builds workflow, hardware, testing, verification, and code-shape constraints. [VERIFIED: local files]
- `Justfile` - command surface. [VERIFIED: local file]
- `scripts/detect-ultra205.sh` - Ultra 205 detector and board-info gate. [VERIFIED: local file]
- `scripts/package-firmware.sh` - canonical package wrapper. [VERIFIED: local file]
- `scripts/phase13-http-static-smoke.sh`, `scripts/phase13-firmware-ota-smoke.sh`, `scripts/phase13-recovery-regression.sh`, `scripts/phase13-monitor-capture.sh` - reusable release evidence helpers and tests. [VERIFIED: local files]
- `tools/flash/src/main.rs` - flash-monitor evidence contract and trusted-output classifier. [VERIFIED: local file]
- `tools/xtask/src/package_manifest.rs` - package manifest v2 and artifacts. [VERIFIED: local file]
- `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs`, `tools/parity/src/safety_allow.rs`, `tools/parity/src/mining_allow.rs` - parity, release gate, and allow patterns. [VERIFIED: local files]
- `crates/bitaxe-api/src/route_shell.rs`, `static_plan.rs`, `update_plan.rs` and `firmware/bitaxe/src/http_api.rs`, `ota_update.rs`, `boot_validation.rs`, `filesystem.rs`, `static_files.rs` - route/static/update/firmware surfaces. [VERIFIED: local files]
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`, `.planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md`, `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md` - prior evidence and blockers. [VERIFIED: local files]

### Secondary (MEDIUM confidence)

- Local environment commands for tool availability and versions. These are current for this machine but may change before execution. [VERIFIED: local commands]

### Tertiary (LOW confidence)

- None. No web-only or unverified external sources were used.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - all tools and command surfaces were verified from local files or local version commands. [VERIFIED: local commands]
- Architecture: HIGH - Phase 13/14/15 helpers and validators provide directly reusable local patterns. [VERIFIED: local files scripts/phase13-*.sh, tools/parity/src/safety_allow.rs, tools/parity/src/mining_allow.rs]
- Pitfalls: HIGH - stale commit, missing `DEVICE_URL`, and pending destructive evidence are documented in Phase 13 and Phase 16 context. [VERIFIED: local files docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md, .planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md]
- Hardware/network availability: MEDIUM - intentionally not probed because the user prohibited hardware, flash, erase, curl, and `DEVICE_URL` mutation/probing during research. [VERIFIED: user prompt]

**Research date:** 2026-07-01  
**Valid until:** 2026-07-08 for hardware/network/tool availability; 2026-07-31 for local architecture patterns unless repo code changes. [VERIFIED: local current date]
