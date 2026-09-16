# Serial Noise v2 implementation and readiness

This report follows the frozen
[base contract](str005-noise-serial-qualification.md) and
[v2 amendment](str005-noise-parity-scope-amendment.md). The earlier
[runtime audit](str005-noise-runtime-readiness.md) and
[fixture preparation report](str005-noise-fixture-readiness.md) retain their
original findings. The dependency remains pinned; no cryptographic fork or
universal in-call cancellation guarantee is introduced.

## Runtime and control ownership

The ordinary fixed Serial/JTAG controller admits one volatile diagnostic job
through authenticated possession. Admission consumes the boot's slot before the
response, and execution waits for the existing completed-response barrier.
Delivered idle observations bind the fresh network input to boot, generation and
transport epoch. Repeated delivery cannot create a second job.

The diagnostic borrows the existing idle primary pool transport worker. Both
primary and fallback lanes must have no connection, queued command, in-flight
command or existing loan. Their short metadata locks exclude ordinary commands
while reserved. This adds no thread or stack. Crypto stays off control, receive,
main telemetry and production safety owners. The replaced boot-time NVS Noise
activation is no longer linked; its historical sources and readers remain.

An independent generation fence enforces heartbeat and absolute authority
expiry. It does not reserve mining work, mutate a ledger or request a fictitious
ASIC shutdown. Cancellation is checked between supported crypto operations and
before protocol I/O. A current opaque crypto call may return after revocation;
that does not authorize the next operation. An invalid monotonic clock revokes
authority and invalidates timing evidence.

The work item's socket, crypto state and inputs drop before its scoped function
returns. The existing worker then records actual job completion. The persistent
pool thread does not claim to exit. Effect fences remain held through actual
completion and resource release. A missed observation horizon remains an
immutable incomplete outcome even if cleanup is observed later.

Idle qualification status exposes existing boot counters and live health with
generation zero when no mining generation has been admitted. It creates no
lease, work reservation, shutdown timestamp or synthetic zero counters.

## Host evidence and independent fixture

The v2 supervisor admits clean published identities, exact contract and
evaluator inputs, the canonical native package and fixture build provenance.
Preparation task completion and the accepted charged predecessor are checked
before an exclusive attempt assignment. The original before-update baseline
stays on the same Gate page through installation and four fresh continuity
cycles.

The installation producer prearms a process observer, derives a fixed command
from its bound claim and requires fresh detector evidence. It cannot accept
arbitrary flash arguments, factory reset or credential flags. Each maximum-size
probe has a fresh claim and before/after authenticated observations; retained
old probe output cannot satisfy a new cycle.

The local fixture independently receives the diagnostic's own 64-byte act one,
authenticates with fresh test authority material and decrypts the exact 22-byte
encrypted proof. It records bounded candidate inventory, socket correlation and
peer EOF. Its private keys and arbitrary logs are not evidence. Host readiness,
lifetime and cleanup remain independently bounded.

The judge joins device stages, fixture receipts, preserved settings and
identity, unchanged accounting and boot-relative work/share counters. Normal
restoration requires fresh same-pair possession and actual device resource
release. Host cleanup joins parent-observed closure with current kernel absence.
Finalization seals exact retained inputs; review reads them without acquiring
authority. Dedicated `recover` remains explicitly unavailable and cannot
implicitly restart, flash or reset anything. No signing or mining route exists
in this supervisor.

## Native resource findings

The first preview app occupied 4,182,400 bytes. After correctness fixes, the
second occupied 4,196,944 bytes and failed the unchanged 4,194,304-byte slot.
Accepted September 15 startup evidence also left only 7,147 internal/DMA/8-bit
heap bytes after required owners started. Adding a 24-KiB internal stack was
therefore not an admissible assumption.

The third preview packages after replacing that new owner with the existing
12-KiB worker and unlinking the obsolete boot activation. Its selected native
authentication path still requires 15,104 bytes, so it does **not** pass stack
readiness. The fourth reduces that path to 12,160 bytes, but the measured Rust
and pthread entry frames add another 208 bytes. That still exceeds the unchanged
12,288-byte stack. Fallible RNG and act-two storage reduce the fifth complete
selected path to 11,888 bytes. Outlining input parsing and disposal reduces the
sixth to 11,648, leaving 640 bytes on the unchanged stack. That preview app is
4,167,728 bytes. The production owner entry is 4,496 bytes on its 24-KiB stack,
and the previously qualified selected telemetry path remains 12,560 of 16,384
bytes. These are dirty-source software previews, never hardware admission.

Before hardware, the software admission guard additionally requires at least 512
bytes remaining after the selected path and measured thread entry frames. This
is a prospective conservative guard, not an upstream guarantee or a complete
bound on omitted paths and platform overhead.

The read-only auditor binds ELF, app, SDK configuration, source and tool hashes.
Its selected-path walk resolves literal call targets and local compiler spills.
GNU linear disassembly can miss a branch start after alignment padding; the
auditor asks the same bound decoder to decode that exact address. Unknown
indirect jumps are retained as limitations. Measured selected frames are not a
complete callgraph, a startup-heap measurement or hardware qualification.

## Verification and handoff

Gate is published at `5dc5ed6f39834cab0144af0a05f6c2535a39d827`; its clean
acceptance bundle digest is
`72f061a8b0b1964caeacf288eafb7cbd061ae001f2e28d42a885d71173f76a03`. Verification
includes 713 browser tests, 352 Rust tests and the actual headless page
composition. One existing SSE timing test failed on the first full run; its
focused reproduction and the full retry passed unchanged.

Ordered firmware-workspace Cargo format, Clippy, build and tests pass: 2,285
tests passed, none failed, three existing tests ignored. The host suite passes
82 tests, including the real process/client/server/finalizer composition with
explicitly synthetic device observations. Real loopback fixture tests exercise
the shared production cryptography separately. Native helper tests cover
alignment gaps, alternative call targets, conditional stores, stack-alias
escape, false panic cutoffs and the margin boundary. These tests are not
hardware-negative evidence.

All 159 canonical Bazel test targets pass. Canonical fixture provenance and CLI
builds pass; the recovery rejection was exercised through the real CLI. Native
packaging, USB ownership, reference, semantic redaction and Bright Builds checks
pass. Cross-package runfiles ownership, restricted child-launcher environment,
observer arming and atomic fixture publication were corrected before admission.

Both software preparation tasks can now close with this report. The published
implementation must still produce a clean exact package and pass its native
audit again before a fresh attempt. Startup heap/health, four continuity cycles,
the positive exchange, unchanged accounting, restoration and actual host cleanup
remain live obligations. No hardware attempt, installation, Noise exchange or
mining reservation was made during preparation. Parity remains 90/95; there is
no checklist or progress-history transition.
