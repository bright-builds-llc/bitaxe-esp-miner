# Parity work plan

- Run ID: `20260824T000902Z-STR-005-RESTORE-RECOVERY`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `c88148be6dded94b0247609f385c96244c3442d6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-installed-package-recovery`
- Continues: `docs/parity/work-plans/20260822T171824Z-STR-005-RUNTIME-ADMISSION/CLOSURE.md`
- Predecessor closure SHA-256: `709e3de59602491ce1531e50f420af94e84fca718405e048ab48c9fbb9e52778`

## Objective

Recover an independently validated exact rollback bundle for the firmware now
installed on the detected Ultra 205, then use it to gate and restore the single
attempt-004 local-fixture Stratum V2 campaign. Prefer an exact schema-v3
historical package. If bounded artifact search and one timestamp-pinned clean
rebuild cannot match the installed app-ELF digest, capture and validate a
firmware-only flash snapshot. Never capture raw NVS, coredump bytes, or adopt a
new baseline.

## Implementation

- [ ] Add one restore-recovery owner that captures private runtime identity,
      searches bounded repository/Bazel locations, performs one owned detached-
      worktree rebuild with the installed timestamp/provenance, and cleans all
      worktree/process state after interruption or completion.
- [ ] Add private schema `bitaxe-stratum-v2-restore-bundle-v1` with tagged
      `package_v3` and `flash_snapshot_v1` variants plus an independently source-
      bound validator and closed public readiness projection.
- [ ] On rebuild ineligibility or digest mismatch, read exactly eight ranges:
      bootloader `0x0/0x8000`, partition table `0x8000/0x1000`, PHY
      `0xf000/0x1000`, factory `0x10000/0x400000`, WWW `0x410000/0x300000`,
      OTA 0 `0x710000/0x400000`, OTA 1 `0xb10000/0x400000`, and OTA data
      `0xf10000/0x2000`. Reject every overlap with NVS `0x9000..0xf000` or
      coredump storage.
- [ ] Add historical package admission against the captured installed identity,
      not current workspace identity. Add one managed multi-range snapshot
      restore transaction, followed by Wi-Fi seed, settings/theme restoration,
      exact runtime attestation, and cleanup.
- [ ] Require the fixed restore bundle in the immutable attempt-004 parser and
      campaign. Preserve earliest failure, safe-stop before restoration, attempt
      every independent restoration step once, and never retry the campaign.

## Commands and effects

After all software gates, clean commit/push, and exact current package build:

1. `just detect-ultra205`
2. `just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-001 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection.json --redact-evidence`
3. `just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-001/restore-bundle.private.json`
4. `just stratum-v2-runtime-admission --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle scratch/str005-installed-package-recovery/recovery-001/restore-bundle.private.json`
5. Only after commands 1-4 succeed and both attempt/projection targets remain
   absent: `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-001/restore-bundle.private.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Commands 1-4 do not consume attempt-004. Recovery may reset/re-enumerate USB for
bootloader reads but performs no writes. Command 5 consumes attempt-004 when it
starts. It inherits the original 180-second lease, 400 MHz/1100 mV/100% fan
profile, safety limits, single accepted-share ceiling, local Noise fixture,
safe-stop, settings restoration, cleanup, privacy, and non-claim contracts.

Snapshot fallback reads only the eight ranges above. Snapshot writes are
ineligible until command 5 has begun and the current-package flash completed or
became uncertain. Restore writes exactly those ranges once through the managed
ESP-IDF esptool, never NVS/coredump/other offsets, then seeds Wi-Fi and restores
the captured settings with `mineonboot=false`.

## Evidence, verification, and completion

Private roots are mode `0700`; identity, bundle, receipt, logs, and binary
artifacts are mode `0600`. Preserve installed identity and firmware bytes only
privately. Public recovery/campaign projections may contain only closed
categories, booleans, bounded counts/durations, bundle kind, digests, current
source/reference provenance, and redaction status—never installed values,
paths, logs, ports, credentials, endpoints, flash bytes, or raw API bodies.

Before hardware: ordered Cargo format/clippy/build/test, Bright Builds, all
Bazel tests, dedicated real-launch tests, canonical firmware build/package,
parity/progress, redaction, reference cleanliness, selector/plan lineage,
sensitive-value review, and final diff review must pass on clean pushed source.

If recovery readiness fails, withhold command 5 and close without mutation. If
attempt-004 fails, safe-stop and restore once, withhold `RESULT.md`, keep
`STR-005` implemented, and authorize no later ordinal. On one independently
accepted campaign plus exact original runtime/package/settings restoration,
create `RESULT.md`, transition only `STR-005` to `verified` with
`unit,golden,workflow,hardware-regression`, synchronize progress, archive both
completed STR-005 task records, final-verify, commit, and push.

Raw NVS/coredump capture, new-baseline flash, external pools, direct UART/pins,
fault injection, OTA, erase, unlisted writes, unbounded mining, and attempt-005
are explicit non-claims and prohibited effects.
