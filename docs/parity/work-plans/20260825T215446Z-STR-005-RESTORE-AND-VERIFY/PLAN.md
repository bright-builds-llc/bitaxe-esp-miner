# Restore Ultra 205 and verify STR-005

- Run ID: `20260825T215446Z-STR-005-RESTORE-AND-VERIFY`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source base: `093f68952a37bf2d418ff16cb67dea573d14ac9f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-restore-and-verify-continuation`
- Continues: `docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/CLOSURE.md`

## Objective

First restore the connected Ultra 205 to the exact pre-campaign recovery-006
firmware, settings, and theme. Then run the local authenticated Stratum V2
fixture through an accepted Noise, channel, job, BM1366 work, nonce, share,
safe-stop, cleanup, and exact-restoration chain. Promote only STR-005 after
independent validation of that complete hardware evidence.

## Fixed restoration contract

- Recovery bundle:
  `scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json`
- Bundle SHA-256:
  `1d5e2e3b76489c36458f63f11bf28b399ea4cd6c2d45f8dab20ef060b03e18f4`
- Recovery projection:
  `docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json`
- Attempt-004 backup SHA-256:
  `ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631`
- Original source/app:
  `a11b579b62cb52a53bbf6072bde209d3eb3f17e2` /
  `32e2de545a0a89f97153baaa0e4c169f5064d47358d208732cf2b2ac8be3a4c4`
- Fresh roots:
  `scratch/str005-exact-restoration/preflight-002` and
  `scratch/str005-exact-restoration/remediation-002`
- Public projection:
  `docs/parity/evidence/str005-exact-restoration/restoration-projection.json`

The restore executor must accept only one non-symlinked managed ESP-IDF
`esptool.py` command for ESP32-S3, the selected port, USB/hard reset, verification,
`dio/16MB/80m`, and the eight recovery-006 addresses backed by admitted immutable
snapshots. It must retain the existing USB lease, owned-child supervision,
same-device reacquisition, cleanup, and retry policy. A pre-transfer connection
failure may retry internally only after objective same-device enumeration
change; partial transfer never retries.

The protected state machine is
`pre_effect_ready → flash_started → firmware_restored → settings_restored → complete`.
Settings-only resume is eligible only from `firmware_restored`. Success proves
the exact original source, app digest, build identity, factory partition,
pinned reference, every restorable key, local Wi-Fi/pool inputs, exact theme,
`mineonboot=false`, `safe_blocked`, and zero hashrate/shares.

## Fixed campaign contract

- First attempt: `attempt-005`
- Root: `scratch/str005-stratum-v2/attempt-005`
- Projection:
  `docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json`
- Profile: Ultra 205, 400 MHz, 1100 mV, 100% fan
- Safety ceiling: 180 seconds
- Pool: local authenticated Noise fixture only

Select exactly one non-loopback host IPv4 whose netmask contains the fresh
device-origin IPv4; reject zero, multiple, VPN-only, or mismatched candidates.
Bind the fixture to that address. Firmware and host evidence must retain closed
transport details (`resolve`, `connect`, `configure`, `handshake`, `write`,
`read`, `frame`) and fixture stages without exposing endpoints or credentials.

Attempt acceptance requires hardware preparation, authenticated Noise, channel
opening, target/job receipt, BM1366 work dispatch, a qualified nonce, encrypted
accepted share, complete safe-stop, USB/process cleanup, and exact recovery-006
restoration. Success and failure both route through the same typed restore
executor. A later attempt is ineligible until the original runtime/settings are
proved and a new authoritative signature has a real-boundary red/green fix.

## Progress, evidence, and stop policy

Every fresh ordinal is recorded with its exact command and changed boundary in
the active task before effects. It requires full gates, clean pushed source,
exact package, fresh roots, no-effect preflight, and one-board detector. Never
retry unchanged or reuse a sealed root. Stop on a repeated post-fix signature,
unresolved partial transfer, hardware blocker, authority boundary, or impossible
evidence contract.

Private roots are mode `0700`; files are mode `0600`. Public evidence contains
only closed categories, booleans, bounded counts/durations, safe provenance,
artifact digests, cleanup, and redaction status. It never contains endpoints,
USB paths, network addresses, credentials, logs, settings values, or flash bytes.

Run ordered Cargo format/clippy/build/test, Bright Builds, all Bazel tests,
real-child regressions, canonical build/package, parity/progress, redaction,
reference cleanliness, selector lineage, sensitive-value review, and diff review
before hardware. On final success, create `RESULT.md`, transition only STR-005 to
`verified` with `unit,golden,workflow,hardware-regression`, synchronize progress,
archive the directly superseded STR-005 tasks, final-verify, commit, and push.

## Non-claims

External production pools, mixed-protocol fallback, other boards, raw
NVS/coredump access, a new baseline, direct UART/pins, fault injection, OTA,
erase, arbitrary writes, unbounded mining, and release readiness remain outside
this plan.
