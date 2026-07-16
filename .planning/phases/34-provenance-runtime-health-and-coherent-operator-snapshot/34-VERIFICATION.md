---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
status: gaps_found
score: "9/10 requirements satisfied"
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T05:30:56Z
lifecycle_validated: true
---

# Phase 34 Verification Report

## Verification Result

Phase 34 remains **gaps found** at **9/10 requirements satisfied**.

Plan 34-09 closes the prior OBS-06 retention gap in the real production path. One publication authority now collects an unnumbered candidate, assigns a monotonic revision after collection, constructs the public and retained projections, transactionally retains the complete marker/runtime-health pair through the named fallible firmware adapter, and only then performs the actual system-info or live-WebSocket issue call. Production retention failures preserve their concrete error, skip issuance, consume the failed revision, release the ordering lock, and permit the next strictly greater revision. OBS-06 passes.

SYS-02 remains blocked by two independently reproduced exact-package admission gaps from the current `34-REVIEW.md`. Current admission accepts both a package whose application entry/load addresses make it non-bootable and a package whose declared `firmware_elf` artifact contradicts the application descriptor's `app_elf_sha256`. The normal package generator emits a coherent package, but the admission boundary does not enforce these relationships. Phase 35 therefore remains blocked.

The repo-local GSD, exact-source/package, software-only Phase 34, evidence-redaction, no-hardware, no-direct-UART/pin, Phase 35, and archived-lineage rules materially informed this result. `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` informed the production-boundary and behavioral-proof assessment. No active standards override applies.

## Requirement Score

| Requirement | Score | Status | Direct evidence |
| --- | ---: | --- | --- |
| OBS-06 | 1/1 | PASSED | `OperatorSnapshotPublisher::publish` collects before locking and holds one mutex through identity allocation, completion, retention, and issue. `runtime_snapshot.rs` returns the named production retention adapter result directly. `log_buffer.rs` validates and appends one `RetainedPair` under one singleton mutex acquisition. `http_api.rs` performs system-info, cadence, and live-connect issuance inside the publisher issue closure. Exact production modules are compiled together in `retained_pair_production_tests`; unavailable, undersized, and poisoned storage skip issue, retain no partial pair, consume the revision, and allow the next revision. |
| SYS-01 | 1/1 | PASSED | `BuildProvenance` validates semantic version and full embedded source commit; `firmware/bitaxe/build.rs` requires generated stamp fields and performs no runtime/host Git substitution. `runtime_snapshot.rs` copies the compiled semantic version and full source commit into the coherent candidate. |
| SYS-02 | 0/1 | GAP | Reference commit, descriptor ELF SHA, manifest digests, unique factory selection, factory/OTA equality, and immutable execution bytes are present. However, entry/load-address mutations and an independently changed packaged ELF both pass the real admission path after enclosing digests are recomputed. The admitted package can therefore be non-bootable or internally contradictory. |
| SYS-03 | 1/1 | PASSED | `platform_identity.rs` uses the embedded static asset plus read-only ESP-IDF calls for IDF version and running partition, with closed board `205` and BM1366 types. |
| SYS-04 | 1/1 | PASSED | The same adapter reads and types reset reason, monotonic uptime, internal free/minimum/largest heap facts, and PSRAM availability. Unknown/nonpositive facts fail to explicit unavailable states. |
| SYS-05 | 1/1 | PASSED | `PlatformFact<T>` is a closed available/unavailable sum type. Fixture candidates authenticate no fact, unknown reset reasons remain unavailable, and compatibility zero/`Unavailable` projections do not promote availability. |
| HLT-01 | 1/1 | PASSED | `PassiveSelfTestState` contains exactly idle, blocked, running, passed, failed, canceled, and unavailable. The Phase 34 adapter reads existing state only and exposes no self-test start operation. |
| HLT-02 | 1/1 | PASSED | The production supervisor records bounded category `telemetry`, checked sequence, and monotonic observation time on every completed step; the coherent candidate copies the latest history and derived age. |
| HLT-03 | 1/1 | PASSED | Supervisor availability/checkpoint health and task-watchdog participation are separate fields. Without direct proof, the evaluator emits task-watchdog `unavailable` with reason `unproved`; the Phase 34 path contains no `esp_task_wdt_*` mutation. |
| HLT-04 | 1/1 | PASSED | Fixed checkpoints age from healthy through stale to unhealthy at checked `3 * cadence` and `10 * cadence` boundaries. The production supervisor regression proves recurring steps continue advancing checkpoints after one-time yield-log suppression. |

**Total: 9/10.**

## Consolidated Must-Have Adjudication

