# ASIC-009 Pure BM1368 Protocol Core

asic009_status: implemented
implementation_commit: 1dc17d9b9a8e12319b5ca01db297d6800bd38d46
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
firmware_dispatch: deferred
hardware_evidence: none
evidence_classes: unit,golden

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T060848Z-ASIC-009/PLAN.md` | `d46fecee642dce61bbd28e5d43f3a569f8d6188a48e124cfbcc13824488ebc81` |
| Golden fixture | `crates/bitaxe-asic/fixtures/bm1368/protocol-cases.json` | `e46580f69f9c7886a4be4c1258f67a8a0948e6198e80c79ebb069c8539ba7cd8` |
| Rust implementation | Git commit | `1dc17d9b9a8e12319b5ca01db297d6800bd38d46` |
| Pinned upstream reference | Git submodule commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

The fixture records the exact upstream BM1368 source and board-profile paths.
It contains protocol facts only and is consumed by current Rust tests.

## Implemented Surface

The pure `bitaxe-asic` crate now provides typed BM1368 behavior for:

- command and job framing with the family CRC5 and CRC16 boundaries;
- chip identity, version mask, read/inactive/address, global and per-chip
  register writes, difficulty, baud, frequency-ramp, nonce-space, and delay
  planning;
- the 82-byte work payload, 88-byte frame, and modulo-128 24-step job IDs;
- strict 11-byte result admission, job/register classification, valid-job,
  core, address-interval, and register checks; and
- nonce byte order, ASIC/core/small-core identity, version-bit decoding, and
  the reference register map.

The design reuses the existing pure BM1366-family CRC, PLL, difficulty-mask,
and hash-counting calculations where upstream uses the same algorithms. It
does not copy upstream function bodies or activate any device effects.

## Verification

The following passed against implementation commit `1dc17d9b`:

- `cargo test -p bitaxe-asic bm1368` (12 passed)
- `cargo test -p bitaxe-asic` (124 passed, 1 ignored)
- `cargo test -p bitaxe-asic dispatch_non_v1_asic_families_are_deferred_without_hardware_scope`
  (1 passed)
- `bazel test //crates/bitaxe-asic:tests`
- `just verify-reference`
- `just package`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `git diff --check`

The package check confirms the existing Ultra 205 artifact path remains
buildable and unaffected. The catalog-dispatch regression confirms every
BM1368 board remains `Deferred` with `NotHardwareVerified` scope.

## Status Boundary and Non-Claims

This evidence supports `implemented` with `unit,golden` only. It does not
implement or verify firmware dispatch, UART ownership, real chip enumeration,
initialization timing, frequency or baud effects, live work/result traffic,
voltage/fan/thermal behavior, safe stop, any BM1368 board, BM1397/BM1370,
mining, OTA/recovery, or release readiness. Promotion to `verified` requires a
separately planned supported BM1368 board and detector-gated, redacted hardware
evidence.
