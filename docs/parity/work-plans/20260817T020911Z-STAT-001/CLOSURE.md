# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `9ce5afb115c874e43ab72ac5ccbe253874d37865a25128d360cb1106afb156b1`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed implementation `edef059bfc1d5dcc79f997c46fa022d8e1bd8ffc`
corrects the source-level backpressure boundary identified after attempts 008,
009, and 010. Campaign-status state remains per-event, but redundant retained
and serial publication is now first-immediate, terminal-immediate, and bounded
to one-second periodic cadence. A closed lock-free owner phase is projected
through runtime health, HTTP/WebSocket, retained text, v7 network evidence,
v13 result evidence, and sealed private-first watchdog diagnostics.

Focused tests prove all closed phases, unavailable/unknown fail-closed
behavior, first and terminal publication, clock regression and overflow, and a
600-second 20-ms event storm with 601 publications from 30,001 events and no
gap above 1,000 ms. Package, format, lint, build, Rust tests, Bright Builds
checks, the complete Bazel suite, privacy, reference, schema/seal, source
ownership, and parity invariance all pass. No hardware, credential, detector,
network-runtime, protected-attempt, or public-projection access occurred.

This software evidence cannot establish live BM1366 hashrate or twenty-window
continuity. STAT-001 therefore remains `implemented`; no checklist or progress
transition is justified.

## Next safe action

A future invocation may consider a new attempt-011 only after creating and
committing a complete immutable detector-gated hardware contract bound to
exact pushed implementation `edef059bfc1d5dcc79f997c46fa022d8e1bd8ffc`.
That contract must preserve the new phase discriminator, v13/v7 seals,
protected evidence, cleanup, retry bounds, and accepted stop conditions. This
closure itself authorizes no device use or retry.

## Non-claims

This closure does not verify STAT-001, prove the inferred serial-backpressure
cause on hardware, authorize attempt-011, prove live BM1366 hashrate accuracy,
complete twenty windows or 600 live seconds, prove watchdog responsiveness
under full device load, establish terminal zero, or claim electrical accuracy,
profitability, arbitrary profiles or pools, other boards or ASICs,
update/recovery behavior, or release readiness.
