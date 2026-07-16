---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
status: issues_found
depth: standard
files_reviewed: 34
files_reviewed_list:
  - Cargo.lock
  - MODULE.bazel.lock
  - crates/bitaxe-api/BUILD.bazel
  - crates/bitaxe-api/src/build_identity.rs
  - crates/bitaxe-api/src/lib.rs
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
  - firmware/bitaxe/src/main.rs
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
  critical: 3
  warning: 1
  info: 0
  total: 4
generated_by: gsd-code-reviewer
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: "2026-07-16T00:57:32Z"
reviewed_source: 566d20d9d4aa99ebc14c2409ad68936b02e3c5de
reviewed_gap_commits: c3bd02c2..312d1273
---

# Phase 34 Code Review

## Summary

Plans 34-05 through 34-07 repair the three previously reported production defects: default factory admission now binds its app partition to the OTA image, recurring supervisor steps advance checkpoints, and one production publisher orders snapshot identity, retention, and HTTP/WebSocket issuance. Four additional defects remain, however. Two paths can still flash bytes other than the bytes admitted, retained snapshot publication is falsely treated as infallible, and the new ESP application parser accepts incomplete or foreign images as structurally valid.

The review covered all 34 requested files and followed the relevant runtime call chains. Repo-local evidence, hardware, GSD, archived-lineage, and no-direct-UART rules plus the Bright Builds architecture, code-shape, verification, testing, and Rust standards materially informed the review. No standards override applies. No hardware, USB, network, credentials, flash, monitor, source change, or Phase 35 operation was used.

## Critical Issues

### CR-01: Explicit image selection bypasses structural admission of the bytes selected for flash

**Evidence:** `tools/flash/src/main.rs:778-805`, `tools/flash/src/main.rs:1002-1014`, `tools/flash/src/main.rs:1055-1077`, and `tools/flash/src/main.rs:1112-1135`

`validate_identity_admission` always structurally validates the unique required OTA and factory artifacts. After that validation, the explicit `--image` branch accepts any manifest-listed artifact path and performs only that entry's self-declared digest check. `flash_command_for_image` then decides between raw factory flashing and `espflash flash` from the selected file name; it does not require the selected path or bytes to be the structurally admitted factory artifact.

This leaves two direct bypasses. A manifest may retain a valid required factory/OTA pair while selecting a modified `firmware_elf` entry, or it may add an extra artifact whose path ends in `bitaxe-ultra205-factory.bin` and select that path. After updating only the selected artifact's manifest digest, admission still validates the unrelated required factory/OTA pair and the selected unbound image reaches `espflash`. In the second case the file name routes the extra bytes to `write-bin 0x0`.

This violates SYS-02 and Plan 34-05's central truth that the exact image selected for flashing is the structurally admitted identity.

**Required fix:** In non-dry-run admission, either require `--image` to resolve to the exact unique `factory_merged_image` artifact already structurally admitted, or dispatch by a closed artifact kind and structurally bind each supported selected kind to the canonical OTA/ELF identity. Reject unknown kinds, extra factory-named paths, and required-kind/path aliases. Add production-path regressions for a selected tampered ELF and an extra factory-named artifact, with recomputed manifest digests, and prove both fail before port discovery, credential handling, or command execution.

### CR-02: The flash process reopens mutable package paths after admission

**Evidence:** `tools/flash/src/main.rs:647-653`, `tools/flash/src/main.rs:757-775`, `tools/flash/src/main.rs:968-1016`, and `tools/flash/src/main.rs:1137-1147`

Admission reads and hashes the package files, parses separate in-memory factory and OTA buffers, then discards those buffers and returns only a filesystem path. Port resolution and optional NVS preparation occur afterward. `run_flash` finally starts a separate `espflash` process, which reopens the path. Replacing or modifying the selected file between the final check and `environment.execute` therefore causes hardware to receive bytes that were never hashed or structurally admitted.

The new bytewise factory-to-OTA comparison closes a static sibling mismatch but does not bind the checked buffers to the bytes consumed by the flash effect. That time-of-check/time-of-use window remains an exact-package evidence and hardware-admission bypass.

