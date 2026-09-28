# Exact-image capture and cutoff verified

Capture003 passed on firmware `f000872f2e436aa7cdaa8cbfa41eee965a27731e`,
full ELF `a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2`,
Gate `9643e87664397a321c715a3a1b1bb6c1183b83ea`, host `53fdd67a`.
The readiness coordinator waited for the actual current-boot core-store receipt
before issuing its sole ASIC-off self-test.

The nonce acknowledgement matched. The explicit panic transition was boot 12→13,
with exact image and healthy return in 8,571 ms. Immediate store diagnostics
reported 77,820 bytes requested, 77,856 prepared, capacity 974,848, and zero results
for initialization, preparation, start, end and store. Panic-boot recovery passed
before ROM acquisition; subsequent recovery passed on boot 14. Both ledgers stayed
next20 / last19 / 1,920,000 ms / pending=false, with no new work reservation.

The actual 974,848-byte region was acquired through the official same-device
owner; application return and cleanup passed. Independent `verify-cutoff` checked
the dump checksum, full matching ELF, native cutoff and actual captured memory.
ASIC outputs were disabled, generation was revoked, and the self-test marker was
present. The dump SHA-256 is
`bc8a8079d97fa2395f63bda78286be6d7c83aaeb39774199e643b8226e11288b`.

Private root: `scratch/str005-renew/capture003/attempt`.
Sealed inventory SHA-256:
`2233a555300f84f5d2802e8caf63ecb88e9e89ec3af9ad6d6353004fb3fd2333`.
The existing capture consumer independently rejudged the complete sealed producer
chain and actual bytes successfully. Browser, server, listener and serial ownership
were released.

The generic observer's `core_capture_verified=false` is preserved: that observer
does not inspect a raw dump. The independent decoder and captured-memory verifier
establish capture. `cause_proven=false` remains unchanged; this deliberate self-test
does not prove Share002's original panic cause. Capture001/002 remain immutable
pre-effect failures. No grant, Start, mining, renewal, clear or parity promotion
occurred. Parity remains 90/95.

Verification process note: commit53fdd67a was created before final Rust-suite
completion had been confirmed. Completion was then confirmed before device access.
Subsequent commits must follow confirmed completion of every required check.
