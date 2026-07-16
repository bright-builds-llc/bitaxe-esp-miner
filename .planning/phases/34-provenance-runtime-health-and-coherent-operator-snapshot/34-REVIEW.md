---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
status: issues_found
depth: standard
files_reviewed: 38
files_reviewed_list:
  - Cargo.lock
  - MODULE.bazel.lock
  - crates/bitaxe-api/BUILD.bazel
  - crates/bitaxe-api/src/build_identity.rs
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/src/logs.rs
  - crates/bitaxe-api/src/operator_snapshot.rs
  - crates/bitaxe-api/src/operator_snapshot_publication.rs
  - crates/bitaxe-api/src/platform_identity.rs
  - crates/bitaxe-api/src/runtime_projection.rs
  - crates/bitaxe-api/src/snapshot.rs
  - crates/bitaxe-api/src/wire.rs
  - crates/bitaxe-core/src/runtime_health.rs
  - firmware/bitaxe/BUILD.bazel
  - firmware/bitaxe/build.rs
  - firmware/bitaxe/src/boot_evidence.rs
  - firmware/bitaxe/src/http_api.rs
  - firmware/bitaxe/src/log_buffer.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/operator_snapshot_retention.rs
  - firmware/bitaxe/src/operator_snapshot_retention_production_tests.rs
  - firmware/bitaxe/src/platform_identity.rs
  - firmware/bitaxe/src/runtime_health_adapter.rs
  - firmware/bitaxe/src/runtime_snapshot.rs
  - firmware/bitaxe/src/safety_adapter/watchdog.rs
  - scripts/build-identity-pathspecs.txt
  - scripts/build-identity-status.sh
  - scripts/build_identity.bzl
  - scripts/package-firmware.sh
  - tools/flash/BUILD.bazel
  - tools/flash/Cargo.toml
  - tools/flash/src/main.rs
  - tools/flash/src/package_admission.rs
  - tools/parity/src/operator_evidence.rs
  - tools/parity/src/operator_snapshot_evidence.rs
  - tools/parity/src/phase33_source_guard.rs
  - tools/parity/src/phase34_source_guard.rs
  - tools/xtask/src/package_manifest.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
blocking_correctness_issue: true
generated_by: gsd-code-reviewer
generated_at: "2026-07-16"
reviewed_source: 5c6d0a8eb9203a787c0fd9d9dc825a4457fa5519
---

# Phase 34 Code Review

## Summary

The explicit-image bypass, mutable-package time-of-check/time-of-use window, and false-success retained-correlation adapter reported by the preceding review are fixed. The admitted factory bytes are now copied into an admission-owned, owner-only temporary snapshot and held through child-process completion. Retained snapshot correlation now performs one preflighted pair append under one production mutex acquisition, and retention and issuance failures preserve distinct typed sources.

Two exact-package admission gaps remain. Both block SYS-02 even though the new tests and the normal package path pass. No hardware, USB, network, credential, flash, monitor, archived-lineage, or Phase 35 operation was used during this review.

## Warnings

### WR-01: The ESP32-S3 image-envelope validator still ignores executable header and segment-address fields

**Evidence:** `tools/flash/src/package_admission.rs:64-160`

`validate_ota_identity` validates the magic, segment count and lengths, descriptor identity, checksum, padding, optional appended SHA-256, and exact end of file. However, `validate_esp32_s3_header` starts at header byte 12. It never checks bytes 2 through 11: SPI mode, SPI speed/size, entry address, write-protect pin, or SPI drive settings. The segment loop likewise reads only `data_len` from each segment header and ignores its `load_addr` field.

Those fields determine whether the admitted image is a bootable ESP32-S3 application and where its segments execute. An image can therefore preserve the expected descriptor and source marker, replace the entry point or segment load addresses with unsupported values, recompute the ESP checksum/appended digest and manifest factory/OTA digests, and pass admission. The pinned ESP-IDF image format defines these fields and validates target memory ranges; they are not opaque payload bytes.

