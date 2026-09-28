# STR-005: current recovery after startup001 verified

Published host `6bcc5c3d` performed one failure-only recovery against the exact
sealed startup001 attempt. It used fresh authenticated possession and queried
the known attempt ID directly. No Start, grant, renewal, fixture, flash, reset,
core clearing or mining occurred. The installed source/ELF/Gate pair is unchanged
from [startup001](20260928-str005-startup001-partial.md).

| Current observation                     | Result                                                                         |
| --------------------------------------- | ------------------------------------------------------------------------------ |
| Authenticated boot / session generation | 8 / 3                                                                          |
| Retained attempt generation             | 1, matching startup001                                                         |
| Retained record                         | Terminal; `asic_dispatch` present                                              |
| Socket                                  | Closed                                                                         |
| Worker                                  | Quiescent                                                                      |
| Fence                                   | Not retained                                                                   |
| Ledger                                  | Next19, last18, 1,740,000 ms, pending=false                                    |
| Original campaign                       | Exhausted at 240,000 ms; unchanged                                             |
| Restoration                             | Confirmed against this page's current baseline                                 |
| Lease / serial                          | Inactive / released                                                            |
| Collection failures                     | None                                                                           |
| Host cleanup                            | Browser/server closed; process, listener and fresh device-holder checks passed |

The retained V2 outcome is `rejected`, with first failure
`worker_quiescent/authority` at device time 102,770,664 microseconds. This follows
the recorded normal Stop gate closure at 102,769 ms. Socket closure and worker
quiescence were observed at 102,774,203 and 102,774,656 microseconds. These values
are preserved as reported; they are not relabeled as accepted-share success or
as a new startup panic.

The corrected collector now passes on the real device. Before correction, the
production Gate regression reproduced the null-attempt query's rejection and
epoch revocation. After correction, the same regression and this fresh hardware
read both succeeded. Other reads and Stop/Close remain independent of a failing
status read. Unknown Start state makes no speculative attempt query.

Private root: `scratch/str005-startup/recovery001/attempt`.
Sealed inventory SHA-256:
`cee3e1fa3d570774dc4f68c2cf576f7be5bee2bed31aafb3a277f3196366b07a`.
Its result reports `current_recovery_complete=true`, no blockers, measured
accounting, current restoration, retained resource release and serial release.
It also reports historical authorization checkpoint and preservation verification
as false, `qualification_complete=false`, `parity_promotion=false` and
`start_replayed=false`.

This new page establishes current state. It cannot reconstruct the previous
page's private preservation baseline or an authorization checkpoint that normal
Start/Stop never captured. Startup001's immutable partial result remains partial;
its original same-session restoration evidence is not erased or overstated.
The task stays open with that precise historical checkpoint blocker. A tested
prospective normal-stop checkpoint path is required before a future qualification
trial. All consumed execution gates are disabled, earlier seals are unchanged,
and parity remains 90/95.
