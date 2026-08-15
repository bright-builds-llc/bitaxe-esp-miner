# Parity work plan

- Run ID: `20260815T192115Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `756749d147109d3850e8fc6ac84af9ee940ad152`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260815T185700Z-THR-001/PLAN.md`

## Selection and prerequisite

The clean synchronized selector has no open plan and ranks THR-001 first; no
higher-ranked candidate was skipped. Attempt-005 is consumed and immutable.
The pushed correction at `6f637e87` now latches the first real fault proof,
parses only canonical ESP-IDF INFO payloads, and makes the already-retained
ordered thermal-stimulus witness available to a late monitor through bounded,
package-scoped replay. Production-shaped fake, late-attachment, malformed-
envelope, timeout, and real-child regressions pass, as does the ESP32-S3 build.

This plan authorizes one fresh attempt-006 only after every ordinal, path, plan
digest, private intent, validator, fixture, test, generated binding, and Bazel
runfile is advanced together. That implementation must pass all software gates,
be committed and pushed separately, and produce a clean exact package before
detector or device access.

Creating this first post-closure plan exposed one deterministic selector defect:
the reconciler requires an explicit link between every historical same-row plan
whenever any newer plan is open, even across a valid terminal closure. THR-001
has one intentionally separate closed-plan boundary between the attempt-002 and
attempt-003 histories, and immutable plans cannot be retroactively linked. The
implementation may make the minimal functional-core correction that treats the
latest valid terminal closure as a lineage reset while retaining fail-closed
link checks among two or more plans after that boundary. A regression must use
the production history shape and prove multiple unclosed unlinked plans still
fail.

## Exact effect and evidence contract

Attempt-006 reuses the consume-before-use private
`esp-thermal-fault-stimulus-intent-v1` contract, bound to board 205, ordinal 6,
the exact source/reference/package/app ELF, this immutable plan, stimulus kind
`emc2101_invalid_sample`, and sample count 5. The intent and runtime artifacts
remain mode-0600 regular non-symlinks beneath fresh mode-0700 ignored roots.
Missing, malformed, misplaced, wrong-mode, wrong-plan, wrong-package, wrong-
board, wrong-ordinal, replayed, or mismatched inputs fail before effects.

The one campaign may perform one exact-package USB flash/reset with private
Wi-Fi and the one-shot NVS stimulus, five one-second typed invalid-temperature
overlays while the production owner continues successful real EMC2101 reads,
bounded retained-marker replay, read-only same-origin HTTP/WebSocket/log
observation, then one ordinary exact-package restoration flash/reset and
verification. The tuple must be erased and committed before use and must not
replay after restoration. The workflow preserves the earliest typed primary
failure and always attempts ordinary restoration after a post-flash failure.

The candidate and final public
`bitaxe-emc2101-thermal-fault-evidence-v1` projection may be published only at
`docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-006.json`
after independent validation. It may contain only closed aggregate categories,
counts, hashes, booleans, and redaction status. It must not expose raw
temperatures, acquisition stamps, boot sessions, settings, hostnames, origins,
ports, USB/network identifiers, credentials, logs, commands, PIDs, private
paths, response bodies, or traces.

No physical heating, fan/voltage/frequency/power change, mining, pool input,
ASIC work, raw I2C/GPIO, public diagnostic setter, erase, OTA, rollback, power
cycle, external UART, pin/pad/header manipulation, injected electrical signal,
non-205 device, retry, attempt-007, or claim of physical overheat/open/short
fault is authorized.

## Implementation and verification

- [ ] Correct terminal-closure lineage reconciliation with one red/green
      production-shape regression; do not edit any immutable historical plan.
- [ ] Advance only the attempt ordinal, protected roots, public projection,
      immutable plan path/digest, fixtures, tests, generated command binding,
      and Bazel runfile from consumed attempt-005 to fresh attempt-006.
- [ ] Prove strict admission, request-once behavior, exact five successful-real-
      read overlays, canonical ordered/replayed markers, fresh recovery,
      restoration, primary-failure precedence, protected modes, atomic
      withholding, bare-marker rejection, and redaction.
- [ ] Run focused and mandatory gates, simplify and review the diff, then commit
      and push before packaging, detection, or hardware use.
- [ ] Build and admit the exact clean package, run one detector, and invoke the
      attempt-006 campaign exactly once.
- [ ] Promote THR-001 only if the full independently validated hardware-
      regression and restoration quorum passes; otherwise withhold evidence,
      preserve `implemented`, record the typed category, and stop without
      attempt-007.

Required gates are the ordered Cargo format, strict Clippy, all-target build,
all-feature tests, Bright Builds, `just build`, `just test`, `just parity`,
`just parity-progress`, redaction, pinned reference, plan/task/generated-
binding checks, exact package, protected modes, sensitive-output review, and
`git diff --check`. The clean selector must resume only this exact open plan
after the lineage correction.

## Sole hardware command and stop rules

After the implementation is pushed, HEAD is clean and synchronized, and the
exact package exists, prove both private roots and the public projection are
absent. Create only the protected wrapper streams, then run `just
detect-ultra205` once. Continue only if exactly one board-205 ESP32-S3 is
admitted and holder/cleanup checks pass. Invoke exactly once:

`just capture-emc2101-thermal-fault-evidence --private-root scratch/thr001-emc2101-fault/attempt-006 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101-fault/wrapper-006/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-006.json --capture-timeout-seconds 120`

Starting that command consumes attempt-006. Promote only if the projection
binds board 205, ordinal 6, exact clean identities and plan, detector admission,
one-shot consumption, successful real reads before/during/after, exactly five
overlays, one canonical complete ordered `baseline_ready`, `fault_observed`,
and `recovered` witness (direct or bounded retained replay), the expected typed
fault projection, final fresh safe HTTP/WebSocket truth, no replay after
ordinary restoration, disabled mining/control, cleanup, protected modes,
independent validation, redaction, and the prior read-only projection.

Non-ready device results map to `hardware_blocked`, malformed or incomplete
evidence to `evidence_invalid`, child timeout to `timeout`, and launch failure
to `process_failed`; restoration remains secondary. Any missing member,
detector ambiguity, unsafe state, recovery failure, cleanup failure, identity
drift, or redaction failure withholds evidence and stops without retry.