| Phase must-have | Status | Evidence |
| --- | --- | --- |
| One completed capture carries read-only telemetry, confirmed settings, platform identity, and passive health before revision assignment. | VERIFIED | `collect_operator_snapshot_candidate` gathers projection state, platform facts, runtime health, sensor observations, storage-confirmed settings, and Wi-Fi facts before `OperatorSnapshotPublisher::publish` acquires its ordering mutex. `complete_operator_snapshot` attaches the assigned identity once. |
| System-info, live WebSocket, retained records, and evidence projections use the same typed boot session/revision and cannot publish in decreasing order. | VERIFIED | System-info and live telemetry serialize the same `SystemInfoWire` identity fields. The production publisher owns retention and actual issuance. The reverse-completion regression observes direct retained and combined HTTP/WebSocket revisions `[1, 2]` without sorting. |
| Retained marker/runtime-health correlation is atomic, fallible, and required before public issue. | VERIFIED | `RetainedPair::try_new` normalizes and validates both records; `RetainedLogBuffer::try_append_pair` preflights complete capacity/counter state; `retain_operator_snapshot_pair` holds one production mutex guard; the named adapter returns its concrete error unchanged; the publisher's `?` skips issue on retention failure. |
| Retention and issuance preserve distinct concrete error sources and ordering-lock health. | VERIFIED | `OperatorSnapshotPublishError<RetentionError, IssueError>` has separate `Retention` and `Issuance` sources. Focused unit tests use different sentinel types; firmware compiles internal `Infallible`, system-info `anyhow::Error`, cadence `LiveCadenceIssueError`, and WebSocket `esp_err_t` issue channels alongside `RetainedPairStorageError`. |
| Only the unique admitted factory artifact reaches non-dry-run flashing, and execution consumes immutable admitted bytes. | VERIFIED | Explicit ELF/extra/alias/factory-like selections fail. `AdmittedFlashImage::Factory` owns the validated factory bytes, and `AdmittedExecutionSnapshot` writes a mode-0600 owner-held private snapshot before port/credential work. Race and cleanup regressions prove the child does not reopen the package path. |
| The admitted OTA is a complete supported, bootable ESP32-S3 application. | GAP | Chip/revision, segment bounds, checksum, padding, appended digest, descriptor, and exact EOF are checked, but main-header entry/SPI fields and segment load addresses are ignored. A zero entry address and zero first-segment load address pass after digest recomputation. |
| The versioned package has one internally consistent ELF/application identity. | GAP | The generator normally emits equal `app_elf_sha256` and `firmware_elf.sha256`, but manifest validation and active admission validate them independently. A changed packaged ELF with only its artifact digest updated is accepted. |
| Missing facts stay unavailable and excluded scopes/evidence gates remain closed. | VERIFIED | Typed platform/health unavailability and source guards remain intact. Verification used no hardware, USB, serial, credentials, network discovery, flash, OTA, UART/pins, Phase 35, or archived-lineage operation. |

## Review Warning Adjudication

### WR-01 — Confirmed: ignored entry and segment load addresses admit a non-bootable image

`tools/flash/src/package_admission.rs:64-160` reads main-header bytes 12 through 23 but never parses bytes 2 through 11. Most importantly, it ignores the four-byte `entry_addr` at bytes 4 through 7. Its segment loop reads only `data_len` from segment-header bytes 4 through 7 and never reads the segment `load_addr` in bytes 0 through 3.

The pinned ESP-IDF contract identifies these as executable fields: `esp_image_header_t.entry_addr` and `esp_image_segment_header_t.load_addr` in `.embuild/espressif/esp-idf/v5.5.4/components/bootloader_support/include/esp_app_format.h`. The bootloader uses the header entry address as the final function pointer and validates segment load/mapping ranges in `components/bootloader_support/src/esp_image_format.c` and `bootloader_utility.c`.

Concrete production-path reproduction at source `02f42eb9c34f5b15af0af0bb7272ea0f465227c5`:

1. Built a fresh current clean package with `just package`.
2. In a temporary copy only, zeroed the OTA main-header entry address and first segment load address, made the identical mutations in the factory partition, recomputed the OTA appended SHA-256, and updated the OTA/factory manifest artifact digests.
3. Invoked the real `bitaxe-flash flash --board 205 --port /dev/null --dry-run` path with that manifest and admitted factory image.
4. Admission succeeded and prepared `espflash write-bin` instead of rejecting before the effect boundary.

The checksum, descriptor identity, source marker, factory/OTA equality, appended digest, and manifest digests all remained self-consistent. The accepted application nevertheless has a zero entry point and invalid first segment destination. This directly blocks SYS-02.

### WR-02 — Confirmed: packaged ELF digest is not bound to `app_elf_sha256`

At `tools/flash/src/main.rs:1182-1205`, active admission validates the top-level `app_elf_sha256`, reads and digest-checks the unique `firmware_elf`, then discards those bytes. Only the independently declared top-level value reaches application-descriptor validation. At `tools/xtask/src/package_manifest.rs:249-332`, manifest validation checks the top-level value and every artifact digest independently but never requires the `firmware_elf` artifact digest to equal `app_elf_sha256`.

