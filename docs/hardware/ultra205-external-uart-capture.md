# Ultra 205 receive-only external UART capture

This procedure gives Phase 28.1.1 an observation channel that remains open while
the Ultra 205 loses both barrel power and native USB. It is an evidence fixture,
not a power, flashing, reset, or command connection.

## Required fixture

- A macOS-compatible USB-UART adapter whose signal level is **3.3 V**.
- Two mechanically secure, insulated, reversible probes.
- An Ultra 205 whose layout matches the official board-205 source.

With barrel power and native USB disconnected, make only these connections:

| Adapter | Ultra 205 | Alternative |
| --- | --- | --- |
| `RX` | `TP18` / `P_TX` | Tag-Connect `J5` pin 3 |
| `GND` | `TP12` / `GND` | Tag-Connect `J5` pin 4 |

Leave adapter `TX`, `VCC`, `RTS`, `DTR`, and every other signal disconnected.
Never attach or reposition a probe while either board power path is present.
Stop if the silkscreen, test-point placement, or board revision does not match
the official Ultra 205 layout. Do not guess an alternate pad.

After insulating and strain-relieving both probes, connect only the adapter USB
to the host. The board remains unpowered until the repo-owned qualification
prints its restore action. During removal checkpoints, disconnect barrel power
and native USB but leave the adapter USB and both probes connected.

## Firmware and reader contract

The firmware console is explicitly UART0 primary at 115200 baud with USB
Serial/JTAG as its secondary output. The repo-owned `uart-native` reader opens
the adapter `O_RDONLY | O_NOCTTY | O_NONBLOCK`, applies 115200 8N1 local
receive-only framing with no flow control, and performs bounded reads. It never
writes a serial byte or manipulates reset, modem, RTS, or DTR lines.

The external UART node and native USB node are distinct inputs. Pass both
explicitly to the qualification command; never let the workflow scan for an
adapter. Raw node names, identities, processes, and UART bytes stay in the
private ignored trace root.

## Safety stop conditions

Stop without running qualification when any of these is true:

- the adapter voltage is unknown or not 3.3 V;
- the adapter or probes can contact adjacent pads;
- adapter `TX` or any power/control line is connected;
- the board layout does not match the official Ultra 205 source;
- the board appears powered after barrel and native USB are removed; or
- either explicit serial node is missing, ambiguous, unexpectedly held, or
  changes identity outside its allowed lifecycle transition.

The supported wiring is derived from the official Ultra 205 schematic: UART0
TX/GPIO43 is `P_TX` at `TP18` and `J5` pin 3; ground is at `TP12` and `J5` pin
4\. ESP32-S3 UART0 uses 115200 baud for the configured firmware console.
