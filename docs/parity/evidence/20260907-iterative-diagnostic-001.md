# Iterative diagnostic 001: real work and owner-stack pressure

Exact installed firmware: `d55248722edd3eb8b295cbbd32d6ff018061544a`.
ELF SHA-256: `602783c432c2b9e342b20ce1a324616fa51df3ce1fcd4abbb39ad09252507e2c`.
Gate: `53bb4fd354e04889b6b0251aa55e316e3a0ac172`.

## Verified hardware observations

Initial installation and four exact-image no-mining update/reconnect cycles
passed. Every cycle retained Device Identity, settings and authorization marks,
exchanged actual 65536-byte request/response payloads, confirmed mining disabled
and released browser/CLI ownership. All thirteen exact runtime artifacts and
the cycle receipts are preserved in protected storage.

Fresh fan-only proof and both ledger reviews preceded the first separately
signed diagnostic allowance. Start completed without a panic, dispatched one
real ASIC work item and performed acknowledged ordered shutdown. Active time
was 7359 ms within the 30000-ms allowance. Work admission closed 408 ms and
shutdown began 446 ms after the last advancing heartbeat. Terminal cooling was
qualified at 33 C with fresh nonzero RPM, restored fan duty and mine-on-boot
false. No nonce correlation, submission or accepted share was observed; those
are not success criteria for this short diagnostic.

The repository judge accepted the diagnostic. The independent ledger reports
30000 ms charged, ordinal 1 complete, next ordinal 2 and no pending reservation.
The original ledger remains unchanged at 240000 ms, masks 7 and no pending
reservation. Device Identity and settings match; authorization marks advanced
after signed Start and were not reset. Browser page/streams/locks, both serial
nodes and the exact supervisor were released.

## Stack evidence and interpretation

The current-boot preparation receipt is valid and reports all nine steps
completed, generation 2, sequence 36, no typed failure, 23999 internal heap
bytes free, a 10752-byte largest block and only **28 bytes** of minimum untouched
owner stack. This is a lifetime high-water reading from the actual production
owner task. Pinned ESP-IDF documentation and its Xtensa StackType_t confirm
that the unit is bytes. The configured stack was 16384 bytes.

The exact ELF's owner trampoline reserves a 6880-byte native frame, with
additional substantial nested preparation frames. This supports stack pressure
as the leading hypothesis, but is not a complete maximum-depth analysis and
does not establish the cause of the earlier panic. The successful diagnostic
alone also does not establish long-run stability. Heap exhaustion and a native
assertion remain unproven alternatives.

The receipt's watermark was captured at the end of preparation and does not
measure later work/shutdown peaks. Before a longer run, increase the stack to
24576 bytes, audit the native owner-entry frame and require fresh matching
owner-thread resource observations with at least 4096 bytes of headroom through
active work and completed shutdown. Measure heap availability again; the
predicted reduction from the additional stack allocation is not hardware proof.

The previous-boot receipt reported corrupt data with no usable payload; no
physical reset-retention success is claimed. No raw memory dump was collected.
The first diagnostic result and exported closed observations remain immutable;
a changed runtime requires a fresh diagnostic and four new cycles.
