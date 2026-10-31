# Access gate peer address on the Gamma 601 and Ultra 205

`task-http-access-gate-peer-address` fixed the HTTP access gate, which read the
peer of its `PF_INET6` server socket through a `sockaddr_in`. Each board then ran
the fixed production firmware, and a private peer was served.

- **Commands** (the task's hardware contract in `TASKS.md`):
  1. `just detect-ultra205`: exactly one device;
  2. `just flash-monitor --board <board> --port <port>
     --capture-timeout-seconds 360`: production, state-preserving;
  3. `just access-gate-check --monitor-log <capture>`, on that same boot.
- **Requests:** three `GET /api/system/info`, sent to the private origin the device
  reported in that capture: no Origin, the device's own Origin, and
  `http://8.8.8.8` as the Origin.
- **Reference:** `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

| Attempt | Board | Source | `usb_board_identity` | App ELF SHA-256 prefix | No Origin | Private Origin | Public Origin |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 002 | Gamma 601 | `6269d90c` | `built=601 stored=601 identity=match` | `9ddeedb0a9dd` | 200 | 200 | 401 |
| 003 | Ultra 205 | `0c0698db` | `built=205 stored=205 identity=match` | `8434582896e8` | 200 | 200 | 401 |

`0c0698db` adds only a `TASKS.md` record to `6269d90c`, so the two sources hold the
same gate. Both runs printed `passed: true`, and both boards stay on these
production builds.

## What this shows

Outside AP mode, the fixed gate denies a peer that is unreadable, zero or not
IPv4-mapped. AP mode allows everything without reading the peer, so the `401`
matters twice: it shows the Origin check still runs, and that the device was not
in AP mode at that request. With that, a `200` with no Origin can only mean the
gate read a private IPv4-mapped peer.

## What this does not show

- That a gated route other than `GET /api/system/info` behaves this way on
  hardware, including settings and OTA. OTA makes its decision on its own path.

- The bug on a device. Firmware from before the fix answers these three requests
  the same way (attempt 001), and `axeos_access_gate=denied` never leaves the
  device, because the USB diagnostics filter drops it and it is not in the retained
  log. The zero peer is read from source.
- A public peer being denied on hardware, which only the unit tests show. This task
  authorizes no public exposure of a device.
- A real IPv6 peer, or the WebSocket routes on hardware, which read the same
  `access_input_from_raw`.

The captures stay private, with their addresses, under ignored mode-0700 roots.
