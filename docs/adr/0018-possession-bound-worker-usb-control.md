# ADR-0018: Bind application Worker control to Device Identity possession

## Status

Accepted

## Context

The browser-facing BWG Controller needs a local physical continuity proof that
cannot be satisfied by cloneable USB identifiers. Existing ROM USB
Serial/JTAG is a flashing and debug surface, ordinary settings NVS contains
operator configuration, and the Production Mining Session is the sole owner of
mining and safe-stop effects. Reusing any of those surfaces as implicit
authority would cross established ownership and privacy boundaries.

Controller 0.3, Worker USB 0.2, and `bwg-worker-possession/0.1` publish the
required strict frames and exact composite topology. Work Lease signing and
key provisioning remain deployment-owned; the published deployment trust
profile supplies the strict full-input verifier contract.

## Decision

The application exposes one TinyUSB composite device with two disjoint
functions: a vendor-specific bidirectional Worker-control function and a
receive-only CDC evidence function. ROM USB Serial/JTAG remains
bootloader/debug-only. Control accepts only bounded possession and Controller
0.3 frames; CDC input never becomes a command.

A dedicated `bwg_worker` NVS namespace stores exactly one private 32-byte
Ed25519 Device Identity seed. A thin identity adapter atomically loads the
blob, generates it with ESP-IDF `esp_fill_random` only when absent, rejects
corruption, and never exposes the seed through ordinary settings, HTTP, logs,
CDC evidence, telemetry, backups, or package artifacts. OTA preserves the
namespace. Identity rotation requires absence after an explicit factory reset
or documented fail-closed corruption recovery; no irreversible eFuse policy is
enabled here.

The pure Worker-control core parses strict frames, signs only the closed
fresh-nonce possession claims, and grants enumeration-local admission after a
proof response is issued. Discover and possession are available before
admission. Start and Renew require current-enumeration admission plus an
injected verifier over the complete normalized request. Authorization and
Stratum material remain volatile. Parsed credential fields, raw payloads,
proof-response buffers, and Device Identity seed buffers zeroize on every
success and error path.

Possession nonce replay state is bounded to one enumeration and fails closed
at capacity without eviction. Work Lease authorization high-water marks are
durable across reboot, firmware update, trust overlap, and rollback. Firmware
never automatically removes a mark when a key leaves current trust; destructive
key retirement requires a separately authorized migration that prevents key-ID
reuse.

Before an authenticated Start can reach the Production Mining Session, the
same dedicated NVS owner advances a closed non-secret journal from `clear` to
`effect_pending`. Confirmed safe stop clears it last. A reboot, failed cleanup,
firmware update, or rollback therefore retains cleanup responsibility without
retaining the lease, credentials, proof, Device Identity, or pool data. On
boot, the existing boot-safe hardware gate advances `effect_pending` to
`reboot_baseline_confirmed` before identity, trust, protocol-owner, or USB
initialization. That confirmation remains durable until a sent status is
followed by another valid host request in the same enumeration, which proves
the host observed the recovery before the journal clears.

The firmware adapter translates accepted leases and every terminal or
continuity-loss reason through the sole Production Mining Session. It allocates
strictly increasing owner-local campaign identities and converts each Worker
window to its exact absolute deadline in the shared boot monotonic domain. It
never adds a second mining owner. Pause, Cancel, expiry, disconnect, reboot,
monotonic reset, lost continuity, and explicit Restore invalidate work, erase
credentials, and retain retry responsibility until the established safe-stop
ordering confirms the Mining Baseline.

The consumed conformance contract is pinned through Bazel to Gate commit
`0b07d36942aa8ca3473771d2f72a373e66cedf58` and archive SHA-256
`c23314d96b33f51119395fafb0dc2aa3f1b0017fd5db5379e42e3f8348f20f96`.
The Device Identity signature includes the compiled firmware source commit so
an exact-package browser can reject a possessed but stale installation.
Bazel verifies the package export paths before fixture-based tests execute.

## Consequences

- USB VID/PID, serial, and enumeration identity remain hints, not authority.
- Admission is cleared on disconnect and reboot; an old lease never resumes.
- The pure core and host conformance runner can be tested without ESP-IDF,
  TinyUSB, NVS, a pool, or hardware.
- Missing, malformed, stale, or mismatched deployment trust remains fail closed.
- `esp_tinyusb` `1.4.5` is the exact ESP-IDF 5.5-compatible device stack. The
  exact `esp-idf-sys` commit `f616563a87595032f06f1fec95b6816b1c11135c`
  supplies its upstream TinyUSB bindgen blocklist fix.
- Pairing, accounts, remote relay, tamper-proof attestation, registry
  publication, eFuse hardening, and hardware qualification remain out of
  scope.
