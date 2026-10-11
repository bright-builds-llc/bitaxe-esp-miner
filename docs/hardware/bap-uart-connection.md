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

Provenance: `esp32.kicad_sch` on `bitaxeorg/bitaxeUltra` branch `ultra-205`
(file commit `034a8a8533193f9c71abb8504178f13483c0947b`; J6 is
`Connector_Generic:Conn_01x06`, footprint `PinHeader_1x06_P2.54mm_Vertical`; pins 3-6
wire directly to ESP32-S3-WROOM-1 pins 32-35). The pinned reference firmware
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

## Making the connection

J6 ships without pins. Either solder a 1x6 2.54 mm male header into J6, or use
test-hook or spring-pin connectors that hold firmly in the plated holes. Loose
jumper pins pushed into the holes are unreliable and can short neighbours.

Use three female-to-female jumper wires from the header to the adapter: GND, and
the two crossed data lines. Check each wire against the table, and confirm pin 1 by
its square pad, before any power is applied.

## macOS host

- Before plugging the adapter in, list serial devices: `ls /dev/cu.*`.
- Plug the adapter in and list again; the new `/dev/cu.usbserial-*` (FTDI or
  CP210x) or `/dev/cu.wchusbserial-*` (CH340) node is the adapter.
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

## Not yet authorized

Opening the adapter, sending BAP frames, subscriptions or settings changes to the
board, powering it for this purpose, and recording evidence each need the task's
hardware contract and the owner's authorization for that step.
