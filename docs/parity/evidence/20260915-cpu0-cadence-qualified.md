# Ultra 205 CPU0 telemetry cadence qualification

**Passed:** ordinal 17, preparation 3, on the exact published pair below.
This closes the remaining CPU0 cadence obligation identified by the
[USB reconciliation](20260913-usb-qualification-reconciliation.md).
It does not advance a parity row or grant BWG/STR successor authority.

| Binding | Value |
| --- | --- |
| Firmware | `8af54d8d8919aeb0f309203f790269803365fb9a` |
| Gate | `72235d884c3605ff77d484f210fcaa0d7218b47e` |
| ELF SHA256 | `35ea53e7a4153fb17f21b8bb8b64653506bb909be1bd9d61cb7e96b130820b6a` |
| Result SHA256 | `75f62388727078793177d549e1fcc1a9eadef9fdbbdba8fcc4b0f47147b911c0` |
| Sealed inventory SHA256 | `fdc6db7219f6c3898b16cc63206da544e0120ed2d3b9b24dd72fd2a2222e48a9` |
| Thirteen-artifact snapshot SHA256 | `836a2ab2367d7fecb2d636b5464697fec9ceb97f4a32d8aac0e0873d816aff5a` |
| Read-only auditor SHA256 | `3f5b00b2accf49b250f7b305a0dd14ba29a36703dea44c2d40c63d8fc56cf1c1` |

The protected evidence root is
`scratch/qualification-implementation-20260907/iterative/cadence-017-preparation-3`.
Its seal covers 124 files and 16 directories. The final journal has 112 rows.
Read-only verification recomputes inventory membership, hashes, permissions,
runtime/artifact bindings, predecessor joins and the acceptance criteria.

Independent companion receipt
`e15c0baf90740cf0f2a10d8bab057808dbdeba147efa66f07285650de08b731a`
also binds fourteen external cleanup, native-audit, operator-helper and
supplementary-observer witnesses. It confirms the sealed and current journals
match and all closure obligations pass without changing an earlier outcome.

## Continuity and measured phases

One detector-admitted Ultra 205 completed the initial state-preserving
installation and four fresh update/reconnect cycles. All five installations
reported healthy trusted startup on this firmware. Every cycle preserved device
identity, settings and authorization state, released serial ownership around the
write, and completed a 65,536-byte request and response through fresh possession.
No factory reset, credential provisioning or NVS reset occurred.

One admitted passive `/api/ws/live` subscriber remained connected through each
phase. Timing below is device-local; intervals crossing capture boundaries are
included. Host observer arrivals are separate observations, not browser rendering.

| Phase | Capture seconds | Complete intervals | At most 750 ms | Maximum interval ms | Maximum processing ms |
| --- | --- | --- | --- | --- | --- |
| Idle | 60.201238 | 101 | 101/101 | 649.701 | 140.379 |
| USB load | 60.275196 | 99 | 99/99 | 686.970 | 176.543 |
| Mining | 60.413063 | 98 | 98/98 | 730.712 | 223.338 |

All 298 intervals met the 750-ms bound; none exceeded the 1,500-ms maximum.
All observed iterations ran on CPU0 at priority 5. Publication/projection,
serialization, queue, asynchronous send, pending-send, subscriber, dropped
observation, overflow and clock-integrity failures were zero. All captures
completed with at least 60 intervals and an intact boundary interval.

USB load included all twelve serialized maximum-size probes, five seconds apart.
Mining capture began from the first successful dispatch of the next admitted
generation, with continued running state and increasing dispatch counts in every
ten-second segment. Live samples reached 33 dispatches; the terminal retained
record contains 34 dispatches and three renewals. The device reports a 71,166-ms
active interval and five accepted shares, but accepted shares are not a
requirement of this cadence judge. The separate normal
acceptance judge was not changed or substituted.

## Shutdown, accounting and cleanup

After the measured mining interval and boundary tail, the qualified harness
suppressed application heartbeats. Device-local generation revocation occurred
2,802 ms after the last valid heartbeat; shutdown began at 2,831 ms. Both satisfy
the unchanged three-second limit. Ordered shutdown, bounded cooling, fresh
authenticated recovery and restored inactive state passed.

The passive observer remained through the fault plus its required tail, then
closed on request with exit 0 and complete cleanup after 218,989 host milliseconds,
within its 360-second lifetime. Fresh possession exported the frozen summaries
after the recovery wait. Mine-on-boot remained false; identity/settings matched,
leases were inactive, the final journal was flushed, and the browser released
serial ownership and closed.

