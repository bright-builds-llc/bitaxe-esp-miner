# Device Noise helper successor amendment

## Status and precedence

Owner-authorized on 2026-10-02 under `task-device-noise-worker-stack`.
Contract ID: `str005-noise-serial-v2`, successor profile `device-noise-helper`.

This amendment, the [v1 contract](str005-noise-serial-qualification.md) and the
[v2 parity amendment](str005-noise-parity-scope-amendment.md) together define
one new positive Noise attempt. It overrides only the requirements named below.
All other v1 and v2 identity, authentication, privacy, ownership, preservation,
accounting, publication and evidence requirements apply unchanged. The archived
`task-str005-noise-auth-205` result, its attempt-001 evidence and its readers
stay unchanged and keep their original meaning.

## Why a successor attempt

Offline analysis of the current firmware found that act-two authentication on
the 12 KiB pool transport worker needed 11,424 bytes through certificate
verification, against a 10,240-byte budget with the 2,048-byte margin. Internal
RAM cannot grow the boot-time workers. See the
[worker stack analysis](../research/20261002-device-noise-worker-stack.md).

The firmware now runs only `complete_diagnostic_into` on a joined 16 KiB helper
whose stack comes from PSRAM. This attempt checks that the changed runtime
completes a real handshake on the device without a fault.

## Overrides

| Requirement                          | Successor meaning                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crypto ownership (v2)                | Initiator construction, act one, socket I/O and proof stay on the existing separately owned 12 KiB worker. Act-two authentication runs on a short-lived 16 KiB PSRAM-stack helper that the worker joins before continuing. No worker stack size, priority, affinity, buffer or internal reserve changes. The helper performs no flash operation. |
| Cancellation (v2)                    | Unchanged in substance: the worker blocks in the opaque authentication call exactly as before, and every check before and after that call is unchanged. A helper spawn failure is a typed `allocation` failure; a helper panic is a typed `authentication` failure.                                                                   |
| Native readiness                     | The worker selected path must reach act-one crypto and must not reach `schnorrsig_verify` or `step_2_with_now`. The helper audit (`just audit-device-noise-stack`) must pass inside the readiness receipt. The borrowed-job edge may be bound by the worker's literal reference to `run_job`, still tied to `borrow.run_job()` in source. |
| Predecessor                          | The sealed safety-recovery006 result of `task-str005-v2-accepted-share-probe`: result SHA-256 `609698fb42c694def0e6f736281d2a31558453707ce1bbc2931fe3e5b5f4aac9`, seal `08b70903f7129ab435945871746d853dd4a297a2205e8c0660b2eb373a158d56`. It is a current-safe-recovery basis, not a qualification pass. It is not the retired fixed-USB recovery-006 plan. |
| Before identity and ledgers          | Installed source `ce8f015811b93385c5aab0bac7bcdf6307452b31`, ELF `453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31`. Expected idle ledger next 22, last 21, charged 2,280,000 ms, with the original budget exhausted. Both must be observed unchanged before and after.                                               |
| Attempt namespace and task gate      | Private roots live under `scratch/device-noise-worker-stack/`. Preflight requires `task-device-noise-worker-stack` under `## Active` with the exact line `Device noise serial hardware: enabled.`                                                                                                                                     |

Everything else stays as v2 requires: state-preserving initial installation plus
four update, reconnect and maximum-exchange cycles; at most five writes; 30-second
installation captures; no mining, Work Lease, grant, signer, pool credentials,
Wi-Fi provisioning, factory reset, erase or rollback; the native-browser
permission and Connect gestures; the same retained page baseline; and mandatory
`finalize` and `review`. `recover` remains unavailable.

## Non-claims

A pass shows that the corrected runtime authenticated one handshake on the
device and restored cleanly. It does not measure the helper's runtime stack
margin, because the Noise status carries no helper high-water field; a helper
overflow would still trip the FreeRTOS stack canary and fail the attempt. It is
not channel, share or mining evidence, and parity stays 90/95.
