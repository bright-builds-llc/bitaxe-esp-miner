---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: "09-2026-06-29T13-16-47"
generated_at: "2026-06-29T13:20:44.365Z"
---

# Phase 9: Flash-Monitor Evidence Wrapper Hardening - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 9 closes the wrapper evidence gap from Phase 8. It must make `just flash-monitor board=205 port=... evidence-dir=...` produce trusted Ultra 205 serial evidence through the repo-owned wrapper, without falling back to raw `espflash` commands and without modifying `reference/esp-miner`.

This phase is limited to flash-monitor evidence capture, recovery guidance, evidence metadata, and workflow/release documentation for that serial-scope proof. It does not establish live HTTP, static, recovery, OTA, rollback, large-erase, failed-update, interrupted-update, ASIC, mining, or safety-control parity.

</domain>

<decisions>
## Implementation Decisions

### Noninteractive Monitor Capture

- **D-01:** Add a first-class wrapper-owned noninteractive evidence path for `flash-monitor --evidence-dir` that invokes `espflash monitor --chip esp32s3 --port <port> --non-interactive` after the flash command.
- **D-02:** Preserve normal interactive `just monitor` behavior for operator/manual use. Evidence capture should not depend on an interactive input reader or PTY-only behavior.
- **D-03:** Keep using esp-rs tooling. Do not add a custom serial backend or PTY dependency unless `espflash monitor --non-interactive` proves unable to capture the required Ultra 205 boot log.
- **D-04:** Define a bounded capture strategy for evidence mode so the wrapper does not hang indefinitely. The captured log must still fail closed if the process exits unsuccessfully, times out without trusted output, or cannot create the evidence log.

### Evidence Record Contract

- **D-05:** Enrich the existing `flash-command-evidence.json` as the canonical machine-readable evidence record instead of relying on manual prose transcription.
- **D-06:** The evidence record for `flash-monitor` must include command kind, board, selected port, source/firmware commit, reference commit, package manifest path, flash image path, exact flash command, exact monitor command, monitor log path, capture mode, capture status, timestamp, and a conclusion/status field.
- **D-07:** It is acceptable to add a generated Markdown evidence summary if the implementation can render it from the same structured record, but the JSON record remains the source of truth.
- **D-08:** Avoid manual evidence-only docs as the primary Phase 9 fix. Manual prose may summarize results, but it must cite wrapper-produced artifacts.

### Failure Handling And Recovery Guidance

- **D-09:** Monitor startup failures must fail visibly and explain that evidence capture is not trusted. The wrapper must not silently retry, silently switch modes, or leave a partial log presented as valid proof.
- **D-10:** Recovery guidance should point to repo-owned steps: rerun `just detect-ultra205`, confirm exactly one port, rerun `just flash-monitor board=205 port=<port> evidence-dir=<path>`, and use `just monitor port=<port>` or PTY/manual monitor only as diagnostic follow-up.
- **D-11:** If an input-reader failure is still observed, the recovery text should specifically steer operators to the wrapper's noninteractive evidence path rather than asking them to run raw `espflash monitor` directly.

### Hardware Evidence And Checklist Boundary

- **D-12:** Capture fresh wrapper-based Ultra 205 serial evidence after implementation when `just detect-ultra205` succeeds with exactly one ESP32-S3 candidate.
- **D-13:** Update workflow/release evidence so raw-monitor fallback is no longer the only proof for the serial boot evidence path. Prefer refreshing `WF-005` and relevant release docs over adding a duplicate checklist row unless the existing row cannot express the closure clearly.
- **D-14:** Do not promote `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, or `REL-003` to verified from Phase 9 serial evidence alone. Those rows still require the live HTTP/static/recovery/OTA/rollback evidence planned for later phases.
- **D-15:** Keep all evidence records free of secrets, private endpoints, pool credentials, Wi-Fi credentials, and NVS secret values.

### the agent's Discretion

The agent may choose exact CLI flag names, capture timeout defaults, evidence schema field names, helper function boundaries, tests, and whether to emit a Markdown summary in addition to JSON. Those choices must preserve the existing `tools/flash` command surface, keep `just` as the user entrypoint, keep `espflash` as the backend, use typed Rust structs for evidence records, and keep `reference/esp-miner` read-only.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 9 goal, gap closure, success criteria, verification expectations, and dependencies.
- `.planning/REQUIREMENTS.md` - FND-07, FND-08, REL-07, and EVD-05 requirements that Phase 9 advances.
- `.planning/PROJECT.md` - Ultra 205 target, evidence policy, ESP-IDF Rust stack, and read-only reference constraint.
- `.planning/STATE.md` - Current project state and accumulated hardware/evidence decisions.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Original package, flash, monitor, and parity evidence decisions.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - Release package/evidence contract and manifest decisions.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - Release evidence workflow, hardware gate, and overclaim prevention decisions.

### Existing Implementation Surfaces

- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, port resolution, evidence record, and command execution implementation.
- `tools/flash/BUILD.bazel` - Bazel binary and test targets for the flash wrapper.
- `tools/flash/Cargo.toml` - Flash tool dependencies.
- `Justfile` - Human command surface for `detect-ultra205`, `flash`, `monitor`, and `flash-monitor`.
- `scripts/detect-ultra205.sh` - Required read-only hardware detection gate before autonomous Ultra 205 runs.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 metadata that evidence records cite.
- `firmware/bitaxe/BUILD.bazel` - Firmware package target that `tools/flash` builds before flashing.

### Evidence, Release, And Parity Documents

- `docs/parity/checklist.md` - Workflow row `WF-005` and release rows that must not overclaim Phase 9 evidence.
- `docs/parity/evidence/phase-08-ultra-205-release-gate.md` - The raw-monitor fallback gap Phase 9 must close.
- `docs/parity/evidence/phase-08-ultra-205-release-summary.md` - Phase 8 release summary and remaining live evidence blockers.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` - Prior successful wrapper evidence style and serial-scope hardware record.
- `docs/release/ultra-205.md` - Operator release guide, flash-monitor evidence command, and remaining HTTP/OTA evidence gates.

