# Gamma 601: the TPS546 configuration and a bounded core-voltage check

Phase D4 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 6) ran the test-only `gamma601-vout-check` variant once:
- it configured the TPS546 as upstream does for the Gamma;
- it read every register back;
- it turned the core voltage on at 1.15 V for 60 s, with the ASIC held in
  reset, then off.

The same run also answered the D2 question: where does the latched CML bit
come from? Attempt 002 passed.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: production, state-preserving;
  3. the owner removed all power and restored it, then `just monitor
     --board 601 --port <port> --capture-timeout-seconds 360`, a
     receive-only capture;
  4. `just package-gamma601-vout-check`, then `just flash-monitor --board
     601 --port <port> --manifest <the 601 package manifest>
     --allow-test-firmware gamma601-vout-check
     --capture-timeout-seconds 360`;
  5. production reinstalled as in step 2.
- **Sources:** attempt 001 at `cc6bf00f`, attempt 002 at `4a7c5717` (the
  fix `ab795002`); the reference at `c1915b0`. Attempt 002 began at step
  3, so its cold boot ran the production `cc6bf00f` that attempt 001's
  step 5 installed.
- **Manifests:** step 4's records `firmware_variant=gamma601-vout-check`,
  board 601. Steps 2 and 5 record `production`.

## Attempt 001: an evidence failure

- **Steps 1-3 and 5 passed.**
- **Step 4 ran the check, but its outcome could not be seen.** `faults`
  and `cml` went to `0x0000`/`0x00`, and only the check's `CLEAR_FAULTS`
  sends that, after a matched readback. The safe-boot records then read
  `off=confirmed` and `reset_held=true`.
- **The cause:** the check wrote its record to the general log only. The
  receive-only console carries the device's replay records, not that log.
- **The fix (`ab795002`):** the variant replays the check's record in turn
  with the safe-boot record. It reads `running` while the check runs.
- **Outcome:** `continue_after_verified_fix`. The controller kept the
  check's configuration, so attempt 002 began after the owner removed all
  power (`4a7c5717`).

## Attempt 002: passed

The `usb_gamma601_vout_check` record, replayed 21 times after the check
(after 2 `pending` and 5 `running`):

| Field | Value | Contract |
| --- | --- | --- |
| `result` / `reason` | `passed` / `none` | `passed` |
| `vout_mode` | `0x97` | the plan's gate |
| `readback` | `match` (the plan writes 32 registers) | `match` |
| `config_faults` / `config_cml` | `0x0002` / `0x02` | recorded |
| `faults_before` / `faults_after` | `0x0000` / `0x0000` | no new fault |
| `samples` | 240 | at least 230 |
| VOUT | 1144 to 1146 mV (1.15 V target; the truncated command is 1148 mV) | 1100 to 1200 mV |
| IOUT peak | 710 mA | under 5000 mA |
| VIN | 5414 to 5429 mV | at least 4800 mV |
| VR / board temperature peak | 36 C / 26 C | under 80 C / 60 C |
| `off` after the check | `confirmed` | `confirmed` |
| Free stack after the check | 8236 bytes | at least 2048 |

The safe-boot record:
- **During the window (5 records):** `off=unread`, readings unavailable
  and `fan_duty_pct=100`, as the contract expects.
- **After it (21 records):** `reset_held=true`, `off=confirmed`, 20 mV,
  `fan_duty_pct=30`, and `faults`/`cml` `0x0000`/`0x00`.
- **One boot:** the capture holds one boot (`boot_ordinal=1`, reset
  category `other`, the install's own reset).
- **Production afterwards (step 5, 56 records):** `built=601 stored=601
  identity=match`, `off=confirmed`, `reset_held=true`, 20 mV, fan 30 % at
  3545 to 3583 rpm. It carries no check record (the variant's capture
  carried 28).

## The CML bit (D2)

| Boot | `pre_id_status` / `pre_id_cml` | `start_status` / `start_cml` | Steady `cml` |
| --- | --- | --- | --- |
| Attempt 001, warm install | `0x0842` / `0x82` | `0x0842` / `0x82` | `0x82` |
| Attempt 001, after all power was removed | `0xffff` / `0x80` | `0x0002` / `0x80` | `0x82` |
| Attempt 001, warm install after the check cleared faults | `0x0840` / `0x00` | `0x0840` / `0x00` | `0x02` |
| Attempt 002, after all power was removed (`cc6bf00f`) | `0x0000` / `0x00` | `0x0000` / `0x00` | `0x02` |

- **CML bit 1 (`0x02`)** is set on every boot by the safe boot's own
  transfers after its start reads: the off writes or the first sweep. The
  configuration writes also left it set (`config_cml=0x02`).
- **CML bit 7 (`0x80`, invalid command)** appeared only after the cold
  boot whose first read returned all-ones. That read most likely came
  before the controller was ready; upstream waits 15 ms before its first
  transfer. A later cold boot read cleanly and set no bit 7.
- **At power-on the output comes up on by itself** (`start_status` without
  `OFF`), as phase C found.

## Effects

- **Attempt 001:** two production installs, one variant install, and one
  owner power cycle.
- **Attempt 002:** one variant install, one production install, and the
  owner's power cycle before it.
- **The variant's writes, each attempt:**
  - the volatile Gamma configuration, never stored;
  - `CLEAR_FAULTS`;
  - the fan at 100 %;
  - one enable for 60 s at 1.15 V, with the ASIC reset held and latched.

There was no NVM write, factory install, ASIC release, HTTP, network
discovery, UART or pin access. The variant is not installed.

## What this shows

On this Gamma 601, upstream's TPS546 configuration, with the two
voltage-bounding deviations, writes and reads back exactly. The
controller regulates 1.15 V steadily: 2 mV of spread over 60 s. With the
ASIC held in reset it draws under 1 A. It turns off and confirms off on
command.

## Not shown here

- **The ASIC powered out of reset, and BM1370 initialization:** phase E/F.
- **Regulation under the ASIC's load,** and any voltage other than 1.15 V.
- **Whether the latched reset pad survives a panic or watchdog reset:** no
  reset happened during the window.
- **Fan speed during the window:** recorded only by the safe boot before
  and after.
- **Parity:** PWR-004 is not promoted. The check is a test variant, and
  production does not configure the controller.
