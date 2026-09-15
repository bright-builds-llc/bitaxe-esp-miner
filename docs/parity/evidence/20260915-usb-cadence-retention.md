# USB cadence failure and diagnostic replay correction

Cadence ordinal 17, preparation 2 remains **unverified**. Five installations and
four update/reconnect cycles passed, as did the idle capture. USB load completed
all twelve maximum-size exchanges, but only 89 of 95 intervals were at most 750 ms
(93.6842%, below the fixed 95% requirement). Mining never armed; no allowance was
issued or consumed.

| Binding | Value |
| --- | --- |
| Firmware | `f056d336b02988d0b56c09ff06f489489aa67b1f` |
| Gate | `72235d884c3605ff77d484f210fcaa0d7218b47e` |
| ELF SHA256 | `b47a10598383b84f36218a9d3db3e40ba00718b5ae971b638015e7f46eeef638` |
| Failed inventory SHA256 | `2003dd26039daa1edad78057e88215a07fffab4883d955648b8a61ad12f8aacb` |
| Audit producer SHA256 | `f894edb393c176edc115e5408530a6ebe1045709f1b17543ca91450e19395c78` |

The protected root is
`scratch/qualification-implementation-20260907/iterative/cadence-017-preparation-2`.
Its immutable inventory covers 115 files and 16 directories. The completed USB
review is retained in journal 34 through the closed final journal 39. USB phase
completion rejected it, so `cadence-usb.json` and a successful result are absent.
The browser error was recovered through read-only inspection and recorded as
parent-observed `cadence_supervisor_rejected`; it is not a browser-generated
failure event or a replacement device summary.

## Measured boundary

| Measurement | Idle | USB load |
| --- | --- | --- |
| Complete intervals | 100 | 95 |
| Intervals at most 750 ms | 100 | 89 |
| Maximum interval, microseconds | 745891 | 828450 |
| Maximum processing, microseconds | 238342 | 320724 |

USB's worst interval decomposes into 320724 us of processing in the previous
iteration and 507726 us of gap. Within that previous processing interval,
retention occupied 153720 us. The independent maximum retention duration was
163131 us; it must not be summed with other independent stage maxima as though
they came from one iteration. CPU0/priority 5, publication, subscriber and capture
integrity checks showed no failures. All 190 queued sends completed.

The retained stage measurements point to processing cost rather than an extended
sleep. Source inspection identifies a concrete contention path: USB diagnostic
replay scans the retained circular log, potentially 512 KiB, while holding the
same mutex used to retain the operator snapshot pair. Each replay search
collects matching lines before selecting the requested boot field; missing or
evicted markers still require a scan. The pair writer itself validates and
copies only its two records. The measured retention stage includes both pair
retention and logging, so the exact hardware lock-wait share is unmeasured.
This is a supported contention hypothesis, not proof that the entire 153720 us
was spent waiting on that mutex.

The correction retains seven typed numeric boot-memory observations and the
first closed Worker startup failure independently at their producers. USB replay
then reads those bounded facts without touching the circular log. General log
download and atomic snapshot-pair retention/issuance stay unchanged. Cached
facts do not assert that an earlier general-log append succeeded. Software
tests must exercise the actual replay reader while the log mutex remains held;
new hardware must establish whether the correction satisfies cadence limits.

## Accounting and cleanup

The last authenticated ledger was the cooling review before capture:
next 17, last 16, charged 1,380,000 ms, pending false. There is no post-failure ledger observation.
No issued grant, consumption, mining arm, work, renewal or heartbeat suppression
was recorded. The original campaign remains exhausted at 240,000 ms with masks 7/7.

The final browser journal is closed with serial ownership released. The passive
observer closed on request, exit 0, after 126492 ms and reported complete cleanup.
The supervisor exited0; separate fresh host checks found no owned children,
listener or serial holders. The supplementary process observation retains
`complete=false`. Its 600-second expiry and exit 1 were separately parent-observed;
that incomplete record supplies no final cleanup proof.

## Evidence scope and continuation

