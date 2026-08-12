# Parity work closure

- Parity row: `NET-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `48796a1c9bdbbce5fbe3b8f07ae7c34ac6f2a6069396d081321b135e6e569877`
- Active task: `task-parity-net002-provisioning-network-attempt-004`

## Closure reason

The exact clean implementation at source commit
`fb9623d6c2f877a716324642311480bdd707a391` passed the complete software
gate and was pushed before hardware use. The sole detector admitted one Ultra
205, one exact device-derived private configuration candidate, protected modes,
and no serial holder. The sole conditional attempt-004 then flashed the exact
AP-only package and captured eleven recurring device-owned AP, DHCP, and DNS
readiness samples after exact-package passive-safety admission.

The attempt closed as `hardware_blocked` at the typed `association` boundary.
The macOS client accepted the detector-bound candidate and observed no different
Bitaxe candidate, but its bounded `networksetup` association transaction either
failed or did not confirm that exact association. It therefore performed no
DHCP, DNS, captive HTTP, or system-info request. The available closed result
cannot safely distinguish host-command rejection from a failed post-command
association confirmation because the raw child outcome is intentionally not
published. Attempt-004 is consumed and must not be retried or reinterpreted as
live client proof.

Host Wi-Fi restoration and the bounded exact-package owner-Wi-Fi recovery flash
both passed. Recovery confirmed the safe connected runtime, no secondary
recovery failure occurred, the private root is mode 0700, every private file is
mode 0600, and no public projection was published.

## Next safe action

Create a fresh continuation that types the association sub-boundary and records
its raw host outcome only in the protected private root. Investigate a
CoreWLAN-owned exact-SSID scan/association transaction or another supported
macOS association API that can join the detector-bound network when
`system_profiler` inventory omits it. Preserve the exact-device candidate,
recurring readiness quorum, ambiguity rejection, cleanup, recovery, and six
closed public boundaries. Only a new immutable plan and fresh ordinal may
authorize another hardware attempt.

## Non-claims

This closure verifies exact-device private candidate derivation, recurring
configuration-network readiness, exact-package passive safety, and complete
host/device recovery only. It does not verify live association, DHCP, wildcard
DNS, captive redirect, settings access, credential submission, station handoff,
repeated provisioning, other boards, mining, hardware controls, updates,
recovery parity, or release readiness. Raw detector, USB, flash, serial, Wi-Fi,
network, route, origin, credential, DNS, HTTP, command, and process material
remains ignored and private. `NET-002` remains `implemented`.
