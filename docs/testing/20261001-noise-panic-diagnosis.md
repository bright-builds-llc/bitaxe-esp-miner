# Virtual Noise crash diagnosis

This continuation belongs to `task-ultra205-virtual-board-validation`.
Physical hardware gates remain disabled, accepted-share remains unresolved, and
parity stays **90/95**. Raw cores, registers and debugger output remain private.
No original seal or diagnostic record is rewritten.

## Retained evidence

Two independent offline inspections verify the run017 dump checksum and full
virtual ELF identity:

- Dump SHA-256: `0f7c3e60e841c6d4f1277135069e1adfb641e7fc3a296f6b2a6c953ed7d32e34`.
- ELF SHA-256: `579fdd34473fe17245f36d7e026fbd68e1041b0a7408a1cb3f958d4b030eb98e`.
- Configured main stack:16,384 bytes; pinned SDK overhead:512 bytes.
- TCB bound distance:16,888 bytes, or16,889 inclusive. The saved task top and
  fault stack pointer are outside those bounds. The fault pointer lies10,168
  bytes below the stack base.
- Deduplicated observed native frames total25,824 bytes. Inlined frames sharing
  a CFA are not added twice. This is observed coverage, not a global upper bound.
- The virtual fixture caller reserves8,464 bytes and its exchange constructor
  reserves7,136 bytes. Their15,600 bytes overlap the6,080-byte Noise completion
  chain and outer controller frames.

The direct fault is a heap-registry node read in SDK `find_containing_heap`.
That node is unaligned and outside configured memory. The vector passed to free
is consistent across SDK and Rust frames:96 bytes, alignment1, word-aligned and
within configured PSRAM. This does not prove registered-heap membership: the
PSRAM payload/allocation header and heap descriptor were not captured. Vector
length/capacity locals were optimized out; capacity96 is inferred from the
verified deallocation layout, not a directly inspected local.

The virtual call site violates its stack contract. The earliest corrupting write
is not observed; neither double-free nor a Noise algorithm defect is established.
The physical status001 failure cause remains unproved.

## Minimal feedback loop

Use the real production Noise initiator, responder and transport with the same
synthetic seed/trust/time and packet shape. Remove controller, board, grants,
ASIC/work and fixture lifecycle context. Complete one authenticated handshake and
one32-byte opaque encrypted frame round trip. Typed phase callbacks preserve
stack observations before potentially failing heap inspection and serialize
results only after resource release.

Local regressions require valid authentication, exact payload round trip, phase
ordering and release after success or ordinary rejection. Wrong authority,
modified/truncated act-two and modified/truncated ciphertext must be rejected by
the real protocol path. No negative fixture may manufacture an ACK or control
success.

Before any target execution, bind the exact marked package/ELF/configuration,
verify the pinned emulator and SDK routing, audit caller frames against the
unchanged16KiB stack and existing2KiB margin, and publish the bounded execution
contract. The initial target gate remains disabled until these checks pass.

A green minimized result identifies context dependence; it does not repair or
qualify the retained full lifecycle. Further experiments must state falsifiable
predictions and change one variable at a time. An evidence-backed correction
must preserve cryptographic decisions, allocation classes and stack/safety limits,
then pass the native audit and the original composed boundary through a separate
published gate.

## Current conclusion

Offline diagnosis identifies a concrete caller-stack deficiency. The isolated
probe and regression audit are being implemented; no new target result or
qualification is claimed by this report.
