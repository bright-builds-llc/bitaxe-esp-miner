---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
status: gaps_found
score: "8/10 requirements satisfied"
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T01:03:59Z
lifecycle_validated: true
---

# Phase 34 Verification Report

## Verification Result

Phase 34 remains **gaps found**. Plans 34-05 through 34-07 repaired the three defects from the prior verification in their production paths:

1. The default factory image is now structurally compared with the admitted OTA image.
2. Duplicate yield-log suppression no longer stops recurring supervisor checkpoints.
3. One production publisher now assigns revisions after collection and holds its ordering mutex through retention and the real HTTP/WebSocket issue call.

The subsequent `34-REVIEW.md` identified four additional defects. Independent source tracing confirms all four. Three defects prevent the bytes consumed by flashing from being the exact complete ESP32-S3 application proved by admission, blocking SYS-02. The fourth permits externally issued snapshots to have no matching retained pair while reporting retention success, blocking OBS-06.

SYS-01, SYS-03 through SYS-05, and HLT-01 through HLT-04 are satisfied by the inspected production paths. Phase 35 remains blocked. No standards override exists or applies.

The repo-local GSD, software-only Phase 34, exact-package, no-hardware, no-direct-UART/pin, archived-lineage, and evidence-redaction rules materially informed this result. The Bright Builds architecture, verification, testing, and Rust standards also informed the production-call-path and regression assessment.

## Goal Achievement

| Phase success criterion | Status | Direct evidence |
| --- | --- | --- |
| System-info, live WebSocket, retained logs, and evidence share one boot session and monotonic operator-snapshot revision. | GAP | `OperatorSnapshotPublisher::publish` in `crates/bitaxe-api/src/operator_snapshot_publication.rs` assigns after collection and holds one mutex through `retain` and `issue`. `http_api.rs` places system-info, live-cadence, and live-connect sends inside issue closures. However, `runtime_snapshot.rs:368-370` converts the production retained adapter into unconditional `Ok(())`, while `log_buffer.rs:11-21` silently returns on poison and `log_buffer.rs:75-84` can install a zero-capacity buffer that drops every append. Public issue can therefore proceed without either retained correlation line. |
| Running identity truthfully reports semantic/source/reference/package and running-platform facts or unavailable. | GAP | Typed runtime identity and platform facts are present, but the explicit-image path may select any manifest-listed artifact after unrelated factory/OTA validation; execution later reopens mutable paths; and the OTA parser does not validate the complete ESP32-S3 image contract. The flashed bytes are not fully bound to the admitted package identity. |
| Passive self-test exposes only the seven approved states without starting a self-test. | VERIFIED | `PassiveSelfTestState` is a closed seven-value enum in `crates/bitaxe-core/src/runtime_health.rs`; the production adapter only reads checkpoint history and invokes the pure evaluator. |
| Supervisor checkpoint health reflects recurring progress and remains separate from unproved task-watchdog participation. | VERIFIED | `transition_supervisor_step` in `firmware/bitaxe/src/safety_adapter/watchdog.rs:109-139` treats duplicate logging as optional and calls `record_supervisor_checkpoint` on every completed step. The production-module regression proves twelve increasing checkpoints. `RuntimeHealthSnapshot::evaluate` reports task-watchdog participation independently as `unavailable` with reason `unproved`. |
| No fixture/host substitution, active health intervention, hardware actuation, credentials, Phase 35, or archived-lineage work occurs. | VERIFIED | Inspected production identity and health adapters are read-only and source guards reject the prohibited effect paths. Verification used no hardware, USB, serial, network, credentials, OTA, direct UART/pins, Phase 35, or archived-lineage operation. |

## Requirement Verification

