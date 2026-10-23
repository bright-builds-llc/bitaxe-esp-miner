# Parity work log

## 2026-10-23 | pure BM1370 protocol implementation

- **Source commit:** `aa7081d1ab4e3dbd800b00a3cc40704c73097c91`.
- **Actions:**
  - implemented the typed BM1370 framing, init plan, ramp, nonce space,
    work encoding and result decoding;
  - added closed faults, a provenance-bound fixture and focused
    behavioral tests;
  - generalized the shared PLL search to take a divider range;
  - kept firmware dispatch deferred.
- **Reference comparison:** checked the pinned `bm1370.c`, `bm1370.h`,
  `pll.c`, `crc.c`, `frequency_transition_bmXX.c`, `asic_common.c`,
  `asic.c`, `power_management_task.c`, `stratum/include/utils.h` and
  `device_config.h` for:
  - framing, CRC and byte order;
  - init order, job ids and the result layout;
  - registers, the PLL range and the errata;
  - the profile constants.
- **Independent review:**
  - recomputed every fixture value;
  - compiled upstream `pll.c` and swept it against the Rust search;
  - compared the ramp over 216,072 pairs.

  It found no correctness defects. Its low findings were applied before
  the source commit: test coverage, fixture provenance and the plan's
  wording.
- **Verification:**
  - 34 focused BM1370 tests passed, and the ASIC crate passed 187 with 1
    existing ignored;
  - `just test` passed 363 Bazel tests;
  - the ordered Rust, Bright Builds, redaction and parity gates passed;
  - `just verify-reference` and `just package` passed.
- **Evidence:** `docs/parity/evidence/asic008-bm1370-core/summary.md`,
  bound to:
  - plan digest
    `cbb023b954890901d5ee00bb031fce799555c53fd29b0df20d83e7cda80a1562`;
  - fixture digest
    `e96472186a4d9dd066c6b017d9e7573c3f2feacb45c0970e56730c0468d606c6`.
- **Outcome:** the pure software surface supports advancing ASIC-008 from
  `deferred` to `implemented`, with `unit,golden` evidence only.
- **Next safe action:** phase F of `task-gamma-601-bring-up`. Firmware
  dispatch and hardware verification wait for these phase F items:
  - the chip-generic seams listed in TASKS.md;
  - a complete effect contract;
  - detector admission;
  - redacted hardware evidence on the Gamma 601.
