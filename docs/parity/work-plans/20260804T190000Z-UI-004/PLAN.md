# Parity work plan

- Run ID: `20260804T190000Z-UI-004`
- Parity row: `UI-004`
- Initial status: `not-started`
- Source commit: `d93e63455e48ab512009a56b0847a4f1996625a4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui004-operator-workflows`

## Selection

The deterministic selector reported no open plan after `BAP-002` closed.
Implemented candidates require their claim-specific live configuration,
network, API, mining, safety-effect, release, or recovery evidence rather than
another software relabel. The in-progress display/input rows require physical
display or input evidence, while the statistics rows still depend on live
runtime producers and parsed share outcomes.

`ASIC-009` and `ASIC-010` are later non-Ultra board expansions without an
eligible board target. `BAP-001` requires an external accessory UART and task
lifecycle; standing USB authorization neither supplies that accessory nor
authorizes a new electrical attachment. `UI-004` is the first actionable row
because the existing firmware routes and static packaging can support a
Rust-owned responsive operator interface with synthetic API and real-browser
evidence without touching device hardware.

## Scope and non-scope

Replace the recovery-only fallback index with an independently designed,
dark-first operator interface. Provide accessible responsive navigation for the
dashboard, network, pool, settings, logs, update, and theme workflows. Reuse the
existing same-origin API contracts for system information, settings PATCH,
theme GET/POST, retained logs, live log WebSocket, firmware OTA, and explicit
command routes. Keep credential fields write-only and absent from diagnostics,
DOM defaults, URLs, and browser persistence.

Add a small pure JavaScript core for route admission, value formatting,
HTML/text safety, write-only patch construction, and closed public errors. Keep
network and DOM effects in separate imperative adapters. Add known SPA route
fallbacks to the pure Rust static planner so direct navigation and reload serve
the same index without weakening traversal protection or unknown-asset
redirect behavior.

The update surface must require an explicit selected `esp-miner.bin` plus a
user confirmation before POSTing firmware. Keep the existing OTAWWW gap visible
and fail closed; do not add whole-`www` updates. Destructive commands must
require user confirmation. Do not enable mining or expose active frequency,
voltage, fan, power, erase, rollback, recovery-fault, or raw hardware controls.

Use only synthetic browser fixtures. The interface may render public device
telemetry in the user's browser, but tests, logs, committed evidence, and
diagnostics must not contain real origins, hostnames, SSIDs, addresses, ports,
workers, credentials, network identifiers, or device identifiers. Do not copy
Angular source expression or generated assets from the GPL reference tree;
reference it only for behavioral breadcrumbs.

## Implementation

- [ ] Add the independent responsive static UI, dark/light theme, accessible
      navigation, safe API client, and write-only configuration forms.
- [ ] Add retained/live log controls, explicit command confirmations, and the
      firmware-only update workflow with the OTAWWW gap kept visible.
- [ ] Add direct SPA route fallback plus pure JavaScript, static-contract, and
      Rust route regressions, including sensitive-value and traversal checks.
- [ ] Exercise production assets in a real browser against synthetic same-origin
      API responses, run every mandatory repository gate, and create a
      commit-bound result before transitioning only `UI-004` to `implemented`.

## Verification and promotion

Run the focused static-contract and `bitaxe-api` route tests, then use the
Playwright CLI against a local synthetic same-origin fixture to navigate every
page, submit a write-only settings form, change theme, filter logs, and confirm
the update workflow remains inert without a selected file and confirmation.
Capture only redaction-safe browser evidence beneath `output/playwright/`.

Then run in order `cargo fmt --all`, strict all-target/all-feature Clippy,
all-target/all-feature build, all-feature tests, Bright Builds checks,
`just test`, `just parity`, `just parity-progress`, redaction, reference
cleanliness, sensitive-value review, static provenance review, and diff checks.

Transition only `UI-004` from `not-started` to `implemented` with
`unit,workflow,browser` evidence if production static assets provide the scoped
navigation, configuration, logs, theme, and firmware-update workflows; direct
route reloads are safe; all sensitive fields remain write-only; the real
browser flow passes; and every repository gate is clean. Live embedded serving,
real device configuration mutation, live logs, firmware upload, OTAWWW,
responsive-device UAT, and end-to-end hardware behavior remain separate
evidence requirements before `verified`.
