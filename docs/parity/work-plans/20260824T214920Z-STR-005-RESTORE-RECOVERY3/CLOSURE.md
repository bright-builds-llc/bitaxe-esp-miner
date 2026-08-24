# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `9c659b0dfd8673c42695e26565deabc3bb5a9af5a750cb86cdf0b1107df8b51b`
- Active task: `task-parity-str005-installed-package-recovery-003`
- Attempt-004 consumed: `no`

## Closure reason

Clean pushed source `b33c89d1c7e38779cf68bf858976142fd214c30f`
and its exact canonical package passed every mandatory software gate, including
the new real nested-Bazel validator-child regression. A fresh detector admitted
exactly one Ultra 205 before recovery-003.

The single recovery-003 command stopped at earliest category
`hardware_blocked`, checkpoint `runtime_monitor_process`, because its initial
15-second receive-only runtime monitor child did not complete successfully. The
failure occurred before installed-identity capture, package search, historical
rebuild, flash readback, bundle creation, validator invocation, or validator
receipt creation. It therefore neither confirms nor disproves the corrected
validator-child behavior on the device path.

The private recovery root is empty and mode `0700`. No public readiness
projection or candidate was created. Attempt-004 remains unused and absent. No
flash write, NVS read/write, firmware snapshot, fixture start, pool connection,
mining, ASIC control, settings change, or campaign effect occurred. No owned
process remains, and a post-run detector again reported exactly one ready Ultra
205.

## Completed software result

The recovery owner now strips inherited nested `JS_BINARY__*` launcher state,
bounds validator output and lifetime, and writes a mode-`0600` closed receipt
covering launcher, workspace binding, exit, timeout, output-limit, bounded byte
counts/digests, invocation digest, acceptance, and source/plan provenance. Later
campaign admission requires that accepted receipt. Focused tests and the real
Bazel child prove accepted/rejected validation, launch failure, timeout, output
limit, protected modes, workspace binding, and secret-canary exclusion.

## Next safe action

Keep `STR-005` at `implemented`. Do not retry recovery-003 or run attempt-004
under this closed plan. A future continuation must first add the same protected
closed-diagnostic treatment to the passive runtime-monitor child, including its
launcher, exit, timeout, bounded output counts/digests, USB ownership/cleanup,
and exact checkpoint classification, plus a real-launch regression for the
observed pre-read boundary. Only a new active task and immutable changed-boundary
plan may authorize a fresh recovery ordinal. Attempt-004 remains available only
because no campaign effect began; attempt-005 remains prohibited.

## Non-claims

This closure does not verify installed-package recovery, the validator child on
the hardware path, an Ultra 205 Noise handshake, V2 channel, ASIC work,
target-qualified nonce, encrypted share, accepted response, terminal safe stop,
package/settings restoration, external-pool interoperability, mixed-protocol
fallback, other boards, unbounded mining, OTA, or release readiness. It does not
create `RESULT.md`, hardware-regression evidence, or `verified` status.
