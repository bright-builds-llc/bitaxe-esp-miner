# STR-005 Channel005: bootstrap drain timeout

Channel005 is **unverified**. Canonical finalization and independent read-only
review agree on `stop_evidence_incomplete`, with first recorded supervisor
failure `noise_install_unqualified`. No channel exchange or mining occurred.
Parity remains **90/95**.

## Exact tested pair and evidence

| Binding | Value |
| --- | --- |
| Firmware | `14f6d6d98ad939db065542db66dd8aad77f097d7` |
| Gate | `e20c0fd52d2216596f904992ffa54fda33be9025` |
| Application ELF SHA-256 | `cbe496ec96b5f20cfaeabf10f6e41105de916310e4dc34fc3a77f07d1091e8cf` |
| Application bytes | 4161024 |
| Context SHA-256 | `dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a` |
| Result SHA-256 | `1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e` |
| Seal SHA-256 | `3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342` |

The published durable-operator implementation is `421aab78`; its metadata-only
closure is `14f6d6d9`. Ordered Cargo checks passed 2346 tests with three existing
ignored tests. All 208 canonical targets, 38 focused Gate tests, native/package,
USB ownership, reference, redaction, standards and parity checks passed before
hardware. Real-process regressions cover client loss, parent loss, failed
result writes, atomic result publication and truthful cleanup.

## Observed boundary

Five state-preserving writes completed: initial installation 0 and updates 1–4.
Installations 0–3 passed startup review. Three fresh continuity cycles passed
65536-byte bidirectional exchanges with preserved identity/settings. Update 4
completed its write and ran the exact image with a stable boot, complete startup
and safe baseline, but its capture contained a current-boot transport diagnostic:

`usb_tx_failure schema=v1 stage=flush_timeout elapsed_ms=2000 queued_bytes=92 record_bytes=92 redacted=true`

The five appearances replay the retained first failure. They do not establish
five distinct timeouts or the total underlying failure count. The observation
means all 92 bytes were queued, but native drain completion was not confirmed
within the existing two-second deadline. It does not prove that no bytes reached
the host, a firmware crash, or a deadlock. The strict assessor correctly rejected
that capture as `error_diagnostic`; the fourth cycle was not executed or credited.

The first bootstrap progress record is also 92 bytes and is a plausible source,
but record identity was not captured in the failure. Host capture begins after
reset and transport-admission checks. Existing evidence lacks the host
reset-to-reader timing, first-failure device uptime/category and native drain
state needed to distinguish reader availability from completion signaling.
Neither explanation is established as the root cause.

## Preservation and cleanup

Fresh pre-write authenticated accounting was next 18, last completed 17,
1560000 ms charged, pending false. The exhausted original campaign remained
240000 ms with reserved/completed masks 7. No fixture process, signed mining
grant, allowance consumption, channel execution or ASIC work followed.

After the failed capture, fresh same-pair possession confirmed the baseline,
inactive leases, matching identity/settings/authorization high-water and
mine-on-boot false. The serial session was closed and flushed; the dedicated
browser window was closed through native UI. The operator observed the real
supervisor exit 0, stopped, and independent resource checks established absence.
A full successful restoration/accounting/cleanup receipt is **not** claimed:
post-failure ledger review was not exported, and normal cleanup preparation
requires a fixture that was never started. That secondary cleanup limitation
does not replace the earlier installation failure.

The first installation request was locally rejected before IPC because its
wrapper directory permissions were too broad. The directory was restricted,
its file policy verified, and the same request ID was admitted once. No duplicate
installation occurred. All failed and successful artifacts remain retained;
no sealed prior campaign was changed or upgraded.

## Required next proof

`task-usb-bootstrap-drain-observability` records the next bounded work: host
monotonic reset/admission/open/first-read timings and device first-failure
category, uptime and flush outcomes. Keep clock domains separate, preserve the
two-second deadline and reject current transmit failures. Optional read-only
FIFO/interrupt observations may narrow the cause; private IDF structure offsets,
a driver fork, concurrent serial owners and an unqualified reset substitution
are excluded. A future hardware attempt needs its own published contract,
verified correction/measurement implementation and fresh guarded admission.
Channel005 and the earlier failed campaigns remain immutable. Share002 has not
been allocated; STR-005 promotion remains blocked.
