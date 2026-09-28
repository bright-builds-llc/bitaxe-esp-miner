# STR-005 normal-stop successor compatibility

This review admits only the next bounded startup successor under ADR-0029/0031.
It does not declare four-cycle qualification of a new firmware/Gate pair, promote
parity, or repair an older sealed result. Accepted-share and heartbeat-loss
probes retain their own claim-specific verification requirements.

## Installed firmware and historical capture

Keep firmware `361425b902a3e214d6d0b052118c9e7a710458aa`, ELF SHA-256
`7f3ea3ce75bf3eb8eb4c9a23a5eb7de5110114e124c22341f5cd2a867f97ebf2`.
Installation007's capture, full ELF/checksum and actual captured cutoff evidence
remain bound to historical Gate `d3ac37435fbf98af76113fe5b962989e3616f707` and
inventory `7131552c725c17c070b52b4238b2a92a742ccf34691ebadb5df2885ac3d5f925`.
The new Gate identity must be recorded separately; do not overwrite that tuple.

The last identified four-cycle evidence belongs to firmware `cf7a3f03` / Gate
`e20c0fd5`. The intervening firmware changes therefore require an impact review,
even though the proposed checkpoint correction itself changes only Gate.

| Boundary                                                              | Actual difference to installed firmware `361425b9`                           | Verification and limits                                                                            |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Native USB driver/receiver, framing, credit window, ordinary liveness | Byte-identical driver, owner/link and worker-control serial mechanisms       | Reuse mechanism evidence; rerun ownership/symbol and relevant serial regressions                   |
| USB diagnostic writer                                                 | Two bounded replay records; indices 21→23, modulo 22→24                      | Validate serialization/replay and fresh application admission; do not claim identical load/timing  |
| Dispatch                                                              | Separate non-inlined signed Start and new off-only self-test command         | Exact ELF native stack audit; actual Gate Start/Stop flow; fresh bounded hardware startup          |
| Ordinary authorization/preservation/settings/NVS                      | Verifier, models, settings adapters and NVS ownership unchanged              | Fresh same-page preservation and both ledgers; no synthetic authorization checkpoint               |
| Generation gate                                                       | Ordinary algorithm unchanged; DRAM placement and panic revocation added      | Exact native placement/cutoff proof; later independent heartbeat-loss measurement                  |
| ASIC startup                                                          | Explicit low reset constructor and cutoff marks                              | Actual captured cutoff plus conservative fresh startup/normal Stop                                 |
| Allocation/build                                                      | Internal allocation threshold 2048; API optz; debug ELF/map; explicit IDF O2 | Retained resolved-config/resource audits and fresh readiness; no historical timing equivalence     |
| Partition/package                                                     | Core region grew 64KiB→952KiB at 0xf12000; other geometry unchanged          | Installation007's actual package/preservation evidence; no new write needed for a host-only change |
| Flash admission                                                       | Stronger physical binding and ROM drift rejection; dump operations added     | Existing segmented-update/NVS protection tests; ordinary segment construction unchanged            |

Installed native audits retain a 13,264-byte signed-Start chain and a 240-byte
store-wrapper addition, plus selected cutoff-region proof. Startup001 observed
Start, dispatch and completed accounting on this image; its historical checkpoint
gap remains. Recovery001 independently verified current restoration/resources.
Neither observation supplies missing four-cycle evidence for this pair.

## Current Gate checkpoint change

Published Gate `bd26128788dd3f04842984e545fa2382c0b50561` preserves Controller 0.4, serial 0.2,
possession 0.2, wire schemas, signed authority and device safety deadlines. The
normal-flow change retains the actual private post-Start/Renew authorization
state, matches it against restored same-generation state at Stop, and keeps the
original preservation baseline through Close and fresh reconnect. Public state
or a caller-supplied hash cannot manufacture the checkpoint.

The actual page/controller regression went red before the correction and green
with it. All 22 focused checks and 840 web tests passed, along with Gate's Rust,
browser, build/package and standards checks. Negative cases cover missing/stale
observations, wrong generations, unexpected advance/rollback, failed restoration,
unknown authorization, repeated Stop/Close/reconnect and preserved heartbeat-fault
checkpoints. Independent review found no blocking issue.

The three production paths changed are `worker-authorization-recovery.ts`,
`worker-normal-authorization.ts` and `worker-serial-acceptance.ts`; the remaining
six paths are tests/support and documentation/task records. The committed JSON
binds all nine paths and the exact diff. Unchanged source blobs for the serial
controller, V2 control, protocol fixtures/specification and dependency manifests
are checked before admission. Archive SHA-256:
`78bffd148a6393398e9347900eb35ad02208640c046c8c5598db0e2208131e05`;
clean bundle SHA-256:
`61801bd68b495d67af728d0ef24d2dbabbe5d3f38a065b8b43f40db935dce38a`.
Fresh evidence binds the current Gate; historical capture remains on d3ac374.

## Normal Stop result interpretation

The V2 share record requires a real written and acknowledged share to become
`accepted`. A startup probe deliberately stopped earlier can truthfully retain
`rejected/evidence`. A network operation interrupted by requested revocation can
instead retain `worker_quiescent/authority`. Preserve those categories.

Startup completion must join the same generation, native `restoration_requested`,
completed safe Stop, exact resource release and the device failure timestamp.
An earlier authority failure, another reason, protocol failure or ambiguous
same-millisecond ordering is not an expected normal-stop cancellation. Retained
V2 `revoked`/`shutdown` event times can describe later observation; actual gate
closure and shutdown initiation come from the native qualification timestamps. Native
millisecond timestamps are floored: require a failure strictly beyond their
possible interval before claiming it followed Stop. An evidence failure is
eligible only at terminalization without an acknowledged share; it is not a
blanket exemption for evidence failures. Tests exercise the real V2Record and
existing generation-gate behavior, as well as the startup evaluator's rejection
of a failure before Stop.

## Fresh preparation and successor admission

Preserve the completed retained record before restarting. Use a fresh V2 recovery
page to read the known attempt directly, both ledgers, restoration and release.
Then use a separate restart-only page: Gate forbids V2/restart combination and
restart-mode downgrade, and those guards remain intact. On that page capture
before/after preservation and accounting, issue one nonce-bound normal restart,
and verify matched ACK, explicit software reset, exact installed identity,
boot+1, healthy readiness, bounded observer completion and actual release.

The new startup page must observe that exact post-restart boot, idle V2 state,
fresh identity/preservation/accounting and qualified cooling. This refreshes
volatile admission without flashing, clearing NVS or consuming a work ordinal.
The already verified erased core region may be reused only with the sealed
clear proof and a complete no-intervening-panic boot chain; never assume it is
empty solely because a previous command reported success.

The resulting evidence covers this bounded startup claim. It does not establish
accepted-share success, heartbeat-loss timing, new-pair four-cycle durability or
Share002's original cause. The later integration review must retain the exact
identities and identify any remaining uncovered interaction. Parity stays 90/95.
