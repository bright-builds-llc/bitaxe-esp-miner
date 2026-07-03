# Phase 18: Firmware OTA And Rollback Evidence - Research

**Researched:** 2026-07-03 [VERIFIED: environment current_date]
**Domain:** ESP-IDF Rust firmware OTA evidence, rollback boot validation, Ultra 205 release evidence [VERIFIED: .planning/phases/18-firmware-ota-and-rollback-evidence/18-CONTEXT.md; .planning/ROADMAP.md]
**Confidence:** HIGH for repo-local architecture and evidence constraints; MEDIUM-HIGH for live hardware planning because the research shell did not run `just detect-ultra205` or a live OTA upload. [VERIFIED: codebase grep; command availability audit]

<user_constraints>
## User Constraints (from CONTEXT.md)

Source for every bullet in this section: `.planning/phases/18-firmware-ota-and-rollback-evidence/18-CONTEXT.md`. [VERIFIED: file read]

### Locked Decisions

#### Current Package And Device Identity

- **D-01:** Treat the source commit present when Phase 18 starts as the only eligible package identity. Run `just package` and manifest-backed release-gate validation before any OTA upload, then record source commit, reference commit, package manifest path, firmware OTA image path, checksum, board, selected port, and `DEVICE_URL` provenance in Phase 18 evidence.
- **D-02:** Do not reuse Phase 17 OTA route-presence evidence as valid OTA proof. Phase 17 proved route reachability only; Phase 18 must upload the firmware image or record an exact blocker.
- **D-03:** Start hardware work with `just detect-ultra205`. Continue only when exactly one likely ESP32-S3 serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds for board `205`.
- **D-04:** Live HTTP/OTA work requires an explicit origin-only `DEVICE_URL` for the just-flashed device, either supplied directly or loaded from a redaction-reviewed target lock or flash evidence artifact. Do not scan the network, infer targets from ARP/mDNS/router state, or guess from serial logs.

#### Valid And Invalid Firmware OTA Evidence

- **D-05:** Reuse the existing firmware OTA helper pattern where practical, but create Phase 18 evidence under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/` with phase-specific summaries, logs, sanitized HTTP artifacts, and redaction review.
- **D-06:** Invalid firmware upload evidence should use a fixed local invalid image fixture and record HTTP status, sanitized body marker, expected rejection text, and conclusion. It is failed-update or invalid-rejection proof only; it is not rollback proof.
- **D-07:** Valid firmware OTA evidence must upload the manifest-listed `esp-miner.bin` to `/api/system/OTA`, record response status/body marker, reboot scheduling behavior, post-reboot serial or HTTP identity, selected partition or explicit unavailable state, and safe post-OTA operation.
- **D-08:** If a valid OTA upload would upload the same current image, that is acceptable for evidence only when the manifest checksum, public response, reboot identity, and boot-validation marker still prove the OTA path exercised the next app partition behavior. If the firmware refuses same-version/self-image updates, record the exact public/internal status and keep valid OTA below verified.

#### Rollback And Boot-Validation Gate

- **D-09:** Prefer non-destructive boot-validation evidence from the firmware adapter and serial logs: `ota_boot_validation=marked_valid`, `ota_boot_validation=not_pending state=...`, selected partition markers, source commit identity, and safe state after reboot.
- **D-10:** Do not run raw rollback invalidation, erase, interrupted update, or forced boot-failure commands unless the active plan documents exact commands, allow flags, stop conditions, recovery image, restore command, expected safe-state markers, and redaction requirements.
- **D-11:** A rejected invalid upload is not rollback or boot-validation proof. Rollback proof requires captured bootloader/ESP-IDF boot-validation state before and after a valid OTA or an explicitly gated rollback/fault case.
- **D-12:** If rollback cannot be safely exercised, record boot-validation evidence when available and keep destructive rollback/fault-injection behavior as a non-claim with owner, blocker, and follow-up path.

#### Checklist, Release Docs, Redaction, And Verification

- **D-13:** Promote checklist rows only to the exact evidence tier supported by Phase 18 artifacts. `OTA-001`, `REL-001`, `REL-002`, `REL-003`, `REL-07`, `REL-08`, and `EVD-05` notes must distinguish valid upload, invalid rejection, reboot identity, selected partition, boot validation, rollback, and non-claims.
- **D-14:** Update release docs, requirements traceability, and phase evidence only after evidence artifacts exist. Documentation should cite commands and artifacts, not implementation existence or goals as proof.
- **D-15:** Redaction review is mandatory before commit. It must cover `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS secret values, raw terminal secrets, request/response bodies, serial logs, and recovery logs.
- **D-16:** Final verification must include repo-native checks for changed paths, helper tests for any modified helper scripts, `just package`, manifest-backed release-gate validation, `just parity`, `just verify-reference`, lifecycle validation, and every hardware/network command actually used. No final commit/push should happen unless `18-VERIFICATION.md` has `status: passed` and lifecycle validation passes for `18-2026-07-03T14-06-29`.

### the agent's Discretion

The agent may choose exact helper names, evidence JSON field names, timeout values, whether Phase 18 wraps or adapts Phase 13 OTA helpers, and the final plan split. Those choices must preserve explicit target input, current package identity, repo-owned ESP/esp-rs tooling, functional core plus imperative shell, read-only reference files, redaction before promotion, no standalone body `---` separators in parsed Markdown, and conservative evidence claims.

### Deferred Ideas (OUT OF SCOPE)

