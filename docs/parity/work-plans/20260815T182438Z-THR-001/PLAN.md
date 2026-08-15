# Parity work plan

- Run ID: `20260815T182438Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `4fdd17db71c448d916eb866d58d0384c2f7a21b1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260815T181534Z-THR-001/PLAN.md`

## Selection and prerequisite

The clean synchronized selector ranks THR-001 first. Attempt-004 is consumed
and immutable. The pushed software correction at `4fdd17db` now has a fast
production-order regression proving the complete five-overlay fault and fresh-
recovery sequence while preserving ordinary one-second stale semantics.

This plan authorizes one fresh attempt-005 only after its ordinal, paths, and
this exact plan digest are advanced throughout the strict private intent,
flash admission, host transaction, deterministic and real-child tests, and
generated command binding; the complete change must pass all gates, be
committed and pushed separately, and produce a clean exact package before USB
admission.

## Exact effect and evidence contract

Attempt-005 reuses the established consume-before-use private
`esp-thermal-fault-stimulus-intent-v1` contract, bound to board 205, ordinal 5,
the exact source/reference/package/app ELF, this immutable plan, stimulus kind
`emc2101_invalid_sample`, and sample count 5. The intent and all runtime
artifacts remain mode 0600 regular non-symlinks beneath fresh mode-0700 ignored
roots. Missing, malformed, misplaced, wrong-mode, wrong-plan, wrong-package,
wrong-board, wrong-ordinal, replayed, or mismatched inputs fail before effects.

The one campaign may perform one exact-package USB flash/reset with private
Wi-Fi and the one-shot NVS stimulus, five one-second typed invalid-temperature
overlays while the owner continues and requires successful real EMC2101 reads,
read-only same-origin HTTP/WebSocket/retained-log observation, then an ordinary
exact-package restoration flash/reset and verification. The tuple must be
erased and committed before use and must not replay after restoration. The
workflow preserves the earliest typed failure and always attempts ordinary
restoration after a post-flash primary failure.

The candidate and final public
`bitaxe-emc2101-thermal-fault-evidence-v1` projection may be published only at
`docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-005.json`
after independent validation. It may contain only closed aggregate categories,
counts, hashes, booleans, and redaction status. It must not expose raw
temperatures, acquisition stamps, boot sessions, settings, hostnames, origins,
ports, USB/network identifiers, credentials, logs, commands, PIDs, private
paths, values, response bodies, or traces.

No physical heating, fan/voltage/frequency/power change, mining, pool input,
ASIC work, raw I2C/GPIO, public diagnostic setter, erase, OTA, rollback, power
cycle, external UART, pin/pad/header manipulation, injected electrical signal,
non-205 device, retry, attempt-006, or claim of physical overheat/open/short
fault is authorized.

## Implementation and verification

- [ ] Advance only the attempt ordinal, protected roots, public projection,
      immutable plan path/digest, fixtures, tests, and generated command binding
      from consumed attempt-004 to fresh attempt-005.
- [ ] Prove strict admission, request-once behavior, exact five successful-real-
      read overlays, ordered markers, fresh recovery, restoration, primary-
      failure precedence, protected modes, atomic withholding, and redaction.
- [ ] Run focused and mandatory gates, simplify and review the diff, then commit
      and push before packaging, detection, or hardware use.
- [ ] Build and admit the exact clean package, run one detector, and invoke the
      attempt-005 campaign exactly once.
- [ ] Promote THR-001 only if the full hardware-regression and restoration
      quorum passes; otherwise withhold evidence, preserve `implemented`, record
      the typed category, and stop without attempt-006.

Required gates are the ordered Cargo format, strict Clippy, all-target build,
all-feature tests, Bright Builds, `just build`, `just test`, `just parity`,
`just parity-progress`, redaction, pinned reference, plan/task/generated-binding
checks, exact package, protected modes, sensitive-output review, and
`git diff --check`.

## Sole hardware command and stop rules

After the implementation is pushed, HEAD is clean and synchronized, and the
exact package exists, prove both private roots and the new public projection are
absent. Create only the protected wrapper streams, then run `just
detect-ultra205` once. Continue only if exactly one board-205 ESP32-S3 is
admitted and holder/cleanup checks pass. Invoke exactly once:

`just capture-emc2101-thermal-fault-evidence --private-root scratch/thr001-emc2101-fault/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101-fault/wrapper-005/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-005.json --capture-timeout-seconds 120`

Starting that command consumes attempt-005. Promote only if the projection
binds board 205, ordinal 5, exact clean identities and plan, detector admission,
one-shot consumption, successful real reads before/during/after, exactly five
overlays, ordered `baseline_ready`, `fault_observed`, and `recovered` markers,
the expected typed fault projection, final fresh safe HTTP/WebSocket truth, no
replay after ordinary restoration, disabled mining/control, cleanup, protected
modes, independent validation, redaction, and the prior read-only projection.

Non-ready device results map to `hardware_blocked`, malformed or incomplete
evidence to `evidence_invalid`, child timeout to `timeout`, and launch failure
to `process_failed`; restoration remains secondary. Any missing member,
detector ambiguity, unsafe state, recovery failure, cleanup failure, identity
drift, or redaction failure withholds evidence and stops without retry.
