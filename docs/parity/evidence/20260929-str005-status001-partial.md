# Bounded diagnostic Start failed before a confirmed reply

One published zero-renewal Start/status attempt ran on the installed diagnostic
image. It did not qualify. The original Share001 panic did not recur in this
attempt: recovery diagnostics remained on boot23. The exact Start rejection
reason and retained V2 status remain unproved.

| Boundary                    | Observed result                                                                                                 |
| --------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Installed source / full ELF | `ce8f015811b93385c5aab0bac7bcdf6307452b31` / `453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31` |
| Before                      | Boot23, next ordinal21/last20, charged2,100,000 ms, pending=false                                               |
| Authorization               | One fresh normal180000 grant, initial60000 window, zero renewals                                                |
| Start                       | Admitted once; no reply confirmed; first client failure `start/operation_failed`                                |
| Stop request                | 14.6s after Start invocation; immediate Stop reported failure                                                   |
| Immediate recovery          | Ledger next22/last21, charged2,280,000 ms, pending=false; original charged240,000 ms, pending=false             |
| Device observations         | Boot23, restored idle/inactive state; retained status read failed                                               |
| Fixture                     | Natural completion failed; termination, reaping and actual release passed                                       |
| Host                        | Server/signer/fixture/serial release verified                                                                   |
| Qualification               | `complete=false`; no accepted share, normal Stop or panic cause claim                                           |

The sealed private root is `scratch/str005-status-repro/status001/attempt` with
inventory SHA-256
`dbda3bed4c468674c2005921e9a3ec1c0e7e9f0587dbc654909a8a92d38a04fc`.
Raw fixture, grant, network and device inputs stay in ignored owner-only storage.
The full self-test core archived earlier was preserved; this attempt did not
produce an observed panic reset.

One blocker in the old sealed result, `status_repro_late_completion_unknown`,
is a finalizer error. The browser reported `lateReplyPending=false` after Start
rejected. Later source corrects this rule, but the old seal remains unchanged.
The actual first failure and the independent fixture-completion failure remain
valid observations. A separate authenticated recovery must measure current
status and retained resources before a new effect can be considered. Parity
remains **90/95**, and the accepted-share task is unresolved.
