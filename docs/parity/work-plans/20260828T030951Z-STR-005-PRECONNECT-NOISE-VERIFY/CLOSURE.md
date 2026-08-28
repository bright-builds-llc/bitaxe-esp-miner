# STR-005 pre-connect Noise continuation closure

- Plan: `20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY`
- Task: `task-str005-preconnect-noise-and-verification`
- Parity row: `STR-005`
- Final parity status: `implemented`
- Terminal decision: `stop_repeated_boundary`

## Outcome

The required fast real-TCP regression reproduced the hardware signature with a
pinned responder deadline: connecting before delayed preparation yielded zero
of 64 act-one bytes. Reversing only the shared effect seam made the same test
pass. A non-debuggable prepared-Noise state now ensures both production and
diagnostic firmware complete keypair and act-one construction before invoking
the TCP connector.

Diagnostic ordinal 4 exercised that exact pushed implementation on the Ultra
205. The local fixture admitted the exact device peer with zero unexpected
peers, but received zero act-one bytes before timeout. No Noise responder,
act-two creation, Noise completion, channel, job, ASIC work, nonce, or share
boundary was reached.

The approved plan defines a repeated zero-byte timeout after precomputation as
terminal. Diagnostic ordinal 5 and campaign attempt 008 are therefore
ineligible and were not created.

## Safety and restoration

The diagnostic used no ASIC, fan, voltage, or mining owner. The approved
recovery-006 snapshot write and separate Wi-Fi seed completed, followed by exact
attempt-004 settings/theme restoration. Final evidence proves the original
source/app/reference/partition identity, `mineonboot=false`, inactive mining,
zero hashrate and shares, USB cleanup, zero owned processes, and fresh one-board
detection.

## Evidence and disposition

The independently validated final failed projection is:

`docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-004.json`

All private roots remain ignored and protected. No `RESULT.md`, campaign
projection, hardware-regression evidence, task archive, or STR-005 promotion is
created. The task remains active but blocked at the repeated pre-Noise local
transport boundary.
