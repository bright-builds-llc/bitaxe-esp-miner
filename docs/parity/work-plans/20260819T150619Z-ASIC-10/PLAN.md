# Parity work plan

- Run ID: `20260819T150619Z-ASIC-10`
- Parity row: `ASIC-10`
- Initial status: `implemented`
- Source commit: `c35cec2c541b989a6cb8af848144abc59f3c7b5e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic10-work-registry`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order is `SELF-001`,
`BAP-002`, `STAT-003`, then `ASIC-10`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`ASIC-10` is therefore first actionable.

The row's missing live dispatch proof now exists in already accepted,
independently validated exact-package evidence. ASIC-003 proves mining-ready
production work, a qualified correlated result, and an accepted submit on the
same Ultra 205 attempt that used the live Stratum production session. Current
pure tests prove the pool-derived registry enqueues valid jobs, preserves pool
context through dispatch, advances generation on reconnect or clean-jobs,
invalidates stale work, and redacts raw context. The live runtime owns one
`ProductionWorkRegistry`.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the
active tracker/checklist, and bounded lessons for protected evidence,
private-first classification, earliest failure, standing authorization, and
agent-runtime timing. Active lesson inputs total 31,758 bytes, so headings
were inventoried and relevant complete blocks loaded; less-relevant blocks
were omitted under the deterministic budget. The August lesson-audit baseline
remains current and no new audit trigger is due.

## Scope and non-scope

Advance only `ASIC-10`. Produce a source-bound evidence summary that joins the
accepted ASIC-003 work-send projection, its ASIC-002 predecessor, current
registry/session tests, current source identity, pinned reference behavior, and
explicit non-claims. No new runtime implementation or projector is needed
because the typed registry and accepted live dispatch already exist.

Bind these accepted projections exactly:

- ASIC-002 initialization:
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 work send:
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`

The evidence summary may contain only repository paths, commits, digests,
closed labels, booleans, counts, and command outcomes. It must contain no raw
ASIC frames, nonce/work/share values, pool or credential data, endpoints,
ports, USB/network identity, telemetry, logs/payloads, commands, PIDs, traces,
or protected identifiers.

This plan authorizes local tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Independently validate the accepted ASIC-002 and ASIC-003 projections.
- [ ] Run current production-work registry and production-session tests.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md` with exact digests,
      conclusions, and non-claims.
- [ ] Commit the evidence as `SOURCE_COMMIT`, transition only ASIC-10, sync
      progress, archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-stratum production_work`
- `cargo test -p bitaxe-stratum production_session`
- the two existing Rust evidence validators over absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, file-size, selector, sensitive-value, source-diff, and final diff
checks.

Promotion requires accepted evidence for one Ultra 205 BM1366 production chain
with pool-derived work registration, live production dispatch, a qualified
correlated result, accepted response, safe stop, cleanup, trusted
identity/safety, current registry source/tests, independent validation, and
redaction. Checklist notes must include the Phase 28 summary path, `accepted
share hardware`, `redaction_status: passed`, and `exact_non_claims`, and must
omit blocker terms.

On success create `RESULT.md`, commit evidence without checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `ASIC-10` to
`verified` with `unit,golden,workflow,hardware-smoke,hardware-regression`, sync
progress, archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave ASIC-10 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify result-correlation policy beyond the accepted
predecessor, submit-response classification ownership, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
clean-jobs or reconnect on live hardware, other ASICs/boards, arbitrary
pools/profiles, unbounded mining, OTA/recovery, or release readiness. It does
not promote ASIC-11, ASIC-12, STR-08, STR-09, SAFE-12, or SAFE-13.
