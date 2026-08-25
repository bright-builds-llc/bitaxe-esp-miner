# Exact restoration remediation plan

- Run ID: `20260825T150417Z-STR-005-EXACT-RESTORATION`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `fca0df14caa69a09c517e7913008ebf711b51f82`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-exact-restoration-remediation`
- Continues: `docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/CLOSURE.md`
- Predecessor closure SHA-256: `98b12c4037ef50777f6743e2278ad9aed06ee55a79f84ee921c09620596e207d`

## Objective

Restore the connected Ultra 205 from the current safe campaign package and NVS
state to the exact pre-campaign recovery-006 firmware snapshot plus settings and
theme. Finish with the original installed runtime identity, `mineonboot=false`,
mining `safe_blocked`, zero hashrate, exact settings/theme, and complete USB and
process cleanup. This remediation is not a Stratum V2 campaign retry and cannot
promote `STR-005`.

## Fixed inputs and outputs

- Recovery bundle:
  `scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json`
- Recovery projection:
  `docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json`
- Recovery capture source:
  `7d5d9504433d54ae28fe853c5827d6dd05693eef`
- Campaign backup:
  `scratch/str005-stratum-v2/attempt-004/settings-backup.private.json`
- Current device source before remediation:
  `78784a4ac3576f39fe451e85508ea878e2941eb4`
- Preflight root: `scratch/str005-exact-restoration/preflight-001`
- Effect root: `scratch/str005-exact-restoration/remediation-001`
- Public projection:
  `docs/parity/evidence/str005-exact-restoration/restoration-projection.json`
- Remediation result:
  `docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/RESULT.md`

Local `wifi-credentials.json` and the sole mode-`0600` ignored
`pool-credentials*.json` input are memory-only secret sources. Their contents,
values, and low-entropy digests never enter terminal output, private evidence,
Git, or public projections.

## Implementation contract

- Add `just stratum-v2-exact-restoration preflight|start|resume` with exact
  board, paths, plan, projection, and redaction admission.
- Extend `restore-installed` with exact `--restore-authorization`,
  `--remediation-plan`, `--private-root`, and `--admission-only` inputs.
- Replace equality between bundle capture source and current host provenance
  with a task-specific authorization contract:
  - recovery capture source equals the fixed commit above;
  - recovery bundle, projection, recovery plan, validator receipt, and all
    snapshot bytes/digests/modes remain exact;
  - current host provenance is clean, pushed, reference-pinned, and equals the
    authorization receipt;
  - the immutable remediation plan digest, board 205, ordinal 1, bundle digest,
    and exact paths match;
  - arbitrary historical bundles, dirty/future replay, path drift, symlinks,
    and authorization tampering fail before USB or execution snapshots.
- `preflight` writes only protected diagnostic/authorization artifacts beneath
  the preflight root and invokes the real Rust adapter in `--admission-only`
  mode. It must return `restoration_pre_effect_ready` while proving no USB
  ownership, snapshot materialization, credential mutation, or device effect.
- `start` must:
  1. pass clean pushed source and canonical host-package gates without flashing
     that package;
  2. require a fresh one-board detector and holder-free USB state;
  3. prove the current device source above, `mineonboot=false`, mining
     `safe_blocked`, and zero hashrate through a fresh monitor/same-origin read;
  4. validate the protected campaign backup and reconstruct every restorable
     setting in memory from backup plus local credentials;
  5. run one admitted eight-range snapshot transaction, excluding NVS and
     coredump, followed by Wi-Fi NVS seed;
  6. reacquire the same device and prove the original bundle runtime identity;
  7. reuse `restoreSelfTestSettings`, then confirm every restorable field, SSID,
     primary pool URL/port/user, fallback disabled, exact theme, and
     `mineonboot=false` without persisting secret values;
  8. prove original runtime, mining `safe_blocked`, zero hashrate, USB cleanup,
     and no owned process before projection.
- Persist a protected state machine:
  `pre_effect_ready → flash_started → firmware_restored → settings_restored → complete`.
  `resume` is settings-only and is eligible exclusively from a proved
  `firmware_restored` state; it can never invoke flash.

## Exact commands

1. `just detect-ultra205`
2. `just stratum-v2-exact-restoration preflight --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --recovery-projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json --campaign-root scratch/str005-stratum-v2/attempt-004 --wifi-credentials wifi-credentials.json --private-root scratch/str005-exact-restoration/preflight-001 --projection docs/parity/evidence/str005-exact-restoration/restoration-projection.json --plan docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/PLAN.md --redact-evidence`
3. After preflight succeeds, refresh `just detect-ultra205` and run:
   `just stratum-v2-exact-restoration start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --recovery-projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json --campaign-root scratch/str005-stratum-v2/attempt-004 --wifi-credentials wifi-credentials.json --private-root scratch/str005-exact-restoration/remediation-001 --projection docs/parity/evidence/str005-exact-restoration/restoration-projection.json --plan docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/PLAN.md --redact-evidence`
4. `resume` uses the identical `start` arguments with action `resume`, only after
   the owner proves `firmware_restored`; otherwise it fails without effects.

## Recovery, evidence, and completion

- The repository USB supervisor retains its existing single internal retry only
  after an objective same-device enumeration change. The host command is never
  rerun unchanged and no second restoration transaction is inferred.
- Admission, identity drift, foreign ownership, pre-transfer failure, or
  unresolved partial/uncertain transfer preserves the earliest category,
  cleans ownership, seals the root, and stops. Settings failure after proved
  firmware restoration may use settings-only `resume`; it must not reflash.
- Private roots/directories are `0700`; authorization, state, child diagnostics,
  HTTP material, settings, and logs are `0600`. Public evidence contains only
  closed categories, booleans, bounded counts/durations, and safe provenance or
  artifact digests.
- Before hardware: ordered Cargo format/clippy/build/test, Bright Builds, all
  Bazel tests, real-process admission, canonical build/package,
  parity/progress, redaction, reference cleanliness, selector/task lineage,
  sensitive-value review, and diff review must pass on clean pushed source.
- Success creates the public restoration projection and remediation `RESULT.md`,
  archives only this remediation task, final-verifies, commits, and pushes.
  `STR-005` stays `implemented`; its campaign closure and non-claims remain.
- Failure withholds projection/result, records current safe state, seals the
  task as blocked, and performs no campaign, external-pool, direct-UART/pin,
  raw-NVS/coredump, fault-injection, OTA, erase, or unlisted write operation.
