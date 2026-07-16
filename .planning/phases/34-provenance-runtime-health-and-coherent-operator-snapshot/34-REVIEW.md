---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "10"
status: issues_found
depth: deep
files_reviewed: 7
files_reviewed_list:
  - tools/flash/src/esp32s3_image.rs
  - tools/flash/src/package_admission.rs
  - tools/flash/src/main.rs
  - tools/flash/BUILD.bazel
  - tools/xtask/src/package_manifest.rs
  - tools/xtask/src/main.rs
  - tools/parity/src/phase34_source_guard.rs
diff_base: 72a66548
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
blocking_correctness_issue: true
generated_by: gsd-code-reviewer
generated_at: "2026-07-16"
reviewed_source: 77430a1a1356bccbb15a7c3fb86805cfe2f2199f
---

# Phase 34 Plan 10 Code Review

## Summary

The two prior SYS-02 reproductions now reach closed production checks. The real flash path hashes the selected ELF bytes and compares that digest with `app_elf_sha256` before reading OTA or factory artifacts, and both OTA bytes and the factory-embedded application are routed through the same typed ESP32-S3 validator before port, credential, snapshot, command, USB, or hardware work.

Two executable-layout invariants remain incomplete. The parser authenticates descriptor bytes at the start of segment 0 without requiring the segment to be DROM, and it validates every destination range independently without rejecting overlaps between segments or overlaps through the ESP32-S3 D/IRAM aliases. Either omission permits a structurally sealed package that does not represent the canonical ESP-IDF application layout. Both warnings block SYS-02.

The repo-local exact-package, immutable-admission, software-only, no-hardware, and frontmatter guidance materially informed this review. `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` informed the boundary parsing, typed-invariant, and regression assessment. No active standards override applies.

## Warnings

### WR-01: Segment 0 is not required to be the canonical DROM application-descriptor segment

**Evidence:** `tools/flash/src/esp32s3_image.rs:183-196` and `tools/flash/src/esp32s3_image.rs:322-340`

`validate_load_address` classifies segment 0 like any other admitted family. `validate_descriptor` then takes the first payload and authenticates its descriptor bytes, but never requires `segments[0].maybe_memory_family == Some(MemoryFamily::Drom)`. The pinned `esp_app_desc.h` defines `esp_app_desc_t` as residing in DROM, `esp_app_desc.c` places it in `.rodata_desc`, and the ESP32-S3 linker script places `.flash.appdesc` at the beginning of the flash-rodata/DROM segment. The ELF therefore continues to reference the descriptor at its linked DROM address even if the image header redirects that payload into DRAM or another family.

Regression shape confirmed through the current parsed dry-run CLI: begin with the freshly generated canonical package; change only segment 0's load address from DROM to an admitted DRAM address; recompute the appended image digest, replace the factory-embedded OTA bytes, and update the OTA/factory artifact digests. Admission reaches flash-command preparation instead of rejecting the noncanonical descriptor placement. A smaller unit regression can use a nonoverlapping DRAM segment 0 containing the descriptor plus a valid executable IRAM segment, then assert a stable descriptor-segment-family rejection.

**Required fix:** Make the descriptor-bearing first segment a typed invariant: it must be non-empty, mapped DROM, contain the complete descriptor at payload offset zero, and satisfy the existing mapped congruence rule. Add direct parser, full OTA/factory admission, and parsed pre-effect CLI regressions for a descriptor-bearing first segment redirected to each other otherwise-admitted family.

### WR-02: Independently valid segments may overlap directly or through D/IRAM aliases

**Evidence:** `tools/flash/src/esp32s3_image.rs:183-195` and `tools/flash/src/esp32s3_image.rs:257-271`

Each non-empty segment is checked for a nonoverflowing end and containment in one admitted envelope, then appended to `segments`. Validation proceeds directly to entry, descriptor, and trailer checks; there is no pairwise destination-range validation. This accepts two ranges that overlap numerically. It also accepts a DRAM range and an IRAM range that refer to the same ESP32-S3 D/IRAM physical SRAM through `SOC_I_D_OFFSET`, even though loading the later segment overwrites bytes from the earlier one. The canonical linker/`elf2image` output does not emit either layout, and the existing conservative ceilings only exclude bootloader-reserved aliases; they do not prevent application segments from colliding with each other.

