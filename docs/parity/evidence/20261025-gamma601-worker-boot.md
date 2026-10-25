# Gamma 601: the Worker boots on the production 601

ADR-0041 item 10 made every 601 build run the BWG Worker. This run installed
the production 601 package once and captured its boot. Attempt 001 passed.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just package --board 601`;
  3. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`, state-preserving.
- **Sources:** the software at `2bf484b8`; the attempt at `e9d3292f` (the
  contract); the reference at `c1915b0`.
- **Manifest,** retained privately with the capture:
  `firmware_variant=production`, board 601, source `e9d3292f`.

## Observed

| Fact | Value | Contract |
| --- | --- | --- |
| USB boot profile (36 records) | `transport=serial_jtag_runtime`, `reason=worker_started`, `baseline=confirmed` | the Worker prepared |
| `boot_baseline_unconfirmed`, `recovery_failed`, Worker start failure | none | none |
| Board identity (56 records) | `built=601 stored=601 identity=match` | match |
| Safe-boot records (56) | `reset_held=true`, `off=confirmed`, 20 mV, fan 30 % | held safe |
| Boots | `boot_ordinal=1` throughout | one |
| USB transmit failures | 0 | recorded |

- **The steady `faults=0x0002`** in the safe-boot records is phase D2's
  CML bit from the safe boot's own transfers.
- **Heap.** The boot's memory checkpoints, against a production Ultra 205
  capture whose Worker runs the same code:

| Checkpoint | 601 free / largest block | 205 free / largest block |
| --- | --- | --- |
| `worker_owner_prepare` | 97303 / 63488 bytes | 99155 / 63488 bytes |
| `statistics_started` | 44207 / 31744 bytes | 46079 / 31744 bytes |

The 601 runs about 2 KB tighter than the 205, with the same largest free
block.

## Effects

One production install and its reset. The first Worker boot on this 601
wrote its Device Identity seed to NVS. There was no Gate session, Worker
Start, test variant, factory install, HTTP request, network discovery, UART
or pin access.

## Not shown here

- **A Gate session,** the capability's admission by the Gate, and a
  production 601 refusing a Start: they come with phase F3.3, which drives
  the Gate tab.
- **Worker recovery after an interrupted effect,** and a recovery failure.
- **The 205 at this commit:** its capability and strings are unchanged by
  host tests; its hardware regression waits for the next 205 session.
- **Parity:** nothing is promoted.
