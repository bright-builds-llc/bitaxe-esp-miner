# USB bootstrap reader-gap measurement

Outcome: **measurement complete; ordinary capture unqualified**. No hardware
qualification, continuity-cycle credit, mining authority or parity promotion is
claimed. Parity remains **90/95** active rows verified.

## Tested pair and evidence

One Ultra 205 received one state-preserving installation under
`task-usb-bootstrap-drain-observability`. The prospective
[measurement contract](../../hardware/usb-bootstrap-drain-measurement.md) and
[interrupted-preflight successor amendment](../../hardware/usb-bootstrap-preflight-successor.md)
were published before the effect.

- Firmware: `3951a441606115798ea47eba0de1052d6590b3a0`.
- ELF SHA256: `0b9d275b5c75c58b3e25f8221d0be7dbb909313c4d9083aba2faf480af7b283f`.
- Gate: `e20c0fd52d2216596f904992ffa54fda33be9025`.
- Private evidence root: `scratch/usb-bootstrap-measure/attempt-002`.
- Context SHA256: `80cf7437feb6df2efa6130a7c8f463e4b679506b96e911d439c57d7c0c864ee0`.
- Independently reviewed result SHA256: `6b3329af0d9188b7618377b18317646b4d24a2b13bcc13725fde2059f932470d`.
- Seal SHA256: `a8ad2ef03819f3f413edb85c2fb85b8623c556fe0077ff01e7574f0265cb6386`.
- Host timing SHA256: `74b6c6b11db502cc7b8198c1cc5bfb7413c0aa2aa38d573254945f2233627915`.
- Device observation SHA256: `6a6cfcd3c04cec759367a56846b18f5677ace0e415829fdabd9e2cc82d818eae`.

Both the finalizer and separate read-only review returned `measurement_complete`.
The ordinary flash/capture command exited 1 with `error_diagnostic`; that verdict
and the first parent-observed `bootstrap_capture_unqualified` remain intact.
The image write completed, exact executing identity and stable boot were observed,
and startup and safe-baseline checks passed. No second write or reset was started.

## Host observations

These are differences within one host monotonic clock. Reset-call bounds include
supervised command launch, execution and reap; they are not physical reset edges.

| Observation                           |      Duration |
| ------------------------------------- | ------------: |
| Qualified reset command call          |    509.335 ms |
| Handoff admission                     |  1,848.076 ms |
| Separate monitor admission            |  2,509.354 ms |
| Reset-command return to reader opened |  4,423.674 ms |
| Descriptor opening itself             |      1.418 ms |
| Reader opened to first nonempty read  |      0.208 ms |
| Actual reader ownership interval      | 27,515.214 ms |

The configured 30-second observation deadline includes monitor admission. It must
not be described as 30 seconds with an open reader. One descriptor was opened,
with no reopen, clock or overflow failure. Reader closure and host cleanup were
confirmed. Host first-read availability does not establish when USB delivery
occurred or whether opening the application reader caused it.

## Device observations

These are device uptime measurements, not host timestamps. The same capture binds
them to the exact tested identity and one stable boot-discriminator stream.

| Observation                            | Value                                       |
| -------------------------------------- | ------------------------------------------- |
| Explicit producer category / kind      | `bootstrap_diagnostic` / `startup_progress` |
| First attempt                          | Failed at `flush_timeout`                   |
| Start / end uptime                     | 91 / 2,092 ms                               |
| Observed elapsed time                  | 2,001 ms                                    |
| Record bytes / queued bytes            | 92 / 92                                     |
| Queue calls                            | One positive call; zero capacity failures   |
| Drain calls                            | 200, all `ESP_ERR_TIMEOUT` (`0x107`)        |
| Maximum individual drain call          | 16 ms                                       |
| Successful drain calls in that attempt | Zero                                        |
| Measurement integrity flags            | Zero                                        |
| Actual write failures retained         | One                                         |
| Measurement replays completed          | Three                                       |

The record identity comes from its explicit producer tag, not an inference from
its length. Repeated retained failure messages are not counted as separate failed
writes. The new first-bootstrap and first-failure observations survived later
successful transmissions and fresh possession.

## Interpretation and next correction

The first record entered the native queue successfully, then exhausted the full
record budget through repeated short timeout returns. This observation is neither
a queue-capacity failure nor the separately fixed premature sub-tick termination.
It does not show one native call blocking for two seconds. ESP-IDF uses the same
timeout result for transmit-mutex and idle-completion waits; these measurements
do not distinguish those internal conditions.

The host spent more than four seconds after reset-command return in admission
work before opening its receive descriptor. That is the leading actionable delay.
Removing only the second admission pass would still leave approximately 1,914 ms
after reset return, without accounting for reset release inside the command. It
would not establish reliable headroom against the unchanged device deadline.

The next bounded correction should start one actively draining receive owner
under the retained physical lease after a fresh valid candidate is observed,
while keeping the existing stability, profile and foreign-owner checks. Bytes
must stay quarantined until admission succeeds. Preserve the reset backend,
single descriptor, strict classifier and device deadlines. This is a proposed
correction, not a demonstrated hardware fix; a fresh prospective contract and
changed published pair are required before another installation.

Do not subtract host timestamps from device uptime, claim exact peer receipt,
claim a specific private IDF wait was responsible, or credit this measurement as
successful ordinary qualification. No driver fork or timeout relaxation is
supported by this evidence.

## Preservation and cleanup

Fresh native Web Serial possession before and after installation verified Device
Identity, settings and authorization-high-water preservation. Mine-on-boot is
false and leases are inactive. Explicit Stop/Restore completed on the tested
candidate, serial was closed and flushed, and the dedicated native browser window
was observed closed. The candidate remains installed; no rollback or factory
reset was performed.

Both fresh accounting reviews report next ordinal 18, last completed 17,
1,560,000 ms charged and no pending reservation. The original campaign remains
exhausted at 240,000 ms with reserved/completed masks 7. No allowance was issued,
loaded, consumed or refunded; no mining, fixture, work, share or heartbeat-fault
campaign ran.

The actual supervisor exited 0 six milliseconds after the parent's stop request.
The daemon stopped, and finalization independently checked absence of owned
processes/groups, serial holders, the supervisor listener and operator socket.
The restoration and cleanup receipts both verified complete.

## Earlier preparations

The native-name validation failure at `def310ec` occurred before assignment.
Its address-based correction retained all native frames and unchanged stack limits.
The following `29cf472d` preflight reserved attempt 001 but hit the unchanged
private-evidence filename guard while snapshotting a published synthetic-fixture
source. Its 1,773 files and assignment remain unchanged and unverified.

The protected sibling closure has SHA256
`e07d772b66fc9c0f31aa978a33517163c966d4cfd54cc34af319be255d6158f7`.
It records `interrupted_before_effects`, not hardware success or fresh device
accounting. The successor used encoded source filenames and validated staging
before assignment. No historical campaign was reopened or reclassified.
