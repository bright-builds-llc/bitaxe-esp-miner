# Idle-review panic: captured, decoded, attributed to memory corruption

During heartbeat005's read-only baseline reviews, the Ultra 205 reset with
`reset_reason=panic` (boot 15 to 16). The device was idle: no grant, no
mining. One current recovery and one core-dump read retrieved the stored dump.
Offline analysis binds the dump to the installed image and identifies the
faulting thread and instruction. The underlying corruption source is not yet
proven.

| Boundary            | Direct evidence                                                                                                                                                                                       |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Image               | Source `654338d0`, ELF `2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193`; exact debug ELF retained                                                                                   |
| Recovery001         | Current safe recovery: idle V2, ledger next 26 / last 25 / 3,000,000 ms (unchanged), restoration and release proven, fresh proof written                                                              |
| Capture001          | One `core-dump-read` inside the proof window; ROM admitted, partition read only, application identity restored, cleanup complete; no erase or write                                                   |
| Dump binding        | Checksum verified; the single ELF identity note equals the installed ELF; the native allocation history is bound to boot 15                                                                           |
| Allocation failures | None on either core                                                                                                                                                                                   |
| Abort message       | No ESP panic-details note: not `abort()`, an assert, the stack-overflow hook or a Rust panic                                                                                                          |
| Original exception  | From the firmware's captured panic-frame record: CPU exception `StoreProhibited` (cause 29) to a near-null address                                                                                    |
| Faulting code       | `std::sync::mpsc::sync_channel` (store through the caller's return-slot pointer), called from `bwg_worker_usb::writer::send_control`                                                                  |
| Faulting thread     | The Worker control owner (`bwg-worker-control`, 16 KiB internal stack, core 1); `send_control` runs only there, while replying to a command                                                           |
| Register state      | The return-slot pointer, return address and stack pointer were all restored as non-stack values after the inner allocation call. The stack pointer points into the general heap, not the task's stack |
| SDK dump view       | Its rewritten view (`exccause` 0xffff, fake frame) only reflects that it rejected the foreign stack pointer; the firmware record preserves the original frame                                         |

## Interpretation

The control owner's register-window spill area was overwritten while
`sync_channel` was allocating. Window underflow then restored a heap address as
the stack pointer and a small integer as the return-slot pointer, and the next
store faulted. The overwritten words look like heap allocator metadata (two
equal pointers and a flagged size). That suggests heap or stack memory
corruption, for example memory the allocator treated as free, rather than a
stack overflow or an out-of-memory abort.

This is a hypothesis for the next diagnostic, not a proven root cause.

## Non-claims

- No correction is claimed. Identifying the writer that corrupted memory needs
  instrumented firmware: heap integrity checks or poisoning on the control
  path, then a bounded reproduction.
- The heartbeat-loss seal stays unverified until a corrected image passes a
  fresh restart and heartbeat attempt.
- The original Share001/Share002 panics are not reclassified by this dump.
- Parity stays 90/95.

Raw dumps, decoder output and memory contents remain in protected private roots
under `scratch/str005-idle-panic*`. This summary carries only categories and
symbol names.

## Reproduction, root cause and correction

A diagnostic image (`c634cc20`) added three things:

- an end-of-stack watchpoint;
- per-command internal-heap integrity checks;
- a captured control-stack trace.

A bounded read-only review loop then reproduced the panic: loop005 failed at
round 20, and the device rebooted with `reset_reason=panic`. The archive-bound
clear had emptied the core partition, so that dump was stored and read.

| Finding              | Direct evidence                                                                                                                                                                                                                                                |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Same fault           | `StoreProhibited` at `std::sync::mpsc::sync_channel<bool>+22` in `writer::send_control`, control owner                                                                                                                                                         |
| Not a stack overflow | At least 6,100 of 16,384 bytes always free across 106 commands; end-of-stack watchpoint never fired                                                                                                                                                            |
| Not heap corruption  | No panic-details note: heap integrity checks passed; no allocation failures                                                                                                                                                                                    |
| Mechanism            | `std::sync::mpmc` channel constructors realign their frame to 64 bytes with `add.n a1, a1, a8` instead of `movsp`. An interrupt in that prologue saves the caller's registers below the old stack pointer, and the return reloads them from below the new one. |
| Reconstructed frames | The callee returned correctly. Only the caller's restored return address and return-slot pointer were stale, giving the near-null store.                                                                                                                       |

The correction (`24af10be`) gives per-request replies a one-shot slot that
holds only a mutex and a condition variable (`bitaxe_runtime::reply`). It is
used for:

- control replies;
- Worker Start, Renew, SafeStop and cooling;
- safety actuation requests;
- deferred HTTP effects.

`just audit-stack-realignment` blocks any non-startup caller of a realigning
function. It blocks the previous image (17 callers, including
`send_control`) and passes the corrected one (9 startup-only callers).

Verification on the corrected image (`7ca3e29c`, ELF `227bc380…`):

- a state-preserving install with all native audits passing;
- 500 read-only review rounds in five fresh sessions with no failure;
- a current recovery shows no reboot during the loop and the ledger unchanged
  at next 26.

Residual risk:

- The toolchain hazard remains for the nine startup-time channel creations.
  Each is one call per boot, and a panic there resets safely.
  The hazard is in LLVM's Xtensa backend and is present in every esp
  toolchain tested, from 1.88.0.0 to 1.99.0.0. It is reported as
  [espressif/llvm-project#140](https://github.com/espressif/llvm-project/issues/140)
  and [esp-rs/rust#284](https://github.com/esp-rs/rust/issues/284).
- A separate defect: after a panic reset, the USB link stayed unusable until
  a physical USB replug.
- Parity stays 90/95.

## Startup channels replaced (queue workaround)

The nine startup-time std channels were replaced by `bitaxe_runtime::queue`.
The realignment audit's allowlist is now empty, and the image contains no
realigning function at all.

The first queue image (`a2052ab0`) panicked on every boot:

- A one-time reproduction, after an archive-verified clear of the core-dump
  partition, stored its dump.
- The decode shows the 16 KiB main task overflowing in the Worker trust
  parse's curve25519 key checks.
- The cause: the change let `production_mining_session::start` (4.4 KiB
  frame) inline into `run_startup`. That frame grew from 1,776 to 4,896 bytes
  and stayed live under the parse.

The fix keeps `start` out of line (1,072 bytes). `just audit-startup-frames`
blocks the failing image and passes the previous and fixed ones.

Verification on `6f268518` (ELF `b6908f6d…`):

- Reinstall attempt-002 `complete` and `hardware_qualified`.
- 500 read-only review rounds (loop007) with no failure.
- recovery016 on the same boot as the install's restoration, with the
  ledger unchanged at next 27, last 26, 3,180,000 ms.

Parity stays 90/95.
