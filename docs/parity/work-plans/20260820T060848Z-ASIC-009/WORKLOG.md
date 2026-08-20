# Parity work log

## 2026-08-20T06:31:18Z | pure BM1368 protocol implementation

- Source commit: `1dc17d9b9a8e12319b5ca01db297d6800bd38d46`.
- Actions: implemented typed BM1368 framing, initialization, work encoding,
  result decoding, closed faults, provenance-bound fixtures, and focused
  behavioral tests while retaining deferred firmware dispatch.
- Reference comparison: checked the exact pinned `bm1368.c`, `bm1368.h`,
  `asic_common.c`, PLL/frequency-transition sources, and `device_config.h`
  facts for framing, CRC, byte order, job IDs, registers, frequency/voltage
  defaults, and profile constants.
- Verification: 12 focused BM1368 tests passed; the ASIC crate passed 124 tests
  with 1 existing ignored test; the deferred-dispatch regression and Bazel ASIC
  target passed; reference and package checks passed; and the ordered Rust and
  Bright Builds gates passed.
- Evidence:
  `docs/parity/evidence/asic009-bm1368-core/summary.md`, bound to plan digest
  `d46fecee642dce61bbd28e5d43f3a569f8d6188a48e124cfbcc13824488ebc81`
  and fixture digest
  `e46580f69f9c7886a4be4c1258f67a8a0948e6198e80c79ebb069c8539ba7cd8`.
- Outcome: the pure software surface supports advancing ASIC-009 from
  `not-started` to `implemented` with `unit,golden` evidence only.
- Blocker or next safe action: firmware dispatch and hardware verification stay
  unavailable until a supported BM1368 board, adapter, complete effect contract,
  detector admission, and redacted hardware regression evidence exist. Leave
  the task active and unarchived.
