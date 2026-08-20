# Parity work plan

- Run ID: `20260820T041751Z-ASIC-12`
- Parity row: `ASIC-12`
- Initial status: `implemented`
- Source commit: `7b6cb38eb7c7819e518cde01158d1cf47d50822f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic12-fail-closed-redaction`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `ASIC-12`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`ASIC-12` is therefore first actionable.

The row's missing live production proof now exists in accepted, independently
validated exact-package evidence. ASIC-002 proves mining-ready initialization;
ASIC-003 proves production-gated work plus an accepted response; ASIC-004 proves
strict result parsing and compatible correlation; ASIC-005 proves the bounded
production UART transport. Current pure tests enumerate closed blocker labels
and redact work, result, target, and submit context. One remaining regression
gap is that the exact public fail-closed status line is rendered only inside the
ESP-IDF firmware shell and is not exercised by a host-runnable unit test.

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

Advance only `ASIC-12`. Move the exact production-status rendering decision
from the ESP-IDF logging shell into the host-testable `bitaxe-asic` functional
core, exercise every closed blocker label through the public fail-closed line,
and leave the firmware adapter as a thin logger. Produce a source-bound evidence
summary joining that current behavior to the already accepted ASIC-002 through
ASIC-005 projection chain and pinned reference behavior.

Bind these accepted projections exactly:

- ASIC-002 initialization:
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 work send:
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC-004 result parsing:
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`
- ASIC-005 serial transport:
  `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`

The evidence summary may contain only repository paths, commits, digests,
closed labels, booleans, counts, and command outcomes. It must contain no raw
ASIC frames, nonces, work/share values, pool or credential data, endpoints,
ports, USB/network identity, telemetry, logs/payloads, PIDs, traces, or
protected identifiers.

This plan authorizes local source edits, tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add host-runnable exact-line coverage for all production status states
      and all eleven fail-closed blocker reasons in the pure ASIC crate.
- [ ] Make the firmware status adapter consume the pure renderer without
      changing its logging levels or observable strings.
- [ ] Independently validate the accepted ASIC-002 through ASIC-005 projections
      and run current production-work/session redaction and blocker coverage.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md`; commit them as
      `SOURCE_COMMIT`, transition only ASIC-12, sync progress, archive this task,
      final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-asic production`
- `cargo test -p bitaxe-stratum production_work`
- `cargo test -p bitaxe-stratum production_session`
- the four existing Rust evidence validators over absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, selector, sensitive-value, source-diff, and final diff checks.

Promotion requires accepted evidence for one Ultra 205 BM1366 production chain
with mining-ready initialization, live production work/result UART, a qualified
correlated result, accepted response, safe stop, cleanup, trusted identity and
safety, plus current exact fail-closed status rendering for every typed blocker,
context-redaction tests, independent validation, and public-artifact redaction.
Checklist notes must include the Phase 28 summary path, `accepted share
hardware`, `redaction_status: passed`, and `exact_non_claims`, and must omit
blocker terms.

On success create `RESULT.md`, commit implementation and evidence without a
checklist change and save that full commit as `SOURCE_COMMIT`; transition only
`ASIC-12` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression`, sync progress,
archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave ASIC-12 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not claim hardware fault injection for every blocker, arbitrary
diagnostic builds, nonzero version-mask or multi-midstate breadth, arbitrary-
load serial behavior, rejected-share hardware, frequency transitions,
voltage/fan/thermal behavior, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, or release readiness. It does not promote
STR-08, STR-09, SAFE-12, or SAFE-13.