The accepted Stage B004 controlled-restart result
`d1487b8aa8fbcb2301683acbdcf7a828fa2771eb6ef19b10b788af5b26016179`
belongs to the F056/Gate722 pair above. It remains transitive ancestry for a
corrected-image attempt, not a restart-success claim for that new image.

The [prospective cadence contract](../../hardware/cpu0-telemetry-cadence-qualification.md)
admits only preparation 3 from this exact sealed failure after verified software
correction and clean publication. It requires five fresh installations, four
cycles, all three captures, fresh accounting before a normal 180,000-ms reservation,
independent heartbeat shutdown, bounded cooling and actual cleanup. No timing
limit changes. Cadence, USB qualification and migration remain active;
parity remains 90/95.

## Software and native verification

The actual old replay reader failed the held-log-mutex regression and joined
after release. The corrected reader completes before release for all eight
rerouted slots, including absent observations. Focused tests pass: nine cache,
eleven replay and nineteen existing retention/publication tests. They cover
first-value retention, concurrent publication, incomplete/corrupt states,
closed wire formatting, log eviction and unavailable general-log storage.
Independent source review found no change to pair append, its two logging calls
or failure-before-issuance behavior.

The initial 53 affected and historical host tests passed, including evidence
corruption, contradictory work/cleanup/accounting, recursive lineage and
interrupted or duplicate assignments. The wider canonical run then exposed an
accessor-evaluation regression in a shared-reader wrapper. That wrapper was
removed: the new reader validates the existing context once, then checks the
already-bound restart receipt bytes by hash. Historical invocation paths stay
unchanged. The original failing test and six focused compatibility/integrity
cases pass, including source changes between validation and receipt rereading.
The read-only CLI revalidates the actual sealed preparation and reports it
unverified, with no post-failure ledger claim.

The corrected complete canonical graph passes all 138 targets using four
concurrent tests and unchanged per-test time limits. The first full run's
300-second context-test timeout and the isolated accessor assertion failure are
retained separately. Ownership, reference, semantic redaction, standards and
parity checks pass; no checklist or progress-history transition occurred.

Ordered Cargo format, Clippy, build and 2,239 tests pass, with one existing ignored
test. The unchanged Gate passes
605 WebCrypto/browser-controller tests. The native preview package builds in
57 seconds without a host launch stall. Its ELF SHA256 is
`8e1e36fa12080213bd533412e96d3abf4d006f500d1e945876b79856a1086dba`;
the app uses 4,118,160 bytes with 76,144 bytes of partition headroom.

Preview audit `16d97887004d1c2fee4d6a3f24d907207464bb7dd75df86d4503803d4137d4fb`
measures 116 cache bytes and 1,436 total diagnostic static bytes, below 2,048.
The selected main path uses 12,560/16,384 bytes; statistics 5,952/8,192;
USB receive 2,208/8,192; control owner 8,208/16,384; production entry
5,056/8,192. These are selected native paths, not complete bounds including
unknown library, ROM, allocator or interrupt frames. The compiler moved the
constant 8,192-byte prepared-thread stack setting into the shared helper; the
audit follows its actual Builder configuration and callback arguments. The
unchanged HTTP setup helper was inlined and its prior standalone-symbol audit
is not claimed for this image. Clean publication and fresh hardware cadence
remain required.

Independent native audit
`7ae7d170da3e32bdbc28f20b9e21dc5ad97e1430ec473e2db1cc9d31dc244a2e`
verifies selected instruction bytes against that ELF. Cache storage is
zero-initialized BSS. Each typed recording section contains one hardware
`s32c1i`, memory barriers, no calls and no backward branches. Producer frames
are 128/112 bytes, dispatcher 96 bytes and marker readers 112/80 bytes.
Error classification before recording and formatting/logging afterward still
allocate; this is specifically a bounded-cache-recording guarantee.
Supplemental proof
`3c25bf061994f1af6ed7fc3104629e3ef3602bd9a4c96a3f6261893532dde7ee`
confirms the shared Builder's explicit stack tag/value and callee loads.