- Recovery fault-injection, failed-update recovery beyond invalid rejection, large erase, interrupted update, and OTAWWW whole-`www` update evidence belong to Phase 19 unless a Phase 18 plan adds explicit recovery gates and the action is necessary for firmware OTA boot-validation.
- Active voltage, fan, thermal, and power-control hardware regression belongs to Phase 20.
- Live mining, share handling, watchdog responsiveness under mining load, and soak behavior belongs to Phase 21.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, and an Angular AxeOS replacement remain outside Phase 18.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| REL-02 | Firmware OTA route behavior accepts, rejects, applies, logs, and recovers from updates with upstream-compatible observable behavior. [VERIFIED: .planning/REQUIREMENTS.md] | Use the existing Rust `/api/system/OTA` handler and ESP-IDF OTA adapter; prove valid upload, invalid rejection, public response, reboot identity, and boot-validation markers through Phase 18 evidence. [VERIFIED: firmware/bitaxe/src/http_api.rs; firmware/bitaxe/src/ota_update.rs; scripts/phase13-firmware-ota-smoke.sh] |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update cases have verification evidence before release parity is claimed. [VERIFIED: .planning/REQUIREMENTS.md] | Phase 18 should capture non-destructive boot-validation after valid OTA and record destructive rollback/fault-injection as gated non-claims unless recovery commands and allow flags are present. [VERIFIED: 18-CONTEXT.md; docs/release/ultra-205.md; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] |
| REL-07 | Build, flash, monitor, OTA, and recovery documentation is sufficient for a developer with a connected Ultra 205 to operate safely. [VERIFIED: .planning/REQUIREMENTS.md] | Update release docs only after Phase 18 artifacts exist, preserving target, recovery, redaction, and non-claim boundaries. [VERIFIED: docs/release/ultra-205.md; 18-CONTEXT.md] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Add or reuse helper tests, run package/release-gate/parity/reference checks, and record hardware/network commands actually used. [VERIFIED: scripts/BUILD.bazel; Justfile; 18-CONTEXT.md] |
</phase_requirements>

## Summary

Phase 18 should be planned as an evidence orchestration phase, not as a new OTA implementation phase. [VERIFIED: 18-CONTEXT.md; firmware/bitaxe/src/http_api.rs; firmware/bitaxe/src/ota_update.rs] The firmware already registers `/api/system/OTA`, streams uploads through ESP-IDF OTA APIs, sends the upstream-compatible success and validation-error bodies, schedules restart, and logs retained boot-validation markers. [VERIFIED: firmware/bitaxe/src/http_api.rs; firmware/bitaxe/src/ota_update.rs; firmware/bitaxe/src/boot_validation.rs; reference/esp-miner/main/http_server/http_server.c]

