# Parity work plan

- Run ID: `20260814T033105Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `bb1b66b4cf5104290a67f424f2f5abb00c05e779`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T023531Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-014 proved exact-package flashing, runtime
identity, protocol admission, fixture lifecycle, and USB cleanup, but closed
`safety_stale` after retaining only two milliseconds active. Its final
readiness transition was armed and hardware-stopped behind `operator_paused`.

The production source and focused host test reproduce that exact boundary:
the command-effects tracker forces `Run` before its first active snapshot, then
immediately follows the unchanged boot-time `Paused` request derived from
`mineonboot=0`. No command has occurred at that point, so the initial safety
default is incorrectly interpreted as an operator pause. This software-only
plan fixes that startup-intent ownership defect. It does not authorize
attempt-015 or any device access.

## Scope and non-scope

Add a lease-scoped command-effects bootstrap that changes only the current
boot's requested operator intent to `Run` after command-effects admission and
before the production owner loop begins. Keep persisted `mineonboot=0`, NVS,
the public setting, and ordinary boot behavior unchanged. After the first
active snapshot, explicit pause and resume API commands must still replace the
requested intent authoritatively; campaign consumption, failure, missing
admission, and later boots must remain paused by the existing owner gate.

Prefer a narrow typed method on the requested-intent owner plus a pure campaign
status predicate used by the production adapter. Do not add a second state
machine, inferred command flag, timing delay, NVS mutation, implicit HTTP
request, evidence relaxation, or backward-compatibility shim. Do not access a
credential, detector, protected attempt trace, USB/device/network interface,
display, mining hardware, or public evidence path. No attempt-015, flash,
monitor, reset, restart, erase, OTA, power cycle, direct UART, or pin/pad/GPIO
action is authorized.

## Implementation and verification

- [ ] Commit and push this immutable software-only plan/task checkpoint before
      editing production or test source.
- [ ] Turn the real campaign-status/runtime-intent seam red: disabled
      mine-on-boot plus admitted command effects must remain `Run` across the
      first active snapshot without pretending an operator command occurred.
- [ ] Add the lease-scoped requested-intent bootstrap and invoke it exactly
      once after admitted command-effects tracker construction and before the
      owner loop reads readiness.
- [ ] Prove explicit pause and resume remain authoritative, consumed or absent
      leases remain paused, non-command campaigns are unchanged, and persistent
      mine-on-boot/NVS state is never mutated.
- [ ] Run focused campaign-status, requested-intent, source-ownership,
      production-session, API-command, and real firmware targets plus every
      mandatory, privacy, reference, selector, digest, sensitive-output, and
      diff gate.
- [ ] Close this plan with API-009 still `implemented`; require a later clean
      selector and separate immutable contract before any hardware ordinal.

Before plan commit and software closure commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run the focused Bazel targets,
`just verify-redaction`, `just verify-reference`, `just build`, immutable plan
digest, unique task binding, selector ownership, source-sensitive scans,
`git diff --check`, and complete diff review.

Success means the red regression turns green with one explicit owner seam:
the command-effects lease starts with a current-boot `Run` request despite the
persisted disabled boot preference, remains running across its first active
snapshot, follows later explicit pause/resume commands, and returns to safe
paused behavior when the lease no longer authorizes actuation. This plan makes
no hardware or API-009 parity-verification claim.