### ADRs And Policy

- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0004-bazel-automation-with-just-wrapper.md` - Bazel and Just command boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as audit evidence.
- `docs/adr/0011-usb-flashing-ergonomics.md` - USB flashing and monitoring command behavior.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and verification status semantics.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 first target and non-205 evidence boundary.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `tools/flash/src/main.rs` already has `CommandSpec`, typed CLI structs, `FlashEnvironment`, `LocalFlashEnvironment`, `prepare_monitor_command`, `run_flash_monitor`, and `EvidenceRecord`.
- `LocalFlashEnvironment::execute_capturing` already captures stdout and stderr into a log path; it currently runs the same interactive monitor command that failed in Phase 8.
- Existing flash-tool tests use `FakeFlashEnvironment` and `tempfile` to verify command construction, evidence file creation, board rejection, and port resolution behavior.
- `scripts/detect-ultra205.sh` already prints `port=<path>` only after exactly one likely port and successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`.

### Established Patterns

- `tools/flash` is a Rust CLI with typed parsing and testable pure command construction around an imperative shell that runs `bazel` and `espflash`.
- Evidence records are JSON and package manifests are machine-readable; docs may summarize evidence but should not become the only source of trusted fields.
- Repo-local hardware guidance requires `just detect-ultra205` before autonomous hardware use and requires exact commands, board, port, source commit, reference commit, logs, observed behavior, and conclusion.
- GSD artifacts must use standalone `---` only for YAML frontmatter delimiters at the top of parsed Markdown files.

### Integration Points

- Extend `prepare_monitor_command` or add a focused evidence-mode variant so `flash-monitor --evidence-dir` can use `--chip esp32s3` and `--non-interactive` without changing ordinary `monitor`.
- Extend `EvidenceRecord` and tests so the wrapper proves flash command, monitor command, log path, manifest, board, port, commits, capture status, and conclusion.
- Update `docs/parity/evidence/`, `docs/parity/checklist.md`, and `docs/release/ultra-205.md` only for the wrapper evidence gap and serial-scope proof.

</code_context>

<specifics>
## Specific Ideas

- Use the Phase 8 fallback command shape as the target wrapper-owned command: `espflash monitor --chip esp32s3 --port <port> --non-interactive`.
- Keep recovery text direct: "evidence capture failed and is not trusted" plus exact repo commands to rerun detection and wrapper capture.
- Evidence should make capture status explicit rather than forcing reviewers to infer validity from file existence.
- Fresh hardware evidence should record the generated `flash-command-evidence.json`, `flash-monitor.log`, selected port, source commit, reference commit, package manifest, observed boot lines, and a narrow conclusion that serial wrapper evidence passed.

</specifics>

<deferred>
## Deferred Ideas

- Live HTTP/static/recovery/OTA/rollback/large-erase/failed-update/interrupted-update evidence remains Phase 13 release evidence scope unless a later roadmap change says otherwise.
- A custom serial monitor backend remains deferred unless `espflash monitor --non-interactive` cannot produce reliable evidence.
- A dedicated new parity checklist row for wrapper evidence capture remains optional; start by refreshing existing workflow evidence unless `WF-005` proves too broad.
- Non-205 board verification, ASIC/mining hardware evidence, and safety-controller hardware regression evidence remain later phases.

</deferred>

*Phase: 09-flash-monitor-evidence-wrapper-hardening*
*Context gathered: 2026-06-29*
