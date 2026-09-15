# HTTP task allocation failure in restart preparation003

Preparation003 remains **unverified**. Its single state-preserving write
completed, but the HTTP server task could not be created. No post-install
authenticated admission, controlled restart or mining occurred.

| Binding | Value |
| --- | --- |
| Firmware | `253658cea424e455456430a399eb509ae04e0bd0` |
| Gate | `72235d884c3605ff77d484f210fcaa0d7218b47e` |
| ELF SHA256 | `6991aab2d448f4d7c8112d5333e9bfb3fe4e72a6b8b1c8a4c9137a7823c92460` |
| Failed inventory SHA256 | `d560b740bcca4936dd6c39722a81165112decc5579ce2c4f78d8d588903c855c` |
| Audit producer SHA256 | `7e90435786f3ab61594991757863ed8d7f3eae4e841e0d2252c6416e2c33acac` |

The protected root is
`scratch/qualification-implementation-20260907/iterative/reset-origin-restart-003`.
Its seal covers 51 files and 15 directories, including all thirteen runtime
artifacts, exact ancestry, fresh pre-install known-network-failure review and
accounting, installation claim, capture, process observations and host cleanup.
No historical inventory was rewritten.

## Failure and source interpretation

The retained marker is:

```text
storage_http_failure schema=v1 phase=http_server error=http_task redacted=true
```

The pinned ESP-IDF `httpd_start` reaches this error only after its context,
semaphore and socket initialization succeeded and task creation failed.
`httpd_os_thread_create` delegates to `xTaskCreatePinnedToCoreWithCaps`.
The configured server requests a 16384-byte stack, priority5, no affinity and
`INTERNAL|8BIT` capabilities2052. The error identifies stack/TCB allocation
failure, not SPIFFS mounting, route registration or an executing HTTP stack
overflow. The exact failed suballocation was not retained.

Later memory observations show 29967 internal/DMA bytes free with a largest
block of 13312 bytes. They were taken after HTTP cleanup and network startup,
and use a stricter capability subset than the task allocation. They support
investigating memory placement but are not the missing matching-capability
snapshot at failure.

There are 58 startup records carrying `first_failure=storage_http`: nine
`network/entered` records and 49 actual `runtime_ready/failed` records. Six HTTP
failure markers, nine statistics `prepared` markers and 49 statistics `active`
markers were captured. All 69 boot records show ordinal1/reset category `other`,
with no captured transition, panic or allocation receipt. Last boot/startup
uptimes are 29801/29802 ms. Initial reset attribution remains unknown.

Preserve command exit1, completed write, untrusted timeout, `trust_basis=none`,
`commit_ready=true`, `startup_complete=false`, `startup_failed=true`, and all
three issues: `error_diagnostic`, `startup_failed`, `startup_incomplete`.
The canonical installation reviewer rejected `restart_install_identity_startup`.
No tool flag overrides that failed qualification.

## Accounting and cleanup

Last authenticated accounting was before installation: next17, last16,
charged1380000 ms, pending false. The original campaign remains charged240000 ms
with masks7/7. Post-install accounting and preservation are unverified.
No allowance, controlled restart or work was issued or consumed.

Detector and installation process observers completed with 13 and 278 samples,
respectively, and no failures or remaining processes. The supplementary
supervisor observer reached its fixed600-second lifetime: 1133 samples,
`complete=false`, exit1. That incomplete record is preserved and supplies no
final cleanup proof. Actual cleanup was separately checked after browser
closure and supervisor exit0: no owned children, listener or holders of either
serial node remained. The flash capture reports a safe baseline; no fresh
post-install application admission is claimed.

## Prospective correction

The HTTP startup routine currently creates its 8192-byte deferred worker before
the 16384-byte server task. The correction creates the larger task first within
that same stage. Route registration and readiness stay after both owners exist;
a later deferred-worker failure drops the server and preserves the first error.
Wi-Fi buffers, the internal reserve, stacks, priorities and deadlines stay fixed.

A deterministic priority-region allocator model reproduces a failure when the
smaller allocation consumes the region needed by the larger task, and succeeds
with the revised order. That model is regression coverage for the orchestration
decision, not a measurement or complete simulation of ESP-IDF heap placement.
Actual allocation success still requires a new exact-package hardware result.

The [restart contract](../../hardware/reset-origin-restart-qualification.md)
permits only an exact sealed003 successor after verified correction, clean
publication, fresh known-failure/accounting review and exclusive assignment.
No old failure or missing proof becomes successful through supersession.
Cadence, USB qualification and migration remain active; parity remains90/95.

## Software verification before publication

Three tests cover the production orchestration helper. Reversing that helper's
actual order in a private mutation makes its regression tests fail. Independent
review confirms that routes/readiness and external activation remain later;
actual ESP-IDF cleanup under memory exhaustion remains a hardware postcondition.

Ordered Cargo checks pass. All 135 canonical targets passed across the full run
and one isolated rerun. The historical context suite timed out during concurrent
verification; its unchanged isolated run passed in 90.3 seconds under the existing
300-second limit. The timeout log remains retained. All 64 affected host tests
and the actual Gate parser probe pass.

Preview native audit SHA256
`fcdcd74e68bcf10e78f30c73f0449a6460d81dbba5632fce21088ea0d8d9c454`
binds ELF `9d518f8d6c3979bcf5f8f43c1539b85b9127d0b6fd3d5988da0e3b78f8fa986c`.
The image is 4119008 bytes with 75296 bytes free. Existing measured runtime stack
paths remain within their bounds; server/deferred setup paths measure2736/2448
bytes in main's16384-byte stack. Static diagnostic storage remains1320/2048.
Unknown library/ROM/allocator/interrupt depth remains excluded. Exact clean
publication and new hardware evidence are still required.