**Required fix:** Validate the concrete main-header policy emitted by the pinned ESP32-S3 package flow and every entry/segment address against supported ESP32-S3 executable ranges, or delegate the complete check to a trusted parser over the already-admitted bytes. Add regressions that mutate each currently ignored field class, recompute all enclosing checksums and digests, and prove rejection before port discovery, credential handling, or process execution.

### WR-02: Admission does not bind `app_elf_sha256` to the packaged `firmware_elf` artifact

**Evidence:** `tools/flash/src/main.rs:1182-1205` and `tools/xtask/src/package_manifest.rs:249-318`

Active admission independently validates the top-level `manifest.app_elf_sha256` format, reads and digest-checks the `firmware_elf` artifact, then discards those bytes. It passes only the top-level value to the OTA application-descriptor check. Neither active admission nor the manifest validator requires the `firmware_elf` artifact's SHA-256 to equal `app_elf_sha256`.

Consequently, a manifest can point `firmware_elf` at different bytes, update only that artifact's self-declared digest, leave the canonical descriptor/factory/OTA identity unchanged, and pass admission with a contradictory release package. The normal package generator currently produces equal values, but the admission boundary must validate the relationship rather than trust that producer behavior.

**Required fix:** Require the unique `firmware_elf` artifact digest to equal `app_elf_sha256` in both manifest validation and active flash admission. Add a production-path regression that replaces the packaged ELF, recomputes `firmware_elf.sha256`, and proves admission fails before port discovery, credential handling, or execution.

## Gap-Fix Adjudication

- The former explicit-image bypass is resolved: explicit selection must lexically equal the admitted factory artifact, and execution dispatch is closed over `AdmittedFlashImage`.
- The former mutable-package race is resolved: admitted factory bytes are copied to an owned mode-0600 temporary snapshot before later effects, and its owner remains live across child execution.
- The former retained-correlation false success is resolved: `RetainedPair` preflights the whole pair, the production adapter acquires the singleton mutex once, storage failures are typed, and issuance is skipped after retention failure while the revision remains consumed.
- The former incomplete-envelope warning is only partially resolved. Chip/revision policy and the complete trailer are now checked, but WR-01 identifies the remaining executable-header and segment-address omissions.

## Verified Strengths

- The immutable admission snapshot binds the exact factory bytes checked in memory to the path consumed by the CLI boundary.
- Retained marker and runtime-health records append atomically with respect to validation, capacity, counter state, and the production lock.
- `OperatorSnapshotPublishError<RetentionError, IssueError>` preserves stage-specific error sources and lock health without collapsing retention into issuance.
- Production-module tests cover unavailable storage, insufficient capacity, mutex poison, skipped issuance, revision consumption, and redaction-safe errors.
- HTTP and WebSocket snapshot paths continue to issue only through the shared publication authority.

## Verification

All executed checks passed:

- `cargo test -p bitaxe-flash package_admission` (22 passed)
- `cargo test -p bitaxe-flash identity_admission` (12 passed)
- `cargo test -p bitaxe-flash admitted_execution` (5 passed)
- `cargo test -p bitaxe-api operator_snapshot_publication` (8 passed)
- `cargo test -p bitaxe-api retained_pair` (8 passed)
- `cargo test -p bitaxe-parity operator_snapshot_evidence` (6 passed)
- `cargo test -p bitaxe-parity phase34_source_guard` (5 passed)
- `bazel test //firmware/bitaxe:retained_pair_production_tests //firmware/bitaxe:operator_snapshot_publication_tests //tools/flash:tests //tools/parity:tests //crates/bitaxe-api:tests //crates/bitaxe-core:tests`
- `just build`
- `just package`
- `just verify-reference`
- `git diff --check`

The passing suite does not exercise the ignored image-header/address fields or the missing relationship between the two ELF digest declarations.

## Conclusion

**Status: issues found.** No critical finding remains from the requested gap repairs, and the production retained-correlation fix is sound. WR-01 and WR-02 are blocking correctness gaps in exact-package admission, so Phase 34 should remain unverified and Phase 35 blocked until both relationships are enforced and covered by pre-effect production-path regressions.
