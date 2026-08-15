# Parity work closure

- Parity row: `IO-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `bc9d9d7be66a1bcf85ee3c1e9692bbcb865eac279f94f8e3d27679991dac9f08`
- Active task: `task-parity-io002-adc-observation-attempt-002`

## Closure reason

Detector-gated attempt-002 used exact clean pushed package source
`d7efb2eab8eaf1ee883ec25297f914ef4c99ab87`. The base transaction admitted
one Ultra 205, observed the exact-package safe boot, kept mining and hardware
control disabled, completed cleanup, and produced an independently valid
protected system-info projection with passed redaction. Protected roots and
files have the planned `0700` and `0600` modes.

ADC post-processing then stopped at the immutable task-contract admission. The
new active task named the plan, attempt-002 paths, and public schema ordinal but
omitted the literal `bitaxe-adc-observation-evidence-v1` identifier required by
`validateTaskAndPlan`. Focused automation tests used a synthetic task fixture
that contained the identifier, so they did not cover the real `TASKS.md`
integration. The terminal category is `evidence_invalid`, the ADC validator was
not reached, no public projection was published, and attempt-002 is consumed
without retry.

This failure does not contradict the corrected millivolt-domain validator and
does not supply ADC parity evidence. Editing the task after observing the
attempt or rerunning under this plan would violate the immutable contract.

## Next safe action

Create a fresh task and immutable plan that add a pre-hardware regression over
the real task block, bind the exact ADC schema identifier, and explicitly
authorize fresh attempt-003 paths and ordinal. Re-run the complete clean pushed
package and detector gates before any device capture. This plan authorizes
neither attempt-003 nor an unchanged retry.

## Non-claims

This closure does not verify live ADC values, calibration accuracy, acquisition
cadence, HTTP/WebSocket numeric correlation, energized-rail behavior, failure
behavior, voltage control, mining, ASIC work, other boards, or release
readiness. IO-002 remains `implemented`.
