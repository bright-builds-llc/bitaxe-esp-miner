# Share001 current recovery verified

Recovery003 on published host `c5513603` passed fresh current-state qualification
on installed firmware f000872f/full ELF a3e25741 and Gate9643. It performed no
Start, grant, renewal, reset, flash, core acquisition or clear.

Authenticated accounting measured next21/last20/2100000ms, pending=false. The
original campaign remains masks7/7/240000ms, pending=false. Current authenticated
V2 status was idle with record=null on boot16. Restoration, inactive authority,
disabled mining, current same-page settings/Device Identity/authorization
preservation, native Close and actual host/listener/serial release all passed.
Every independent collection stage succeeded. A current recovery proof was
produced from the original fresh collection timestamp after host release.

Private root: `scratch/str005-share-recovery/recovery003/attempt`.
Inventory SHA-256:
`9f8b0f41b9976c386674ad38cf22243d0243e69b02960764b591058ff08e7c5d`.
Result: current_safe_recovery=true, fresh_effect_proof=true, blockers=[],
accounting_measured=true, host_resources_released=true.

This is current-state recovery only. Historical Share001/Share002 retained
resource proof and original authorization checkpoints remain unavailable;
qualification_complete=false, retained_resource_proof=false,
historical_resource_unavailable=true. Their original seals and failed outcomes
remain unchanged. The active accepted-share task is not archived. Parity90/95.

Recovery001 independently measured accounting but exposed a diagnostic receipt
filename mismatch and post-Stop possession invalidation. Recovery002 passed all
collection operations but exposed the current judge's ready-only status check.
Both partial roots remain immutable. Corrections were separately regression-tested,
reviewed, committed and pushed before each new collection. Current admission now
accepts Gate's actual baseline_confirmed state only through a collector-local
opt-in with all restoration/preservation/authority checks; no state is relabeled.

Verification included production Gate Stop, context and diagnostic-consumer
boundaries, real HTTP/process/listener tests, original-part offline reproduction,
eight affected Bazel targets, ordered Cargo checks (2427 passed,3 existing
ignores), standards, redaction, reference and scoped Markdown/diff checks.

Core acquisition requires a separate published stage and another fresh proof
from that exact host revision. This sealed observation cannot be reused after
its freshness window or across a source revision. No dump validity or panic cause
is claimed here.