| Requirement | Status | Direct code evidence |
| --- | --- | --- |
| OBS-06 | PENDING | Typed session/revision correlation and monotonic external ordering are implemented, but production retention cannot report failure. An HTTP/WebSocket revision can be issued after both retained appends were dropped, so all four projections are not guaranteed to derive from the same retained snapshot record. |
| SYS-01 | PASSED | `BuildProvenance` validates semantic version and full source commit; `firmware/bitaxe/build.rs` requires the generated stamp and exports compile-time fields without invoking Git; `runtime_snapshot.rs:543-550` copies those fields into runtime projection. |
| SYS-02 | PENDING | The manifest and runtime expose pinned reference commit and application-descriptor ELF SHA, and default factory-to-OTA equality is checked. Nevertheless, explicit selection can bypass the structurally admitted artifact, the subprocess reopens a mutable path after admission, and the OTA parser accepts incomplete/foreign image structures. Exact flashed-package correlation is not closed. |
| SYS-03 | PASSED | `firmware/bitaxe/src/platform_identity.rs` reads embedded static release identity, `esp_get_idf_version`, the running partition, and closed board `205`/ASIC `BM1366` values into typed platform facts. |
| SYS-04 | PASSED | The same adapter reads `esp_reset_reason`, `esp_timer_get_time`, internal heap free/minimum/largest-block values, and PSRAM availability from ESP-IDF, with typed unavailable fallbacks. |
| SYS-05 | PASSED | `PlatformFact<T>` separates `Available` from explicit `Unavailable` reasons. Compatibility zero/`Unavailable` projections never change the typed fact to available, and production collection does not read host fixtures or Git. |
| HLT-01 | PASSED | The passive self-test vocabulary is exactly idle, blocked, running, passed, failed, canceled, and unavailable. The read adapter cannot start or mutate a self-test. |
| HLT-02 | PASSED | `run_supervisor_step` obtains the checkpoint state, the transition validates and records bounded category `telemetry`, increasing sequence, and monotonic observation time, and `runtime_health_adapter::collect` projects latest category/sequence/age into the coherent snapshot. |
| HLT-03 | PASSED | Supervisor availability/checkpoints and task-watchdog participation are separate fields; the evaluator does not infer task-watchdog participation and reports it unavailable/unproved. No `esp_task_wdt_*` call exists in the Phase 34 health path. |
| HLT-04 | PASSED | Recurring production checkpoints now advance after the first yield. The pure evaluator derives healthy through `3 * cadence`, stale after that boundary, and unhealthy after `10 * cadence`; a truly frozen sequence ages while a running loop refreshes it. |

**Score: 8/10 requirements satisfied.** The four review defects collapse into two requirement gaps: OBS-06 and SYS-02.

## Plan-by-Plan Adjudication

### Plan 34-01 — Canonical Build Provenance

The shared build identity, four label states, full machine commits, runtime/API/LCD projections, schema-v3 manifest, scoped dirty classification, and dirty-package rejection are present. Its strongest admission must-have remains partial because the final bytes consumed by the flashing process are not always the structurally admitted immutable bytes. SYS-01 passes; SYS-02 remains pending.

### Plan 34-02 — Coherent Snapshot Identity

The typed boot session, nonzero revision, checked sequence, public wire fields, retained marker format, and evidence parser are present. Plan 34-07 corrected concurrent revision reservation/publication order. The remaining production-retention false-success path means the retained projection is not guaranteed for every externally issued identity, so OBS-06 remains pending.

### Plan 34-03 — Truthful Running Platform

The production adapter is the sole read-only ESP-IDF/static-asset source for platform facts. Closed board/ASIC/reset vocabularies and per-field unavailable states are attached to the same candidate before projection. SYS-03, SYS-04, and SYS-05 pass.

### Plan 34-04 — Passive Runtime Health

The pure vocabulary, age thresholds, task-watchdog separation, read-only adapter, additive system-info/live fields, and redacted retained record are implemented. The original producer defect was outside the pure evaluator and is repaired by Plan 34-06. HLT-01 through HLT-04 pass.

### Plan 34-05 — Structural Factory Admission

The default package path now requires unique required kinds, parses a factory partition table, parses bounded descriptor fields, and compares the OTA-length factory application prefix byte-for-byte with the OTA image. A recomputed-digest factory-app tamper is rejected before device effects. The plan closes the prior default-path mismatch but does not close explicit selection, immutable execution, or the complete ESP image format.

### Plan 34-06 — Recurring Supervisor Checkpoints

