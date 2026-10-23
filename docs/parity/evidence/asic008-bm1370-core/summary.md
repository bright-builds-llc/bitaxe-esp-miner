# ASIC-008 Pure BM1370 Protocol Core

asic008_status: implemented
implementation_commit: aa7081d1ab4e3dbd800b00a3cc40704c73097c91
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
firmware_dispatch: deferred
hardware_evidence: none
evidence_classes: unit,golden

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20261023T180521Z-ASIC-008/PLAN.md` | `cbb023b954890901d5ee00bb031fce799555c53fd29b0df20d83e7cda80a1562` |
| Golden fixture | `crates/bitaxe-asic/fixtures/bm1370/protocol-cases.json` | `e96472186a4d9dd066c6b017d9e7573c3f2feacb45c0970e56730c0468d606c6` |
| Rust implementation | Git commit | `aa7081d1ab4e3dbd800b00a3cc40704c73097c91` |
| Pinned upstream reference | Git submodule commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

The fixture records the exact upstream sources, the license posture, and
how it was derived. It holds protocol facts only, and the current Rust
tests consume every field.

## Implemented Surface

The pure `bitaxe-asic` crate now provides typed BM1370 behavior for:

- **Framing:** command and job frames, through the shared BM13xx framing,
  CRC5 and CRC16-FALSE.
- **The init plan, in upstream's order:**
  - the version mask;
  - the `CHIP_ID` read and the split for chip detection;
  - the global, addressing and per-chip register writes;
  - the B9/54/3C block.
- **Frequency:**
  - the 6.25 MHz frequency ramp in both directions;
  - the PLL plan within the BM1370's feedback-divider range (160-239);
  - the nonce space with the BM1370's hash-counting errata.
- **Timing:** the max-baud switch, the job interval and the expected
  hashrate.
- **Work:** the 82-byte payload, the 88-byte frame, and job ids in steps
  of 24 modulo 128.
- **Decoding:** strict 11-byte decoding of nonce results, register reads
  and chip identity:
  - the job lookup;
  - the small core, core, ASIC index and version bits;
  - the register map;
  - closed faults.

The shared PLL search now takes a feedback-divider range. The BM1366 path
keeps its range and is byte-identical, and upstream's own `test_pll.c`
vector passes. The design does not copy upstream function bodies or
activate any device effect.

## Verification

The following passed against implementation commit `aa7081d1`:

- `cargo test -p bitaxe-asic bm1370` (34 passed);
- `cargo test -p bitaxe-asic` (187 passed, 1 existing ignored);
- `bazel test //crates/bitaxe-asic:tests`, within `just test` (363
  passed);
- `just verify-reference` and `just package`;
- the ordered gates:
  - `cargo fmt --all`;
  - `cargo clippy --all-targets --all-features -- -D warnings`;
  - `cargo build --all-targets --all-features`;
  - `cargo test --all-features`;
  - `bun scripts/bright-builds-check.ts all`;
  - `just verify-redaction`;
  - `just parity`;
  - `git diff --check`;
- the firmware stack audits, plus the Gamma 601 production build and
  `gamma601-vout-check` package.

**The fixture was computed independently.** A separate reimplementation
of upstream's algorithms computed its values: the float PLL search, the
CRCs, `BM1370_init`, the ramp and the nonce space. The fixture covers all
99 init frames for one chip at 525 MHz. All 12 frames in `bm1370.c`'s
hand-dump comments match it.

**An independent review agreed.** It recomputed every fixture value with
its own reimplementation, and all matched. It also compiled upstream's
`pll.c` and compared it with the Rust search over 1-1200 MHz in 0.25 MHz
steps, under three divider ranges, with no differences. Its ramp
comparison covered 216,072 start and target pairs with no differences.
Its findings were all low and are applied.

The catalog-dispatch regression confirms that the Gamma 601's BM1370
stays `Deferred`, with `NotHardwareVerified` scope.

## Status Boundary and Non-Claims

This evidence supports `implemented` with `unit,golden` only. It does not
implement or verify:
- firmware dispatch or UART ownership;
- real chip enumeration, or the chip-counting loop;
- initialization timing, frequency or baud effects;
- live work or result traffic;
- voltage, fan or thermal behavior with the ASIC running;
- safe stop or mining;
- any other board or ASIC;
- OTA or recovery, or release readiness.

Promotion to `verified` needs the Gamma 601's phase F:
- a firmware adapter;
- detector admission;
- a hardware smoke and regression;
- safe-stop evidence;
- redaction.

ASIC-008 cannot reuse Ultra 205 evidence.
