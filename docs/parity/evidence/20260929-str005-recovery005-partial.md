# Current accounting saved; restoration not confirmed

Recovery005 was a fresh read-only continuation after the failed status001 Start.
It preserved authenticated accounting and diagnostics before the retained-status
request failed. Ordinary Stop then failed. Close released the serial connection,
but its receipt did not confirm restoration or an inactive lease. Current safety
and historical resources therefore remain unproved.

| Evidence                           | Result                                                                                                          |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Installed source / full ELF        | `ce8f015811b93385c5aab0bac7bcdf6307452b31` / `453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31` |
| Current ledger                     | Next22/last21/charged2,280,000 ms/pending=false                                                                 |
| Original campaign                  | Charged240,000 ms/pending=false                                                                                 |
| First failure                      | `status/operation_failed`                                                                                       |
| Cleanup                            | Stop failed; Close and host resource release were observed                                                      |
| Current safety                     | Unproved; restoration and lease inactivity were not confirmed                                                   |
| Retained status                    | Unavailable                                                                                                     |
| Share001 historical resource proof | Unavailable                                                                                                     |

The private root `scratch/str005-share-diagnostic/recovery005/attempt` is sealed
with inventory SHA-256
`b5c2ebf1c0805f90498759f9fe798255f4a5f33892d33943b658e407663b1741`.
No grant, Start, mining, flash, reset or core operation occurred in this recovery.
The next separately published stage gathers current state and accounting first,
then performs Stop and Close without a status query. A successful serial Close
alone is not counted as device restoration. The accepted-share task remains
unresolved and parity remains **90/95**.
