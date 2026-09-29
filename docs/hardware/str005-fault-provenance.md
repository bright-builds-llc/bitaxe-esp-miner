# Fixed-size Share001 fault provenance

This diagnostic update observes failure boundaries without changing mining,
lease, heartbeat, stack-size, allocation-capability or power policy. It preserves
all old dumps and results. Device effects require the separately published
[diagnostic owner contract](../../scripts/str005-share-diagnostic/CONTRACT.md).

## Native records

The existing panic wrapper first cuts ASIC output latches and revokes generation,
then completes its unchanged cutoff receipt. Only afterward does the new typed
C shim examine diagnostic pointers and delegate to the original SDK handler.
The component is compiled through pinned ESP-IDF5.5.4/esp-idf-sys and uses SDK
headers with compile-time size/offset assertions; no guessed TCB layout is used.

A192-byte core-dump record retains image/boot/core/cycle identity, panic reason
metadata and pointer values, the original exception-frame pointer, and guarded
PC/PS/A0/A1/exception fields. Alignment and the entire internal-DRAM span must be
valid before loads; external frame memory is never dereferenced. Invalid spans
are recorded as unavailable. String pointers are never dereferenced by the hook.

Two SDK cross-object hooks record the actual task header before and after
`esp_core_dump_check_task`, then commit it only when the SDK identifies the same
crashed TCB. Record the four stack-sanity predicates, actual SDK result and fake
replacement separately: check_task=true is not proof of a real stack. Repeated
consistent passes are supported; mismatches, nesting and incomplete commits
remain explicit failures. SDK external-stack predicate configuration is preserved.

A separate1536-byte history holds each core's first allocation failure plus its
latest eight failures. It retains sequence, raw TCB, core, CCOUNT, size/capabilities,
allocator-name pointer, image/boot/stage and task-bound command breadcrumbs.
The SDK callback is recorded before the legacy first-failure RTC receipt; that
receipt's layout/checksum/projection remain compatible. Native registration failure
fails initialization explicitly, and Rust verifies the legacy RTC type sizes.

The recorder uses bounded local interrupt masking and the pinned SDK inline
S32C1I compare-and-set primitive, with a compile-time ISA guard. It does not use
generic compiler atomics that can lower to SDK critical-section locks. There is
no allocation, logging, string read, stack walk, cross-core lock or retry loop.
Overflow, overwritten history, incomplete rows, early unbound identity, ISR origin
and unavailable owner slots are represented explicitly. CCOUNT is per-core and
wraps; it is not a globally ordered timestamp.

## Command breadcrumbs and interpretation

A scoped registration identifies the BWG owner only while that task lives.
Each frame has a sequence and a closed numeric phase: idle, parse, dispatch,
Start preparation, Renew preparation, V2 snapshot, V2 value construction, reply
serialization, queued reply or cleanup. RAII clears command/owner state on exits.
No request contents, grants, endpoints or names enter these records.

The recorded phase-site PC identifies the last instrumentation boundary. It is
**not** the exact allocating instruction. The SDK allocator-name pointer identifies
an allocator API when the exact ELF can resolve it. Task identity, phase, timing
and the independently captured panic frame must be correlated; an allocation
failure can be recoverable and is never labeled the panic cause by itself.

## Independent verification

`just audit-fault-provenance --elf ELF --output PRIVATE_JSON` checks actual native
ABI constants, selected core-dump regions, internal state placement, hot call
closure, literal/data access and SDK wrapper routes. Unresolved/forbidden calls
reject. Panic additions have a128-byte stack limit; allocation additions256 bytes.
The immutable SDK ROM interrupt helper is a narrowly verified exception: SDK
selector, artifact hash, application veneer/literal, exact leaf instructions and
its16-byte stack are checked. Other ROM addresses are not implicitly admitted.

The development image passed all five native audits (cutoff, core store, signed
Start, signed Renew and fault provenance). Added panic stack measured96 bytes and
allocation192 bytes. Development builds never authorize installation; repeat the
checks against the exact clean published package before an effect.

Host tests execute production C record logic and the pure pointer/predicate model.
Controller tests verify failed snapshots retain the correct boundary without
changing response/effect behavior. Decoder tests cover torn/checksum/identity/
source/boot/overlap/truncation errors, original versus synthetic frames, chronology
limits and preservation of independent raw evidence when later validation fails.

`just core-dump analyze` now uses an official-loader-verified deterministic
`core.elf` and bounded pinned GDB batch commands. It avoids the former high-level
metadata-report wait and suppresses frame arguments. Full output remains private.
`just core-dump verify-provenance` requires the new captured records, reads only
actual core PT_LOAD bytes, and joins them to the exact program ABI and SDK notes.
An original frame must be meaningful; a synthetic SDK frame alone cannot qualify.
A meaningful separately captured original frame may explain an SDK replacement.

Private raw records are retained even when subsequent integrity or qualification
checks fail. Public summaries contain only reviewed flags/counts/digests; ordinary
historical inspect/analyze still work on images without the new records. A valid
record alone does not establish a root cause. Raw dumps/debugger output remain
under ADR-0030 private evidence controls.

## Hardware sequence

Keep the original b665 crash archive immutable. Under explicit stage gates:
recover, verify/archive-bound clear of the old on-device copy, state-preserving
install, fresh candidate recovery, and actual empty-core verification. Then run
one ASIC-off self-test and require original-frame provenance plus cutoff/recovery.
Archive that self-test dump and perform a second separately admitted clear before
any bounded Start/status reproduction. Never infer emptiness from a flash.

The initial owner has no signer, mining fixture, Start or renewal route. A later
reproduction needs its own published bounded contract, fresh accounting and zero
renewals. Stop on missing evidence or a repeated corrected boundary; parity stays
90/95 until independent qualification and promotion criteria pass.
