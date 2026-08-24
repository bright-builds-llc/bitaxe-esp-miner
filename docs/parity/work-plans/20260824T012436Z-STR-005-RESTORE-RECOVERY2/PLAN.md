# Parity work plan

- Run ID: `20260824T012436Z-STR-005-RESTORE-RECOVERY2`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `1dd7f3e32a9376c8c5578ec1e27186e2f956e9d4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-installed-package-recovery-002`
- Continues: `docs/parity/work-plans/20260824T000902Z-STR-005-RESTORE-RECOVERY/CLOSURE.md`
- Predecessor closure SHA-256: `0da849a91fcda59773688339984f81a262b0da900b9dbcb3dc8fefdf7070f888`

## Objective

Use the regression-backed explicit 460800-baud, 600-second-per-range readback
path to make one fresh recovery-002 attempt for the firmware currently
installed on the detected Ultra 205. Independently validate the exact rollback
bundle, then run the still-unused attempt-004 local-fixture Stratum V2 campaign
only if every no-effect readiness gate passes. Never capture raw NVS or
coredump bytes, retry an unchanged boundary, or adopt a new baseline.

## Changed boundary

Recovery-001 proved that bounded search and the one timestamp-pinned rebuild do
not yield an exact installed package. Its snapshot fallback completed the three
small ranges and then exhausted a 300-second implicit-baud factory read. Current
source pre-creates every target at mode `0600`, reasserts that mode after every
child outcome, passes `--baud 460800`, and permits 600 seconds per range. Pure
tests bind the exact corrected command. Recovery-002 is therefore a changed,
regression-backed boundary, not an unchanged retry.

## Implementation

- [ ] Bind the recovery owner, campaign restore bundle, historical restore
      adapter, and tests to this task/plan, the fresh private recovery-002 root,
      and a fresh public readiness projection.
- [ ] Keep package search bounded to repository/Bazel locations and the
      timestamp-pinned rebuild limited to one owned detached worktree. Clean the
      worktree and every child process on interruption or completion.
- [ ] If no exact package is recovered, read exactly the eight previously
      approved firmware ranges at explicit 460800 baud with a 600-second
      per-range ceiling. Pre-create and retain all private targets at `0600`.
- [ ] Independently validate range allowlisting, NVS/coredump exclusion,
      partition layout, running-image identity, digests, containment, file
      modes, source/plan binding, and the closed public readiness projection.
- [ ] Preserve the existing attempt-004 safety, fixture, safe-stop,
      restoration, exact runtime/settings proof, cleanup, privacy, and
      independent validation contracts without adding another campaign ordinal.

## Commands and effects

After all software gates, clean commit/push, and exact current package build:

1. `just detect-ultra205`
2. `just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-002 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-002.json --redact-evidence`
3. `just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-002/restore-bundle.private.json`
4. `just stratum-v2-runtime-admission --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-002/restore-bundle.private.json`
5. Only after commands 1-4 succeed and both attempt/projection targets remain
   absent: `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-002/restore-bundle.private.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Commands 1-4 do not consume attempt-004. Recovery may reset/re-enumerate USB for
bootloader reads but performs no writes. Command 5 consumes attempt-004 when it
starts. It retains the single host-owned local SV2 Noise fixture, 180-second
lease, 400 MHz/1100 mV/100% fan ceiling, one accepted-share ceiling, continuous
safety/watchdog checks, terminal safe stop, exact original bundle/settings
restoration with `mineonboot=false`, and complete process/USB cleanup.

Snapshot fallback reads only bootloader `0x0/0x8000`, partition table
`0x8000/0x1000`, PHY `0xf000/0x1000`, factory `0x10000/0x400000`, WWW
`0x410000/0x300000`, OTA 0 `0x710000/0x400000`, OTA 1
`0xb10000/0x400000`, and OTA data `0xf10000/0x2000`. Snapshot restoration is
ineligible until command 5 has begun and current-package flashing completed or
became uncertain. It writes those eight ranges once in one managed transaction,
never NVS/coredump/other offsets, then seeds Wi-Fi and restores settings/theme.

## Evidence, verification, and completion

Private roots are mode `0700`; identity, bundle, receipt, logs, and binaries are
mode `0600`. Public recovery/campaign projections contain only closed
categories, booleans, bounded counts/durations, bundle kind, digests, current
source/reference provenance, and redaction status. They never contain installed
runtime values, paths, logs, ports, credentials, endpoints, flash bytes, or raw
API bodies.

Before hardware: ordered Cargo format/clippy/build/test, Bright Builds, all
Bazel tests, focused real-launch tests, canonical firmware build/package,
parity/progress, redaction, reference cleanliness, selector/plan lineage,
sensitive-value review, and final diff review must pass on clean pushed source.

Recovery-002 runs exactly once. If readiness fails, withhold command 5, create
`CLOSURE.md`, keep `STR-005` implemented, and authorize no retry. If attempt-004
starts, it runs once and is never retried unchanged. On any campaign terminal,
safe-stop first, restore the original bundle/settings once, validate cleanup and
original runtime identity, and preserve the earliest category. Failure withholds
`RESULT.md` and keeps `STR-005` implemented. One independently accepted
campaign plus exact restoration creates `RESULT.md`, transitions only `STR-005`
to `verified` with `unit,golden,workflow,hardware-regression`, synchronizes
progress, archives completed task records, final-verifies, commits, and pushes.

Raw NVS/coredump capture, new-baseline flash, external pools, direct UART/pins,
fault injection, OTA, erase, unlisted writes, unbounded mining, and attempt-005
are prohibited effects and explicit non-claims.
