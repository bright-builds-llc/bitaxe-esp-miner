# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `0328084c0157831b9d85ac6369777b48ed4fc32cb5d709c4fc3570e9fc373fdf`
- Active task: `task-parity-str005-autonomous-continuation`
- Attempt-004 consumed: `yes`

## Closure reason

Attempt-004 ended without protocol/share evidence and its single required
original-restoration attempt failed. The board is currently reachable and
fail-closed, but a second restoration write exceeds this plan's authority.
`STR-005` therefore remains `implemented` and the hardware policy stops at
`stop_authority_boundary`.

## Completed recovery

The rolling continuation diagnosed and fixed three distinct pre-campaign
boundaries with red/green real-process regressions:

- runtime monitor capture increased from the incorrect 15-second value to the
  device-session contract's 60-second value;
- outer monitor supervision increased from 75 to 210 seconds so independent
  probe, admission/reacquisition, capture, and final-cleanup bounds can finish;
- the independent validator child now uses the Bazel wrapper's resolved Node
  binary and direct compiled CLI rather than an unreliable nested launcher.

Recovery-006 on pushed source/package `7d5d9504` passed both qualified runtime
monitors, installed identity, runtime continuity, all eight 460800-baud
firmware-only reads, NVS/coredump exclusion, protected modes, bundle creation,
validator-child acceptance, independent validation, cleanup, and redaction.
The accepted public recovery projection is retained with this closure.

## Campaign outcome

After a narrowly admitted host-only preflight descendant, exact current package
`78784a4a`, no-effect `pre_effect_ready`, fresh one-board detection, and
`runtime_admission_ready`, the single attempt-004 campaign started and was
consumed.

The exact current factory image and temporary campaign NVS writes both completed
and verified once. The campaign did not reach active mining: `active_ms=0`, no
readiness transition or protocol gate was observed, no share was accepted or
rejected, and the private campaign result classified the missing submit response
before the outer owner ended `timeout/unclassified`. USB cleanup completed.

The outer owner attempted original snapshot/settings restoration once, as
required, but its protected recovery receipt records `restored=false`. No public
campaign projection was published. Exact original package/settings restoration
is therefore not claimed.

## Current safe state

A fresh post-campaign detector admitted exactly one ready Ultra 205. A protected
receive-only monitor found exactly one current runtime origin and no active
mining/control markers. A same-session protected system-info read proves:

- the current campaign package is running;
- the original package is not running;
- `mineonboot=false`;
- mining activity is `safe_blocked`;
- hashrate is zero; and
- accepted/rejected share counts are both zero.

The board is reachable and fail-closed, but it remains on the current package
with campaign-era configuration rather than the exact pre-campaign package and
settings.

## Terminal decision

The closed policy outcome is `stop_authority_boundary`. Attempt-004 and its one
restoration attempt are consumed. A second restoration write would exceed this
plan's explicit once-only rollback contract even though the retained snapshot
bundle remains valid. No further device write was issued.

## Next safe action

Any remediation now requires a separate active exact-restoration task and
repo-owned command that admits the current safe runtime, the retained
recovery-006 snapshot, a fresh write ordinal, recovery/cleanup, and proof of the
original package/settings. It is not an STR-005 campaign retry and must not
promote STR-005 by itself.

## Non-claims

This closure does not verify the local Noise handshake on hardware, channel/job
work, ASIC nonce, encrypted share, submit response, accepted share, campaign
safe stop, exact restoration, external-pool interoperability, mixed-protocol
fallback, other boards, unbounded mining, OTA, or release readiness. It does not
create `RESULT.md`, hardware-regression evidence, or `verified` status.