The current tests cannot expose this because their synthetic positive fixtures use one small disjoint range per family and assert only per-segment containment. A focused regression should construct two individually admitted numeric-overlap ranges, then a separate DRAM/IRAM alias-overlap pair using the pinned ESP32-S3 alias mapping, and require rejection before descriptor/trailer success. Include adjacent half-open ranges as a positive boundary case.

**Required fix:** After parsing all non-empty segments, reject pairwise numeric intersections and normalize the D/IRAM portions into one physical-address representation before checking alias intersections. Keep zero-length segments range-free. Store the resulting nonoverlapping segment set in the validated type so later entry/descriptor checks cannot observe an unchecked layout.

## Prior SYS-02 Reproduction Adjudication

- The prior entry/load-address reproduction is closed for the classes now encoded: zero, unaligned, out-of-envelope, overflowing, cross-boundary, reserved-envelope, and mapped-congruence mutations fail in `esp32s3_image::validate`, which is called for both OTA and factory application bytes.
- The prior contradictory-ELF reproduction is closed. `tools/xtask/src/package_manifest.rs:143-224` hashes the real ELF while constructing the manifest and validates the relationship; `tools/xtask/src/package_manifest.rs:304-320` enforces unique-kind validation followed by digest equality for supplied manifests; `tools/flash/src/main.rs:1186-1195` hashes the selected ELF bytes and rejects disagreement before any later application artifact is read.
- The immutable factory snapshot remains closed: active admission returns owned factory bytes, and later execution consumes the admission-owned snapshot rather than reopening the package artifact.

## Verified Strengths

- Header parsing covers the exact canonical DIO, 80 MHz/16 MiB, WP, drive, chip/revision, reserved-byte, and appended-hash fields.
- Segment length, offset, load-end, trailer, and digest arithmetic uses checked operations and bounded slices. Loader-valid zero-length semantics are preserved while mapped zero-length segments retain congruence checks.
- Entry validation requires four-byte alignment and containment in a non-empty admitted IRAM or IROM segment.
- Descriptor version, ELF SHA, MMU log2 value, source marker, segment checksum, zero padding, appended SHA-256, and exact end-of-file are checked.
- Manifest construction, manifest validation, and active admission independently enforce the unique firmware ELF/application digest relationship using actual ELF bytes at producer and consumer boundaries.
- Active rejection remains ordered before OTA/factory reads for ELF mismatch and before port discovery, credentials, command execution, USB, or hardware for executable-image mismatch.

## Verification

All software-only checks executed for this review passed:

- `cargo test -p bitaxe-flash esp32s3_image` — 12 passed.
- `cargo test -p bitaxe-flash package_admission` — 26 passed.
- `cargo test -p bitaxe-flash executable_admission` — 2 passed.
- `cargo test -p bitaxe-flash firmware_elf_app_sha` — 2 passed.
- `cargo test -p xtask package_manifest` — 8 passed.
- `cargo test -p bitaxe-parity phase34_source_guard` — 5 passed.
- `just package` completed successfully for source `77430a1a1356bccbb15a7c3fb86805cfe2f2199f`.
- The clean current package passed the parsed software-only dry-run admission path.
- `git diff --check` passed before the review artifact was written.

No hardware, USB, serial, credentials, network discovery, flashing, OTA execution, direct UART/pins, Phase 35, or archived-lineage operation was used.

## Conclusion

**Status: issues found.** The original zero-entry/load-address and contradictory-ELF gaps are materially repaired, and the ELF identity ordering is sound. WR-01 and WR-02 leave the claimed canonical executable layout incomplete, so SYS-02 and Phase 34 should remain unverified until both invariants have production-path regression coverage and a fresh independent verifier passes.
