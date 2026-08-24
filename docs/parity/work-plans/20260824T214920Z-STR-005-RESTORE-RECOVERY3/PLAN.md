# Parity work plan

- Run ID: `20260824T214920Z-STR-005-RESTORE-RECOVERY3`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `5e1114f30da98cbf9e96fc8f5490185731337abc`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-installed-package-recovery-003`
- Continues: `docs/parity/work-plans/20260824T012436Z-STR-005-RESTORE-RECOVERY2/CLOSURE.md`
- Predecessor closure SHA-256: `e4565fa9aa9adf1a4c667db3a82ca238c2b8fd2e9f402f33cac36d74f5b87d27`

## Objective

Make the independent restore-validator child boundary closed, protected, and
reproducible before one fresh recovery-003 attempt. Recovery must retain a
private diagnostic receipt covering the launcher, exit, timeout, working
directory, bounded output counts/digests, and acceptance result. A real Bazel
launcher regression must prove the exact child path accepts a valid fixture and
classifies rejection without leaking values. Only then may recovery-003 run and
unlock the still-unused attempt-004 Stratum V2 campaign if readiness succeeds.

## Changed boundary

Recovery-002 resolved the firmware readback timeout and created a valid exact
eight-range snapshot bundle, but its in-owner validator child returned nonzero
while the same retained inputs passed a bounded post-run validator invocation.
The owner retained no child receipt, so that discrepancy could not be safely
classified. Recovery-003 adds the missing diagnostic owner and real-launch
regression. It is not an unchanged recovery retry.

## Implementation

- [ ] Add a bounded validator-child runner that always returns a typed outcome
      and writes one mode-`0600` private receipt before success or failure.
- [ ] The receipt contains only closed launcher/cwd categories, exit code,
      timeout/output-limit booleans, bounded stdout/stderr byte counts and
      SHA-256 digests, invocation digest, validation acceptance, and source/plan
      provenance. It never contains arguments, paths, output text, runtime
      values, credentials, endpoints, or flash bytes.
- [ ] Add a real Bazel launcher test using protected synthetic package inputs.
      Prove accepted, rejected, launcher failure, timeout, output limit, working-
      directory binding, protected mode, and secret-canary exclusion.
- [ ] Bind recovery, restore admission, campaign, flash restore, runfiles, task,
      and tests to this plan, `recovery-003`, and the fresh readiness projection.
- [ ] Preserve bounded package search, one timestamp-pinned rebuild, exact
      eight-range 460800-baud snapshot fallback, installed-runtime continuity,
      independent projection validation, cleanup, and all prior non-claims.

## Commands and effects

After all software gates, clean commit/push, and exact current package build:

1. `just detect-ultra205`
2. `just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-003 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-003.json --redact-evidence`
3. `just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-003/restore-bundle.private.json`
4. `just stratum-v2-runtime-admission --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-003/restore-bundle.private.json`
5. Only after commands 1-4 succeed and both campaign targets remain absent:
   `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-003/restore-bundle.private.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Commands 1-4 perform no campaign effect and do not consume attempt-004.
Recovery-003 may reset/re-enumerate USB for readback but performs no write.
Command 5 consumes attempt-004 when it starts and retains the existing
180-second lease, host-owned local Noise fixture, 400 MHz/1100 mV/100% fan
ceiling, one accepted-share ceiling, continuous safety/watchdog checks, safe
stop, exact original bundle/settings restoration with `mineonboot=false`, and
complete cleanup.

Snapshot fallback reads only bootloader `0x0/0x8000`, partition table
`0x8000/0x1000`, PHY `0xf000/0x1000`, factory `0x10000/0x400000`, WWW
`0x410000/0x300000`, OTA 0 `0x710000/0x400000`, OTA 1
`0xb10000/0x400000`, and OTA data `0xf10000/0x2000`. It never reads NVS or
coredump. Snapshot restoration remains ineligible until command 5 begins and
current-package flashing completes or becomes uncertain; it writes exactly the
eight ranges once, then seeds Wi-Fi and restores settings/theme.

## Evidence, verification, and completion

Private roots/directories are mode `0700`; identity, bundle, receipt, logs, and
binaries are mode `0600`. Public projections contain only closed categories,
booleans, bounded counts/durations, bundle kind, digests, current provenance,
and redaction status. They never contain runtime values, paths, ports, logs,
credentials, endpoints, flash bytes, validator output, or raw API bodies.

Before hardware: ordered Cargo format/clippy/build/test, Bright Builds, all
Bazel tests, dedicated real-launch validator tests, canonical firmware
build/package, parity/progress, redaction, reference cleanliness, selector/plan
lineage, sensitive-value review, and final diff review must pass on clean pushed
source.

Recovery-003 runs once. Failure withholds command 5, creates `CLOSURE.md`, keeps
`STR-005` implemented, and authorizes no retry. Attempt-004 runs once only after
accepted preflight/runtime admission. Every campaign terminal safe-stops first,
restores original bundle/settings once, validates exact runtime and cleanup,
and preserves the earliest category. Failure withholds `RESULT.md`. One
independently accepted campaign plus exact restoration creates `RESULT.md`,
transitions only `STR-005` to `verified` with
`unit,golden,workflow,hardware-regression`, synchronizes progress, archives
completed tasks, final-verifies, commits, and pushes.

Raw NVS/coredump capture, new-baseline flash, external pools, direct UART/pins,
fault injection, OTA, erase, unlisted writes, unbounded mining, and attempt-005
are prohibited effects and explicit non-claims.
