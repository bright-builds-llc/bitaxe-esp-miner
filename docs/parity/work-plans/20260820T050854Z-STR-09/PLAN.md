# Parity work plan

- Run ID: `20260820T050854Z-STR-09`
- Parity row: `STR-09`
- Initial status: `implemented`
- Source commit: `646e0d0008ba7120bddeb427d33bd6cec5329d34`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str09-submit-response-classification`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `STR-09`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`STR-09` is therefore first actionable.

Later accepted evidence supersedes the Phase 27/28 prerequisite artifacts and
the original Phase 30 no-promotion disposition for this exact row. STR-001
proves an authorized production socket with an accepted submit response.
STR-006 binds that response to hardware preparation, ASIC-derived work, a
qualified correlated result before submit, and ordered safe stop. ASIC-004
independently proves the parsed BM1366 result and compatible correlation.
Current submit-response, live-runtime, and production-session tests prove
classification requires a matching current submit intent and response identity.

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

Advance only `STR-09`. Produce a source-bound evidence summary joining the
accepted STR-001, STR-006, and ASIC-004 projections to current submit-response
classification tests. Add the three exact STR-09 proof fields to the canonical
Phase 30 conclusion, update its requirement table/evidence basis/non-claims,
and change the checked-in current-artifact regression to require every promoted
Phase 30 row. No new runtime implementation or hardware projector is needed.

Bind these accepted projections exactly:

- STR-001 socket:
  `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`
- STR-006 protocol coordinator:
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- ASIC-004 result parsing:
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`

The evidence summary and Phase 30 update may contain only repository paths,
commits, digests, closed labels, booleans, counts, and command outcomes. They
must contain no pool or credential values, endpoints, ports, IP/MAC/USB
identity, device URLs, raw Stratum messages, work/share values, logs/payloads,
PIDs, traces, or protected identifiers.

This plan authorizes local source/tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Independently validate the accepted STR-001, STR-006, and ASIC-004
      projections.
- [ ] Run current submit-response, live-runtime, and production-session tests.
- [ ] Add exact STR-09 structured proof to the canonical Phase 30 conclusion
      and make the current-artifact regression require all promoted rows.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md`; commit implementation
      and evidence as `SOURCE_COMMIT`, transition only STR-09, sync progress,
      archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-stratum submit_response`
- `cargo test -p bitaxe-stratum live_runtime`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //tools/parity:tests`
- the existing STR-001, STR-006, and ASIC-004 Rust evidence validators over
  absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, selector, sensitive-value, source-diff, and final diff checks.

Promotion requires one accepted Ultra 205 hardware share whose response is
classified only after a current-generation ASIC result correlates to active
work and creates matching submit intent; exact-package admission, trusted
identity/safety, ordered safe stop, cleanup, current classification tests,
independent validation, redaction, and the canonical Phase 30 fields
`STR-09.live_submit_response_classified: true`,
`STR-09.asic_correlation: passed`, and
`STR-09.safe_stop_status: complete` must all pass. Checklist notes must include
the Phase 28 and Phase 30 paths, `accepted share hardware proof`,
`asic bridge correlation`, `redaction_status: passed`, and `exact_non_claims`,
and must omit blocker terms.

On success create `RESULT.md`, commit implementation/evidence without a
checklist change and save that full commit as `SOURCE_COMMIT`; transition only
`STR-09` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`, sync progress, archive only
this task, run final gates, and push. On failure create `CLOSURE.md`, leave
STR-09 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify rejected-share hardware, mismatched/stale response
paths on hardware, fallback or reconnect on hardware, exact upstream timeout or
keepalive equivalence, arbitrary pools, TLS, Stratum v2, unbounded mining,
other boards/ASICs, updates, recovery, profitability, or release readiness. It
does not promote SAFE-12 or SAFE-13.
