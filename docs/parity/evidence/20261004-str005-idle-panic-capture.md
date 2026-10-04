# Idle-review panic: captured, decoded, attributed to memory corruption

During heartbeat005's read-only baseline reviews, the Ultra 205 reset with
`reset_reason=panic` (boot 15 to 16). The device was idle: no grant, no
mining. One current recovery and one core-dump read retrieved the stored dump.
Offline analysis binds the dump to the installed image and identifies the
faulting thread and instruction. The underlying corruption source is not yet
proven.

| Boundary            | Direct evidence                                                                                                              |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Image               | Source `654338d0`, ELF `2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193`; exact debug ELF retained            |
| Recovery001         | Current safe recovery: idle V2, ledger next 26 / last 25 / 3,000,000 ms (unchanged), restoration and release proven, fresh proof written |
| Capture001          | One `core-dump-read` inside the proof window; ROM admitted, partition read only, application identity restored, cleanup complete; no erase or write |
| Dump binding        | Checksum verified; the single ELF identity note equals the installed ELF; the native allocation history is bound to boot 15  |
| Allocation failures | None on either core                                                                                                          |
| Abort message       | No ESP panic-details note: not `abort()`, an assert, the stack-overflow hook or a Rust panic                                 |
| Original exception  | From the firmware's captured panic-frame record: CPU exception `StoreProhibited` (cause 29) to a near-null address           |
| Faulting code       | `std::sync::mpsc::sync_channel` (store through the caller's return-slot pointer), called from `bwg_worker_usb::writer::send_control` |
| Faulting thread     | The Worker control owner (`bwg-worker-control`, 16 KiB internal stack, core 1); `send_control` runs only there, while replying to a command |
| Register state      | The return-slot pointer, return address and stack pointer were all restored as non-stack values after the inner allocation call. The stack pointer points into the general heap, not the task's stack |
| SDK dump view       | Its rewritten view (`exccause` 0xffff, fake frame) only reflects that it rejected the foreign stack pointer; the firmware record preserves the original frame |

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
