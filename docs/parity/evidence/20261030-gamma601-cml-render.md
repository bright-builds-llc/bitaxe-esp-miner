# Gamma 601: the TPS546 CML bit and the startup screen

The follow-up in `task-gamma-601-bring-up` asks where the TPS546's latched
`STATUS_CML` bit 1 (`0x02`, other communication fault) comes from. The
existing captures placed it between the boot's start read and the steady
records, and the startup screen was the suspect. This run read `STATUS_CML`
right before and right after the screen, on a boot that began clear. The
bit was still clear after the screen's step. The captures do not show that
the screen's transfers ran.

- **Commands** (the render-bracket contract in `TASKS.md`):
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`, state-preserving;
  3. the owner's power cycle: barrel and USB off, about 10 s, barrel, then
     USB;
  4. the detector again, then `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`, receive-only.
- **Sources:** `56bcc2ea`; the reference at `c1915b0`.
- **Package:** `firmware_variant=production`, board 601, source `56bcc2ea`.
- **Record:** `usb_gamma601_safe_boot` schema v4. It adds `pre_render_cml`
  (after the off writes) and `post_render_cml` (after the screen), both
  reads only.

## The readings

Every record in each capture carries the same values.

| Field | Install (warm, 56 records) | After the power cycle (56 records) |
| --- | --- | --- |
| Reset reason | `other` | `power_on` |
| `pre_id_status` / `pre_id_cml` | `0x0842` / `0x82` | `0x0000` / `0x00` |
| `start_status` / `start_cml` | `0x0842` / `0x82` | `0x0000` / `0x00` |
| `pre_render_cml` | `0x82` | `0x00` |
| `post_render_cml` | `0x82` | `0x00` |
| `cml` (the record's latest sweep) | `0x82` | `0x02` |
| `faults` | `0x0002` | `0x0002` |

The install's boot says nothing about the question: it began with the bits
already latched. The cold boot does:
- **The bit was clear at the `post_render_cml` read,** after the identity
  read, the start reads, the off writes and the screen's step. A fault that
  latches after that read is not excluded.
- **It is set after that read and before the captured records' sweep.** The
  cold capture's first safe-boot record comes at about 40 s of uptime, so
  this boot bounds it to the first 40 s, about 70 sweeps. Earlier warm boots
  that began clear (`scratch/self001-gamma601/attempt-009`,
  `scratch/gamma-601-promotion/run-005`) already read `cml=0x02` at about
  5.6 s.

Both captures held the safe-boot readings: `reset_held=true`,
`off=confirmed`, `tps546=d24s`, fan 30 %, `built=601 stored=601
identity=match`, one boot, and no panic. The fan read 3545 to 3564 rpm on
the install and 3583 to 3602 rpm after the power cycle.

## What this changes

Comparing boots had pointed at the screen, as the only kind of transfer
unique to the window. If the screen's transfers ran on this boot, that is
refuted, and the bit does not follow from any kind of transfer seen so far:
- what remains in the window is the early sweeps, whose transfers repeat
  about every 500 ms, and the safe boot's conditional off re-assert;
- the same transfers leave the bit clear after a `CLEAR_FAULTS`, in direct
  captures of 5 to 6 minutes. Longer spans are inferred from the next boot's
  `pre_id_cml`.

So something present early in a boot and absent later is the likelier
cause, not a particular command. Other startup work runs alongside the early
sweeps, though the safe-boot thread alone owns the bus, so it could act only
indirectly. This is a direction for the next step, not a finding.

## Not shown here

- Which transfer or moment sets bit 1.
- That the startup screen was drawn. Its success goes only to the general
  log, which the receive-only console does not carry, and a configuration
  error skips its transfers.
- Bit 7 (`0x80`): the `0x82` on the install's boot was latched before that
  boot's first read, probably by an earlier boot's too-early read (phase
  D4), perhaps at the board swap. No capture covers it.
- Any harm from the bit. Mining's start clears it, and the regulator's
  baseline accepts it.

## Effects

- the detector checks;
- one state-preserving production install with its resets;
- the owner's power cycle;
- one receive-only capture;
- the installed production firmware's own effects. Mining on boot is not
  recorded directly; `off=confirmed` in every record is consistent with it
  being off.

There was no factory install, settings or NVS write, HTTP, test build, 205
package, mining, direct UART or pin access.

The 601 stays on production `56bcc2ea`. That is one commit past the code its
F4 evidence covers; the change adds these two reads and the record's fields.
