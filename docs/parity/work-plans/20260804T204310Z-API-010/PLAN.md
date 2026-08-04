# Parity work plan

- Run ID: `20260804T204310Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `ca98393afed140576e411d3322df69df53a766f8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-004`

## Selection

The clean synchronized `main` branch and deterministic selector resumed only
`docs/parity/work-plans/20260804T200849Z-API-010/PLAN.md`. This immutable plan
is its explicitly linked continuation for one fresh progress-backed ordinal;
no other parity candidate is considered while the `API-010` lineage is open.

`attempt-003` stopped at an initial flash-monitor `process_failed` boundary
whose stdout/stderr, effect state, and closed child marker were not durable.
Source commit `8c93b1b73a0e62ba4fecb1ae46604d30ac29916a`
and finalization commit `ca98393afed140576e411d3322df69df53a766f8`
enable and regression-test the existing private flash-effect result, bounded
exit/timeout facts, allowlisted dual-evidence marker, and public theme failure
projection through real child processes. That verified production-boundary
instrumentation is the new information required by the hardware-attempt policy.

The ignored Wi-Fi credential input exists without being read. The private
attempt root, private wrapper root, and public projection destination for
`attempt-004` are absent.

## Scope and non-scope

Freeze the exact package from the pushed planning commit, privately capture one
detector admission for exactly one Ultra 205, and run exactly one bounded
`verify-theme-durability` transaction. The workflow may perform its admitted
exact-package flash, read the original theme, POST one generated non-secret
alternate theme, confirm immediate readback, request one normal software
restart through `device-session reboot-live`, prove same-device exact-build
boot ordinal `N+1` and persisted theme state, restore the original theme, and
confirm restoration and cleanup before publishing.

The supervisor-owned attempt root is
`scratch/api010-theme-durability/attempt-004`. It must be absent before launch,
mode `0700` after creation, and contain only mode-`0600` private artifacts. The
caller-owned sibling is `scratch/api010-theme-durability/wrapper-004`; its
detector and wrapper streams are mode `0600` and never pre-create the child.
The only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after the closed workflow succeeds and semantic redaction passes.

Do not read or expose credentials; discover network origins; change Wi-Fi or
pool configuration; mine; enable ASIC work; change voltage, frequency, fan,
thermal, or power controls; exercise display input; perform OTA or raw
partition writes; use direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals; claim installed AxeOS browser behavior; or run
a second hardware attempt. The repository hardware, device-session, evidence,
architecture, code-shape, verification, testing, Rust, and TypeScript standards
govern this plan.

## Implementation

- [ ] Commit and push this immutable plan and complete active task contract
      after all pre-hardware software gates pass.
- [ ] Build the exact Ultra 205 package, capture one private detector transcript
      with mode `0600`, and require exactly one admitted board 205.
- [ ] Run exactly one `attempt-004` capture with a 360-second workflow timeout
      and a shell wall clock exceeding 420 seconds.
- [ ] Validate private modes, the closed public projection or typed failure,
      exact package identity, restoration/cleanup facts, and non-claims.
- [ ] Create `RESULT.md` and transition only `API-010` to `verified` if every
      criterion passes; otherwise record the earliest closed terminal signature,
      withhold evidence, keep `implemented`, and stop without retry.

## Verification and promotion

Before hardware, run focused automation/flash real-process regressions and the
repository-required checks. After the pushed plan, run only the exact commands
recorded in the active task. After the single attempt, run formatting, strict
Clippy, all-target/all-feature Cargo build and tests, Bright Builds, `just test`,
`just parity`, `just parity-progress`, semantic redaction, pinned-reference
cleanliness, immutable-plan checks, sensitive-output review, private-mode
checks, and diff review.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact clean
package and reference, one admitted board 205, a ready closed device session for
the same physical device, one restart request, exact build recovery, changed
boot session, ordinal `N+1`, software reset, immediate and post-restart theme
equality, exact original-theme restoration, disabled mining and hardware
control, complete cleanup, and passed redaction. It may contain no origin, URL,
theme value, hostname, port, USB/network identifier, credential, raw child,
HTTP/serial/process material, or private path.

For an initial-child failure, the authoritative signature is the automation
category plus `stage`, `flash_monitor_exit_code`,
`flash_monitor_timed_out`, `flash_monitor_terminal_marker`,
`flash_effect_result_status`, and `flash_effect_status`. Missing or malformed
closed facts after their verified fix select `stop_repeated_boundary`. Any
other failure records its first closed signature, withholds promotion, and ends
this invocation without an unchanged retry.