The standard plan should create Phase 18-specific evidence under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/`, reuse or thinly wrap `scripts/phase13-firmware-ota-smoke.sh`, require same-commit package/release-gate identity, require `just detect-ultra205`, require an explicit origin-only target, upload the manifest-listed `esp-miner.bin`, and capture post-OTA serial markers. [VERIFIED: 18-CONTEXT.md; scripts/phase13-firmware-ota-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

**Primary recommendation:** Add a small Phase 18 wrapper/ledger around the existing OTA smoke helper, not a second OTA implementation; make the wrapper enforce Phase 18 target provenance, same-commit package identity, artifact checksums, redaction review, and exact checklist promotion boundaries. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase13-firmware-ota-smoke-test.sh; 18-CONTEXT.md]

## Project Constraints (from AGENTS.md)

- Load and honor `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` when present, and relevant standards pages before planning or implementation. [VERIFIED: AGENTS.md; AGENTS.bright-builds.md; standards/index.md]
- Keep `reference/esp-miner` read-only; use it as behavioral evidence and do not modify it. [VERIFIED: AGENTS.md; docs/adr/0005-read-only-reference-implementation.md]
- Use ESP-IDF and esp-rs tooling first for OTA, flashing, monitoring, image generation, partitions, SPIFFS, NVS, FreeRTOS, and logging. [VERIFIED: AGENTS.md]
- Treat `.embuild/` as generated, local, gitignored ESP-IDF/esp-rs state. [VERIFIED: AGENTS.md]
- Start autonomous Ultra 205 hardware use with `just detect-ultra205`; continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md; scripts/detect-ultra205.sh]
- Do not read, print, summarize, or commit `wifi-credentials.json`; agents may pass the file path to repo-owned `just flash` or `just flash-monitor` commands when the detector gate succeeds. [VERIFIED: AGENTS.md]
- Do not run ad hoc erase, rollback, interrupted update, voltage/fan/mining stress, or raw write commands outside documented phase-gated procedures. [VERIFIED: AGENTS.md]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Use functional core plus imperative shell: pure decisions in crates, ESP-IDF/HTTP/OTA/serial effects in thin firmware or helper adapters. [VERIFIED: AGENTS.md; standards/core/architecture.md; standards/languages/rust.md]
- Run repo-native verification before committing changed paths; for Rust code commits, run format, clippy, build, and tests in the required order. [VERIFIED: AGENTS.md; standards/core/verification.md; standards/languages/rust.md]
- Unit tests should prove one concern and use Arrange/Act/Assert when structure is not trivial. [VERIFIED: AGENTS.md; standards/core/testing.md]
- In GSD artifacts and Markdown parsed with YAML frontmatter, do not use standalone body `---` separators. [VERIFIED: AGENTS.md]
- No project-local skill directories were found under `.claude/skills/` or `.agents/skills/`. [VERIFIED: `find .claude/skills .agents/skills -maxdepth 2 -type f -name SKILL.md`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| ESP-IDF | `v5.5.4` pinned through `esp-idf-sys` metadata | OTA app slots, OTA data partition, rollback boot validation, HTTP server body streaming, reboot, partition identity. [VERIFIED: firmware/bitaxe/Cargo.toml; .cargo/config.toml; firmware/bitaxe/sdkconfig.defaults] | The project pins ESP-IDF through `esp_idf_version = "tag:v5.5.4"` and enables `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y`. [VERIFIED: firmware/bitaxe/Cargo.toml; firmware/bitaxe/sdkconfig.defaults] |
| `esp-idf-svc` / `esp-idf-sys` | `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2` | Rust ESP-IDF services plus raw OTA FFI calls. [VERIFIED: Cargo.lock; `cargo search esp-idf-svc`; `cargo search esp-idf-sys`] | Existing firmware imports `esp_idf_svc::sys` for OTA, boot validation, restart, and partition identity. [VERIFIED: firmware/bitaxe/src/ota_update.rs; firmware/bitaxe/src/boot_validation.rs; firmware/bitaxe/src/http_api.rs] |
| `scripts/phase13-firmware-ota-smoke.sh` | repo-owned helper | Invalid fixture upload, valid `esp-miner.bin` upload, checksum verification, sanitized HTTP artifacts, and post-OTA serial marker validation. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh] | It already implements the Phase 18 core evidence pattern and has fixture tests for missing URL, valid/invalid flow, missing boot marker, and wrong invalid response markers. [VERIFIED: scripts/phase13-firmware-ota-smoke-test.sh] |
| Phase 17 target-lock pattern | repo-owned evidence pattern | Explicit target provenance with `network_scan: disabled`, sanitized URL, board, port, source commit, reference commit, and flash evidence link. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json] | Phase 18 must reuse explicit target input and must not infer targets from network scanning. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh] |
| Package manifest v2 / release gate | repo-owned `tools/xtask` and `tools/parity` | Source/reference commits, artifact paths, offsets, and SHA-256 checksums for `esp-miner.bin` and related release artifacts. [VERIFIED: tools/xtask/src/package_manifest.rs; tools/parity/src/release_gate.rs] | Phase 18 must prove the uploaded OTA image is the manifest-listed `firmware_ota_image`. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json] |
| `espflash` | local `4.0.1`; crates.io latest reported `4.4.0` | USB board-info, flash, monitor, and package-adjacent Espressif device workflows. [VERIFIED: `espflash --version`; `cargo search espflash`] | Repo guidance and existing commands use `espflash`; the detector gate requires `espflash board-info --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: AGENTS.md; scripts/detect-ultra205.sh] |
| `curl` | local `8.7.1` | Live HTTP upload to `/api/system/OTA`. [VERIFIED: `curl --version`; scripts/phase13-firmware-ota-smoke.sh] | Existing live evidence helpers use bounded `curl` requests and redacted artifacts rather than manual terminal transcripts. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase17-live-http-api-smoke.sh] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Typed manifest and JSON evidence parsing. [VERIFIED: Cargo.lock; tools/xtask/src/package_manifest.rs; tools/parity/src/release_gate.rs] | Use in Rust tooling if Phase 18 adds machine-readable evidence or release-gate checks. [VERIFIED: standards/core/architecture.md; tools/parity/src/release_gate.rs] |
| `clap` | `4.6.1` | CLI parsing for host tools. [VERIFIED: Cargo.lock; `cargo search clap`] | Use only if Phase 18 extends Rust host tooling; shell is sufficient for a thin evidence wrapper. [VERIFIED: existing helper scripts; tools/parity/src/main.rs] |
| `anyhow` / `thiserror` | `anyhow 1.0.103`, `thiserror 2.0.18` workspace request with `thiserror 1.0.69` also resolved transitively | Error handling in host tools and reusable crates. [VERIFIED: Cargo.lock; Cargo.toml; `cargo search anyhow`; `cargo search thiserror`] | Use `anyhow` for CLI/tool errors and `thiserror` for reusable library errors when code changes are needed. [VERIFIED: AGENTS.md] |
| `shasum` / `sha256sum` | local tools present | SHA-256 verification in shell helpers. [VERIFIED: `command -v shasum sha256sum`; scripts/phase13-firmware-ota-smoke.sh] | Use shell checksum fallback only in evidence helpers; Rust manifest logic should keep structured checksum validation. [VERIFIED: tools/xtask/src/package_manifest.rs; scripts/phase13-firmware-ota-smoke.sh] |
| Bazel `sh_test` / Rust `rust_test` | Bazel `9.1.1` local | Helper and tool regression tests. [VERIFIED: `bazel --version`; scripts/BUILD.bazel; tools/parity/BUILD.bazel] | Add tests for any new Phase 18 wrapper or parity evidence guard. [VERIFIED: 18-CONTEXT.md; standards/core/testing.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Phase 18 wrapper around Phase 13 OTA helper | Manual `curl` commands and pasted summaries | Manual transcripts risk stale package identity, unredacted bodies, and unsupported checklist promotion; existing helpers already have redaction and marker tests. [VERIFIED: 13-REVIEW-FIX.md; scripts/phase13-firmware-ota-smoke-test.sh] |
| ESP-IDF OTA APIs | Custom image validation or direct flash writes | ESP-IDF already validates OTA app images, manages OTA data, and selects next partitions; custom validation would diverge from project stack and reference behavior. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html; VERIFIED: firmware/bitaxe/src/ota_update.rs] |
| Explicit `DEVICE_URL`/target lock | Network scan, ARP, mDNS, or serial-log guessing | Phase 18 forbids target inference and requires origin-only explicit target provenance. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh] |
| Non-destructive boot-validation evidence | Raw rollback invalidation or forced boot failure | Destructive rollback/fault actions need exact commands, allow flags, recovery image, restore command, stop conditions, and redaction rules. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md] |

**Installation:** No new dependency installation is recommended for Phase 18 planning. [VERIFIED: Cargo.lock; command availability audit]

**Version verification:** Relevant package/tool versions were verified with `Cargo.lock`, `cargo search`, and local `--version` commands. [VERIFIED: `python3 tomllib Cargo.lock query`; `cargo search`; local command availability audit]

## Architecture Patterns

### Recommended Project Structure

```text
docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/
├── package-release-gate.md                  # Phase 18 package and release-gate identity ledger
├── package-release-gate/
│   ├── bitaxe-ultra205-package.json          # copied manifest; no generated binaries
│   ├── package-command.log
│   └── release-gate.log
├── serial-boot/
│   ├── detect-ultra205.log
│   ├── flash-command-evidence.json
│   └── flash-monitor.log
├── target-lock.json                          # redacted target provenance
├── firmware-ota/
│   ├── firmware-ota-smoke.log
│   ├── invalid-firmware.bin
│   ├── invalid-firmware-ota.headers.txt
│   ├── invalid-firmware-ota.body.txt
│   ├── invalid-firmware-ota.curl-error.txt
│   ├── valid-firmware-ota.headers.txt
│   ├── valid-firmware-ota.body.txt
│   ├── valid-firmware-ota.curl-error.txt
│   └── post-ota-monitor.log
├── firmware-ota.md                           # phase ledger and exact claim boundaries
├── rollback-boot-validation.md               # boot-validation and rollback/non-claim ledger
├── redaction-review.md
└── summary.md
```

This structure mirrors Phase 17 package/target/redaction evidence and the Phase 13 OTA helper artifact layout. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md; scripts/phase13-firmware-ota-smoke.sh]

