# STR-005 TCP payload plan closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `14bd8aef5d78f38881a3da1a99a6808f7f6e8c93bb1d1a02d7972fcaaeb1d843`
- Active task: `task-str005-tcp-payload-205`

## Closure reason

Diagnostic-001 stopped before effects at `timeout:fixture_ready` because the
supervisor supplied a 360-second fixture session timeout to a fixture contract
bounded at 300 seconds. The production argument-constructor regression fixed
that exact boundary with a 120-second fixture session timeout, every required
software gate passed, and the clean fix commit was pushed before diagnostic-002.

Diagnostic-002 then passed fixture readiness, used the exact clean package, and
reached the device transaction, but the exact admitted peer delivered zero of
the fixed 64 payload bytes. The fixture sealed `payload_read` with zero bytes,
no digest match, and no extra bytes. No Noise, Stratum V2 message, ASIC, fan,
voltage, or mining boundary was accepted.

The mandatory recovery-006 restoration then stopped at `hardware_blocked`.
The current restore adapter admits only its archived Noise diagnostic root and
ordinal or prior campaign roots; it rejects this decomposed task's
`diagnostic-002/restoration` root before producing a restore result. Exact
recovery-006 firmware/settings restoration is therefore unproved. The installed
diagnostic seed set `mineonboot=false` and the diagnostic firmware has no
hardware-actuation owner, but exact prior package/settings identity and final
USB/process cleanup are not claimed.

The plan requires `stop_hardware_blocker` when exact restoration cannot be
proved and the active task explicitly authorizes no diagnostic-003. The STR-005
checklist fields remain unchanged at `implemented | unit,golden,workflow`; no
transition or progress synchronization is permitted.

## Next safe action

Create a fresh active recovery-only task and immutable plan before any further
device effect. It must add and software-verify a current-task recovery root and
authorization in the repo-owned restore adapter, run only exact recovery-006
package/settings restoration, and prove inactive zero-work runtime plus
USB/process cleanup. It must not send another TCP payload, start Noise, run a V2
session, mine, or promote STR-005.

Only after that recovery task succeeds may a separate new TCP diagnostic plan
diagnose the zero-byte `payload_read` boundary. That later plan requires a new
closed discriminator and regression-backed fix; it may not treat an ordinal,
timing change, or unchanged rerun as progress.

## Non-claims

This closure does not verify TCP payload delivery, Noise authentication,
encrypted traffic, setup/channel/job receipt, ASIC work, nonce or share
submission, mining, exact restoration, hardware regression, other pools,
other boards, OTA/recovery parity, release readiness, or verified STR-005
parity.
