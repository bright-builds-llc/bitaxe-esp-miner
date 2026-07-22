---
status: investigating
trigger: "Attempt 21 stopped at detector connection_failure before the Phase 35 checksum probe, credential access, or writes. Diagnose whether this is transient device state or espflash 4.5.0 USB-JTAG-Serial reset compatibility without issuing an unchanged retry."
created: 2026-07-22T04:47:59Z
updated: 2026-07-22T04:47:59Z
---

## Current Focus

hypothesis: The 4.5.0 Windows-oriented reset-order change may be incompatible with this macOS native USB-JTAG-Serial boundary, but one clean USB/barrel-power remediation is required to distinguish a persistent tool/device contract failure from transient target state.
test: After user-confirmed full non-invasive power/USB remediation, rerun exact-head preflight and one fresh Attempt 22. Do not issue a separate diagnostic; the sole detector and in-invocation checksum probe remain the discriminating boundaries.
expecting: A detector pass followed by a typed probe result disproves a persistent detector reset incompatibility. The same pre-probe `connection_failure` confirms the boundary and stops as `stop_hardware_blocker`.
next_action: Wait for the user's remediation confirmation.

## Symptoms

expected: Exact-tool detector admission succeeds, then the bounded checksum probe classifies the first flash boundary before credentials or writes.
actual: The detector selected one candidate but its reset-capable board-info command failed to connect. The probe, credential validation, and writes never started.
errors: Shareable signature is `connection_failure` with `boundary_schema=none`, `flash_stage=none`, and `flash_boundary=none`.
reproduction: Attempt 21 is sealed and must not be reused. Hermetic detector regressions reproduce the closed category but cannot establish the physical USB reset result.
started: Attempt 21 at exact source `e007c06a5350b197a7f2a1af1bb6a41472be651d` after all software and preflight gates passed.

## Eliminated

- Tool absence, version mismatch, and executable digest failure: `just doctor` passed with espflash 4.5.0 before the attempt.
- Ambiguous or missing serial candidate: the detector reached board-info rather than the missing/ambiguous-node categories.
- Probe, credential, factory, NVS, and monitor defects: none of those boundaries started.

## Evidence

- timestamp: 2026-07-22T04:47:59Z
  checked: Attempt 21 private seal and detector boundary, without printing protected identifiers or raw child output.
  found: Fresh root mode was `0700`; the seal is non-promotion, non-reusable, primary `connection_failure`, with no restoration or cleanup secondary failure and no typed flash stage.
  implication: The failure precedes the new checksum probe and is distinct from Attempts 19–20.
- timestamp: 2026-07-22T04:47:59Z
  checked: Official espflash 4.0.1 versus 4.5.0 reset source and upstream pull request 999.
  found: 4.5.0 changed USB-JTAG-Serial DTR/RTS ordering as part of a Windows connection repair; upstream review explicitly stated that path lacked USB-JTAG-Serial validation.
  implication: Tool/device reset compatibility is a strong causal hypothesis, but the repository must not claim it confirmed until the physical boundary is re-tested after one clean remediation.

## Resolution

root_cause:
fix:
verification:
files_changed: []
