# Parity work closure

- Parity row: `NET-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `79e58eac03213637cceb99304402faf5b50c99db3a3b0f06cfd00a9bdf84a4ba`
- Active task: `task-parity-net001-reconnect-lifecycle-attempt-001`

## Closure reason

The sole detector admitted exactly one Ultra 205. Attempt-001 used the exact
schema-v3 package built from clean pushed implementation commit
`daa27a3337c834285580362ab6dcfe0e5d6c428f`. The firmware consumed and
confirmed deletion of the private one-shot marker before deliberately
disconnecting the station.

Private retained observations recorded exactly one armed, disconnected,
attempted, connected, recovered, and stable marker. The disconnect followed
probe arming by 2,063 ms; the first retry followed disconnect by 5,033 ms;
DHCP recovery followed reconnect launch by 4,466 ms; the recovered marker
followed connected publication by 28 ms; and stable client-only service was
retained 15,026 ms after recovery.

The host capture nevertheless terminated as `reconnect_timing_invalid`
because its validator incorrectly required the sequential connected and
recovered log calls to share an identical millisecond timestamp. It therefore
withheld the public evidence projection before the final same-origin HTTP
postcondition. The bounded ordinary exact-package recovery flash succeeded and
removed the probe marker from the final package state.

Both ignored private roots were mode 0700 and every contained artifact was
mode 0600. The typed failure envelope contained only the closed category and
safe recovery booleans. No public projection was published and attempt-001 is
consumed.

## Next safe action

The validator now admits a bounded nonnegative delay between the sequential
connected and recovered publications, with regression coverage using the
observed 28-ms separation. Run the complete software gate, commit and push the
correction, then create a fresh immutable continuation plan and hardware
ordinal. The new attempt must independently reach the final same-origin HTTP
quorum and validate a closed public projection before `NET-001` promotion.

## Non-claims

This closure does not verify the final HTTP postcondition, accepted public
evidence, repeated or long-running reconnect behavior, provisioning-client
suppression, live IPv6 behavior, router or RF failure modes, other boards,
mining, hardware controls, updates, recovery parity, release readiness, or
`NET-001` parity. Raw detector, USB, flash, serial, network, origin, credential,
process, and HTTP material remains ignored and private. `NET-001` remains
`implemented`.