**Required fix:** Read each admitted artifact once and carry an admission-owned snapshot through execution. Prefer flashing through an API that consumes the already-admitted bytes. If the CLI boundary must remain, materialize the admitted bytes to a private owner-only temporary artifact outside the mutable package tree, retain its ownership for the entire child-process lifetime, and execute only that snapshot. Add a deterministic fake-environment race test that mutates the workspace package after admission but before execution and proves the executed bytes remain the admitted bytes or execution fails closed.

### CR-03: Production snapshot retention silently succeeds when the retained store drops records

**Evidence:** `firmware/bitaxe/src/runtime_snapshot.rs:368-375` and `firmware/bitaxe/src/runtime_snapshot.rs:433-437`; supporting call-chain behavior at `firmware/bitaxe/src/log_buffer.rs:11-21` and `firmware/bitaxe/src/log_buffer.rs:61-84`

The publication transition correctly skips issuance when its retention adapter returns an error, but the firmware adapter makes that outcome unreachable. `retain_completed_operator_snapshot` returns `()` and performs two independent `append_runtime_log_line` calls; the closure then unconditionally returns `Ok(())`. The log adapter silently returns on mutex poison. More importantly for the production abort/reboot profile, failure to allocate both the normal and fallback retained buffers creates `RetainedLogBuffer::empty()`, whose appends intentionally drop all bytes while still looking successful to this caller.

An HTTP or WebSocket snapshot can therefore be externally issued with a new session/revision while neither its retained marker nor its matching runtime-health record exists. The generic publisher's fallible-retention tests do not exercise this production adapter. This breaks OBS-06's required correlation between public and retained projections and weakens fail-closed evidence behavior under memory pressure.

**Required fix:** Add one fallible retained-log operation that acquires the log-buffer lock once, rejects unavailable/zero-capacity storage, appends both correlation lines as one bounded transaction, and returns a typed redacted failure. Propagate that result directly from the publisher retention closure so issuance is skipped and the revision is consumed on failure. Add a production-module regression for unavailable storage and for lock failure, asserting that the issue closure is not called and no partial pair is reported as retained.

## Warnings

### WR-01: The structural OTA parser does not validate the complete ESP32-S3 image contract

**Evidence:** `tools/flash/src/package_admission.rs:62-123`

The parser validates the `0xE9` magic, segment count, segment bounds, first-segment application descriptor, expected version/SHA, and an embedded source-commit marker. It does not validate the ESP image header's chip identifier or the checksum, alignment padding, optional appended SHA-256, and trailer after the declared segments. Those bytes are part of the flashable ESP application contract, not arbitrary padding.

A foreign-chip header or corrupted payload/trailer can keep the expected descriptor fields and source marker, be copied identically into the factory partition, receive recomputed manifest artifact digests, and pass `validate_factory_ota_identity`. The selected bytes are then consistent with each other but are not a fully validated ESP32-S3 application and may fail only after hardware has been modified.

**Required fix:** Parse the complete ESP image header and require the ESP32-S3 chip ID and supported revision/header flags. Validate the segment checksum, required 16-byte alignment/padding, appended digest when the header declares it, and reject unconsumed or malformed trailers. Prefer an established format parser when it exposes these checks. Add focused mutations for wrong chip, checksum mismatch, digest mismatch, malformed padding, and unexpected trailing data.

## Verified Strengths

- Default factory selection now validates exactly one required artifact set, parses bounded descriptor and partition fields, and compares the factory app prefix byte-for-byte with the admitted OTA image.
- The supervisor fix separates optional duplicate-log suppression from mandatory recurring checkpoint mutation; the production-module test covers repeated monotonic steps.
- The production `OperatorSnapshotPublisher` collects outside its ordering mutex and owns identity allocation, projection, retention, and injected issuance without revision reset or reuse.
- HTTP system-info, live cadence, and live-connect send calls now execute inside the shared publication authority, while stale-client cleanup occurs after unlock.
- Existing build identity, platform availability, runtime-health vocabulary, wire shapes, package schema, and source guards remain strongly typed and additive.

## Conclusion

**Status: issues found.** The original SYS-02, HLT-02/HLT-04, and OBS-06 defects were addressed in their default success paths, but CR-01 and CR-02 still allow hardware to receive bytes other than the bytes proven by admission, and CR-03 allows public snapshot issuance without retained correlation. WR-01 also makes the claimed structural ESP application check incomplete. Phase 34 should remain unverified and Phase 35 blocked until these defects are fixed and covered by production-path regressions.