### Pattern 1: Same-Commit Identity Chain

**What:** Run `just package`, copy the package manifest into Phase 18 evidence, run manifest-backed release gate, flash or confirm a just-flashed device from that manifest, then run OTA against the exact manifest-listed `esp-miner.bin`. [VERIFIED: 18-CONTEXT.md; tools/parity/src/release_gate.rs; scripts/phase13-firmware-ota-smoke.sh]

**When to use:** Use this before any valid OTA, invalid OTA, rollback, or boot-validation claim. [VERIFIED: 18-CONTEXT.md]

**Example:**

```bash
just package
bazel run //tools/parity:report -- release-gate \
  --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
```

Source: Phase 17 package/release-gate command pattern and `tools/parity` release-gate command. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md; tools/parity/src/main.rs]

### Pattern 2: Explicit Target Lock, No Discovery

**What:** Accept a direct `DEVICE_URL`, explicit `--device-url`, or trusted local flash evidence source, then write only redacted target provenance into committed evidence. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]

**When to use:** Use before any live HTTP upload; block if the URL has userinfo, query, fragment, or path. [VERIFIED: scripts/phase17-live-http-api-smoke.sh; 18-CONTEXT.md]

**Example:**

```bash
scripts/phase17-live-http-api-smoke.sh \
  --use-flash-log-device-url \
  --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --flash-evidence-json target/phase18-dev-raw/flash-command-evidence.json \
  --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-preflight \
  --target-lock-out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json
```

Source: Phase 17 helper and target-lock pattern; planner may choose a Phase 18-specific wrapper instead of reusing the Phase 17 helper name. [VERIFIED: scripts/phase17-live-http-api-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]

### Pattern 3: OTA Helper Wrapper Rather Than Reimplementation

**What:** Add `scripts/phase18-firmware-ota-evidence.sh` as a thin wrapper that validates Phase 18 paths, target provenance, and same-commit manifest identity, then calls `scripts/phase13-firmware-ota-smoke.sh --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota`. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; 18-CONTEXT.md]

**When to use:** Use if the planner wants Phase 18-specific artifact names and tests without changing the proven upload/marker core. [VERIFIED: scripts/phase13-firmware-ota-smoke-test.sh; 13-REVIEW-FIX.md]

**Example:**

```bash
scripts/phase13-firmware-ota-smoke.sh \
  --device-url "$DEVICE_URL" \
  --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port "$PORT" \
  --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota \
  --monitor-seconds 45
```

Source: existing Phase 13 helper CLI. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh]

### Pattern 4: Boot Validation As the Primary Non-Destructive Rollback Evidence

**What:** Treat post-OTA retained log markers as proof of ESP-IDF boot-validation behavior when destructive rollback is not safely exercised. [VERIFIED: 18-CONTEXT.md; firmware/bitaxe/src/boot_validation.rs]

**When to use:** Use after valid OTA reboot when serial logs contain `firmware_commit=`, `reference_commit=`, and `ota_boot_validation=` markers. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; firmware/bitaxe/src/boot_validation.rs]

**Example markers:**

```text
firmware_commit=<source commit or prefix>
reference_commit=<reference commit>
ota_boot_validation=marked_valid
```

Source: firmware retained boot-validation adapter and helper marker validation. [VERIFIED: firmware/bitaxe/src/boot_validation.rs; scripts/phase13-firmware-ota-smoke.sh]

### Anti-Patterns to Avoid

- **Route presence as OTA proof:** Phase 17 route evidence explicitly remains route-presence/validation-path only and does not prove valid upload or rollback. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md]
- **Invalid rejection as rollback proof:** A rejected invalid upload can support invalid-rejection or failed-update evidence, but rollback proof requires bootloader or boot-validation state after a valid OTA or gated fault case. [VERIFIED: 18-CONTEXT.md; docs/release/ultra-205.md]
- **Ad hoc destructive commands:** Raw rollback invalidation, erase, interrupted upload, and forced boot failures are out of bounds unless the active plan names exact commands and recovery gates. [VERIFIED: 18-CONTEXT.md; AGENTS.md]
- **Committing raw target or credentials:** Raw `DEVICE_URL`, Wi-Fi credentials, pool credentials, tokens, private endpoints, and NVS secret values must be absent or redacted in committed evidence. [VERIFIED: AGENTS.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| OTA image writing and validation | Custom app-image parser, manual partition writes, or custom boot slot toggling | ESP-IDF OTA calls through `esp-idf-sys`: `esp_ota_get_next_update_partition`, `esp_ota_begin`, `esp_ota_write`, `esp_ota_end`, `esp_ota_set_boot_partition`. [VERIFIED: firmware/bitaxe/src/ota_update.rs; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] | ESP-IDF manages OTA slots, app validation, OTA data, and boot partition semantics. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] |
| Boot validation and rollback state | Custom rollback state files or NVS flags | ESP-IDF rollback APIs: `esp_ota_get_state_partition`, `esp_ota_mark_app_valid_cancel_rollback`, `esp_ota_mark_app_invalid_rollback_and_reboot`. [VERIFIED: firmware/bitaxe/src/boot_validation.rs; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] | ESP-IDF stores OTA states in `otadata` and the bootloader interprets those states. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] |
| Target discovery | ARP/mDNS/router scans or serial-log guessing | Explicit `DEVICE_URL`, target-lock, or trusted local flash evidence source with committed redaction. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh] | Phase 18 forbids network inference and requires origin-only target provenance. [VERIFIED: 18-CONTEXT.md] |
| OTA evidence capture | Manual terminal transcript | Repo-owned helper with sanitized headers/body/error files and marker validation. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase13-firmware-ota-smoke-test.sh] | Prior review found helper overclaim/redaction pitfalls and fixed them with tests. [VERIFIED: .planning/phases/13-final-ultra-205-release-evidence/13-REVIEW-FIX.md] |
| Release readiness validation | Prose-only checklist updates | `just parity` and `bazel run //tools/parity:report -- release-gate --manifest ...`. [VERIFIED: Justfile; tools/parity/src/main.rs; tools/parity/src/release_gate.rs] | Parity tooling rejects verified OTA rows without valid OTA, invalid image rejection, and boot-validation terms. [VERIFIED: tools/parity/src/main.rs] |
| Redaction | Truncate-first snippets or manual memory | Existing redaction stream patterns and a required `redaction-review.md`. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md] | Phase 13 review fixed redaction-before-truncation and private-endpoint leaks in live helpers. [VERIFIED: 13-REVIEW-FIX.md] |