Concrete production-path reproduction against the same fresh package:

1. Appended different bytes to the temporary packaged `firmware_elf`.
2. Updated only that artifact's manifest SHA-256; left the OTA, factory, descriptor, and top-level `app_elf_sha256` unchanged.
3. Invoked the real dry-run flash admission path.
4. Admission succeeded and prepared the flash command.

The package therefore claims one packaged ELF while the descriptor and top-level identity attest another. The generator's current equal output is useful evidence but cannot replace enforcing the relationship at manifest validation and active admission. This also blocks SYS-02.

## Focused Verification Evidence

All commands ran without hardware or external inputs.

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-flash package_admission` | PASS — 22 tests. Confirms Plan 34-08 chip/revision, bounds, checksum, padding, digest, EOF, descriptor, source, partition, and factory/OTA checks; no entry/load-address regression exists. |
| `cargo test -p bitaxe-flash identity_admission` | PASS — 12 tests. Confirms explicit artifact/basename/alias closure and fail-before-effect ordering; no ELF/top-level digest relationship regression exists. |
| `cargo test -p bitaxe-flash admitted_execution` | PASS — 5 tests. Confirms immutable private execution bytes and success/failure cleanup. |
| `cargo test -p xtask package_manifest` | PASS — 6 tests. Confirms required-kind uniqueness and normal manifest construction; no contradictory ELF hash test exists. |
| `cargo test -p bitaxe-api operator_snapshot_publication` | PASS — 8 tests covering completion order, distinct stage errors, retention/issue failure, poison, reentrancy, and exhaustion. |
| `cargo test -p bitaxe-api retained_pair` | PASS — 8 atomic pair validation/capacity/counter tests. |
| `cargo test -p bitaxe-api platform_identity` | PASS — 6 typed platform truth/unavailability tests. |
| `cargo test -p bitaxe-api runtime_projection` | PASS — 11 shared identity/health and compatibility tests. |
| `cargo test -p bitaxe-core runtime_health` | PASS — 11 vocabulary, transition, age-boundary, freeze, recovery, and watchdog-separation tests. |
| `cargo test -p bitaxe-parity operator_snapshot_evidence` | PASS — 6 coherence, chronology, redaction, and production-publisher tests. |
| `cargo test -p bitaxe-parity phase34_source_guard` | PASS — 5 identity, publication, platform, health, and package guards. |
| Focused Phase 34 Bazel test set | PASS — API, core, exact production retention, production supervisor, flash, and parity targets; seven unique targets after suite expansion. |
| `just package` | PASS — current clean package built at source `02f42eb9c34f5b15af0af0bb7272ea0f465227c5`; normal package has equal top-level and ELF artifact SHA-256. |
| Adversarial real dry-run admission: zero entry/load address | ACCEPTED — concrete WR-01 gap; no hardware/process effect executed. |
| Adversarial real dry-run admission: contradictory packaged ELF | ACCEPTED — concrete WR-02 gap; no hardware/process effect executed. |

## Required Gap Closure

1. Extend active OTA admission to validate the concrete supported main-header policy and every entry/load address needed for a bootable ESP32-S3 application. At minimum, reject invalid entry points, invalid/overflowing segment destination ranges, and invalid mapped-segment alignment/range combinations. Add production-path mutations that recompute all enclosing hashes and still fail before port, credential, command, USB, or hardware work.
2. Require the unique `firmware_elf.sha256` artifact digest to equal `manifest.app_elf_sha256` in both `validate_package_manifest_v3` and active flash admission. Add a production-path regression that changes the packaged ELF and its artifact digest while leaving the descriptor identity unchanged, then proves rejection before later effects.
3. Rerun the exact Rust sequence, focused Cargo/Bazel regressions, repository-wide Bazel tests, canonical build/package/reference gates, code review, and a fresh independent Phase 34 verifier. Phase 35 remains blocked until that verifier passes.

## Human Verification

None. The remaining gap is deterministic at the software admission boundary; hardware cannot repair or supersede it.

## Exact Non-Claims

- This report does not claim Phase 34 passed or SYS-02 is complete.
- It does not claim every admitted application is bootable on ESP32-S3.
- It does not claim `firmware_elf` is bound to the application descriptor's ELF SHA-256.
- It does not qualify any package on hardware or provide Phase 35 evidence.
- It does not promote CFG-12, EVD-10 through EVD-15, parity, mining, OTA/recovery, active health/control, credentials, other boards, or archived-lineage claims.

## Completion Summary

Plan 34-09 successfully closes OBS-06, and SYS-01, SYS-03 through SYS-05, and HLT-01 through HLT-04 remain satisfied. Phase 34 is not complete because current exact-package admission accepts a non-bootable ESP32-S3 application and a contradictory packaged ELF. The final score is **9/10 requirements satisfied**, with SYS-02 as the sole remaining requirement gap.
