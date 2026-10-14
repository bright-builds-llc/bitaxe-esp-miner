# Ultra 205 BAP connection over a USB-to-UART adapter

Connection guidance for `task-parity-bap-live-accessory-verification`, given under the
owner's explicit direct-UART authorization of 2026-10-11. This page authorizes no
UART traffic, settings change or other hardware effect; those need their own task
contract and owner authorization.

## The BAP header: J6

The Ultra 205's accessory port is **J6**, an unpopulated 1x6, 2.54 mm through-hole
header at the top edge of the board's back side. Pin 1 is the **square** pad.

| J6 pin | Net | Use for BAP |
| --- | --- | --- |
| 1 (square pad) | 5V | **Do not connect.** Board supply output. |
| 2 | GND | Adapter GND |
| 3 | GPIO39 (MTCK) | Board TX: to the adapter's **RX** |
| 4 | GPIO40 (MTDO) | Board RX: to the adapter's **TX** |
| 5 | GPIO41 (MTDI) | Not used |
| 6 | GPIO42 (MTMS) | Not used |

Provenance, from two independent sources in `bitaxeorg/bitaxeUltra` branch
`ultra-205` (branch head `842b42676ef54ceba5bc92dcc2b0ff66b02e0fb5`):

- schematic `esp32.kicad_sch` (file commit
  `034a8a8533193f9c71abb8504178f13483c0947b`): J6 is `Connector_Generic:Conn_01x06`;
  pin 1 is on the `5V` power symbol, pin 2 on `GND`, and pins 3-6 wire directly to
  ESP32-S3-WROOM-1 pins 32-35;
- layout `bitaxeUltra.kicad_pcb` (blob `4e8dcdef5ee1b43cecca401c872112667c790a6d`):
  J6 pad 1 is the rectangular pad on net `/5V`, pad 2 is on `GND`, and pads 3, 4, 5
  and 6 share their nets only with module U12 pads 32 (`MTCK/GPIO39`), 33
  (`MTDO/GPIO40`), 34 (`MTDI/GPIO41`) and 35 (`MTMS/GPIO42`).

The pinned reference firmware
(`reference/esp-miner/main/Kconfig.projbuild`, `main/bap/bap_uart.c`) and this
repository's firmware (`firmware/bitaxe/src/bap_adapter.rs`) both run BAP on UART2
with TX on GPIO39 and RX on GPIO40 at 115200 baud, 8-N-1, no flow control.

## Electrical rules

- The ESP32-S3 GPIOs are **3.3 V only**. Use an adapter set to 3.3 V logic; a 5 V
  logic adapter can damage GPIO40. If the adapter has a voltage jumper or switch,
  set it to 3.3 V and confirm it before wiring.
- Connect only **GND, RX and TX**. Never connect the adapter's VCC, 3V3 or 5V pin
  to the board, and never connect anything to J6 pin 1.
- Cross the data lines: board TX (pin 3) to adapter RX; board RX (pin 4) to adapter TX.
- Wire with the board fully unpowered: barrel power and USB both unplugged.
- The board keeps its normal power: the 5 V barrel jack, plus its USB-C link to the
  Mac for the repository's usual control path. The adapter is a second USB device
  on the same Mac, so both share the Mac's ground.

## What a meter shows on a powered, idle board

Measured against pin 2 (GND): pin 1 reads about 5 V. Pin 3 reads about 3.3 V,
because a UART transmit line idles high; that is the board's BAP TX at rest, not a
supply. Pin 4, the board's receive input, may also read near 3.3 V through the
UART's internal pull-up. Pins 5 and 6 are unused by the firmware, and what they
read is not meaningful.

## Making the connection

J6 ships without pins. Either solder a 1x6 2.54 mm male header into J6, or use
test-hook or spring-pin connectors that hold firmly in the plated holes. Loose
jumper pins pushed into the holes are unreliable and can short neighbours.

Use three female-to-female jumper wires from the header to the adapter: GND, and
the two crossed data lines. Check each wire against the table, and confirm pin 1 by
its square pad, before any power is applied.