**Key insight:** The risky domain logic belongs to ESP-IDF and existing repo evidence helpers; Phase 18 should plan guardrails, current identity, redaction, and exact claim promotion rather than implement new OTA primitives. [VERIFIED: AGENTS.md; firmware/bitaxe/src/ota_update.rs; scripts/phase13-firmware-ota-smoke.sh]

## Common Pitfalls

### Pitfall 1: Same-Image OTA May Not Prove the Intended Behavior

**What goes wrong:** Uploading the same `esp-miner.bin` returns a response but does not clearly show next-slot boot behavior, or firmware refuses self-image updates. [VERIFIED: 18-CONTEXT.md]

**Why it happens:** Phase 18 is constrained to the current package identity, and D-08 allows same-image evidence only when checksum, response, reboot identity, and boot-validation markers still prove the OTA path exercised next app partition behavior. [VERIFIED: 18-CONTEXT.md]

**How to avoid:** Require the helper ledger to record manifest checksum, response, reboot marker, selected partition if available, and `ota_boot_validation=`; if self-image update is refused, record the exact public/internal status and keep valid OTA below verified. [VERIFIED: 18-CONTEXT.md; scripts/phase13-firmware-ota-smoke.sh]

**Warning signs:** Evidence has `valid OTA status: 200` but no post-reboot firmware/reference/boot-validation markers. [VERIFIED: scripts/phase13-firmware-ota-smoke-test.sh]

### Pitfall 2: Running OTA While Current App Is Pending Verify

**What goes wrong:** ESP-IDF can reject OTA begin with `ESP_ERR_OTA_ROLLBACK_INVALID_STATE` if rollback is enabled and the running app has not confirmed itself valid. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html]

**Why it happens:** ESP-IDF rollback requires the running app to be confirmed valid before another OTA update is started. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html]

**How to avoid:** Capture boot-validation marker before upload and after reboot; if `ota_boot_validation=not_pending state=factory` appears before valid OTA, expect the next OTA boot to produce a pending/marked-valid style marker. [VERIFIED: firmware/bitaxe/src/boot_validation.rs; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md]

**Warning signs:** Valid OTA body contains validation/activation failure or logs include rollback invalid state. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html; VERIFIED: firmware/bitaxe/src/http_api.rs]

### Pitfall 3: Invalid Upload Evidence Overclaims Rollback

**What goes wrong:** A non-200 invalid image response gets cited as rollback or boot-validation proof. [VERIFIED: 13-REVIEW-FIX.md; 18-CONTEXT.md]

**Why it happens:** Rejected invalid uploads happen before successful new-image boot validation; rollback proof requires post-update bootloader/ESP-IDF state. [VERIFIED: 18-CONTEXT.md; docs/release/ultra-205.md]

**How to avoid:** Keep fields separate: `invalid_image_rejection_status`, `valid_ota_status`, `boot_validation_status`, `rollback_status`, and `checklist_promotion_boundary`. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota.md; scripts/phase13-firmware-ota-smoke.sh]

**Warning signs:** Notes say "invalid image rejection captured" but lack `ota_boot_validation` or selected partition evidence. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; tools/parity/src/main.rs]

### Pitfall 4: Target Provenance Stales Out After Flash

**What goes wrong:** Package and flash evidence are from one source commit, while live OTA upload targets a device running another image or a stale `DEVICE_URL`. [VERIFIED: 16-CONTEXT.md; 18-CONTEXT.md]

**Why it happens:** A raw `DEVICE_URL` can outlive a flash cycle or point to another device on the bench. [VERIFIED: 17-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]

**How to avoid:** Plan a fresh Phase 18 package/flash/target chain or explicitly validate the accepted target against current manifest, selected port, source commit, reference commit, and `/api/system/info` sanity. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh]

**Warning signs:** `DEVICE_URL` is sourced only from environment with no target lock, selected port, source commit, or board `205` binding. [VERIFIED: scripts/phase17-live-http-api-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]

### Pitfall 5: Redaction Review Happens After Documentation Updates

**What goes wrong:** Release docs or checklist cite artifacts before request bodies, serial logs, raw targets, and error files are redaction-reviewed. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

**Why it happens:** Evidence files are generated during hardware/network runs, and helper outputs can include response bodies, curl errors, or serial network identifiers. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase17-live-http-api-smoke.sh]

**How to avoid:** Make `redaction-review.md` a prerequisite for docs/checklist/requirements updates and run a targeted secret scan over Phase 18 evidence before promotion. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

**Warning signs:** `redaction_status: passed` is missing or artifacts are marked absent/not reviewed. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

## Code Examples

Verified patterns from repo sources.

### ESP-IDF OTA Streaming Adapter

```rust
// Source: firmware/bitaxe/src/ota_update.rs
let ota_partition = unsafe { sys::esp_ota_get_next_update_partition(ptr::null()) };
let begin_result = unsafe {
    sys::esp_ota_begin(
        ota_partition,
        sys::OTA_SIZE_UNKNOWN as usize,
        &mut ota_handle,
    )
};
```

This is the correct OTA effect boundary for Phase 18; plans should not replace it with custom binary manipulation. [VERIFIED: firmware/bitaxe/src/ota_update.rs; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html]

### Boot-Validation Marker Contract

```rust
// Source: firmware/bitaxe/src/boot_validation.rs
if startup_diagnostics_passed {
    mark_running_slot_valid()?;
    info_retained("ota_boot_validation=marked_valid");
    return Ok(());
}
```

Phase 18 post-OTA monitor evidence should require `ota_boot_validation=` plus source/reference identity markers before any valid OTA claim. [VERIFIED: firmware/bitaxe/src/boot_validation.rs; scripts/phase13-firmware-ota-smoke.sh]

### Invalid Upload Is Not Rollback Proof

