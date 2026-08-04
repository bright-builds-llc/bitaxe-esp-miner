# Parity work plan

- Run ID: `20260804T180000Z-BAP-002`
- Parity row: `BAP-002`
- Initial status: `not-started`
- Source commit: `ff2d1114c7e08868ffd58ce4cb523dfcca81364d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap002-protocol`

## Selection

The deterministic selector reported no open plan after `NET-003` closed.
Implemented candidates require their separately documented live configuration,
network, API, mining, safety-effect, release, or recovery evidence rather than
another software relabel. The in-progress display/input rows require device
display or input evidence, and the in-progress statistics rows depend on live
runtime producers and parsed share outcomes that are not presently admitted.

`ASIC-009` and `ASIC-010` are later non-Ultra board expansions without eligible
hardware or a current board target. `BAP-001` requires an external accessory
UART and task lifecycle; standing USB authorization neither supplies that
accessory nor authorizes a new electrical attachment. `BAP-002` is the first
actionable row because its bounded protocol and handler-decision behavior can be
implemented as a pure Rust core without a UART, accessory, credentials,
network requests, or hardware effects.

## Scope and non-scope

Add an independently designed `bitaxe-core` BAP module with the exact ten
command tokens, eighteen parameter tokens, 256-byte wire bound, NMEA-style XOR
checksum, canonical `$BAP,...*XX\r\n` encoder, and a typed parser. Preserve the
reference's compatibility rules that checksum-free `SUB` and `UNSUB` frames are
admitted and a `SUB` checksum mismatch is tolerated, while every other missing
or mismatched checksum fails closed through a value-free category.

Add a bounded stateful ingress admission helper that rejects the same complete
frame repeated within 1,000 milliseconds without exposing retained bytes in
`Debug`. Add pure command planning for connected/AP mode, supported request
projection, subscription default and positive interval selection, and setting
validation. Preserve exact public BAP error values for AP restrictions, missing
values, invalid ranges, and unsupported settings. Model setting and restart
needs as intents only; do not apply NVS writes, credentials, voltage,
frequency, fan, block state, or restart effects.

Use only synthetic golden cases. Custom `Debug` output must expose command and
closed category information but never frame values, SSIDs, passwords, pool
fields, workers, addresses, or raw bytes. Keep framing, parsing, validation,
and response projection in the functional core.

Do not initialize UART2, select pins, attach an accessory, register FreeRTOS
tasks, implement the firmware imperative shell, persist settings, start
subscriptions, send network traffic, enable mining, submit ASIC work, change
frequency/voltage/fan/power, restart the device, flash hardware, use direct
UART, or manipulate pins. Those remain `BAP-001` or separate safety-evidence
work and cannot be inferred from pure protocol evidence.

## Implementation

- [ ] Add exact command/parameter vocabularies, checksum, bounded canonical
      encoding, typed parsing, and compatibility admission.
- [ ] Add duplicate suppression plus pure request, subscription, and setting
      decisions with redaction-safe errors and no imperative effects.
- [ ] Add synthetic golden and behavior-focused regressions, update Cargo/Bazel
      ownership, and run every mandatory repository gate.
- [ ] Create a commit-bound `RESULT.md`; transition only `BAP-002` to
      `implemented` when all pure criteria pass, leaving UART interoperability
      and hardware effects explicitly pending.

## Verification and promotion

Run focused `cargo test -p bitaxe-core bap --all-features` and
`bazel test //crates/bitaxe-core:tests`, inspect golden provenance and sensitive
value scans, then run in order `cargo fmt --all`, strict all-target/all-feature
Clippy, all-target/all-feature build, all-feature tests, Bright Builds checks,
`just test`, `just parity`, `just parity-progress`, redaction, reference
cleanliness, and diff checks.

Transition only `BAP-002` from `not-started` to `implemented` with `unit,golden`
evidence if the exact vocabulary, bounded wire format, checksum compatibility,
duplicate boundary, connected/AP command decisions, request response shapes,
setting range decisions, value-free error categories, redacted diagnostics, and
all repository gates pass. Live accessory UART interoperability, request/task
lifecycle, subscription delivery, persistence, restart, and any hardware
effect require a later `BAP-001` implementation plus explicit eligible evidence
before either row can be `verified`.
