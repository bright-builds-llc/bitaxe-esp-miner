# Parity work plan

- Run ID: `20260812T193941Z-PWR-002`
- Parity row: `PWR-002`
- Initial status: `implemented`
- Source commit: `24843096bf0750e481efe9a49b877c83a7fae8a1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr002-asic-power-initialization-audit`

## Selection

The clean synchronized selector ranks `API-009` first and `PWR-002` second.
`API-009` is temporarily unavailable because its next IDENTIFY campaign must
begin only after an operator explicitly reports being present and watching in
the immediate pre-effect occurrence. No such fresh occurrence exists in this
invocation. Earlier display observations do not satisfy or weaken that physical
observation gate, so starting the campaign would consume an ordinal without a
valid IDENTIFY quorum.

`PWR-002` is the first currently actionable row. The validated public
`bitaxe-asic-initialization-evidence-v1` projection binds the sealed accepted
Ultra 205 attempt to exact-package and trusted-runtime identity, eighteen
accepted and zero invalid preparation events, terminal
`retain_production_uart/completed`, exactly one detected BM1366, mining-ready
initialization, retained production UART, accepted live work, fresh safety,
confirmed safe stop, cleanup, and no later hardware rerun. The deterministic
production preparation transaction orders fresh safety, fan actuation and
post-command RPM proof, conservative core voltage, a 500 ms stabilization
delay, active-low ASIC enable, reset/detect, mining-ready initialization, and
retained UART. The accepted attempt therefore supplies the row-specific
hardware regression needed for a typed audit without touching the device.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. The complete
safety, authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, and host-stall lesson blocks remain
loaded; the previously disclosed unrelated omitted set remains unchanged.

## Scope and non-scope

Add a typed `bitaxe-asic-power-initialization-evidence-v1` functional-core
contract and a narrow automation projector. Derive only closed PWR-002 facts
from the independently validated ASIC-002 projection, its exact SHA-256 and
commits, the pinned reference, the immutable plan and archived accepted-task
lineage, and current-versus-attempt source compatibility. Require byte-identical
admission for unchanged power-initialization modules and unique semantic-span
admission for modules changed since the accepted attempt.

The projection must bind the closed conservative profile to 400 MHz, 1100 mV,
and 100% fan; prove the deterministic nine-step preparation order; prove fresh
safety before power effects, post-command nonzero fan-RPM admission, a 500 ms
core-voltage stabilization boundary, active-low ASIC enable, reset and exactly
one chip, mining-ready frequency initialization, retained production UART,
accepted downstream work, confirmed safe stop, cleanup, no rerun, and
redaction. It must also bind the deterministic rollback contract: every
partial preparation failure attempts all idempotent safe-shutdown steps while
the original preparation failure remains primary. Fault recovery is a
source-backed fail-closed property, not a claim that the accepted campaign
injected a fault.

The projector must independently validate its source projection, reject any
incomplete or mutated quorum member, reject ambiguous or drifted semantics,
write through a private candidate, validate the candidate with the Rust-owned
contract, and publish atomically only after every check passes. Public output
may contain only schemas, commits, digests, fixed safe constants, closed
categories, counts, and booleans.

No detector, flash, reset, USB session, serial monitor, network request,
credential access, mining rerun, GPIO effect, direct UART, pin manipulation,
fault injection, voltage/fan/power change, or other hardware interaction is in
scope. This plan does not prove analog voltage accuracy, rail rise time,
electrical waveform shape, arbitrary power sequencing outside the closed
profile, automatic fan behavior, or recovery from an injected physical fault.
It proves the commanded production initialization transaction through the
exact admitted firmware, successful downstream chip/work behavior, safe stop,
and source-compatible implementation.

## Implementation

- [ ] Add the minimum typed ASIC-power-initialization evidence contract,
      independent validator, generated binding, projector command, and human
      command surface.
- [ ] Add behavior-focused Rust and TypeScript regressions for the complete
      quorum, step/order and profile constants, source compatibility,
      validator and workflow identity, publication withholding, candidate
      cleanup, and sensitive-output absence.
- [ ] Produce and independently validate one redacted public PWR-002 evidence
      projection from the accepted ASIC-002 projection; do not rerun hardware.

## Verification and promotion

Focused tests must prove that only a validator-accepted ASIC-002 source
projection with its exact digest, trusted package/runtime identity, complete
nine-step preparation, exactly one chip, accepted work, safe stop, cleanup,
and no hardware rerun can produce PWR-002 evidence. They must prove the exact
profile and ordered power boundaries, active-low enable semantics, rollback
and primary-failure semantics, source/reference/task/plan/workflow binding,
source-drift rejection, candidate cleanup, final-evidence withholding on every
failed quorum member, and absence of hostnames, origins, ports, USB/network
identifiers, credentials, raw traces, or private paths.

Run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require focused automation tests, exact generated contracts, independent
Rust validation of both source and final projections, `just verify-redaction`,
`just verify-reference`, unique task/plan binding, immutable-plan digest,
reference cleanliness, sensitive-output review, `git diff --check`, and full
diff review. Commit and push this immutable plan/task checkpoint before
implementation.

Promote `PWR-002` from `implemented` to `verified` only if the final closed
projection passes every validator and repository gate and establishes that the
exact admitted Ultra 205 firmware completed the upstream-aligned ASIC power
initialization transaction, produced successful initialized work, and then
completed safe stop and cleanup. Otherwise retain `implemented`, withhold final
evidence, record the exact blocker, and stop this row without a hardware
attempt.
