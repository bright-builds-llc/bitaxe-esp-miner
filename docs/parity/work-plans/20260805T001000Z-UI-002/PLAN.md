# Parity work plan

- Run ID: `20260805T001000Z-UI-002`
- Parity row: `UI-002`
- Initial status: `in-progress`
- Source commit: `4e50e0e07b5c06f37380301d57d18b65a1d13edf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui002-screen-flow`

## Selection

The deterministic selector reported no open plan. The earlier `implemented`
rows require their claim-specific live configuration, network, mining,
safety-control, release, accessory, other-board, or physical-interface evidence
rather than another software relabel. UI-002 is the first software-actionable
row: UI-001 now owns configured panel initialization and power, but the runtime
still alternates one debug line and does not implement the pinned priority
screens, identify/notification layers, intro sequence, or bounded carousel.

The pinned screen implementation updates every 500 ms, gives pre-carousel
states priority, overlays identify and mining notifications, pins statistics
for a new-block notification, shows two bounded intro pages once, and then
cycles URL, statistics, mining, and Wi-Fi pages. It updates content without
reinitializing the display and keeps button-driven navigation separate.

## Scope and non-scope

Add a pure, bounded Ultra 205 screen-flow model with independent page and
overlay decisions. Model the pinned priority order for self-test, firmware
update, ASIC status, overheat, welcome, and connection states; the one-shot
Bitaxe/open-source intro; the URL, statistics, mining, and Wi-Fi carousel; the
identify overlay; accepted/rejected/work and paused notifications; and the
new-block statistics pin. Preserve exact 500 ms evaluation and pinned dwell
semantics using checked monotonic time.

Render every page into exactly four bounded SSD1306 text lines. Sanitize control
characters, truncate by display-cell width, and represent unavailable runtime
facts explicitly. Pool hosts, SSIDs, IP addresses, and other local values may
exist only in the private in-memory frame sent to the panel; do not log, debug,
persist, commit, or expose them through new evidence.

Add a read-only firmware projection from already-owned runtime, Wi-Fi, safety,
settings, identify, block-notification, and production-session state. It must
not issue an operator snapshot, drain statistics markers, append retained logs,
or mutate mining state. Retain one screen-flow owner beside the UI-001 display
owner, refresh it on the shared absolute 500 ms schedule, redraw only changed
frames, and pass priority visibility into the existing power policy. Display
failure must continue to isolate itself from sensor acquisition.

This plan does not integrate or exercise a physical button; UI-003 owns button
wake, short/long press, and manual navigation. Missing live coinbase fields may
render as unavailable until a separate production owner supplies them, and no
claim is made for exact LVGL animation or bitmap assets on the text-only Rust
stack. This plan authorizes local software, synthetic fixtures, and build work
only: no flash, credentials, external service, mining, pool connection,
voltage/frequency/fan/power effect, OTA, recovery, direct UART, pins, or
physical input.

## Implementation

- [ ] Add the pure page, overlay, counter-delta, priority, one-shot intro, and
      carousel state machine with bounded four-line frame formatting.
- [ ] Add a side-effect-free firmware screen snapshot projection, including
      the production work-received count without exposing sensitive values.
- [ ] Replace alternating debug content with the retained screen owner on an
      absolute 500 ms cadence and redraw only when the complete frame changes.
- [ ] Add focused transition/content/privacy tests and production ownership
      regressions, build the real firmware, and run every required gate.

## Verification and promotion

Focused tests must cover every priority route, priority release back to the
carousel, the one-shot intro, exact dwell boundaries, wraparound, identify and
notification overlays, accepted/rejected/work counter deltas, paused state,
new-block pinning, unavailable values, bounded lines, control-character
sanitization, regressed clocks, and checked overflow. Firmware tests must prove
that projection is read-only, no private value reaches logs or debug output,
the 500 ms schedule is absolute, only changed frames flush, priority visibility
feeds display power, and a display failure cannot stop sensor acquisition.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`, `just
parity`, `just parity-progress`, redaction, reference cleanliness, immutable-
plan, and diff checks. Transition only UI-002 from `in-progress` to
`implemented` with `unit,workflow` evidence after every gate passes. Do not
claim UI-003 input parity, live screen behavior, exact LVGL animation/bitmap
parity, mining success, hardware-control effects, or `verified` status.
