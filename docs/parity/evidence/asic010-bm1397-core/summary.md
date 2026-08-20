# ASIC-010 Pure BM1397 Protocol Core

asic010_status: implemented
implementation_commit: 3909a304213f81babf9d3fed38800bd2b515c0a5
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
firmware_dispatch: deferred
hardware_evidence: none
evidence_classes: unit,golden

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T064119Z-ASIC-010/PLAN.md` | `4882bc9e7e96a47b3d8b0777e659337970298013945ebeca9caeb4f00f558ac8` |
| Golden fixture | `crates/bitaxe-asic/fixtures/bm1397/protocol-cases.json` | `77a8096b6c16d39435b4ae95027971f9c102a1b306fcf9af34e7c741853d464d` |
| Rust implementation | Git commit | `3909a304213f81babf9d3fed38800bd2b515c0a5` |
| Pinned upstream reference | Git submodule commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

The fixture names all six pinned BM1397, PLL, frequency-transition, mining,
and board-profile sources it derives protocol facts from. It contains no
device, network, credential, or hardware-observation data.

## Implemented Surface

The pure `bitaxe-asic` crate now provides typed BM1397 behavior for:

- command and 146-byte job-payload framing with exact CRC5/CRC16 boundaries;
- chip identity, read/inactive/address commands, init writes, difficulty,
  default and maximum baud, and the Max-family profile facts;
- model-specific PLL parameters, raw post-divider encoding, duplicated
  prefrequency/frequency writes, 10 ms internal delays, and the shared 6.25 MHz
  transition with 100 ms step delays;
- deterministic one- and four-midstate 152-byte work frames plus modulo-128
  four-step job rotation;
- strict nine-byte job/register decoding, valid-job and address admission,
  midstate-index version rolling, nonce byte order, ASIC/core/small-core
  identity, hashrate/error register classification, and previous-nonce
  duplicate suppression; and
- explicit fail-closed rejection for malformed CRC/preamble/length/register,
  missing or out-of-range job IDs, zero address intervals, and zero chip counts.

The upstream BM1397 version-mask setter is modeled truthfully as a no-frame
placeholder; version rolling remains represented through the four precomputed
midstates and result correlation. Unused one-midstate slots are zeroed instead
of reproducing upstream uninitialized memory. The shared PLL search was
deepened to accept explicit family bounds without changing BM1366 behavior.

## Verification

The following passed against implementation commit `3909a304`:

- `cargo test -p bitaxe-asic bm1397` (16 passed)
- `cargo test -p bitaxe-asic` (140 passed, 1 ignored)
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
buildable and unaffected. The catalog-dispatch regression confirms BM1397
remains `Deferred` with `NotHardwareVerified` scope.

## Status Boundary and Non-Claims

This evidence supports `implemented` with `unit,golden` only. It does not
implement or verify firmware dispatch, UART ownership, real chip enumeration,
initialization timing, frequency or baud effects, live work/results, analog
voltage behavior, fan/thermal behavior, safe stop, any BM1397/Max board,
BM1368/BM1370 breadth, mining, OTA/recovery, or release readiness. Promotion
to `verified` requires a separately planned supported BM1397 board and
detector-gated, redacted hardware evidence.