```bash
# Source: scripts/phase13-firmware-ota-smoke.sh
log "invalid image rejection conclusion: captured - not rollback proof"
log "invalid image rejection is not rollback proof"
```

This wording should remain present in Phase 18 artifacts unless a separate gated rollback/fault case is executed. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; 18-CONTEXT.md]

### Marker-Based Helper Pass Condition

```bash
# Source: scripts/phase13-firmware-ota-smoke.sh
for marker in "firmware_commit=" "reference_commit=" "ota_boot_validation="; do
    if grep -Fq "$marker" "$post_ota_monitor_log"; then
        log "post_ota_marker: ${marker} present"
        continue
    fi
    log "post_ota_marker: ${marker} missing"
    missing=1
done
```

The planner should keep this exact class of guard because `tools/parity` also requires valid OTA, invalid image rejection, and boot-validation terms before an OTA verified claim. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; tools/parity/src/main.rs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Serial route registration or empty POST route presence treated as OTA support | Valid OTA evidence requires a real `esp-miner.bin` upload, invalid rejection, reboot identity, and boot-validation marker. [VERIFIED: 17-CONTEXT.md; 18-CONTEXT.md] | Phase 17/18 boundary, 2026-07-03. [VERIFIED: 17-VERIFICATION.md; 18-CONTEXT.md] | `OTA-001` remains below verified until Phase 18 artifacts exist. [VERIFIED: docs/parity/checklist.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] |
| Successful OTA response treated as enough | ESP-IDF rollback-capable OTA requires first-boot validation or explicit rollback state. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html] | ESP-IDF rollback docs and Phase 7 implementation decisions. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-RESEARCH.md] | Phase 18 must record `ota_boot_validation=...` or keep rollback/boot-validation below verified. [VERIFIED: 18-CONTEXT.md] |
| Ad hoc network discovery | Explicit origin-only target provenance and `network_scan: disabled`. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] | Phase 17 target-lock pattern, 2026-07-03. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json] | Phase 18 helpers should fail closed on missing/invalid target URL. [VERIFIED: 18-CONTEXT.md] |
| Destructive rollback/fault tests as casual smoke | Recovery/fault actions require plan-owned commands, allow flags, recovery image, restore command, stop conditions, and redaction. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md] | Phase 13 review fixes and Phase 18 decisions. [VERIFIED: 13-REVIEW-FIX.md; 18-CONTEXT.md] | Phase 18 should prefer non-destructive boot-validation and defer broader recovery regressions to Phase 19. [VERIFIED: 18-CONTEXT.md] |

**Deprecated/outdated:**

- Do not cite Phase 17 `POST /api/system/OTA` empty-body evidence as valid OTA proof. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md]
- Do not claim rollback from invalid-image rejection alone. [VERIFIED: 18-CONTEXT.md; docs/release/ultra-205.md]
- Do not add PlatformIO, direct `esptool.py` upload flows, or a custom OTA stack for Phase 18. [VERIFIED: AGENTS.md; firmware/bitaxe/src/ota_update.rs]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

All claims in this research were verified from local files, command output, cargo registry search, or cited official documentation; no `[ASSUMED]` claims are intentionally present. [VERIFIED: research source audit]

## Open Questions

1. **Is a live Ultra 205 currently connected and uniquely detectable?**
   - What we know: The repo requires `just detect-ultra205` and board-info before hardware work. [VERIFIED: AGENTS.md; 18-CONTEXT.md]
   - What is unclear: This research did not run the detector because it is planning research, not evidence execution. [VERIFIED: command log for this research]
   - Recommendation: First execution task should run `just detect-ultra205` and write blocked evidence on zero/multiple/wrong-board results. [VERIFIED: AGENTS.md; 18-CONTEXT.md]

2. **Can Phase 18 obtain a raw origin-only `DEVICE_URL` without committing it?**
   - What we know: `DEVICE_URL` was unset in the research shell, `wifi-credentials.json` exists but was not read, and Phase 17 has a committed redacted target lock plus a local developer-raw flash evidence artifact. [VERIFIED: environment audit; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]
   - What is unclear: Whether that raw target remains correct after a fresh Phase 18 flash/package run. [VERIFIED: 18-CONTEXT.md]
   - Recommendation: Refresh target provenance during Phase 18 and never commit the raw target. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

