# Parity work closure

- Parity row: `NET-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `2d705f1a10befc1e235d383b9b33e5fe620e6522fcba4f6b0ba814d92bc99028`
- Active task: `task-parity-net002-provisioning-network-attempt-005`

## Closure reason

The exact pushed implementation at source commit
`2af62ad54600dd6c7bffcf3481cc8cd94b98ab3d` passed the complete software gate.
The sole detector admitted one Ultra 205 and one protected exact-device
candidate. The sole attempt-005 passed exact-package passive safety, recurring
AP/DHCP/DNS readiness, the directed CoreWLAN scan and association, DHCP,
wildcard DNS, captive redirect, and same-origin system-info retrieval.

The attempt then closed as `service_recovery_failed` because the evidence
checker required the persisted `startMiningOnBoot` preference to be false. The
live API returned the valid boolean value true, while the authoritative runtime
attestation independently proved mining, work submission, and hardware control
disabled. Repository wire fixtures also establish true as a valid/default
configuration value. The capture therefore confused an operator preference
with active runtime safety and correctly withheld evidence.

Host restoration and exact-package owner-Wi-Fi recovery passed, with no
secondary recovery failure. All private files are mode 0600 beneath a mode-0700
root and no public projection exists. Attempt-005 is consumed.

## Next safe action

Create a fresh continuation that removes the invalid
`startMiningOnBoot === false` API postcondition. Continue requiring the exact
runtime passive-safety attestation plus the AP state, exact build identity,
CoreWLAN, DHCP, DNS, captive redirect, system-info, cleanup, and recovery
quorums. Add a regression proving either boolean preference cannot substitute
for or contradict the authoritative disabled runtime state. Only a new
immutable plan and fresh ordinal may authorize another attempt.

## Non-claims

This closure verifies live exact-candidate association, DHCP, wildcard DNS,
captive redirect, system-info access, and complete recovery, but does not
publish accepted NET-002 evidence or verify credential submission, station
handoff, repeated provisioning, other boards, mining, hardware controls,
updates, recovery parity, or release readiness. Raw local identities and
network material remain private. `NET-002` remains `implemented`.