Start and renewals legitimately advanced authorization high-water state. Its
post-work recovery checkpoint matched; the original pre-work authorization
baseline did not match and is not claimed unchanged. Active observations stayed
within the declared safety profile: 5.3325–5.3525 V input, at most 8.85 W and
44 C, with at least 9,424 bytes of observed production-owner stack headroom.
The terminal restored sample reported 30 C and 3,200 RPM.

| Accounting | Before | After |
| --- | --- | --- |
| Next ordinal | 17 | 18 |
| Last completed ordinal | 16 | 17 |
| Total charged milliseconds | 1,380,000 | 1,560,000 |
| Pending reservation | false | false |

The full normal 180,000-ms reservation was charged without refund. The original
campaign remains exhausted at 240,000 ms with masks 7/7. No ledger was reset.

The exact supervisor exited 0 after identity-bound shutdown. Independent fresh
host checks found no listener, owned process/group or serial holders. The
supplementary process observer's incomplete record and parent-observed nonzero
exit are retained. Its configured lifetime was 600 seconds; its exact exit time
and termination cause were not independently observed. It supplies no final cleanup proof.
Actual cleanup is supported by the separate browser/exit observations and kernel
process/socket/serial checks.

## Correction and verification scope

The [preceding failure](20260915-usb-cadence-retention.md) remains unverified and
sealed. Its USB phase reached only 89/95 intervals within 750 ms, with maximum
retention time 163.131 ms. After moving typed boot-diagnostic replay out of the
retained-log mutex, USB's maximum retention time was 10.838 ms; mining's was
21.303 ms. The old reader fails and the corrected reader passes the real held-log-
mutex regression. This demonstrates removal of that blocking path; the earlier
hardware delay was not separately measured as mutex-wait time.

General logs, atomic snapshot-pair retention/issuance, the 500-ms sleep, affinity,
priority, stacks, safety deadlines and acceptance thresholds remain unchanged.
Ordered Cargo checks passed 2,239 tests with one existing ignored test. The
corrected complete canonical graph passed all 138 targets with four concurrent
tests and unchanged per-test limits. Gate's 605 tests, standards, ownership,
reference, redaction and parity checks passed. Earlier verification failures are
retained and explained in the correction report.

Clean native audit
`6c33e204cde6a74ce03f8bb029e27bbb9325abdfe332a4027d9b164e99bfbfce`
binds this ELF and measures 1,436 diagnostic static bytes, including the 116-byte
cache, below the 2,048-byte limit. The app is 4,118,080 bytes with 76,224 bytes of
partition headroom. Selected main/statistics/USB/control stack paths and the
production entry fit their unchanged budgets. Independent byte-level audit
`3a2ab1b48f7d209edc8185b88a13eeba95f433326d162e295d21549a3f259f96`
confirms one inline hardware CAS per typed recording section, no calls or backward
branches there, and the explicit 8,192-byte prepared-thread setting. These audits
exclude unknown library/ROM/allocator/interrupt depth; surrounding formatting and
logging still allocate.

## Broader milestone composition and limitations

- [September 8 live acceptance](20260908-worker-preparation-live-acceptance.md)
  retains its own firmware/Gate scope for normal acceptance, shares, renewals,
  foreground loss, heartbeat suppression, shutdown and durable accounting.
- [September 11 Hello recovery](20260911-hello-recovery-live.md) retains its own
  pair for no-drain authenticated recovery and subsequent mining resumption.
- [September 13 reconciliation](20260913-usb-qualification-reconciliation.md)
  resolves the earlier migration obligations and identifies measured CPU0 cadence
  as the remaining gap now closed by this result.
- Accepted Stage B004 remains a controlled-restart result for F056/Gate722. It is
  ancestry, not a controlled-restart claim for this cache image.

Every earlier failed preparation, exhausted campaign and charged predecessor
remains unchanged. This is bounded Ultra 205 evidence with one subscriber and
the declared load; arbitrary scheduling/load, thermal soak, other boards,
hardware stale-control-reply injection and exhaustive physical read-batch
fragmentation/coalescence remain outside the claim. Deterministic transport
coverage remains distinct from observed hardware behavior. BWG restoration and
STR005 require their own current successor contracts and independent evidence.
Parity remains **90/95 active rows verified**.