3. **Will same-commit OTA be accepted and exercise the next app slot?**
   - What we know: D-08 allows same-image evidence only if checksum, public response, reboot identity, and boot-validation marker prove the path. [VERIFIED: 18-CONTEXT.md]
   - What is unclear: The live firmware response for same-image upload was not executed during research. [VERIFIED: no live OTA command run in this research]
   - Recommendation: Plan a blocked-acceptable branch that records exact public/internal status if same-image/self-image OTA is refused. [VERIFIED: 18-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `git` | source/reference commit recording | yes [VERIFIED: `command -v git`] | `2.53.0` [VERIFIED: `git --version`] | none |
| `node` | GSD tools and some repo helper scripts | yes [VERIFIED: `command -v node`] | `v24.13.0` [VERIFIED: `node --version`] | none |
| `python3` | helper JSON parsing and audits | yes [VERIFIED: `command -v python3`] | `3.14.4` [VERIFIED: `python3 --version`] | none |
| `just` | human command surface | yes [VERIFIED: `command -v just`] | `1.48.0` [VERIFIED: `just --version`] | call underlying Bazel/script only for diagnostics |
| `bazel` / `bazelisk` | canonical build/test/package graph | yes [VERIFIED: `command -v bazel bazelisk`] | `9.1.1` [VERIFIED: `bazel --version`; `bazelisk --version`] | none |
| `cargo` / `rustc` | Rust tool and tests | yes [VERIFIED: `command -v cargo rustc`] | `1.88.0-nightly` [VERIFIED: `cargo --version`; `rustc --version`] | none |
| `espflash` | detector board-info, flash, monitor | yes [VERIFIED: `command -v espflash`] | local `4.0.1`; crates latest `4.4.0` [VERIFIED: `espflash --version`; `cargo search espflash`] | use repo-owned ESP-IDF tools only if a plan documents why `espflash` is insufficient |
| `curl` | live OTA upload | yes [VERIFIED: `command -v curl`] | `8.7.1` [VERIFIED: `curl --version`] | none for live HTTP upload |
| `shasum` / `sha256sum` | helper checksums | yes [VERIFIED: `command -v shasum sha256sum`] | system tools [VERIFIED: command availability audit] | Rust manifest SHA-256 logic in `tools/xtask` |
| Ultra 205 hardware | valid/invalid OTA and serial reboot capture | not probed in research [VERIFIED: no `just detect-ultra205` run] | unknown | write blocked evidence if `just detect-ultra205` fails |
| Explicit `DEVICE_URL` | live OTA upload | unset in research shell [VERIFIED: environment audit] | n/a | direct user-supplied URL or trusted local flash evidence source; no network scan [VERIFIED: 18-CONTEXT.md] |
| `wifi-credentials.json` | optional post-flash Wi-Fi join | file present but contents not read [VERIFIED: `test -f wifi-credentials.json`; AGENTS.md] | n/a | omit if not needed or unavailable |

**Missing dependencies with no fallback:**

- Live OTA evidence requires a detector-approved Ultra 205 and an explicit origin-only target; neither was proven during research. [VERIFIED: AGENTS.md; 18-CONTEXT.md; environment audit]

**Missing dependencies with fallback:**

- `DEVICE_URL` was not set, but Phase 18 may refresh or derive a raw target from trusted local flash evidence while committing only redacted provenance. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `sh_test` for helper scripts and Bazel `rust_test` for Rust crates/tools. [VERIFIED: scripts/BUILD.bazel; tools/parity/BUILD.bazel; crates/bitaxe-api/BUILD.bazel] |
| Config file | `MODULE.bazel`, `Cargo.toml`, `Justfile`, `scripts/BUILD.bazel`, `tools/parity/BUILD.bazel`, `firmware/bitaxe/BUILD.bazel`. [VERIFIED: file scan] |
| Quick run command | `bazel test //scripts:phase13_firmware_ota_smoke_test //tools/parity:tests` when helper/parity behavior changes; `just parity` for docs/checklist-only changes. [VERIFIED: scripts/BUILD.bazel; tools/parity/BUILD.bazel; Justfile] |
| Full suite command | `just test && just package && just parity && just verify-reference`, plus manifest-backed `release-gate` for the copied Phase 18 manifest and every hardware/network command actually used. [VERIFIED: Justfile; 18-CONTEXT.md] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| REL-02 | Invalid image rejection must require a rejection marker and not count unrelated non-200 bodies. [VERIFIED: scripts/phase13-firmware-ota-smoke-test.sh] | shell unit | `bazel test //scripts:phase13_firmware_ota_smoke_test` | yes [VERIFIED: scripts/BUILD.bazel] |
| REL-02 | Valid OTA must upload manifest-listed `esp-miner.bin`, return success response, and require post-OTA identity/boot-validation markers. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh] | shell unit + hardware smoke | `bazel test //scripts:phase13_firmware_ota_smoke_test`; live command gated by plan | yes for helper test; hardware run requires board [VERIFIED: scripts/BUILD.bazel; AGENTS.md] |
| REL-08 | Boot-validation/rollback claims must not promote from invalid rejection alone. [VERIFIED: 18-CONTEXT.md; tools/parity/src/main.rs] | parity unit/checklist | `bazel test //tools/parity:tests && just parity` | yes [VERIFIED: tools/parity/BUILD.bazel; Justfile] |
| REL-07 | Release docs must cite exact Phase 18 artifacts and preserve recovery gates/non-claims. [VERIFIED: docs/release/ultra-205.md; 18-CONTEXT.md] | docs/parity validation | `just parity && just verify-reference` plus targeted `rg` checks for Phase 18 citations | docs exist; Phase 18 artifacts not yet [VERIFIED: docs/release/ultra-205.md; docs/parity/checklist.md] |
| EVD-05 | Redaction review must cover Phase 18 logs, bodies, target, serial, and docs before commit. [VERIFIED: 18-CONTEXT.md] | manual + grep audit | `rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence` | directory not yet [VERIFIED: evidence file scan] |

### Sampling Rate

- **Per task commit:** Run targeted helper/parity checks for changed code or `just parity` for docs/checklist changes. [VERIFIED: standards/core/verification.md; 18-CONTEXT.md]
- **Per wave merge:** Run `just test` for code/helper changes, then `just parity` and `just verify-reference`. [VERIFIED: Justfile; AGENTS.md]
- **Phase gate:** `18-VERIFICATION.md` must have `status: passed`, lifecycle validation must pass for `18-2026-07-03T14-06-29`, and every hardware/network command used must be cited. [VERIFIED: 18-CONTEXT.md]

### Wave 0 Gaps

- [ ] `scripts/phase18-firmware-ota-evidence.sh` and `scripts/phase18-firmware-ota-evidence-test.sh` if the planner chooses a Phase 18-specific wrapper instead of calling Phase 13 helper directly. [VERIFIED: current file scan; 18-CONTEXT.md]
- [ ] `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/` ledgers and `redaction-review.md` before checklist/docs promotion. [VERIFIED: current file scan; 18-CONTEXT.md]
- [ ] Target-lock refresh or explicit target input contract for Phase 18, because `DEVICE_URL` is unset in the research shell. [VERIFIED: environment audit; 18-CONTEXT.md]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not set `security_enforcement: false`. [VERIFIED: .planning/config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | limited | The firmware OTA route uses network/origin access gates rather than a user login flow; do not add new authentication in Phase 18 unless existing route policy changes. [VERIFIED: crates/bitaxe-api/src/update_plan.rs; CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V3 Session Management | no | Phase 18 does not add browser sessions or server-side session state. [VERIFIED: 18-CONTEXT.md; crates/bitaxe-api/src/update_plan.rs] |
| V4 Access Control | yes | Preserve private network/origin access checks and AP/APSTA rejection before upload effects run. [VERIFIED: crates/bitaxe-api/src/update_plan.rs; reference/esp-miner/main/http_server/http_server.c] |
| V5 Input Validation | yes | Validate manifest image identity/checksum, origin-only `DEVICE_URL`, invalid image rejection markers, and ESP-IDF app-image validation result. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase17-live-http-api-smoke.sh; firmware/bitaxe/src/ota_update.rs] |
| V6 Cryptography | yes, limited | Use SHA-256 checksums from package manifests and do not hand-roll crypto; Phase 18 does not introduce signed OTA or secure boot changes. [VERIFIED: tools/xtask/src/package_manifest.rs; 18-CONTEXT.md] |
| V7 Error Handling and Logging | yes | Log sanitized status markers and redact secrets/targets before committing evidence. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md] |
| V12 Files and Resources | yes | Treat uploaded firmware images and generated invalid fixture files as bounded update artifacts, not arbitrary committed release files. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; 18-CONTEXT.md] |
| V13 API and Web Service | yes | `/api/system/OTA` is an HTTP API route that must preserve access gates, method, response status/body, and error behavior. [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml; firmware/bitaxe/src/http_api.rs] |