### Plug-in adapters: keep their power pins out of J6

Common 5-pin CP2102 adapters have their pins in the order 3V3, TXD, RXD, GND, +5V.
Plugged straight into J6 with +5V on the square pin 1, the data lines happen to
land correctly (GND on 2, RXD on 3, TXD on 4), but two power pins land where they
must not:

- adapter **+5V** on J6 pin 1 ties the board's 5 V rail to the adapter's USB 5 V.
  The adapter's LED lights from the board alone; plugged into a Mac as well, the
  two supplies would be joined and could drive current into each other;
- adapter **3V3** on J6 pin 5 holds GPIO41 at the adapter's 3.3 V output, which
  would short if that pin were ever driven low.

Use jumper wires for GND, RXD and TXD only. If the adapter must plug in directly,
its +5V and 3V3 pins must not enter or touch J6: remove them from the header, or
cover them, rather than relying on a bend.

### Power-on order

An adapter without USB power is fed by the board through its RXD wire: the
board's idle-high TX line leaks through the adapter's input protection into its
supply, enough to light its LED. That loads the ESP32's TX pin and is not a state
to leave connected. So:

1. Plug the adapter into the Mac first; its LED lights from USB and the Mac lists
   it (for a CP2102, Silicon Labs `0x10c4:0xea60`, as `/dev/cu.usbserial-*`).
2. With the board powered as usual, connect the jumpers to the adapter: GND
   first, then RXD, then TXD.
3. To finish, disconnect the jumpers before unplugging the adapter from the Mac.

If the adapter's LED goes dark when plugged in and the Mac does not list it, the
USB port or dongle is not powering it; try another port.

## macOS host

- Before plugging the adapter in, list serial devices: `ls /dev/cu.*`.
- Plug the adapter in and list again; the new node is the adapter:
  `/dev/cu.usbserial-*` (FTDI, or CP210x with Apple's built-in driver),
  `/dev/cu.SLAB_USBtoUART` (CP210x with Silicon Labs' driver) or
  `/dev/cu.wchusbserial-*` (CH340).
- The board's own USB node stays separate. Do not confuse the two: the repository's
  detector and flash tools own the board's USB node, and the BAP client owns only
  the adapter's node.

## Host client

`just bap --port /dev/cu.<adapter> <command>` runs the repository's BAP client
(`tools/bap-client`), which frames with the firmware's own BAP core:

- `request <parameter>`: prints each value the board returns;
- `subscribe <parameter> [--interval-ms N] [--listen-seconds S]`: prints updates,
  then always unsubscribes;
- `unsubscribe <parameter>`;
- `set <parameter> (--value V | --value-file PATH) --confirm <parameter>`.

It opens only a `/dev/cu.*` adapter node and refuses the board's own
`/dev/cu.usbmodem*` node. Each inbound frame is one JSON line, then one outcome
line; the exit status is 0 only for a confirmed or answered operation. Values of
`ssid`, `password`, `wifi_ssid`, `wifi_password`, `wifi_ip`, `pool`, `poolPort`
and `poolUser` are never printed. `ssid` and `password` are accepted only from a
private file (mode 0600, owned by you, not a symlink), never on the command line.
Changing `frequency` or `asic_voltage` also needs `--allow-asic-setting`. An
adapter that disappears is reported as an error, not as a quiet line.

`probe` runs the task contract's read-only sequence (`systemInfo`, `shares`,
then a short `hashrate` subscription) once. A request is answered only for
`systemInfo`, `shares`, `block_height`, `found_block` and `show_new_block`;
`hashrate` arrives only through a subscription. It needs a mode-0700 parent holding
the detector's saved report as a mode-0600 sibling, creates the attempt directory
itself, refuses one that already exists, stops at the first failed step and
seals one category-only record read-only. Its exact use is in the task contract.

## Not yet authorized

Opening the adapter, sending BAP frames, subscriptions or settings changes to the
board, powering it for this purpose, and recording evidence each need the task's
hardware contract and the owner's authorization for that step.