Verified. `transition_supervisor_step` no longer returns when the one-time yield log is suppressed. Every completed step attempts validated checkpoint publication, while the adapter remains passive and contains no task-watchdog or hardware effect. The production-source Bazel regression passes.

### Plan 34-07 — Ordered Snapshot Publication

Revision assignment, retained-call ordering, and actual HTTP/WebSocket issuance now share one production authority. Reverse collection completion produces publication order `[1, 2]` without sorting. Poison, reentrancy, sequence exhaustion, and generic fallible retention/issuance behavior are modeled correctly. The firmware's concrete retention closure defeats the generic retention guarantee by always returning success after two infallible-looking append calls; therefore the OBS-06 goal remains incomplete.

## Review-Finding Adjudication

### CR-01 — Confirmed: explicit image selection bypasses structural admission

`validate_identity_admission` validates only the unique required ELF, OTA, and factory artifacts and their relationship (`tools/flash/src/main.rs:1019-1079`). The later explicit branch (`1002-1007`) accepts any path found anywhere in `manifest.artifacts`. `require_manifest_artifact_for_path` (`1112-1124`) neither restricts artifact kind nor binds the selected entry to the validated factory artifact. `flash_command_for_image` (`778-805`) dispatches by filename, not a closed admitted kind.

Consequently, a modified manifest-listed ELF or extra artifact can be selected after the unrelated required factory/OTA pair passes. An extra path named `bitaxe-ultra205-factory.bin` is routed to raw `write-bin 0x0`. Recomputing only that selected entry's digest satisfies the final selected-path digest check. No test covers either bypass.

**Impact:** blocks SYS-02.

### CR-02 — Confirmed: admitted bytes are reopened from a mutable path

Admission reads and hashes package bytes at `main.rs:1055-1068` and `1137-1147`, but `resolve_flash_image` returns only a path. `prepare_flash` then performs port resolution and optional NVS work (`757-775`), and `run_flash` later calls `environment.execute` with that path (`647-655`). The child `espflash` process reopens the package file. No file handle, admission-owned byte snapshot, immutable temporary artifact, or final atomic identity check bridges validation to consumption.

Replacing the selected path after admission but before subprocess open can therefore flash bytes that were never admitted. The current fake environment has no race regression for this boundary.

**Impact:** blocks SYS-02 and exact-package evidence readiness.

### CR-03 — Confirmed: production retention silently reports success

The generic publisher correctly skips issuance when `retain` returns an error. The firmware adapter at `runtime_snapshot.rs:368-370`, however, calls `retain_completed_operator_snapshot` and unconditionally returns `Ok(())`. That helper performs two independent `append_runtime_log_line` calls (`433-437`). Each append silently returns on mutex poison (`log_buffer.rs:11-21`). If both buffer allocations fail, `fallback_log_buffer` installs `RetainedLogBuffer::empty()` (`75-84`), whose `append` increments a counter but retains no bytes (`crates/bitaxe-api/src/logs.rs:197-202`).

An external issue call can therefore succeed with no retained marker or health line, or potentially only part of the pair. Generic publisher tests use an injected fallible closure and do not compile or exercise this concrete adapter failure behavior.

**Impact:** blocks OBS-06.

### WR-01 — Confirmed: OTA structural validation is incomplete

`validate_ota_identity` checks the magic byte, segment count/ranges, first-segment descriptor magic/version/ELF SHA, and later searches validated segment payloads for the source commit. After the final declared segment, it immediately returns payload ranges (`tools/flash/src/package_admission.rs:62-123`). It never validates the ESP image header's chip identifier or supported header fields, segment checksum, required alignment/padding, an appended SHA-256 when declared, or exact trailer consumption.

Thus factory and OTA bytes can agree and carry expected descriptor/source fields while still representing a foreign-chip or corrupted ESP application. Artifact digests only prove consistency with the manifest after recomputation; they do not prove the application format.

**Impact:** blocks SYS-02's flashed-package identity/admission claim.

## Focused Verification Evidence

