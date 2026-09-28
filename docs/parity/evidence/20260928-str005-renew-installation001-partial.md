# Renew installation001: write verified, candidate recovery incomplete

The state-preserving flash and trusted runtime observation passed. The overall
qualification did not: the candidate recovery collector failed before creating
its first recovery round. The sealed result is `complete=false`, with the exact
blocker `candidate_recovery_missing`.

| Observation                       | Result                                                                                                             |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Host / installed candidate source | `8d4e470e4cd3e18be510d3912586aeda65960275`                                                                         |
| Candidate full ELF SHA-256        | `d0dd775335e55e9d009a0a779bc938bebf64e76fe78d8d7c8144a06ef1c9d88e`                                                 |
| Gate                              | `bd26128788dd3f04842984e545fa2382c0b50561`                                                                         |
| Before image                      | `361425b902a3e214d6d0b052118c9e7a710458aa`, ELF `7f3ea3ce75bf3eb8eb4c9a23a5eb7de5110114e124c22341f5cd2a867f97ebf2` |
| Fresh before accounting           | next20 / last19 / 1,920,000 ms / pending=false                                                                     |
| Before retained attempt           | Boot9, generation6, terminal, socket closed, worker quiescent, no fence                                            |
| Installation duration             | 511,496 ms; exit0, no timeout/interruption                                                                         |
| Runtime observation               | `timed_out_after_trusted_output`, trusted, stable safe boot                                                        |
| NVS provisioning                  | `not_provided`; ordinary disjoint state-preserving update                                                          |
| Final host resources              | Released; fresh detector and owner/listener/serial checks passed                                                   |
| Candidate recovery rounds         | None; post-update accounting/preservation qualification remains incomplete                                         |

The original page connected to the expected new source/full ELF and displayed
matching settings, Device Identity and authorization preservation under the same
baseline ID. These interactive observations do not replace the missing candidate
recovery producer. The failed collector left the connection open; explicit
Stop/Close subsequently confirmed restoration and serial release before the tab
and server were closed. No Start, grant, renewal, mining, self-test or core clear
occurred in this attempt, and no reservation was refunded.

A production page/controller/WebCrypto regression reproduced the collection boundary: Gate's
page-owned V2 history retains the old terminal record across an intentional
firmware transition and rejects an empty record on the new boot. The UI failed
before the candidate-recovery-begin request produced any artifact. The exact
exception was not persisted by this attempt. The independent regression reproduced
`v2_retained_evidence` on the same old-terminal/new-idle transition; it does not
retroactively add that exception to the sealed attempt. A correction must retain old
history, restrict any transition to a verified changed image/new boot, and preserve
ordinary reconnect and one-use Start guards. The collector also needs unconditional
Stop/Close when its pre-collection status request fails.

Private evidence root: `scratch/str005-renew/installation001/attempt`.
Sealed inventory SHA-256:
`8d6a89683e92b6d798b92a01b38882138b5db5f177058c3c0ed8f9a04b85a5ad`.
The canonical candidate package is privately retained at
`scratch/development-core-dumps/build-8d4e470e`.

The native selected-path audits passed on this exact ELF: Start13,264/3,120 bytes
used/margin; Renew11,760/4,624, with a 2,048-byte minimum. These audits are not
complete stack bounds or runtime renewal proof. No new-image capture, accepted
share, heartbeat shutdown or original Share002 panic cause is claimed. Startup003
remains qualified on its actual older image; parity remains 90/95. The active
accepted-share task stays unresolved, and this partial root is immutable.
