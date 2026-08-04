# Parity work plan

- Run ID: `20260804T200000Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `not-started`
- Source commit: `f10e50514abc95f1d98806b9788ed591d484c63a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The deterministic selector reported no open plan after `UI-004` closed.
Implemented rows require their own claim-specific live evidence rather than a
status-only pass. The in-progress display and statistics-consumer rows require
live producer or physical-interface evidence. `ASIC-009` and `ASIC-010` target
later non-Ultra boards, while `BAP-001` requires an unavailable external
accessory and separately authorized electrical attachment. `STAT-001` is the
first actionable row because its counter conversion, rolling windows, reset
semantics, production ownership, and API projection can be implemented and
verified without starting a mining campaign or changing hardware controls.

## Scope and non-scope

Add a pure bounded hashrate monitor matching the pinned reference behavior for
BM1366 instantaneous and wrapping counter measurements, four hash domains,
error percentage, and hierarchical 1-minute, 10-minute, and 1-hour averages.
Reject out-of-range observations and ignore sub-second counter intervals. Reset
counter history when the ASIC session stops so a later counter reset cannot
produce a wraparound spike.

Keep the existing production-session owner authoritative. Preserve parsed
register values in its typed worker events, pass them to the pure monitor, and
request the existing passive register-read burst at the one-second cadence only
while the session is already active. Publish the resulting current, rolling,
and error values through the existing mining runtime snapshot and AxeOS system
wire fields. No second UART owner or independent mining lifecycle is allowed.

This work does not authorize or initiate a mining campaign, pool connection,
device flash, credential use, hardware attempt, frequency/voltage/fan/power
effect, OTA, recovery, direct UART, or pin interaction. Software verification
uses synthetic counter/register observations only. Live counter accuracy and
hardware behavior remain below verified.

## Implementation

- [ ] Add the pure bounded monitor, typed observations, exact conversion and
      reset semantics, hierarchical rolling averages, and focused regressions.
- [ ] Extend runtime hashrate state and system-info projection to carry current,
      1-minute, 10-minute, 1-hour, and error values without placeholder fields.
- [ ] Route parsed production register reads through the sole ASIC worker,
      schedule passive reads only for an already-active session, and publish the
      monitor snapshot through the existing owner.
- [ ] Run focused tests and every mandatory repository gate, then create a
      commit-bound result before transitioning only `STAT-001` to `implemented`.

## Verification and promotion

Focused tests must cover instantaneous scaling, wrapping counters, sub-second
suppression, invalid ASIC/domain admission, stop/resume reset, error percentage,
and 1-minute/10-minute/1-hour rolling behavior. Firmware ownership tests must
prove one production UART owner, value-carrying register events, active-only
read scheduling, reset on stop, and runtime publication. API tests must prove
all four hashrate fields and error percentage come from the same typed snapshot.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`,
`just parity`, `just parity-progress`, redaction, reference cleanliness, and
diff checks. Transition only `STAT-001` from `not-started` to `implemented` with
`unit,workflow` evidence when those gates pass. Do not claim live register
traffic, mining hashrate accuracy, API/UI hardware parity, or verified status.
