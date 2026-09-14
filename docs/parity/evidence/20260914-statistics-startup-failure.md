# Statistics startup failure during guarded restart qualification

Stage-B installation 001 remains **unverified**. Its single state-preserving
write completed, but statistics startup failed. No controlled restart,
allowance issuance, reservation or mining followed.

| Binding                 | Identity                                                           |
| ----------------------- | ------------------------------------------------------------------ |
| Firmware                | `a74701d68505070dd00435005641fc3882ae8af8`                         |
| Gate                    | `e3f4ce6283ebeb4f58479aa66e530f9ea7bcbfd0`                         |
| ELF                     | `a3f11024e17aa091157c4b44e410cbd07e0e2eb516f350e9e087a4b029680299` |
| Failed inventory SHA256 | `883b0a4f40eb97c59ec03f517d283d556e4ea3412de9e807e44371fdcf7ab3f1` |

The protected root is
`scratch/qualification-implementation-20260907/iterative/reset-origin-restart-001`.
An independent read-only audit verified all 50 files, 15 directories, thirteen
retained artifacts, hashes, lengths and owner-only permissions. Historical
results and assignments are unchanged.

## Observed failure and limits

The thirty-second capture contained 49 `runtime_ready/complete` observations
whose first failure was `statistics`, spanning 24,160 device milliseconds.
Eleven boot observations spanned 27,470 ms on ordinal 507 with category `other`.
Six identity records matched the installed pair. No boot transition, panic,
allocation-failure receipt or fatal signature was captured.

The original flash result remains unchanged: write completed, capture untrusted,
exit 1, `startup_failed`, and `commit_ready:true`. That flag is preserved
verbatim and is not treated as a passing startup result.
The canonical reviewer rejected `restart_install_identity_startup`.

Authenticated accounting was collected **before** installation: next ordinal 17,
last completed 16, 1,380,000 ms charged, pending false; the original campaign was
exhausted at 240,000 ms. Post-install accounting and settings preservation were
not authenticated. Both before-install journal boundaries and actual cleanup
were verified: page, supervisor, three process observers, owned children,
listener and both serial-node holders are gone.

## Root-cause investigation

At the tested source, `statistics_runtime::start()` has exactly one fallible
operation: creation of its 8192-byte thread. It does not access statistics disk,
NVS or schema storage before returning this error. This identifies the failing
operation separately from the previously fixed oversized execution frames.

Memory pressure is the leading explanation:

| Runtime                  | Largest block before spawn | Observed free-byte decrease | Outcome      |
| ------------------------ | -------------------------- | --------------------------- | ------------ |
| Accepted daa recovery    | 8704 B                     | 9112 B                      | Started      |
| Failed a747 installation | 7936 B                     | 88 B                        | Spawn failed |

Those checkpoints measure `INTERNAL|DMA|8BIT`; pthread stacks use
`INTERNAL|8BIT`. The values support the hypothesis but do not prove the exact
failed allocation or errno. The decreases are observations, not allocator-cost
measurements. Original a261 also had this spawn failure before its later reboot,
so it predates the guarded restart interface.

## Targeted correction and continuation

The correction prepares the same 8 KiB thread before hardware/Wi-Fi allocation,
holds it behind a one-shot activation gate, and starts its existing sampling
schedule at the original statistics boundary. Unused preparation cancels.
Thirty-six bytes of retained diagnostics record the actual capability class,
matching-capability heap snapshots and explicit errno availability. No stack,
telemetry sleep, priority, safety deadline or ledger limit is reduced.

The [prospective successor contract](../../hardware/reset-origin-restart-qualification.md)
requires exact failed evidence, a verified changed pair, fresh current-image
idle accounting and known-failure observations, and a separate exclusive
assignment. The failed context cannot resume. A complete installation,
new-image stability observation, authenticated restart and independent cleanup
review remain necessary before fresh cadence qualification. The correction has
not yet been qualified on hardware; parity remains 90/95.

## Software verification checkpoint

Ordered firmware-repository Cargo checks pass, with 2239 tests and one existing
ignored test; all 130 canonical targets pass. Ownership, reference, redaction,
standards and parity/progress checks pass. The initial additional code exceeded
the 4 MiB OTA bound by 3,248 bytes. Removing unused private dynamic error formatting
and redundant ownership wrapping retained the checks and reduced the image to
4,193,968 bytes, leaving 336 bytes. The partition layout is unchanged.

The refreshed native audit binds preview ELF
`f587f219591a2741c1078cfe0a7e7f075dd477526cdfb67c1edbf77a294c2cfb`:
selected statistics path 5,936/8,192 bytes, main telemetry 12,560/16,384, and combined
static diagnostics 1,320/2,048. Audit SHA256 is
`436f7054998284bb5084374135cc89acae655610f311adef3da459bd9d6993c5`.
These exclude unmeasured whole-callgraph/IRQ/pthread/allocator depth. The clean
published package must still pass size and identity checks.

The new Gate passes 605 WebCrypto tests, TypeScript, browser build, package and
standards checks. Its required Rust suite stopped at two Docker container-create
timeouts, and later/headless checks remain incomplete. Docker's API and normal
restart are unresponsive. Gate publication and fresh hardware qualification are
blocked. The current published Gate lacks the required decoder; successor
preflight rejects it before reserving any assignment. No successor 002 or mining
reservation has been created.
