# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Terminal category: `evidence_invalid`
- Verification claimed: `no`
- Plan SHA-256: `8e8049fd6fbb19575f6abe593afcdd9ac2303eee0204b5f188d4b65aa7607d58`
- Attempt: `005` consumed

## Closure reason

The clean exact package and one detector-admitted Ultra 205 entered the sole
attempt-005 transaction. The device-side stimulus no longer reproduced the
prior projection loss: the protected production log contains one
`fault_observed` marker and one `recovered` marker, with no closed stimulus abort
reason. The `baseline_ready` marker is absent from the post-flash monitor
capture.

The host then rejected the marker sequence as `evidence_invalid`. The two
captured production markers begin after the canonical ESP log prefix, while
`validateMarkerSequence` currently accepts only lines whose first byte begins
the marker payload. Its deterministic fixture emits invented bare marker lines,
so it does not represent the production logger. Separately, the baseline marker
occurs early enough to precede post-flash monitor attachment. The complete
three-marker ordered witness is therefore unavailable even though the later
fault and recovery states ran.

The transaction performed ordinary exact-package recovery. Its typed public
facts report recovery complete, recovery flash used, no secondary recovery
failure, final projection withheld, child cleanup complete, and USB holder
cleanup complete. No candidate or public fault projection exists.

## Next safe action

Create a software-only continuation that first makes a real-process regression
use canonical prefixed production log lines and reproduce the current rejection.
Replace invented bare-line parsing with one strict shared ESP-log payload
boundary. Then make the baseline witness replayable or reader-armed so a
post-flash observer cannot miss it; preserve exact order, one-shot semantics,
redaction, and failure closure. Only a separately committed plan after that fix
may define attempt-006. Never reuse attempt-005.

## Non-claims

This closure does not publish hardware-regression evidence or verify THR-001.
It does not claim the absent baseline witness, exact five injected samples from
public evidence, physical overheat, electrical sensor failure, calibration,
mining, controls, other boards, or release readiness.
