# Parity work plan

- Run ID: `20260804T230000Z-UI-001`
- Parity row: `UI-001`
- Initial status: `in-progress`
- Source commit: `8e4399758a77e6f6bda1b0f4c46f9627178e79ca`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui001-display-behavior`

## Selection

The deterministic selector reported no open plan. CFG-001 has no authorized
retry after its bounded soak lineage closed at `stop_repeated_boundary`.
CFG-005, CFG-006, NET-001 through NET-003, the implemented ASIC/Stratum/API,
power/thermal/IO/self-test rows, BAP-002, UI-004, LOG-001, STAT-001 through
STAT-003, REL-001 through REL-003, and SAFE-10 through SAFE-13 require their
claim-specific live configuration, network, mining, safety-control, release,
other-board, accessory, or physical-interface evidence rather than another
software relabel. ASIC-009 and ASIC-010 need unavailable non-Ultra boards, and
BAP-001 needs an unavailable accessory plus separately authorized electrical
attachment.

UI-001 is the first software-actionable row. The current Ultra 205 adapter
hardcodes SSD1306 128x32, rotation zero, normal colors, and effectively always-
on behavior. It ignores the already-confirmed `display`, `rotation`,
`invertscreen`, and `displayTimeout` settings and recreates an unconfigured
driver for each runtime frame. The pinned display implementation validates the
configured panel, applies rotation and inversion at initialization, and owns
display power independently from screen content.

## Scope and non-scope

Add a pure Ultra 205 display contract for the exact `SSD1306 (128x32)` panel,
the four supported rotations, inversion, and the upstream timeout vocabulary:
negative means always on, zero means off outside an explicit wake/priority
window, and a positive value means an inactivity timeout in minutes. Use
checked time arithmetic, make state changes edge-triggered, and preserve the
earliest configuration or driver failure without affecting sensor acquisition.

Load the display contract from the confirmed settings snapshot before the I2C
display is initialized. Keep one firmware display owner responsible for panel
initialization, rendering, and on/off commands. Apply rotation and inversion
before the first published frame, retain an initialized driver across runtime
frames, and service display timeout decisions on an absolute bounded cadence.
The current startup/debug frame remains the content source for this row.

UI-002 continues to own upstream screen priority, notification, and carousel
content. UI-003 continues to own boot-button input, short/long press semantics,
and wake activity. This plan does not authorize a device flash, credentials,
network access, mining, pool connection, voltage/frequency/fan/power effects,
OTA, recovery, direct UART, pins, or physical button interaction. Live panel
orientation, inversion, timeout, current draw, and operator-visible behavior
remain below verified.

## Implementation

- [ ] Add the pure exact-panel configuration, rotation, inversion, timeout,
      activity, and edge-triggered display-power state machine with focused
      boundary and overflow regressions.
- [ ] Project the contract from the confirmed settings snapshot with upstream
      defaults and fail closed on malformed or unsupported stored values.
- [ ] Refactor the firmware adapter into one retained display owner that applies
      configuration before first render and owns runtime render/power commands.
- [ ] Preserve the independent 500 ms sensor cadence, add source-ownership and
      command-order regressions, build the real firmware, and run every gate.

## Verification and promotion

Focused tests must cover exact panel admission, every rotation, inversion,
missing-key defaults, malformed and unsupported settings, negative/zero/
positive timeout policy, exact timeout boundaries, priority/wake overrides,
edge-triggered commands, regressed clocks, and checked overflow. Firmware tests
must prove one retained owner, configuration before first flush, no per-frame
reinitialization, sole ownership of display power, and display failures isolated
from sensor acquisition.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`, `just
parity`, `just parity-progress`, redaction, reference cleanliness, immutable-
plan, and diff checks. Transition only UI-001 from `in-progress` to
`implemented` with `unit,workflow` evidence after every gate passes. Do not
claim UI-002 carousel parity, UI-003 input parity, live display behavior, or
verified status.
