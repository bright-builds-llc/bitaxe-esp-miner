# Parity work plan

- Run ID: `20260804T211000Z-STAT-002`
- Parity row: `STAT-002`
- Initial status: `in-progress`
- Source commit: `445fb17a4bc1df78c06d0cb289f3780befa72c02`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat002-statistics-history`

## Selection

The deterministic selector reported no open plan after STAT-001 finalized and
ranked implemented evidence gaps before in-progress work. CFG-001 still needs a
fresh purpose-bound 485 MHz/1200 mV hardware actuation, while its existing soak
lineage closed at `stop_repeated_boundary` and explicitly forbids attempt-005.
The remaining earlier implemented rows require broader live settings, network,
ASIC, Stratum, command, safety-control, logging, release, or recovery evidence
than a software-only iteration can supply. The display/input rows require
physical-interface evidence, and ASIC-009, ASIC-010, and BAP-001 require
unavailable non-Ultra or external-accessory hardware.

STAT-002 is the first actionable row. Its API shape and one-shot projection
marker are implemented, and STAT-001 now supplies typed current and rolling
hashrate values. The missing production behavior is a bounded, cadence-owned
history that records runtime snapshots independently of HTTP request timing.

## Scope and non-scope

Implement the pinned statistics task's 720-sample bounded history and exact
full-buffer removal behavior as a pure module. A zero `statsFrequency` clears
and disables history; a positive value retains one sample per one-second
producer tick and uses the configured frequency only for the reference
retention-span decision. Reject regressed sample timestamps rather than
allowing unsigned wraparound.

Add a dedicated firmware statistics producer with an absolute one-second
deadline. It reads the confirmed `statsFrequency` setting, captures current
hashrate, safety, Wi-Fi, platform, and response-time-compatible values through
the existing runtime snapshot boundary, and appends to the sole history owner.
The GET route returns a cloned history and never creates, drains, or mutates a
sample because of a request. Preserve the Phase 26 marker mechanics for their
existing projections without using them as the production history store.

This work does not authorize a device flash, monitor, credentials, network
access, mining campaign, pool connection, voltage/frequency/fan/power effect,
OTA, recovery, direct UART, or pin interaction. Live cadence, hardware-derived
telemetry accuracy, browser chart behavior, and long-duration thinning remain
below verified.

## Implementation

- [ ] Add a pure 720-entry statistics history with zero-frequency clearing,
      monotonic timestamp admission, exact full-buffer eviction, and focused
      boundary tests.
- [ ] Correct statistics samples to project the typed STAT-001 error percentage
      and preserve every existing AxeOS-compatible column and filter rule.
- [ ] Add the one-second firmware producer, confirmed-setting read, snapshot
      capture seam, sole history ownership, and request-time read-only API path.
- [ ] Add production wiring/ownership regressions, run every mandatory gate,
      and create a commit-bound result before transitioning only STAT-002.

## Verification and promotion

Focused tests must cover empty/disabled behavior, timestamp regression, append
through capacity, oldest eviction after the configured span, median-time
eviction inside the span, chronological output, history clearing, typed
hashrate/error projection, and request reads that cannot create or consume
samples. Firmware ownership tests must prove one producer, one-second absolute
cadence, authoritative `statsFrequency`, sole history storage, and no HTTP-time
sampling.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`, `just
parity`, `just parity-progress`, redaction, reference cleanliness, and diff
checks. Transition only STAT-002 from `in-progress` to `implemented` with
`unit,workflow,api-compare` evidence when all gates pass. Do not claim live
firmware cadence, hardware telemetry accuracy, long-duration retention, UI
behavior, or verified status.
