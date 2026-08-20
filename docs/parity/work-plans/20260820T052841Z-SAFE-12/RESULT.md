# Parity work result

- Parity row: `SAFE-12`
- Final status: `verified`
- Implementation commit: `308f312f63951daceb2e49ead2a515e979e91453`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed SAFE-10, STR-006, PWR-002, and
  PWR-003 evidence was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/safe12-production-safe-stop/summary.md` joins four already
accepted public projections after independent Rust validation:

- SAFE-10 `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`
- STR-006 `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- PWR-002 `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
- PWR-003 `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`

SAFE-10 proves a detector-admitted 600,746-ms accepted production campaign with
20/20 safety windows, fresh safety, watchdog continuity, final consumed state,
serial finish, confirmed device-local safe stop, and cleanup. STR-006 proves
ordered terminal safe stop after accepted submit. PWR-002 proves all eight
rollback steps were attempted and ASIC disable was commanded. PWR-003 proves
the active-low core-rail disable mechanism. All projections are mode `0644`.
No new projector or protected artifact was used.

Current session tests prove submissions are blocked, work is invalidated, ASIC
interaction stops, pool transport closes, hardware stop runs, and the final
disabled snapshot publishes in order and idempotently. Firmware tests prove the
typed physical shutdown sequence, safe-stop status confirmation, and owner
progress heartbeats. Only successful physical completion produces the hardware
confirmation event; failures remain fail closed.

The following focused and required gates passed on source `308f312f`:

- independent validation of SAFE-10, STR-006, PWR-002, and PWR-003
- `cargo test -p bitaxe-stratum safe_stop`
- `cargo test -p bitaxe-stratum production_session`
- the three focused firmware Bazel targets
- the ordered format, strict Clippy, all-target build, and all-feature test gates
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Conclusion

`SAFE-12` has detector-gated live safety hardware proof that an accepted Ultra
205 production campaign completed the ordered software and physical safe-stop
path, consumed its lease, published disabled state, and cleaned up. This
supports `SAFE-12` at `verified` with
`unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify fault-injected safe stop on hardware, per-step
electrical timing or waveform measurement, power-loss interruption, automatic
thermal or fan fault recovery, arbitrary profiles or pools, other boards or
ASICs, unbounded mining, OTA/recovery, or release readiness. It does not
promote SAFE-13. No credential or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work occurred
during this plan.
