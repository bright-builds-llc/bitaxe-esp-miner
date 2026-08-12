# Parity work plan

- Run ID: `20260812T222308Z-PWR-006`
- Parity row: `PWR-006`
- Initial status: `implemented`
- Source commit: `e3f96fadaad2a4865bb019a25ae617fe930ab869`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr006-ina260-live-projection`

## Selection

The clean synchronized selector reports no open plan and ranks `API-009`
first, followed by `PWR-006`. `API-009` remains temporarily unavailable: its
latest closure requires a fresh pre-effect occurrence in which the operator
explicitly reports being present, watching the display, and ready to answer
both live prompts. This continuation reports improved display content but does
not contain that complete two-prompt readiness statement, so it cannot consume
another command-effects ordinal.

`PWR-006` is the first actionable row. Its checklist note predates the accepted
API-002 system-info capture and says no fresh INA260 values or HTTP/WebSocket
correlation exist. The ignored protected API-002 attempt still contains the
exact HTTP and WebSocket snapshots that produced the committed
`bitaxe-system-info-evidence-v1` projection. Both snapshots carry fresh power,
bus-voltage, and current observations with one identical acquisition stamp and
identical values inside the Ultra 205 safety envelope. The capture source,
reference, package, boot session, revisions, field contract, mining-disabled
state, hardware-control-disabled state, cleanup, and redaction were already
independently accepted. Every current INA260 acquisition and API projection
path relevant to this claim is byte-identical to that admitted source commit.

The active lesson set exceeds its deterministic loading budget, but its
2026-08-03 audit hashes still match both active inputs, so no new audit is
triggered. Safety, authorization, evidence, privacy, retry, redaction,
source-boundary, real-process, ESP-IDF, and host-stall lessons informed this
plan. The unrelated caption, small-table deletion, GSD separator, and manual-
removal blocks were not loaded. Repo-local guidance, Bright Builds rules,
architecture, code-shape, verification, testing, Rust, and TypeScript standards
were reviewed; no local override changes this work.

## Scope and non-scope

Add one typed `bitaxe-ina260-evidence-v1` functional-core contract, independent
validator, generated binding, and narrow automation projector. The projector
must independently validate the existing public API-002 projection, the exact
private attempt modes and expected artifacts, source/reference/boot coherence,
fresh complete INA260 power/current/bus-voltage status, finite safe-range
predicates, exact HTTP/WebSocket value and acquisition-stamp equality, current
source compatibility, cleanup, and redaction. It must write a private
candidate, validate it through the Rust-owned contract, and publish atomically
only after every member passes.

Public evidence may contain only schemas, commits, SHA-256 digests, fixed
INA260 address/register constants, counts, categories, and booleans. It must
never contain raw power, voltage, current, acquisition stamps, boot sessions,
hostnames, origins, ports, USB/network identifiers, credentials, retained
logs, traces, or private paths.

No detector, package build, flash, reset, USB/serial access, network request,
credential read, mining, voltage, fan, power, I2C/GPIO, direct UART, pin,
fault-injection, recovery, or other hardware effect is authorized or required.
The API-002 attempt is not rerun. This plan does not claim analog calibration
accuracy, rail waveform or timing, INA260 alert/configuration registers,
hardware writes, dynamic loads, automatic control, fault recovery, another
board, or another sensor family.

## Implementation

- [x] Audit PWR-006, the pinned INA260 reference, current sensor/API owners,
      protected API-002 snapshots, public projection, and source compatibility.
- [ ] Add the minimum Rust evidence contract, validator, generated binding,
      projector command, and human command surface.
- [ ] Add behavior-focused regressions for the exact source quorum, freshness,
      safe ranges, HTTP/WebSocket equality, source drift, private modes,
      publication withholding, candidate cleanup, real-child validation, and
      sensitive-output absence.
- [ ] Produce and independently validate one public redacted PWR-006 projection
      from the existing protected capture without touching the device.
- [ ] Transition only PWR-006, synchronize progress, complete the task review,
      and archive the task atomically if the full quorum passes.

## Verification and promotion

Focused verification must include independent validation of the API-002 source
projection and new INA260 projection; the Rust contract tests; TypeScript
projector, invocation, validator-boundary, and real-child tests; exact private
and public digests; mode-0700/0600 source protection and mode-0644 final
publication; source/reference ancestry; byte compatibility for the INA260
adapter, acquisition reducer, observation store, snapshot, wire, and statistics
paths; generated contracts; repository redaction; pinned-reference cleanliness;
candidate absence; immutable-plan and task binding; sensitive-output denial;
and `git diff --check`.

Run the mandatory sequence in order: `cargo fmt --all`, `cargo clippy
--all-targets --all-features -- -D warnings`, `cargo build --all-targets
--all-features`, `cargo test --all-features`, `bun
scripts/bright-builds-check.ts all`, `just test`, `just parity`, and `just
parity-progress`. Also run `just verify-redaction`, `just verify-reference`,
and full diff review before each required commit boundary.

Promote PWR-006 only if the final typed projection binds board 205; the exact
API-002 source, current source, pinned reference, package and workflow lineage;
validated protected modes and digests; fresh complete INA260 current,
bus-voltage, and power observations; finite safe-range predicates; exact
HTTP/WebSocket value, state, and acquisition-stamp correlation; read-only
production ownership; source compatibility; mining and hardware control
disabled; cleanup; no hardware rerun; and passed redaction. Any missing,
malformed, drifted, unsafe, incoherent, or privacy-invalid member withholds the
projection, keeps PWR-006 at `implemented`, records a truthful closure, and
stops without hardware recovery or retry.
