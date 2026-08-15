# Parity work plan

- Run ID: `20260815T042822Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `8f21102fd9ea71ec37dd1d09e457e933c60264df`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260815T041103Z-API-009/PLAN.md`

## Selection

Clean synchronized HEAD has no open plan and API-009 remains first. The sole
attempt-026 passed the repaired dismissal ordering through notification clear
but failed count preservation before IDENTIFY. The sealed categorical record
and production source agree: the host compares post-dismissal count with the
first active-notification count, although pause convergence permits an in-
flight result to advance the cumulative count before dismissal.

## Scope and non-scope

Add a red state-machine regression with an initial notification count lower
than the paused pre-dismissal count. At the joined safe-stopped pause, require
a positive current count, capture it immediately before issuing the one
dismissal request, and compare the first cleared readback with that captured
value. Preserve genuine notification provenance, notification clear, all
request counts, pause/safe-stop ordering, IDENTIFY, resume, recovery, terminal,
sealing, privacy, and evidence semantics.

This plan is software-only. It may modify source, tests, task records, worklog,
and closure; run loopback processes; and build firmware. It may not read
credentials or protected traces; access detector, USB, device/network,
display, mining, or hardware-control interfaces; create public evidence;
promote API-009; run attempt-027; or perform direct UART, pin, destructive,
fault-injection, OTA, rollback, erase, reset, or power effects.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before source edits.
- [ ] Add a red regression proving an increment during pause convergence does
      not invalidate dismissal preservation of the paused pre-request count.
- [ ] Capture the positive paused count immediately before the sole dismissal
      POST and preserve all existing fail-closed ordering and recovery.
- [ ] Run focused command-effects and campaign tests plus all mandatory gates.
- [ ] Close with API-009 still `implemented`; attempt-027 requires a separate
      immutable exact-package hardware plan after this fix is pushed.

Before plan and final source commits, run in order: `cargo fmt --all`, strict
Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run redaction, reference,
firmware, immutable-plan digest, unique-task, selector, sensitive-output, and
diff checks.

Promotion is prohibited. Accepted completion is the regression-backed root-
cause fix with every software gate green and no hardware access.
