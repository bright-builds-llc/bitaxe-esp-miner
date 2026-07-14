# Phase 32 Software Evidence and Hardware-Pending Record

Phase 32 implements a shared, bounded I2C0 lifecycle for startup display and read-only INA260/EMC2101 acquisition on Ultra 205. This record admits software evidence only. Phase 35 owns any later hardware-backed parity promotion.

## Software Evidence Inventory

The implementation evidence consists of:

- pure signed/sentinel decoder and failure-isolation tests in `bitaxe-safety`;
- source guards for finite I2C timeouts, repeated-start reads, one-owner handoff, read-only capabilities, producer ordering, and clone-only consumers;
- repeated API-read tests proving stored stamps and failed-source state do not change until a producer replacement;
- the ordered Rust format, lint, build, and test gate;
- canonical firmware build and package targets; and
- the read-only reference-clean guard.

These checks prove code structure, deterministic behavior, firmware compilation, package construction, and reference cleanliness. They do not prove physical sensor behavior.

## Hardware Evidence Status

hardware_evidence: pending
hardware_accessed: false
credentials_accessed: false
raw_artifacts_committed: no
checklist_promoted: false
promotion_owner: phase-35

No detector, flash, monitor, device API, credential, reset, or other hardware command ran during Phase 32 execution.

The pending blocker is exact: no current repo-owned wrapper records the required private mode-0700 root and mode-0600 serial-session trace containing device/session identity, PID and process-group identity, descendants, serial file-descriptor holders, and post-session cleanup proof while also producing a separate sanitized shareable summary. The current `redact-evidence=true` path sanitizes network and secret values but still preserves raw serial paths in command evidence. Therefore it is not eligible for this phase's hardware proof contract.

## Exact Exclusions and Non-Claims

Phase 32 did not perform or verify:

- fan, thermal-control, voltage, frequency, power-control, reset, GPIO, or other actuator writes;
- ASIC initialization, work submission, mining, pool access, Stratum traffic, or credential use;
- fault injection, self-test, stress, erase, recovery, rollback, OTA, or interrupted-update behavior;
- direct UART, pins, pads, headers, test points, probes, jumpers, solder, or injected signals;
- device discovery, flashing, monitoring, on-device API/WebSocket requests, or serial-session reuse;
- any board other than Ultra 205 board `205`;
- archived Phase 28.1.1 lineage work; or
- any parity-checklist transition to `verified`.

Startup display continuity, physical INA260 readings, physical EMC2101 temperature/tachometer readings, naturally occurring sensor-failure behavior, and live API availability remain hardware evidence pending. A later phase may evaluate them only through a compliant repo-owned, detector-gated, bounded, read-only, redacted workflow.
