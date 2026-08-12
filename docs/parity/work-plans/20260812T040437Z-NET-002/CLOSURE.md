# Parity work closure

- Parity row: `NET-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `c1d92c65a0c121d4f35ddec5cc810a840caa23ee82d2af5d658ff7f78b195c77`
- Active task: `task-parity-net002-provisioning-network-attempt-001`

## Closure reason

The exact clean implementation at source commit
`4c42f182b265ec0d4d2b40a04106ffcc4de745ce` passed the complete software
gate. The sole detector admitted one Ultra 205, and the sole conditional
attempt-001 flashed the exact package without owner Wi-Fi credentials.

The orchestration closed as `evidence_invalid` before host association because
the captured serial document did not contain the one-shot safe-state or
configuration-network startup lines. Private aggregate diagnosis shows that
the capture was healthy but attached after those startup lines: it retained
118 recurring runtime-heartbeat records and 197 recurring runtime-health and
operator-snapshot records from the exact build. Thus the device was running,
not boot-looping, while the host contract incorrectly treated an inherently
late serial attachment as proof that AP startup had failed.

The host remained powered on and unassociated. The bounded ordinary recovery
flash restored the same exact package with the opaque owner Wi-Fi input and
confirmed the safe connected state. The typed terminal envelope reports host
restoration and device recovery complete with no secondary recovery failure.
Both ignored private roots are mode 0700, every contained file is mode 0600,
and no public projection was published. Attempt-001 is consumed.

## Next safe action

Create a fresh immutable continuation that does not require one-shot boot lines
from a late-attached flash monitor. Admit the exact AP-only build through the
recurring redacted runtime stream, then bind AP mode, credentials-missing
status, exact build identity, disabled mining, and disabled hardware control to
the same-origin API response after the unique configuration-network
association. Add a production-shaped regression in which startup lines are
absent but recurring runtime records and the complete client/API quorum are
present. Preserve the existing admission, DNS/captive checks, host cleanup,
exact-package recovery, privacy policy, and one-attempt bound under fresh
wrapper/attempt paths.

## Non-claims

This closure does not verify live configuration-network visibility,
association, DHCP, wildcard DNS, captive redirect, settings access, credential
submission, station handoff, repeated provisioning, other boards, mining,
hardware controls, updates, recovery parity, or release readiness. It does not
reinterpret late serial attachment as AP proof. Raw detector, USB, flash,
serial, Wi-Fi, network, route, origin, credential, DNS, HTTP, command, and
process material remains ignored and private. `NET-002` remains `implemented`.
