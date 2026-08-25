# Exact restoration remediation closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `55d821773eb277ed9e6e27ff0bc8cdd7aeb7c19fbe304ff5fe72cf8e452b8ceb`
- Active task: `task-str005-exact-restoration-remediation`
- Remediation ordinal 1 consumed: `yes`

## Closure reason

The exact no-effect preflight passed, but the single restoration invocation
stopped at `snapshot_restore` before launching a write child. The remediation
state is `flash_started`, not `firmware_restored`, so settings-only resume is
ineligible. A second host invocation would exceed this immutable plan's fixed
ordinal and stop contract.

## Completed implementation and admission

Pushed source/package `276bb178` passed ordered Cargo formatting, clippy,
all-target build, and all-feature tests; Bright Builds; all Bazel tests;
canonical firmware build/package; parity/progress; redaction; reference
cleanliness; selector lineage; sensitive-value review; and diff review.

The task-specific historical-package authority admits only recovery-006, its
fixed capture source, the exact recovery/remediation plan and bundle digests,
board 205, ordinal 1, a clean current source, and the pinned reference. The real
adapter's admission-only path validated every protected snapshot range without
execution snapshots, USB write ownership, or device mutation. Fresh detection
then admitted exactly one ready Ultra 205.

## Restoration outcome

The outer owner proved the expected current package and safe baseline, wrote
the protected `pre_effect_ready` and `flash_started` states, and invoked the
Rust restore adapter once. The child exited 1 at `snapshot_restore`; no public
projection was published.

The child stderr digest exactly equals the deterministic local
`executor_program_mismatch` error. The snapshot adapter correctly renders a
single managed `esptool.py` multi-range transaction, but the shared flash
environment rejects every command whose program is not `espflash`. That guard
returns before `UsbSession::run_espflash`, proving that no write child or flash
transfer was launched. USB cleanup completed.

## Current safe state

A fresh post-failure detector admitted the sole board. A protected receive-only
monitor plus same-session system-info read prove:

- the unchanged campaign package is running;
- the reference remains pinned;
- `mineonboot=false`;
- mining is `safe_blocked`;
- hashrate is zero; and
- accepted and rejected share counts are both zero.

The original recovery-006 firmware and settings are still not restored, but no
additional device mutation occurred during this remediation.

## Terminal decision

The closed policy result is `stop_authority_boundary`. Remediation ordinal 1 is
consumed, `firmware_restored` was never proved, and no second restoration or
settings mutation is authorized. `RESULT.md`, the public restoration
projection, STR-005 hardware-regression evidence, and STR-005 promotion are all
withheld. The blocked remediation task remains active and is not archived.

## Next safe action

A separate immutable continuation must extend the shared execution environment
with a task-gated managed-`esptool.py` transaction that retains the existing USB
lease, supervision, effect diagnostics, retry rules, and eight-range allowlist.
It must prove the pre-transfer executor regression, pass all gates, and admit a
fresh remediation ordinal before any device write.

## Non-claims

This closure does not claim exact firmware/settings restoration, any flash
write, NVS mutation, Stratum V2 campaign retry, protocol/share evidence,
external-pool behavior, direct UART/pin work, fault injection, OTA, erase,
other-board behavior, release readiness, or verified STR-005 parity.
