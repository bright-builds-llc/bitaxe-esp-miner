# Parity work plan

- Run ID: `20260818T040753Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `7d4b1670edee357c8ec93b84c7cdc8693ce5db9c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
has no open plan. Candidate order begins `SELF-001`, `BAP-002`, then
`STAT-001`. `SELF-001` remains dependency- and safety-blocked by the absence of
a production-safe hardware self-test route and complete fan/voltage/power/ASIC/
reboot recovery contract. `BAP-002` remains blocked by not-started `BAP-001`
UART/request/subscription ownership and an unauthorized external electrical
accessory path.

`STAT-001` is first actionable. Audited attempt-018 passed plan, task, source,
package, detector, watchdog, safety, privacy, and cleanup admission, then
rebooted after 273,286 active milliseconds. The sealed session changed with
closed reset category `panic`; no raw panic line or backtrace is retained, so
the task, panic mechanism, and root cause cannot be distinguished. A blind
attempt-019 is prohibited. The next useful work is a complete in-memory,
value-free panic discriminator at the existing serial analyzer boundary.

Material guidance includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards/core/architecture.md`, `standards/core/code-shape.md`,
`standards/core/verification.md`, `standards/core/testing.md`,
`standards/languages/rust.md`, the active task/checklist, and bounded lessons
for diagnostic completeness, runtime stack capacity, real process boundaries,
redaction, evaluator identity, earliest failure, and hardware retry discipline.
The over-budget lesson ledger was inventoried, omitted blocks were disclosed,
and its current above-limit audit baseline has no new trigger.

## Scope and non-scope

Advance only `STAT-001` through local software changes. Add a pure classifier
for complete serial lines that recognizes the closed ESP-IDF/Rust panic
families `stack_overflow`, `stack_smashing`, `heap_corruption`, `assertion`,
`abort`, `rust_panic`, and `guru_meditation`, plus `unknown` when a panic reset
has no observed signature. Classify only closed known task families such as the
production mining owner/ASIC worker, live WebSocket, deferred effects, safety,
sensor, fan, statistics, Wi-Fi reconnect, HTTP server, and main task; use
`other` for non-allowlisted names. Retain only first-signature labels and
saturating counts, never raw lines, task names, addresses, backtraces, or
payload fragments.

Integrate the classifier into campaign serial diagnostics, rotate the private
diagnostic schema, add fragment/coalescing/CRLF and privacy regressions, and
bind the serial reducer plus diagnostic vocabulary into the transitive
hashrate evaluator source inventory and Bazel runfiles. Rotate the expected
source-path count and preserve result v16/network v11, closed reset category,
earliest failure precedence, protected modes, seals, legacy units, and public
projection behavior.

Software-only authorization: repository source, fixtures, deterministic tests,
builds, documentation, and Git operations. Do not read protected attempt-018
artifacts beyond the already recorded closed facts; do not access credentials,
detector/device/USB/network runtime, private values, or projection candidates.
Do not flash, reset, monitor, mine, actuate, update, erase, inject faults,
manipulate power, use external UART/BAP, touch electrical interfaces, reuse
attempt-018, or create/run attempt-019.

## Implementation

- [ ] Add a pure closed panic-signature and task-family classifier that stores
      no raw serial text and handles fragmented/coalesced/CRLF input through the
      existing complete-line analyzer.
- [ ] Publish only closed signature/task/count fields in private serial
      diagnostics, rotate its schema, and preserve earliest observation order.
- [ ] Bind every new reachable reducer/vocabulary source into the hashrate
      evaluator inventory and Bazel runfiles; update the exact source count.
- [ ] Add behavior, privacy, schema, source-drift, generated-contract, and real-
      process regressions; run every focused and mandatory gate.
- [ ] Create `WORKLOG.md` and a non-verifying `CLOSURE.md`; leave `STAT-001`,
      checklist, progress history, and README unchanged.

## Verification and promotion

Focused verification: panic classifier unit cases for every closed signature
and task family; fragmented/coalesced/CRLF analyzer coverage; first-signature
precedence; saturating count; raw-line/address/backtrace absence; diagnostics
schema; source inventory/runfiles and drift; hashrate automation/validator;
campaign result/network compatibility; real firmware/package build;
`just verify-redaction`; `just verify-reference`; diff review.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

This plan cannot verify `STAT-001`: it authorizes no hardware and cannot
retroactively classify attempt-018. Success means the complete value-free
diagnostic passes every gate and a closure records the exact next hardware
prerequisites. Do not request a no-op checklist transition or progress sync.
Any later hardware plan must start from a clean pushed diagnostic commit,
define a fresh ordinal and complete authority contract, and prove a materially
changed information boundary before device access.

## Non-claims

No claim about the cause of attempt-018's panic, task identity, stack overflow,
hashrate accuracy, complete live runtime, attempt-019 eligibility, or parity
verification. No hardware, credentials, external services, private evidence,
or public projection are used by this plan.
