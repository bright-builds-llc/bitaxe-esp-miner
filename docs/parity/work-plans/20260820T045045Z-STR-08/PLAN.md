# Parity work plan

- Run ID: `20260820T045045Z-STR-08`
- Parity row: `STR-08`
- Initial status: `implemented`
- Source commit: `009d1df778d1367e6fc539f491afd26f82e0cd35`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str08-live-socket-lifecycle`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `STR-08`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`STR-08` is therefore first actionable.

Later accepted evidence supersedes the Phase 25 and Phase 27 static prerequisite
artifacts for this exact row. The independently validated STR-001 projection
proves one real authorized production TCP session through accepted submit
response and safe stop. The STR-006 projection binds that socket proof to
hardware preparation, authorization, ASIC dispatch/result correlation, ordered
terminal safe stop, and the current single-owner lifecycle. Current live-runtime,
production-session, and loopback transport tests cover the remaining pure and
host-adapter lifecycle behavior.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, managed architecture, code-shape, verification,
testing, and Rust standards, the active tracker/checklist, and bounded lessons
for protected evidence, redaction, hardware authorization, retries, units, and
agent-runtime timing. Active lesson inputs total 31,758 bytes, so every heading
was inventoried and priority complete blocks plus the complete global file were
loaded; remaining non-priority repository blocks were omitted under the
deterministic budget. The August lesson-audit baseline remains current and no
new audit trigger is due.

## Scope and non-scope

Advance only `STR-08`. Produce a source-bound evidence summary that joins the
accepted STR-001 socket and STR-006 coordinator projections, current live-
runtime and production-session tests, the production transport loopback test,
current source identity, pinned reference behavior, and explicit non-claims.
No new runtime implementation or projector is needed because the typed socket
lifecycle and accepted live socket session already exist.

Bind these accepted projections exactly:

- STR-001 socket:
  `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`
- STR-006 protocol coordinator:
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`

The evidence summary may contain only repository paths, commits, digests,
closed labels, booleans, counts, bounded durations, and command outcomes. It
must contain no pool or credential values, endpoints, ports, IP/MAC/USB
identity, device URLs, raw Stratum messages, work/share values, logs/payloads,
PIDs, traces, or protected identifiers.

This plan authorizes local tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Independently validate the accepted STR-001 and STR-006 projections.
- [ ] Run current live-runtime, production-session, and transport-loopback
      lifecycle tests.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md` with exact digests,
      conclusions, and non-claims.
- [ ] Commit the evidence as `SOURCE_COMMIT`, transition only STR-08, sync
      progress, archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-stratum live_runtime`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //firmware/bitaxe:production_transport_tests`
- the existing STR-001 and STR-006 Rust evidence validators over absolute
  projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, selector, sensitive-value, source-diff, and final diff checks.

Promotion requires one accepted Ultra 205 production session with exact-package
admission, trusted identity and safety, a real live socket connection,
authorization before ASIC dispatch, live work/result correlation, accepted
submit response, ordered safe stop and cleanup, current typed lifecycle and
loopback tests, independent validation, and redaction. Checklist notes must
include the Phase 28 summary path, `live socket success`, `accepted share
hardware`, `redaction_status: passed`, and `exact_non_claims`, and must omit
blocker terms.

On success create `RESULT.md`, commit evidence without a checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `STR-08` to `verified`
with `unit,workflow,hardware-smoke,hardware-regression`, sync progress, archive
only this task, run final gates, and push. On failure create `CLOSURE.md`, leave
STR-08 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify fallback or reconnect on hardware, exact upstream
timeout or keepalive option equivalence, DNS/IP-family preference parity,
arbitrary pools, TLS, Stratum v2, rejected-share hardware, unbounded socket
stability, other boards, updates, recovery, profitability, or release readiness.
It does not promote STR-09, SAFE-12, or SAFE-13.
