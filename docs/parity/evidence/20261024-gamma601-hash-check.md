# Gamma 601: the BM1370's first hashing

Phase F3.2 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 9) ran the test-only `gamma601-hash-check` variant once:
- the chip check's powered sequence: the TPS546 on at 1.15 V with the
  reset held and latched, then the chip detection, exactly one BM1370;
- the BM1370 init through the executor, ramped to 300 MHz in segments,
  and upstream's 2 s stabilization;
- the diode gate, then the Bitcoin genesis header as one job, every
  result verified in firmware against the known header;
- the reset held and latched, and the output turned off, confirmed.

Attempt 001 passed.

- **Commands:**
  1. the owner removed all power (barrel and USB) and restored it;
  2. `just detect-ultra205`, here and again before step 5's install;
  3. `just package-gamma601-hash-check`;
  4. `just flash-monitor --board 601 --port <port> --manifest <the 601
     package manifest> --allow-test-firmware gamma601-hash-check
     --capture-timeout-seconds 360`, state-preserving;
  5. `just package --board 601`, then `just flash --board 601 --port
     <port>`, state-preserving, and `just monitor --board 601 --port
     <port> --capture-timeout-seconds 360`, a receive-only capture.
- **Sources:** the software at `48324b00`; the attempt at `92d865c4` (the
  contract); the reference at `c1915b0`.
- **Manifests,** retained privately with the captures: step 3's records
  `firmware_variant=gamma601-hash-check`, board 601, source `92d865c4`;
  step 5's records `production`.

## The record

The `usb_gamma601_hash_check` record, replayed 25 times after the check
(after 2 `pending` and 1 `running`):

| Field | Value | Limit (the contract's pass rule, or the firmware stop) |
| --- | --- | --- |
| `result` / `reason` | `passed` / `none` | `passed` |
| `chips` / `blocker` | `1` / `none` | `1` |
| `freq_mhz` / `target_mv` | 300 / 1150 | 300 / 1150 |
| VOUT at the verify | 1145 mV | within 50 mV |
| `verified` / `invalid` / `duplicate` / `discarded` | 8 / 0 / 0 / 0 | at least 3; invalid at most a tenth |
| `genesis_hit` | `true` | information only |
| `best_difficulty` | 3406 | recorded |
| `first_result_ms` / `window_ms` | 96 / 7704 | the window at most 20000 |
| Diode before / at the gate / peak | 127.8 C / 37.5 C / 40.0 C | the gate 10 to 65 C; under 65 C |
| VIN minimum | 5289 mV | at least 4600 mV |
| IOUT / power peak | 6320 mA / 7234 mW | under 20 A / 22 W |
| VR temperature peak | 39 C | under 90 C |
| `on_ms` (power-on sequence start to the off command) | 16899 ms | under 45000 |
| `off` after the check | `confirmed` | `confirmed` |
| Free stack after the check | 10512 bytes (16 KiB thread) | recorded |

- **The known answer itself:** `genesis_hit=true` means the chip returned
  the genesis block's nonce `0x7c2bac1d` with no version bits rolled, and
  it verified. The design review expected the chip's version rolling to
  make that unlikely; it did not prevent it.
- **The window ended early,** at the eighth verified result after 7.7 s,
  as designed.

The safe-boot record:
- **Before the check (2 records):** `reset_held=true`, `off=confirmed`,
  10 mV, `faults`/`cml` `0x0002`/`0x02`.
- **During it (2 records):** `off=unread`, readings unavailable,
  `fan_duty_pct=100`, as designed.
- **After it (24 records):** `reset_held=true`, `off=confirmed`, 20 mV,
  `fan_duty_pct=30`, `faults`/`cml` `0x0000`/`0x00`.
- **One boot:** `boot_ordinal=1` throughout, `built=601 stored=601
  identity=match`; no restart during or after the check.
- **Production afterwards (step 5, 56 records):** `identity=match`,
  `off=confirmed`, `reset_held=true`, 20 mV, fan 30 %, no check record. Its
  steady `cml=0x02` is phase D2's own transfer bit, and its 57
  `usb_tx_failure stage=flush_timeout` records are the reader gap between
  `just flash` and `just monitor`, as in F3.1.

## The diode (F3.1's open item)

F3.1 read 127.75 C from the EMC2101's external channel with the ASIC
unpowered and after detection. Here the same channel read 127.8 C before
the enable, then 37.5 C at the gate after the init and 40.0 C at its peak
while hashing. The init's write of register `0x54` (the analog mux,
`bm1370.c`) comes between, and upstream reads the chip temperature only
after its init. So on this board the diode is usable once the BM1370 is
initialized, and the high value before is an unpowered or uninitialized
channel, not a temperature.

## Effects

- **The owner's power cycle** before the attempt.
- **One variant install, one production install** and the receive-only
  capture.
- **The variant's writes:**
  - the volatile Gamma configuration, never stored, and `CLEAR_FAULTS`;
  - the fan at 100 % for the check;
  - one enable at 1.15 V for 16899 ms from the start of the power-on
    sequence to the off command;
  - the chip detection, then the BM1370 init and the ramp to 300 MHz;
  - one genesis job, hashed for 7.7 s.

There was no pool or Stratum traffic, NVM write, factory install, HTTP
request, network discovery, UART or pin access. The variant is not
installed.

## Not shown here

- **Any frequency other than 300 MHz,** or upstream's 525 MHz default.
- **Sustained hashing,** a pool, shares and the thermal steady state: the
  hashing lasted 7.7 s.
- **The power at full load:** 7.2 W peak here, against the 22 W stop that
  still has no evidence at the default frequency.
- **The pad latch and the restart handler through a real reset:** no
  reset happened during the window.
- **Parity:** nothing is promoted. The check is a test variant, and
  production 601 dispatch stays refused until phase F4.