All commands ran at source `1ca5efe7d9ddab86dace88ea6efae96ccf6cd4ca`. No hardware or external input was used.

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-api operator_snapshot_publication` | PASS — 7 production publisher tests, including reverse completion, retained failure, issue failure, poison, reentrancy, and exhaustion. These use injected retention and do not exercise firmware log storage. |
| `cargo test -p bitaxe-core runtime_health` | PASS — 11 health vocabulary, transition, boundary, freeze, recovery, and watchdog-separation tests. |
| `cargo test -p bitaxe-flash package_admission` | PASS — 14 structural factory/OTA tests. There are no chip-ID, checksum, appended-digest, padding/trailer, explicit-selected-artifact, or TOCTOU regressions. |
| `cargo test -p bitaxe-parity phase34_source_guard` | PASS — 5 source guards. The guards prove ordering syntax and default factory linkage but do not reject the four confirmed paths. |
| `bazel test //firmware/bitaxe:supervisor_checkpoint_production_tests //firmware/bitaxe:operator_snapshot_publication_tests //tools/flash:tests //tools/parity:tests //crates/bitaxe-api:tests //crates/bitaxe-core:tests` | PASS — five unique test targets after suite expansion; all were cache hits. |
| `just verify-reference` | PASS — pinned reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |

The plan summaries record successful mandatory Rust sequences, repository-wide Bazel tests, canonical build/package, and reference checks for each implementation commit. Those runs are credible regression evidence, but they do not supersede the inspected production blind spots above.

## Required Gap Closure

### SYS-02: close the admission-to-consumption boundary

1. Replace path-based selection with a closed admitted artifact kind. For non-dry-run hardware admission, either allow only the exact unique `factory_merged_image` artifact or structurally bind every supported selected kind to the canonical OTA/ELF identity. Reject extra/unknown kinds, path aliases, and factory-name dispatch outside that closed kind.
2. Carry one admission-owned byte snapshot through the flash effect. Prefer a byte-consuming API; otherwise write admitted bytes once to a private owner-only temporary artifact, retain its owner for the entire child lifetime, and execute only that snapshot. Add a deterministic race regression that mutates the package tree after admission and proves executed bytes remain admitted or execution fails closed.
3. Validate the complete supported ESP32-S3 application image: chip ID/revision/header flags, segment checksum, alignment/padding, declared appended digest, and exact trailer consumption. Add one focused mutation per rejected field.
4. Prove all three failures occur before port discovery, credential handling, subprocess execution, or hardware interaction.

### OBS-06: make retained correlation transactional and fallible

1. Add one typed fallible log-buffer operation that acquires the mutex once, rejects poisoned/unavailable/zero-capacity storage, and appends the marker and runtime-health lines as one bounded retained transaction.
2. Return that result directly from the production publisher retention closure. A retention failure must consume the revision, skip HTTP/WebSocket issue, release the publisher lock, and expose only a redacted failure category.
3. Compile the production log-storage adapter in host tests. Cover unavailable storage, lock poison, and pair atomicity; assert the issue closure is not called and no partial pair is reported retained.

After implementation, rerun the exact mandatory Rust sequence, focused Cargo/Bazel regressions, `bazel test //...`, canonical build/package/reference checks, and fresh Phase 34 code review and verification. Phase 35 must remain blocked until the replacement verification passes.

## Human Verification

None. These are deterministic software-boundary defects. Hardware cannot repair or reliably disprove them and must not be used to qualify Phase 34.

## Exact Non-Claims

- This report does not claim Phase 34 passed.
- It does not claim an arbitrary explicit image is structurally admitted.
- It does not claim the subprocess consumes the bytes that admission hashed.
- It does not claim the current OTA parser proves a complete ESP32-S3 application image.
- It does not claim every issued snapshot has a matching retained marker and runtime-health pair.
- It does not qualify any package or source commit on hardware.
- It does not provide Phase 35 evidence, CFG-12/EVD closure, parity promotion, mining, OTA/recovery, active health, credential, other-board, or archived-lineage proof.

## Completion Summary

Phase 34 is not complete at the production boundary. Eight requirements pass. OBS-06 and SYS-02 remain pending because retained correlation can silently disappear and exact admitted bytes are not yet guaranteed to be the complete bytes consumed by flashing. Another `--gaps` planning cycle should address the two closure groups above before Phase 35 begins.
