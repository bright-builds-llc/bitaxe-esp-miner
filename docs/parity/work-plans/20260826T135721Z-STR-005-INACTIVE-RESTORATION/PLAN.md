# Finalize inactive restoration and verify STR-005

- Run ID: `20260826T135721Z-STR-005-INACTIVE-RESTORATION`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source base: `8b7952672309abc75e0a63f55aae159d6ee509e1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-inactive-restoration-and-campaign-continuation`
- Continues: `docs/parity/work-plans/20260825T215446Z-STR-005-RESTORE-AND-VERIFY/CLOSURE.md`

## Objective

Close the already completed remediation-002 through read-only validation using
the restored firmware's truthful inactive state, then run the untouched local
Stratum V2 attempt-005 and promote only after accepted hardware plus exact final
restoration.

## Read-only restoration finalization

Use fresh root `scratch/str005-restoration-finalize/finalize-001` and public
projection `docs/parity/evidence/str005-exact-restoration/restoration-projection.json`.
Validate the sealed remediation-002 state, authorization, snapshot and Wi-Fi
diagnostics, bundle and backup digests, exact original source/app/reference/
partition, settings, theme, `mineonboot=false`, zero hashrate/shares, cleanup,
and one closed inactive category: `paused` or `safe_blocked`.

The expected current category is `paused`, matching the restored source's
operator-paused semantics. Public evidence records `mining_inactive=true` and
the closed category; it does not claim `safe_blocked`. Finalization may detect,
monitor, and read same-origin runtime APIs, but it must not flash, seed NVS,
patch settings, change theme, restart, start a fixture, or create attempt-005.

## Campaign attempt-005

After accepted restoration evidence is committed and pushed, rebuild the exact
package and bind attempt-005 to this plan:

- root `scratch/str005-stratum-v2/attempt-005`;
- projection `docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json`;
- local same-subnet authenticated Noise fixture;
- Ultra 205 at 400 MHz, 1100 mV, and 100% fan;
- 180-second safety ceiling; and
- exact recovery-006 restoration on every terminal outcome.

Acceptance requires hardware preparation, authenticated Noise, channel open,
target/job receipt, BM1366 work, qualified nonce, encrypted accepted share,
complete safe-stop, USB/process cleanup, exact original identity/settings/theme,
`mineonboot=false`, zero terminal work, and inactive category `paused` or
`safe_blocked`.

## Progress and completion

Every hardware ordinal requires ordered Cargo gates, Bright Builds, all Bazel
tests, canonical build/package, parity/progress, redaction, reference cleanliness,
selector lineage, sensitive-value review, clean push, fresh roots, and fresh
one-board detection. Never retry unchanged or reuse a sealed root. Continue only
after a new closed signature receives a real-boundary red/green fix and exact
original restoration; stop repeated, partial-transfer, hardware, authority, or
impossible outcomes.

Private roots are mode `0700` and files mode `0600`. Public evidence contains
only closed categories, booleans, bounded counts/durations, safe provenance,
artifact digests, cleanup, and redaction status—never endpoints, network or USB
identifiers, credentials, settings values, logs, or flash bytes.

On success, create `RESULT.md`, transition only STR-005 to `verified` with
`unit,golden,workflow,hardware-regression`, synchronize progress, archive the
directly superseded STR-005 tasks plus this continuation, final-verify, commit,
and push.

## Non-claims

External pools, mixed-protocol live fallback, other boards, direct UART/pins,
raw NVS/coredump access, a new baseline, fault injection, OTA, erase, arbitrary
writes, unbounded mining, and release readiness remain outside this plan.
