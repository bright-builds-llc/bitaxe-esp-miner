# Inactive restoration and STR-005 campaign closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `14c7676fb26b6291a24d08d229bc38717691835978d61ae24fd8cff91736470a`
- Active task: `task-str005-inactive-restoration-and-campaign-continuation`
- Campaign attempts consumed: `attempt-005`, `attempt-006`, `attempt-007`
- Final remediation consumed: `remediation-005`

## Closure reason

Attempt-007 repeated the exact authoritative post-fix signature from
attempt-006: firmware terminal `transport/handshake` and fixture terminal
`noise`, after TCP connection but before channel, work, or share. The hardware
policy therefore requires `stop_repeated_boundary`. No attempt-008 or STR-005
promotion is authorized.

## Implemented and verified software

The continuation added and verified a typed managed-esptool restore executor,
read-only inactive restoration finalization, same-subnet fixture selection,
closed transport/fixture stages, a 300-second listener window, descendant
recovery admission, corrected relative-path rollback, and signed full-`u32`
fixture certificate validity.

Every implementation boundary passed ordered Cargo gates, Bright Builds, all
55 Bazel tests, canonical build/package, parity/progress, redaction, reference
cleanliness, selector lineage, sensitive-value review, and diff review before
hardware. Timing-sensitive aggregate failures passed unchanged focused and
isolated reruns before the full suite passed.

## Hardware outcomes

- Remediation-002 restored exact firmware and exposed the historical truthful
  `paused` state; read-only finalization accepted the inactive contract.
- Attempt-005 stopped `transport/connect` after its listener expired before the
  device connection. Safe-stop and cleanup passed.
- Remediation-003 exactly restored the board.
- Attempt-006 connected TCP and stopped `transport/handshake`; the fixture
  stopped `noise`. Safe-stop and cleanup passed.
- Remediation-004 exactly restored the board.
- Attempt-007 repeated the same handshake/noise signature after the signed
  full-time-domain certificate fix. Safe-stop and cleanup passed.
- Remediation-005 exactly restored the board after campaign rollback settings
  verification remained incomplete.

No campaign reached channel-ready, BM1366 work dispatch, nonce correlation, or
share submission. No campaign projection was published.

## Final safe state

The independently validated remediation-005 projection proves original source
`a11b579b62cb52a53bbf6072bde209d3eb3f17e2`, the original app digest, pinned
reference, factory partition, exact settings/theme, `mineonboot=false`, inactive
`paused`, zero hashrate/shares, cleanup, and redaction.

## Terminal decision

STR-005 remains `implemented`. `RESULT.md`, campaign evidence, checklist
transition, hardware-regression claim, task archival, and promotion are all
withheld. The blocked task remains active.

## Next safe action

Any future work requires a separate task that diagnoses responder/initiator
Noise interoperability below the current `handshake/noise` discriminator using
a real-boundary regression that does not consume another campaign. A renamed
category, another certificate-time variation, or unchanged hardware retry is
not progress.

## Non-claims

This closure does not verify Noise authentication on hardware, channel/job
handling, ASIC work, nonce/share behavior, accepted or rejected shares,
external pools, mixed-protocol fallback, other boards, direct UART/pins, raw
NVS/coredump access, fault injection, OTA, erase, unbounded mining, release
readiness, or verified STR-005 parity.
