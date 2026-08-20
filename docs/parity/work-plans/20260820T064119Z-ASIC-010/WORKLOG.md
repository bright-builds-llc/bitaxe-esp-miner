# Parity work log

## 2026-08-20T07:07:53Z | pure BM1397 protocol implementation

- Source commit: `3909a304213f81babf9d3fed38800bd2b515c0a5`.
- Actions: implemented typed BM1397 framing, initialization/frequency plans,
  deterministic one/four-midstate work, stateful result decoding, closed
  faults, provenance-bound fixtures, and focused behavioral tests while
  retaining deferred firmware dispatch.
- Reference comparison: checked the pinned `bm1397.c`, `bm1397.h`, `pll.c`,
  shared frequency-transition and version-roll sources, `asic_common.c`, and
  `device_config.h` for framing, CRC, init order, PLL bounds/encoding, delays,
  byte order, job/midstate IDs, registers, duplicate filtering, and profile
  constants.
- Verification: the first focused run exposed an odd-length hand-authored
  golden string; correcting the fixture made all 16 focused tests pass. The
  review then found and closed high-bit job-ID wraparound. The ASIC crate passed
  140 tests with 1 existing ignored helper; deferred dispatch and Bazel tests,
  reference/package checks, ordered Rust gates, and Bright Builds passed.
- Evidence:
  `docs/parity/evidence/asic010-bm1397-core/summary.md`, bound to plan digest
  `4882bc9e7e96a47b3d8b0777e659337970298013945ebeca9caeb4f00f558ac8`
  and fixture digest
  `77a8096b6c16d39435b4ae95027971f9c102a1b306fcf9af34e7c741853d464d`.
- Outcome: the pure software surface supports advancing ASIC-010 from
  `not-started` to `implemented` with `unit,golden` evidence only.
- Blocker or next safe action: firmware dispatch and hardware verification stay
  unavailable until a supported BM1397 board, adapter, complete effect contract,
  detector admission, and redacted hardware regression exist. Leave the task
  active and unarchived.
