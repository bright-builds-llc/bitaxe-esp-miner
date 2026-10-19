# Ultra 205: handover loop and Wi-Fi power save

Two hardware runs followed up the lease interruption check
(`task-local-mining-handover-panic-and-outages`):
- a lease-free handover loop, to reproduce the native panic seen once
  during a lease handover;
- soak 004, to explain the outages that had interrupted local mining about
  every 1.5 hours.

The loop passed without reproducing the panic. Soak 004 caught one outage
in the device's own retained log: a Wi-Fi `beacon_timeout` disconnect,
with modem power save on, which upstream turns off. That makes power save
the leading hypothesis for the outages. With power save off, a repeat
2-hour soak ran without a single break. Parity is not promoted. The panic
stays an open defect.

## Handover loop (`--soak handover-001`, attempt 001)

| Observation   | Result                                                                    |
| ------------- | ------------------------------------------------------------------------- |
| Firmware      | `83337e33` (clean package), installed state-preserving                    |
| Cycles        | 20 lease-free handovers in 38 minutes, one boot session                   |
| Each cycle    | Connect refused at `hello`; ASIC down; admitted Connect idle 48 s; closed |
| Breaks        | 87–94 s each; local mining back after every one                           |
| Break logs    | All 20 show the handover markers; no Wi-Fi reconnect line                 |
| Stops         | None: no reset, device pause, fault or overheat                           |
| Gate          | No grant signed; the page server sealed `unverified` as expected          |
| Device        | Same physical device before and after; `lsof` found no serial holder      |

With the lease check's attempts 003 and 004, the panic has not recurred in
22 admitted handovers since it occurred. The loop is weak evidence for that
trigger, though. The panic came with the admitted page idle and no page
operation, while the loop's admitted pages ran status polls. Only lease
check attempt 003's 90 s idle hold matched it. The core-dump partition was
cleared beforehand and will keep the next panic's dump.

## The outage soak 004 caught (attempt 001)

Attempt 001 ran on the same firmware and stopped after 43 minutes. An
outage passed the 60 s bound (61,333 ms), and the run paused the device,
which powered down with no reset. The retained log, read at once, showed
this sequence:
1. `wifi_reconnect=disconnected reason=beacon_timeout`;
2. reconnected about 10 s later;
3. the local run ended as the pool session dropped
   (`ending=Some(Requested)`);
4. a new run began, and was not yet mining when the bound expired.

The earlier outages have the same shape: the device unreachable for about
20 s, then a new pool session. Their cause was never logged, so this one
`beacon_timeout` is the only direct observation. The attempt left no final
named-port detector; attempt 002's detectors show the same device.

Upstream `components/connect/connect.c` calls
`esp_wifi_set_ps(WIFI_PS_NONE)` in both of its branches ("Disable power
savings for best performance"). This firmware never did, so it ran
ESP-IDF's station default, modem sleep. The fix (`44309653`) turns power
save off after each driver start and before any association. On hardware,
the retained line `wifi_power_save=none` (seen in the private log)
confirms it.

## Soak 004 on the fix (attempt 002)

| Observation    | Result                                                                    |
| -------------- | ------------------------------------------------------------------------- |
| Firmware       | `9408968b` (clean package), installed state-preserving                    |
| Power save     | `wifi_power_save=none` in the retained log after boot                     |
| Mining time    | 7,201,760 ms, unbroken: no outage, maximum sample gap 7,824 ms            |
| Shares         | 282 accepted, 0 rejected                                                  |
| Wi-Fi          | Connected on every sample; RSSI −50 to −31 dBm; no disconnected sample    |
| Limits         | At most 12.69 W and 61 C; input 5,204–5,266 mV; readings fresh            |
| Stop           | Paused; powered down 27 s and 29 s after (0.83 W); one boot session       |
| Device         | Same physical device before and after; `lsof` found no serial holder      |

The retained log read at the end covers only about the last 40 minutes,
and it holds no `beacon_timeout`. The rest of the 2 hours shows no Wi-Fi
drop indirectly: no disconnected sample and no sample gap over 7.8 s.

## How strong this is

Before the fix, local mining met six outages in about 9.1 hours of local
runs (soak 002 attempts 001 and 002, soak 003, lease check attempt 004 and
soak 004 attempt 001): about one every 1.5 hours. At that rate, a 2-hour run
with no outage happens by chance about a quarter of the time. Attempt 002
is consistent with the fix, but does not prove it. A longer soak on the
fixed firmware, for example 8 hours, would test it properly.

## Not shown here

- The cause of the lease handover panic, or that it cannot recur.
- That power save caused every earlier outage, that beacon timeouts cannot
  recur with power save off, or any behavior past 2 hours on the fixed
  firmware.
- Why the access point's beacons were missed (the access point itself, or
  radio conditions).
