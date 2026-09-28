# STR-005 startup001: Start passed; recovery collection failed

The published `651b9a53` host probe ran once against installed firmware
`361425b902a3e214d6d0b052118c9e7a710458aa`, full ELF
`7f3ea3ce75bf3eb8eb4c9a23a5eb7de5110114e124c22341f5cd2a867f97ebf2`,
and Gate `d3ac37435fbf98af76113fe5b962989e3616f707`.

Preflight reverified installation007's actual checksum/full-ELF/captured-cutoff
proof. Fresh authenticated baseline measured next18/last17/1,560,000 ms and no
pending reservation. The official same-device clear compared the complete raw
partition against the preserved dump, verified erased readback, returned the
application and released ownership. No flash or capture self-test occurred.

| Measurement                       | Result                                                |
| --------------------------------- | ----------------------------------------------------- |
| New normal reservation            | Ordinal18, 180,000 ms; measured from fresh ledger     |
| Signed lease                      | 60,000 ms, zero renewals                              |
| Completed Start reply             | 10,182.5 ms after invocation                          |
| Work dispatched                   | 0→1, same generation                                  |
| Stop requested                    | 10,523.2 ms after invocation; 340.7 ms after reply    |
| Retained qualification after Stop | Safe stop complete; restoration requested; fan paused |
| Measured active time              | 6,493 ms                                              |
| Same-session accounting           | Next19/last18/1,740,000 ms; pending=false             |
| Original campaign                 | Remains exhausted at 240,000 ms                       |
| Final result                      | Partial; no parity promotion                          |

The same-session state before the failing status query confirmed the safe
baseline, restoration and inactive lease. Its resource observation reported
shutdown complete. These observations do not substitute for the missing
independent retained V2 resource record or authorization checkpoint.

The recovery collector sent a null-attempt status query first and expected a
local `v2_idle_correlation` error before retrying with the known attempt. The
firmware instead rejects a null query when a retained record exists; that
rejection revokes the serial epoch. Thus the expected fallback never runs, and
Close reports `close_failed`. A fresh native connection reached ready state, but
the fresh collector repeated the same invalid query before recording its session.
The final page reported closed serial ownership with restoration flags unconfirmed.
No second Start was attempted.

A separate static review found that normal Start/Stop does not capture the Gate
`WorkerAuthorizationRecoveryCheckpoint`: the pinned capture calls belong to
planned fault paths. The startup evaluator requires that checkpoint, but the
probe never captured one. It cannot be reconstructed from the public post-stop
state or a new page. This remains an independent qualification blocker even
after the status-query correction; no later current-state observation can be
relabeled as the missing original checkpoint.

The controlled fixture exited and its listener was released. The browser tab and
server were closed, and independent process/listener/device-holder checks passed
before sealing. The evaluator preserves `start_proven=true` and
`dispatch_increased=true`, but rejects completion because the fresh recovery
quorum and retained dispatch proof are missing. Its `first_failure=null` refers
to the successful main Start/dispatch phase; it does not mean recovery succeeded.

Private root: `scratch/str005-startup/startup001/attempt`.
Sealed inventory SHA-256:
`950a8e55b5efb468905a4edd2e739cfd02e47013797e026bf0218af955234cb7`.
Raw archives and all earlier seals remain unchanged. Ordinal18 is now measured
as consumed and completed; no refund or budget reset is implied.

The consumed Start and clear gates are disabled. The targeted software correction
queries the known attempt directly; broadening the error fallback would be wrong
because the rejected query has already revoked authority. Any new recovery must
use its own published failure-only contract. Original Share002 cause remains
unproved, and startup001 cannot be archived as qualified success. Parity stays
90/95; accepted-share and heartbeat-loss claims remain separate.

Subsequent [recovery001](20260928-str005-startup-current-recovery.md) verified
current restoration, unchanged accounting and retained resource release after
the collector fix. It supplements this immutable partial result and leaves the
missing historical authorization checkpoint unresolved.
