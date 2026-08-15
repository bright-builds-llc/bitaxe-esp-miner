# Parity work closure

- Parity row: `IO-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `bb0db9d7338e79d86bd4a97105e85805db599593f82da06360505836b4506fb6`
- Active task: `task-parity-io002-adc-observation`

## Closure reason

Detector-gated attempt-001 used exact clean pushed package source
`0bd2dfff2e662431fba3bb95d5654b1dbce3c80a`. The base transaction admitted
one Ultra 205, observed the exact-package safe boot, kept mining and hardware
control disabled, completed cleanup, and passed its independent system-info
validator and redaction contract. The independent lossless ADC input validator
then rejected both fresh live samples because they were outside the immutable
400–2,000 mV admission range. The terminal category was `evidence_invalid`.
No public projection was published, and attempt-001 is consumed without retry.

The reference and Rust implementations both report the calibrated ADC1
channel-1 pin reading directly in millivolts. This plan did not establish that
a passive safe-state core rail must be energized above 400 mV, so accepting the
observed values or weakening the range after the attempt would be circular.

## Next safe action

Create a fresh active task and immutable plan that first establishes the
expected Ultra 205 ADC reading when mining and hardware control are disabled,
using authoritative board electrical evidence or a separately authorized safe
stimulus. Define a justified closed range and retry eligibility before any new
device attempt. This plan authorizes neither attempt-002 nor an unchanged
retry.

## Non-claims

This closure does not verify live ADC calibration accuracy, the observed
millivolt values, energized-rail behavior, failure behavior, exact numeric API
correlation, cadence under load, voltage control, mining, ASIC work, thermal or
fan behavior, other boards, or release readiness. IO-002 remains
`implemented`.
