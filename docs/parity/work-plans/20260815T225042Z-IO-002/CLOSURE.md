# Parity work closure

- Parity row: `IO-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `e656644026e008f79b71696172cb5665f1379e7d677b22ac3a6be5253c40d78f`
- Active task: `task-parity-io002-adc-observation-attempt-003`

## Closure reason

Detector-gated attempt-003 used exact clean pushed package source
`9f48a1dbc07d7df83b05452e10edee4ff8989d12`. It admitted one Ultra 205,
observed the exact-package boot and same origin, kept mining and hardware
control disabled, completed cleanup, and passed redaction. Protected
system-info evidence independently validates, and the protected ADC input
validator passed finite integer millivolt-domain values with fresh, monotonic,
coherent HTTP/WebSocket observations.

Final ADC projection processing stopped at source-provenance admission. The
guard requires every configured semantic breadcrumb to occur exactly once, but
the upstream breadcrumb `.bitwidth = ADC_BITWIDTH_DEFAULT` legitimately occurs
three times in pinned `reference/esp-miner/main/adc.c`. The typed terminal
category is `evidence_invalid` with safe summary `ADC source semantic fragment
is not unique`. The public candidate was removed, no ADC projection was
published, and attempt-003 is consumed without retry.

The units are not the cause of this stop. Upstream's calibrated ADC call writes
millivolts, the Rust adapter preserves millivolts, the API field is bound to
that millivolt observation, and the live protected input validator accepted the
wire values as integer millivolts. The defect is an ambiguous provenance
breadcrumb plus a regression input/cache gap that did not expose the pinned
reference multiplicity before hardware.

## Next safe action

Create a fresh task and immutable plan that replace the broad bit-width
breadcrumb with exact initializer context and declare the pinned reference file
as an explicit source-semantics regression input. The complete software and
clean-package gates must pass before a separately authorized fresh hardware
ordinal. This plan authorizes neither attempt-004 nor an unchanged retry.

## Non-claims

This closure does not verify the final public ADC evidence projection,
energized-rail values or accuracy, external calibration, induced failure, load
behavior, long-duration drift, voltage control, mining, ASIC work, other
boards, or release readiness. IO-002 remains `implemented`.
