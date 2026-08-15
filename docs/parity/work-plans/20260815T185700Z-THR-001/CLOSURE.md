# THR-001 marker-observation software closure

- Run ID: `20260815T185700Z-THR-001`
- Status: `complete_software_only`
- Implementation commit: `6f637e87557084aa9c7d34861d2c16f1e7a083b1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Parity status: `implemented`
- Hardware effects: `none`

## Root cause and correction

Attempt-005 proved the device reached both the fault and recovery states, but
the host validator required invented byte-zero marker lines. Production emits
those payloads inside the canonical ESP-IDF INFO envelope. The baseline marker
could also complete before the post-flash reader attached, while the existing
retained replay allowlist and lifecycle were limited to the accepted-state
diagnostic.

The host now extracts only a numeric-uptime INFO envelope with the exact
`bitaxe_firmware` tag and accepts evidence only when the extracted marker
payloads contain one complete contiguous baseline/fault/recovery triplet. The
admitted thermal-stimulus package requests the existing bounded retained-log
replay, whose exact first-token allowlist now includes complete redaction-safe
thermal-stimulus state records. Ordinary packages do not request this replay.

## Regression evidence

The canonical automation target first failed with `evidence_invalid` for a
complete production-prefixed fixture, a late-attachment fixture, and the real
child-process seam. After the correction it passes those cases and proves that
bare lines, malformed timestamps, wrong levels, wrong tags, missing states,
wrong ordering, and child timeout cannot publish evidence. A late observer may
see an incomplete prefix before a replayed complete triplet; repeated retained
markers do not weaken the required ordered witness. Ordinary exact-package
restoration still proves the one-shot stimulus is not replayed.

## Verification and non-claims

The ESP32-S3 firmware build, ordered Cargo gates, Bright Builds checks, all 45
Bazel tests, parity validation, parity progress, redaction, pinned-reference
cleanliness, and diff checks passed. The direct host-target firmware test was
inapplicable because `esp-idf-sys` correctly rejects the macOS host target; the
canonical cross-target firmware build and Bazel firmware tests passed instead.

No detector, package admission, USB, serial, HTTP, device, NVS, sensor, reset,
control, mining, OTA, erase, or other hardware effect ran under this plan. It
does not reinterpret attempt-005, promote THR-001, or authorize attempt-006.
A distinct immutable hardware plan is required before the corrected production
witness may be exercised on the Ultra 205.
