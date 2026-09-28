# STR-005 startup003: startup and normal-stop qualification passed

The independently evaluated and sealed startup003 result is complete, with no
blockers or recorded owner failure. The normal-stop authorization checkpoint
matched after a fresh connection on the original page and remained matched
through the final Stop/Close.

| Identity / measurement                    | Verified result                                                                     |
| ----------------------------------------- | ----------------------------------------------------------------------------------- |
| Host source                               | `482de031`                                                                          |
| Installed firmware                        | `361425b902a3e214d6d0b052118c9e7a710458aa`                                          |
| Full ELF SHA-256                          | `7f3ea3ce75bf3eb8eb4c9a23a5eb7de5110114e124c22341f5cd2a867f97ebf2`                  |
| Current Gate                              | `bd26128788dd3f04842984e545fa2382c0b50561`                                          |
| Historical capture Gate                   | `d3ac37435fbf98af76113fe5b962989e3616f707`; unchanged                               |
| Boot / started generation                 | 9 / 6                                                                               |
| Start reply                               | 10,108.6 ms after invocation                                                        |
| Dispatch                                  | 0→1; matching retained `asic_dispatch`                                              |
| Stop request                              | 10,528.9 ms after invocation; 420.3 ms after reply                                  |
| Authority                                 | One 60,000-ms grant; zero renewals                                                  |
| Accounting                                | Ordinal 19 completed; next 20; 1,920,000 ms charged; pending=false                  |
| Native active_ms                          | 6,460                                                                               |
| Native gate closure / shutdown initiation | 1,161,836 / 1,161,885 ms                                                            |
| Stop reason / completed stage             | `restoration_requested` / `fan_paused`                                              |
| Checkpoint / preservation                 | Fresh match; original page baseline retained                                        |
| Cleanup                                   | Socket closed, worker quiescent, no retained fence; serial and host owners released |

The retained V2 outcome remains `rejected`, with
`worker_quiescent/authority` at 1,161,838,795 microseconds. This is demonstrably
after the native gate-closure millisecond interval. Socket closure and worker
quiescence occurred at 1,161,841,242 and 1,161,841,669 microseconds. The evaluator
requires this exact normal-stop correlation; it does not broadly ignore failures
or label the record accepted. No accepted share was required or claimed.

Preparation003 separately proved one no-mining software restart from boot 8 to 9
in 8,447 ms, with matched ACK, unchanged same-page preservation and both ledgers,
followed by release. Startup preflight independently revalidated those producer
artifacts, the erased-core boot lineage, the actual historical capture/cutoff,
the reviewed Gate source difference and the exact installed native ELF. The
selected Start ancestry is 13,264 of 16,384 bytes, leaving 3,120 against a 2,048-byte
minimum; this remains selected-path evidence, not a complete callgraph bound.

Private roots and sealed inventory SHA-256:

- `scratch/str005-startup/preparation003/attempt`:
  `5922c9cc8e39316924d5c9f3cfe607f6ecc6e97cd430ca4c67758a2e04d8a391`
- `scratch/str005-startup/startup003/attempt`:
  `cc5bab7bd34da6f3b50388c31f5e1fc4aefb6a7faf12bd14678a6dc08593e7a1`

Preparation001/002 and startup002 remain failed, immutable pre-Start attempts;
none issued a reset claim or mining grant. Their clock/handoff and failure-recording
corrections were tested before continuation. Startup001's missing historical
checkpoint remains missing. This successful new result does not modify those
records or establish Share002's original panic cause.

The startup task's criteria pass and its full native record is archived. The
consumed startup/preparation gates are disabled. The original campaign remains
exhausted at 240,000 ms and no reservation was refunded. Parity remains 90/95;
there is no new-pair four-cycle durability or heartbeat-loss claim.

The accepted-share successor has a separate prerequisite: signed Renew on this
installed image has only 352 bytes of selected-path stack margin. Its isolated
replacement and native guard must qualify a new image before the planned
renewal-bearing share test. Renew was not exercised by startup003.
