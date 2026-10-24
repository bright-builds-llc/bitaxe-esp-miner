# Gamma 601: the first BM1370 chip detection

Phase F3.1 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 8) ran the test-only `gamma601-chip-check` variant once:
- it turned the TPS546 on at 1.15 V through the production power path,
  with the ASIC reset held and the pad latched;
- it ran the BM1370's chip detection through the executor: the reset
  pulse, the `CHIP_ID` read and the count;
- it held the reset low again and turned the output off, confirmed.

Attempt 002 passed: exactly one BM1370 answered.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just package-gamma601-chip-check`;
  3. `just flash-monitor --board 601 --port <port> --manifest <the 601
     package manifest> --allow-test-firmware gamma601-chip-check
     --capture-timeout-seconds 360`, state-preserving;
  4. `just package --board 601`, then `just flash --board 601 --port
     <port>`, state-preserving, and `just monitor --board 601 --port
     <port> --capture-timeout-seconds 360`, a receive-only capture.
- **Sources:** the software at `cedd5f25`; attempt 002 at `824d7e20` (the
  contract's command fix); the reference at `c1915b0`.
- **Manifests:** read at packaging time and not retained: step 3's
  recorded `firmware_variant=gamma601-chip-check`, board 601, source
  `824d7e20`; step 4's recorded `production` at the same source. The flash
  tool checks the manifest's variant against the ELF marker, and accepted
  each install only with the matching allowance (step 3) or none (step 4).

## Attempt 001: refused, no effect

- **The contract's step 3 omitted `--manifest`.** `flash-monitor --board
  601` therefore built the production 601 package, and the flash tool's
  identity admission refused the test allowance before any write
  (`test_firmware_allowance_without_test_package`); its output holds no
  flash command.
- **The fix (`824d7e20`):** the contract names the chip check's manifest,
  as the core-voltage check's did.
- **Outcome:** `continue_after_verified_fix`. The refused `flash-monitor`
  had rebuilt the package output as production, so attempt 002 packaged
  the variant again; its manifest recorded the variant at `824d7e20`.

## Attempt 002: passed

The `usb_gamma601_chip_check` record, replayed 26 times after the check
(after 2 `pending`):

| Field | Value | Contract |
| --- | --- | --- |
| `result` / `reason` | `passed` / `none` | `passed` |
| `chips` / `blocker` | `1` / `none` | `1` |
| VOUT at the verify | 1146 mV (1.15 V target) | within 50 mV |
| IOUT peak | 1648 mA | recorded |
| VR temperature peak | 33 C | recorded |
| `on_ms` (power-on sequence start to the off command) | 2257 ms | recorded |
| `off` after the check | `confirmed` | `confirmed` |
| Free stack after the check | 7108 bytes (12 KiB thread) | recorded |
| Diode before / after | 127.8 C / 127.8 C | recorded; see below |

The safe-boot record:
- **Before the window (2 records):** `reset_held=true`, `off=confirmed`,
  10 mV, `faults`/`cml` `0x0202`/`0x02`, the boot's state before the
  check's `CLEAR_FAULTS`.
- **During the window (1 record):** `off=unread`, readings unavailable,
  `reset_held=true`, `fan_duty_pct=100`, as designed.
- **After it (25 records):** `reset_held=true`, `off=confirmed`, 20 mV,
  `fan_duty_pct=30`, `faults`/`cml` `0x0000`/`0x00`.
- **One boot:** `boot_ordinal=1` throughout, the install's own reset; no
  restart during or after the check.
- **Production afterwards (step 4, 56 records):** `built=601 stored=601
  identity=match`, `off=confirmed`, `reset_held=true`, 20 mV, fan 30 %.
  It carries no check record (the variant's capture carried 28). The
  steady `cml=0x02` is the safe boot's own transfer bit that phase D2
  recorded.
- **USB transmit failures:** the production capture also replays 57
  `usb_tx_failure stage=flush_timeout` records, and the variant's none.
  Production booted after `just flash` with no host reading until `just
  monitor` attached, the reader gap of
  `20260927-usb-bootstrap-reader-gap-measurement.md`; the variant's
  capture followed its install directly.

## The diode reading

The EMC2101's external channel read 127.8 C both with the ASIC unpowered
and after the count, powered and out of reset. In the decoder's 0.125 C
steps only 127.75 C (raw `0x3FE`) and 127.875 C (`0x3FF`, the rejected
short code) print as 127.8, so the raw code was `0x3FE`. That is not one
of the two fault codes the decoder rejects (`0x3F8` open, `0x3FF` short),
so it was accepted as a value.
- Upstream reads the 601's chip temperature only once the ASIC is
  initialized (`Thermal_get_chip_temp` returns -1 before), with ideality
  `0x24` and beta `0x00`, which the safe boot also writes.
- Detection is not initialization, so this run does not show what the
  diode reads after the full BM1370 init. Phase F3.2 must read it after
  the init before the cooling proof's board rule is settled. A reading
  this high trips the overheat stop, which is the safe direction.

## Effects

- **Attempt 001:** none; refused at identity admission.
- **Attempt 002:** one variant install, one production install and the
  receive-only capture.
- **The variant's writes:**
  - the volatile Gamma configuration, never stored, and `CLEAR_FAULTS`;
  - the fan at 100 % for the check;
  - one enable at 1.15 V for 2257 ms from the start of the power-on
    sequence to the off command;
  - the chip detection: the reset pulse, the baud setting, the version
    mask writes and the `CHIP_ID` read, at the chip's reset-default
    frequency.

There was no work frame, frequency change, NVM write, factory install,
HTTP request, network discovery, UART or pin access. The variant is not
installed.

## Not shown here

- **The precondition's power removal:** the owner disconnected the 601
  for the 205 regression, but this run cannot show that all power was
  removed since the core-voltage check. The check writes and reads back
  the full configuration before the enable, which the contract's
  precondition was there to protect.
- **The BM1370 initialization, the frequency ramp, work and nonces:**
  phase F3.2.
- **The diode after the init:** above.
- **The pad latch and the restart handler through a real reset:** no
  reset happened during the window.
- **Parity:** nothing is promoted. The check is a test variant, and
  production 601 dispatch stays refused until phase F4.
