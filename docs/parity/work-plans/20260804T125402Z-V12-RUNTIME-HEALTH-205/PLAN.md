# Parity work plan

- Run ID: `20260804T125402Z-V12-RUNTIME-HEALTH-205`
- Parity row: `V12-RUNTIME-HEALTH-205`
- Initial status: `implemented`
- Source commit: `74ef3921352381356e13ec08f9249f3a32f976b9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-v12-runtime-health-typed-capture`

## Selection

The deterministic selector reported no open plan after successful closure of
`V12-OPERATOR-SNAPSHOT-205`. Earlier candidates retain the safety-critical,
broad behavior, consumed-attempt, non-promotable evidence, or incomplete
hardware-effect gaps already recorded in the predecessor plan. The remaining
narrow v1.2 runtime row is the first actionable candidate: Phase 36 names the
exact correction `runtime_health_insufficient`, while current firmware already
publishes one coherent runtime-health value through HTTP, WebSocket, and the
paired retained log record.

## Scope and non-scope

Implement one typed passive capture that freezes an exact package, admits one
Ultra 205, confirms safe disabled boot, captures HTTP system info and a later
same-boot live WebSocket snapshot, and validates each projected runtime-health
value against the exact correlated retained-log record. Require an available
healthy supervisor checkpoint, positive non-regressing checkpoint sequence,
participating task watchdog with a fresh positive feed sequence, exact package
identity, same boot, monotonic snapshot revision, cleanup, and closed privacy.

Private artifacts remain mode `0600` beneath one mode-`0700`
absent-before-use attempt root. The committed projection may contain only
schema names, cryptographic identities, bounded revisions/sequences/age
categories, booleans, source identities, and closed state labels. It must not
contain hostnames, origins, URLs, IP or MAC values, Wi-Fi or pool values, ports,
USB identities or paths, raw serial/HTTP/WebSocket/log content, or process
identifiers.

Do not invoke self-test, restart, mining, settings mutation, ASIC work,
hardware control, OTA, recovery, discovery, direct UART, or pin work. Do not
promote operator snapshots, broader system API, watchdog intervention,
self-test lifecycle, mining safety, or any other checklist row.

## Implementation

- [ ] Add a Rust-owned runtime-health evidence contract, validator, and strict
      detector-gated automation command.
- [ ] Add a private retained-record validator that binds the full health tuple
      to each HTTP/WebSocket snapshot identity and rejects unavailable, stale,
      unhealthy, regressing, or contradictory observations.
- [ ] Preserve earliest typed failure through bounded exact-package recovery
      and publish the closed projection only after private validation.
- [ ] Add behavior-focused unit and real-child-process regressions for valid
      joins, session/revision/sequence mismatch, health-state failure,
      malformed child output, recovery precedence, and sensitive-value denial.
- [ ] Record exact commands and evidence in `WORKLOG.md`; create `RESULT.md`
      only if the single hardware capture satisfies every promotion condition.

## Verification and promotion

Run the focused contract, automation, parity-validator, and real-process tests,
then the mandatory Rust sequence, Bright Builds checks, full Bazel suite,
parity/progress, semantic redaction, reference cleanliness, and diff checks.
After a clean pushed implementation, run exactly the task-recorded package,
detector, and capture commands once. Promote only
`V12-RUNTIME-HEALTH-205` when every substantive passive-health, exact-package,
safe-state, cleanup, and privacy fact passes. Any missing fact withholds final
evidence and ends the attempt without retry.