### Known Threat Patterns for Phase 18

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Wrong device receives OTA because target was inferred or stale | Spoofing/Tampering | Require explicit target lock or direct origin-only `DEVICE_URL`, board `205`, selected port, source/reference commits, and `network_scan: disabled`. [VERIFIED: 18-CONTEXT.md; scripts/phase17-live-http-api-smoke.sh] |
| Invalid firmware upload accepted or misclassified | Tampering | Fixed invalid fixture, non-200 requirement, rejection body marker requirement, and blocked evidence on unrelated responses. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; scripts/phase13-firmware-ota-smoke-test.sh] |
| OTA response cited as rollback proof | Repudiation | Separate valid upload, invalid rejection, boot-validation, rollback, and non-claim fields; run `just parity` verified-row guards. [VERIFIED: 18-CONTEXT.md; tools/parity/src/main.rs] |
| Secrets or private endpoints leak into evidence | Information Disclosure | Redact response bodies, curl errors, serial logs, target locks, and docs; run a redaction review before commit. [VERIFIED: 18-CONTEXT.md; scripts/phase13-firmware-ota-smoke.sh; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md] |
| Device becomes unavailable after OTA/fault action | Denial of Service | Prefer non-destructive boot validation first; run destructive/fault actions only with recovery image, restore command, stop conditions, and safe-state markers documented. [VERIFIED: 18-CONTEXT.md; docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md] |
| Checklist overclaims release parity | Elevation of Privilege/Repudiation | Promote only exact evidence tier and keep `REL-003`/rollback/fault claims below verified unless terms and evidence artifacts exist. [VERIFIED: 18-CONTEXT.md; tools/parity/src/main.rs] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/18-firmware-ota-and-rollback-evidence/18-CONTEXT.md` - locked decisions, discretion, deferred scope, canonical refs. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - REL-02, REL-08, REL-07, EVD-05 definitions and traceability. [VERIFIED: file read]
- `.planning/ROADMAP.md` - Phase 18 goal, success criteria, verification expectations, and research flags. [VERIFIED: file read]
- `.planning/STATE.md` - current milestone state and accumulated evidence decisions. [VERIFIED: file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md` - repo constraints and standards. [VERIFIED: file reads]
- `scripts/phase13-firmware-ota-smoke.sh` and `scripts/phase13-firmware-ota-smoke-test.sh` - reusable OTA helper and tests. [VERIFIED: file reads]
- `scripts/phase17-live-http-api-smoke.sh`, Phase 17 target lock, summary, and redaction review - explicit target and redaction patterns. [VERIFIED: file reads]
- `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/ota_update.rs`, `firmware/bitaxe/src/boot_validation.rs`, `crates/bitaxe-api/src/update_plan.rs` - current Rust OTA and boot-validation implementation. [VERIFIED: file reads]
- `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs`, `tools/xtask/src/package_manifest.rs` - release-gate and overclaim validation. [VERIFIED: file reads]
- ESP-IDF OTA docs for `v5.5.4` - OTA partition, rollback, boot validation, and OTA API semantics. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html]
- Espressif ESP-IDF release page - confirms `v5.5.4` release and docs availability. [CITED: https://github.com/espressif/esp-idf/releases]
- OWASP ASVS project page - ASVS as a framework for web application/service security requirements. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Secondary (MEDIUM confidence)

- `cargo search` registry results for `esp-idf-svc`, `esp-idf-sys`, `embuild`, `espflash`, `clap`, `serde`, `anyhow`, and `thiserror`. [VERIFIED: cargo registry search]
- Local command availability audit for `git`, `node`, `python3`, `just`, `bazel`, `bazelisk`, `cargo`, `rustc`, `espflash`, `curl`, `shasum`, and `sha256sum`. [VERIFIED: command availability audit]
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-RESEARCH.md` and `.planning/phases/13-final-ultra-205-release-evidence/13-REVIEW-FIX.md` for prior OTA research and helper hardening history. [VERIFIED: file reads]

### Tertiary (LOW confidence)

- None. [VERIFIED: source audit]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - existing versions and commands are pinned or installed locally, and no new dependencies are recommended. [VERIFIED: Cargo.lock; command availability audit]
- Architecture: HIGH - Phase 18 maps directly onto existing helper/firmware/parity boundaries. [VERIFIED: scripts/phase13-firmware-ota-smoke.sh; firmware/bitaxe/src/ota_update.rs; tools/parity/src/main.rs]
- ESP-IDF rollback semantics: HIGH - verified against official ESP-IDF OTA docs and current firmware calls. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/system/ota.html; VERIFIED: firmware/bitaxe/src/boot_validation.rs]
- Live hardware feasibility: MEDIUM - tools and prior target evidence exist, but research did not run detector or OTA upload. [VERIFIED: environment audit; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json]
- Redaction/promotion pitfalls: HIGH - prior review fixes and Phase 17 redaction patterns are explicit. [VERIFIED: 13-REVIEW-FIX.md; docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md]

**Research date:** 2026-07-03 [VERIFIED: environment current_date]
**Valid until:** 2026-07-10 for live-tool/hardware assumptions; 2026-08-02 for repo-local architecture if no OTA/helper changes land first. [VERIFIED: current date; ASSUMED validity window based on phase volatility]

## RESEARCH COMPLETE

Phase 18 research is complete and ready for planning. [VERIFIED: this file]
