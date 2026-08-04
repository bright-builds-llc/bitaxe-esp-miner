# UI-003 bounded Ultra 205 boot-button plan

- Run ID: `20260805T020000Z-UI-003`
- Parity row: `UI-003`
- Starting status: `in-progress`
- Target status: `implemented`
- Source commit: `81ec8b87ab6396329fbfb3066464310a55655f2e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui003-boot-button`
- Hardware attempts: none

## Selection and gap

Deterministic selection has one remaining `in-progress` row: UI-003. The
pinned `input.c` configures active-low GPIO0 with a pull-up, classifies a short
click versus one long press at 2,000 ms, routes a normal short click to identify
cancellation or `screen_next`, routes a normal long press to configuration-AP
toggle, and routes a self-test long press to self-test reset. The Rust firmware
owns the display and screen flow but does not own GPIO0, classify input, or
route any physical button effect; its runtime marker still declares input
unavailable.

This is software-actionable without pressing or electrically manipulating the
button. Physical observation remains a separate hardware-regression gate.

## Contract and ownership

Add a pure active-low button owner with a bounded poll/debounce contract. A
stable press begins after 30 ms, a stable release before 2,000 ms emits exactly
one short click, and a held stable press emits exactly one long press at the
2,000 ms boundary. Release after a long press emits no short click. Regressed
monotonic time and checked deadline overflow fail without advancing state.

Add one firmware GPIO0 owner using the ESP-IDF Rust GPIO driver with pull-up.
The owner lives in one named thread, samples every 10 ms, retains no input
history, and logs only closed status/effect categories. Driver, thread, or
classification failure disables input only; display, sensor, network, mining
safe-stop, and hardware-control owners continue independently.

Normal short clicks cancel an active identify effect atomically; otherwise
they enqueue a bounded screen-next request. The display owner consumes pending
requests at its existing 500 ms boundary, renders the manual successor before
priority reassertion on the next evaluation, records display activity, and
coalesces a bounded number of queued clicks without leaking frame text.

Normal long presses call a typed Wi-Fi-owner configuration-AP toggle. Retain
the validated AP configuration and optional station configuration inside the
existing sole Wi-Fi owner, switch between AP/AP+STA and STA/none as appropriate,
start captive DNS at most once, and update the private runtime snapshot only
after ESP-IDF accepts the configuration. No SSID, address, credential, or
network identifier may appear in new input logs or evidence.

The current Rust runtime has no active self-test effect owner and publishes
that state as unavailable. Preserve the pinned routing decision in pure tests,
but fail closed with a typed `self_test_reset_unavailable` category rather than
pretending to reset a nonexistent owner.

## Implementation

- [ ] Add the pure debounce/classification and short/long routing vocabulary.
- [ ] Add the GPIO0 firmware owner, bounded event queue, identify cancellation,
      manual screen advance, activity wake, and redaction-safe status marker.
- [ ] Extend the sole Wi-Fi owner with a typed configuration-AP toggle and
      one-time captive-DNS admission without changing credential handling.
- [ ] Add focused pure, adapter, ownership, failure-isolation, and firmware
      build coverage, then run every required repository gate.

## Verification and promotion

Focused tests must cover active-low idle/press/release, bounce, exact debounce
boundaries, short release, exact 2,000 ms long boundary, held one-shot behavior,
long-release suppression, regressed time, overflow, identify cancellation,
manual screen ordering/wrap, priority reassertion, display activity, normal AP
toggle routing, self-test fail-closed routing, unavailable owners, and absence
of sensitive values from public debug/log/evidence surfaces. Production source
tests must bind GPIO0 pull-up ownership, the 10 ms sampling contract, single
owner installation, post-configuration snapshot publication, and failure
isolation.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`, `just
parity`, `just parity-progress`, redaction, reference cleanliness, immutable-
plan, and diff checks. Transition only UI-003 from `in-progress` to
`implemented` with `unit,workflow` after every gate passes. Do not claim live
button behavior, exact LVGL timing, live AP toggling, self-test cancellation,
display content, mining, hardware effects, or `verified` status.
