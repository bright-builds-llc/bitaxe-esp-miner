# Parity work result

- Parity row: `ASIC-12`
- Final status: `verified`
- Implementation commit: `30e0340695e1f307dfcdc7aa6949da07beb616f5`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed ASIC-002 through ASIC-005 evidence
  was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/asic12-fail-closed-redaction/summary.md` joins four already
accepted public projections after independent Rust validation:

- ASIC-002 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC-004 `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`
- ASIC-005 `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves mining-ready initialization,
retained production UART, live production work TX and result RX, a qualified
correlated result, accepted response, fresh safety, confirmed safe stop,
cleanup, trusted identity, and passed redaction. Each projection is mode
`0644`. No new projector or protected artifact was used.

Current source moves the exact public status-line decision into the pure ASIC
core and leaves the firmware adapter as a thin logger. Host tests cover all
three successful states and every one of the eleven typed fail-closed reasons.
Every failure line includes the closed reason plus disabled mining and work-
submission state. Existing production-work and session tests prove raw work,
result, target, submit, registry, and transport context is redacted and typed
ASIC failure subcategories survive terminal safe stop.

The following focused and required gates passed on implementation source
`30e03406`:

- independent validation of ASIC-002 through ASIC-005
- `cargo test -p bitaxe-asic production`
- `cargo test -p bitaxe-stratum production_work`
- `cargo test -p bitaxe-stratum production_session`
- the ordered format, strict Clippy, all-target build, and all-feature test gates
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Conclusion

`ASIC-12` has a closed current-source contract for public fail-closed blocker
rendering and redaction plus an accepted live Ultra 205 BM1366 production chain.
This supports `ASIC-12` at `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not claim hardware fault injection for every blocker,
arbitrary diagnostic builds, nonzero version-mask or multi-midstate breadth,
arbitrary-load serial behavior, rejected-share hardware, frequency transitions,
voltage/fan/thermal behavior, other ASICs or boards, arbitrary pools or
profiles, unbounded mining, OTA/recovery, or release readiness. It does not
promote STR-08, STR-09, SAFE-12, or SAFE-13. No credential or protected-attempt
access, detector, device/USB/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work occurred during this plan.
