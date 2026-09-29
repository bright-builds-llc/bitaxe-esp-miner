# Current safety restored; failed Start cause remains unresolved

Recovery006 deliberately omitted retained-status queries after the earlier query
interfered with Stop. It collected authenticated state, accounting and diagnostics,
then ordinary Stop and Close completed without a first failure. The finalizer
reports `current_safe_recovery=true` and complete host release. Its overall
result remains `complete=false` because retained status was intentionally not
requested; no historical resource or mining qualification success is implied.

| Boundary                    | Direct evidence                                                                                                             |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Installed source / full ELF | `ce8f015811b93385c5aab0bac7bcdf6307452b31` / `453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31`             |
| Accounting                  | Next22/last21/charged2,280,000 ms, pending=false; original charged240,000 ms, pending=false                                 |
| Device and host             | Same boot23, exact identity, inactive lease, confirmed restoration, Stop/Close and host release                             |
| Failed generation           | Generation3, `unsafe_observation` revocation, safe stop complete, budget complete, zero work dispatched or shares submitted |
| Preparation receipt         | Current boot23, generation3, steps1–4 completed; step5 (500-ms core-voltage stabilization) cancelled                        |
| Preparation resources       | Internal free heap3,451 bytes; largest block2,176 bytes; stack free9,516 bytes at the failed boundary                       |
| Retained V2 record          | Not requested in the safety-only stage                                                                                      |

The sealed private recovery006 root is
`scratch/str005-share-diagnostic/recovery006/attempt`, inventory SHA-256
`08b70903f7129ab435945871746d853dd4a297a2205e8c0660b2eb373a158d56`.
Its prior recovery005 seal remains
`b5c2ebf1c0805f90498759f9fe798255f4a5f33892d33943b658e407663b1741`.
Raw device, fixture and debugger inputs remain private.

The preparation receipt and revocation show where Start ended, but they do not
identify whether the unsafe observation came from a stale sensor sweep, a zero
fan sample, or another unsafe reading. Low free internal heap is a concrete
pressure signal, not proof that an allocation caused this failure. There was no
observed panic reset in status001, so the newly qualified core instrumentation
could not decode this Start rejection. The original Share001 panic cause and
historical retained-resource gap remain unresolved. Another Start requires a
separate published diagnostic contract that captures a closed failure category;
no grant replay or blind retry is authorized. Parity remains **90/95** and the
accepted-share task stays active.
