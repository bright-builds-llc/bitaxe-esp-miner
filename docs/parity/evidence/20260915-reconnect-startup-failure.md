# Reconnect thread allocation failure after statistics reservation

Stage-B successor002 remains **unverified**. Its one state-preserving write
completed, and the corrected statistics task became active. Startup then
reported a returned out-of-memory error creating the Wi-Fi reconnect thread.
No controlled restart or cadence qualification was attempted on this image.

## Exact runtime and retained evidence

| Binding | Value |
| --- | --- |
| Firmware | `582fc2bf94779e6ea867e5de87b97695df5a20cd` |
| Gate | `72235d884c3605ff77d484f210fcaa0d7218b47e` |
| Application ELF SHA256 | `8095cfcf3991129188a39bc28f56796ee1859fd22d70ada6a1dc8c7359051558` |
| Failed inventory SHA256 | `29c8f6bb14b1962676bdd8af4a5c9b406ea81ab0f68c61882c51c2680eb0d7ad` |
| Failure audit producer SHA256 | `4946a65aaa619c0dd291ca09932cadaa87ed2c7f1eccb7507cf8ef2a34e5efab` |

The protected local evidence root is
`scratch/qualification-implementation-20260907/iterative/reset-origin-restart-002`.
Its new failure-only inventory contains 51 files and 15 directories, including
all thirteen retained runtime artifacts. Independent checks covered the exact
context and predecessor chain, fresh pre-install known-failure review and
accounting, exclusive installation claim, completed ordinary write, capture,
mandatory installation observer, final closed pre-install journal and actual
host cleanup. Inventory equality was checked again after sealing.

The canonical installation reviewer rejected
`restart_install_identity_startup`. Preserve the legacy values: command exit 1,
`flash_status=completed`, `capture_status=timed_out_without_trusted_output`,
`trusted_output=false`, `trust_basis=none`, `commit_ready=true`, and issues
`error_diagnostic` plus `startup_failed`. `commit_ready` is a retained tool field,
not qualification acceptance.

## Observations and diagnosis

The closed failure marker is:

```text
wifi_startup_failure schema=v1 phase=reconnect_spawn error=no_memory redacted=true
```

In the tested source, that phase wraps the fallible spawn of the 8192-byte
`wifi-reconnect` thread. The error classifier maps the returned I/O
`OutOfMemory` category without formatting private error text. Wi-Fi driver
preparation and station association had already succeeded. The marker therefore
identifies thread creation failure; it does not indicate bad credentials,
association failure or NVS corruption. The exact pthread errno and failing
suballocation were not retained.

Statistics preparation now succeeded before late startup allocations. Its
matching `INTERNAL|8BIT` capability 2052 snapshots showed free 270983/largest 129024
bytes before preparation and free 261883/largest 120832 afterward. Nine retained
`prepared` and 49 `active` markers used the unchanged 8192-byte stack. These
snapshots are observations across preparation, not an isolated allocator-size
measurement. They support the previous statistics allocation diagnosis while
showing that successful statistics startup alone does not prove the whole
service set fits.

The capture contains 49 `runtime_ready/complete/first_failure=network` samples,
five reconnect failure markers and 69 boot records. All captured boot records
report ordinal 1/reset category `other`; the last boot/startup uptimes are
29751/29752 ms. No captured boot transition, panic receipt or allocation-failure
receipt appears. Initial reset attribution remains unknown.

The remaining required late thread is the 8192-byte USB receive loop. The
targeted correction reserves both reconnect and receive threads before later
startup fragmentation, with activation kept at their existing boundaries.
The statistics activation gate is shared rather than duplicated. Stack sizes,
priorities, CPU affinity, heartbeat enforcement and telemetry cadence stay fixed.
Conditional captive-DNS and explicit reconnect-probe threads remain separate
paths; this campaign does not qualify them.

## Accounting, cleanup and limitations

Fresh authenticated accounting before installation was next 17, last 16,
charged 1380000 ms, pending false. The original campaign remained charged 240000 ms
with masks 7/7. The browser captured a new private preservation baseline on the
installed a747 image and verified its same known statistics failure before
releasing USB. There is **no authenticated post-install accounting or
preservation proof** for successor002. No allowance was issued or consumed,
no controlled restart was claimed, and no mining occurred.

The mandatory prearmed installation observer completed with 281 observations,
no recorded failures and no remaining processes. The supplementary supervisor
observer reached its 600-second lifetime during earlier browser-connection
troubleshooting: 1133 observations, `complete=false`, exit 1. Its original record
is preserved and supplies no final cleanup proof. It is not the mandatory
installation observer and is not treated as complete coverage.

Actual cleanup was checked separately: the released browser page was closed,
the exact owned supervisor exited 0 after SIGTERM, all three observers exited,
and fresh process/listener/serial checks found no owned children, listener or
holders of either serial node. The flash capture reports a safe baseline;
fresh post-install application admission remains missing.

## Prospective continuation

[The restart contract](../../hardware/reset-origin-restart-qualification.md)
defines one narrowly guarded successor003 after software correction, tests,
native audits and exact clean publication. The failed002 seal is the immutable
anchor; closure itself grants no effects. Before the next installation, fresh
authenticated review must match the actual installed002 image, boot 1/other,
advancing known reconnect-spawn failure, active statistics and unchanged ledger.

Only a passing installation, healthy observation and independently verified
authenticated restart may admit fresh cadence17-2. Its four cycles, three
captures, normal 180000-ms reservation and heartbeat shutdown criteria remain
unchanged. USB qualification and migration remain active; parity remains 90/95.

## Correction verification before publication

The correction reserves the existing reconnect and USB receive workers before
late allocations, preserving their activation gates. A prepared credential enum
requires the station credentials and reconnect worker together; missing/invalid
credentials cause no reconnect allocation. Direct tests exercise the production
factories, and private mutation tests reject removed gates and unconditional
AP-only reservation. Independent review found no remaining actionable issue.

The firmware-only release `z` profile reduces the preview to 4118992 bytes, with
75312 bytes remaining in the fixed OTA slot. Ordered Cargo checks pass (2239
tests, one existing ignored); all 133 canonical targets and Gate's 605 WebCrypto
tests pass. The narrow telemetry auditor update passes 18 tests and retains its
original failure limits.

Preview audit SHA256
`ceabd7d69d723c330bf89e12a0ca215d5bd257a1814e950a8ae81f5a2f7fe992`
binds ELF `5f14c644ea27b2d1321ca79ac6a777ba833e2114d11862760143bcb69a227485`.
Its selected main path is 12448/16384 bytes, statistics 6960/8192, USB receive
2224/8192, reconnect 944/8192, control owner 8224/16384, and the production owner
entry 5024/8192. Static cadence/diagnostic storage is 1320/2048 bytes. Callback
vtable and constructor bindings are retained; unknown library, ROM, allocator
and interrupt depth are excluded. No whole-task stack or hardware safety bound
is claimed.

This preview still requires clean publication/package identity and fresh
hardware qualification. None of these software results changes failed002's
outcome or supplies its missing post-install proof.
