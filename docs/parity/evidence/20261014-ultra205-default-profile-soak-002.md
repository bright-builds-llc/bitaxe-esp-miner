# Ultra 205: upstream-default soak 002 passed after a revocation-clock fix

Soak attempt 002 under `task-ultra205-default-profile-soak-002` passed. The Ultra
205 mined for 600 active seconds at the upstream-default profile (485 MHz,
1200 mV, 100% fan). It ran through the browser Gate (ADR-0033), on Stratum V1
against the owner's pool. The device closed the work gate by its budget,
completed safe-stop and returned to the paused baseline. An independent review
of the private evidence agrees with `passed`, with the caveats below. Parity is
not promoted.

| Observation            | Measured result                                                       |
| ---------------------- | --------------------------------------------------------------------- |
| Firmware commit        | `a46f95684bb940c0773c9fe2168a72f573d77f2d`                            |
| App ELF SHA-256        | `058bfaf2c96fe526f6b9d11f3968c696027fca6d78205992eacf90e284409289`    |
| Package manifest       | `9a2ee80221a2a04f30f09bcabfce01f64709df973184a9ea0bb730dbd630efd3`    |
| Gate                   | `d9023cb176336f74ebea20c2bb6f35319e489a75`                            |
| Reference              | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                            |
| Board / profile        | 205 / signed `upstream-default`                                       |
| Pool                   | `pool_config: local-owner-supplied`; Stratum V1                       |
| Install                | State-preserving `flash-monitor`: `observed_exact_package`            |
| Active time at stop    | 607,680 ms (work gate 600,000 ms; budget 619,050 ms)                  |
| Soak ledger            | Ordinal 3 charged once; 1,857,150 ms total; next ordinal 4            |
| Renewals               | 28 of 36 pre-signed                                                   |
| Shares                 | 22 accepted, 0 rejected; 22 nonce-work correlations; 308 dispatched   |
| Windows                | 20/20 credited; HTTP 14–16 and WebSocket 46–54 samples per window     |
| Continuity             | Max WebSocket gap 3,551 ms; max HTTP gap 4,632 ms; zero reconnects    |
| Clock residual         | 157 ms (limit 1,000 ms)                                               |
| Input voltage / power  | Work gate open: 5.24–5.29 V / 11.02–12.40 W                           |
| ASIC temperature / fan | Work gate open: 35–54 C / 7,267–7,346 RPM                             |
| Terminal               | Mining stopped on HTTP and WebSocket 12.2–13.5 s after the gate closed |
| Safe-stop              | `fan_paused` about 13 s after the gate closed                         |
| Persistence            | `mineonboot=false`; pool settings retained                            |
| Cleanup                | Server gone, port free, same-device detector, no serial holder        |

The idle WebSocket pre-phase passed over 80,476 ms with 41 HTTP and 139
WebSocket samples, one connection and no broken events. The cooling proof
reached 5,901 RPM at 100% duty before the fan was restored to 30%.

## Attempt 001 and the fix

Attempt 001, on firmware `ecc7f122`, was revoked the instant mining activated:
`worker_revocation_detail reason=unsafe_observation trigger=no_safe_sample
fact=none since_safe_ms=4294967294`. No work was dispatched. Soak ordinal 2 was
charged, as designed, and sealed `unverified`.

`since_safe_ms` is 2^32 − 2. A safety sample was stamped 2 ms after
`GenerationGate::check_deadline` read its clock, and the unsigned
`now.wrapping_sub(last_safety_ms) > 1_000` saw that fresh sample as about 49.7
days old. The heartbeat cutoff had the same flaw.

Commit `a46f9568` treats a stamp newer than the caller's clock reading as zero
elapsed. Two regressions, which fail on the old code, cover the safety sample
and the heartbeat. Attempt 002 did not recur that signature.

## Caveats from the independent review

- The largest HTTP gap (4.6 s) and WebSocket gap (3.6 s) fell together about
  56 s into work, inside a read-only BAP probe run over the separate UART2
  adapter (BAP ordinal 007). The timing suggests BAP traffic caused them but
  does not prove it. Both are within the limits, but larger than in the
  2026-10-06 soak.
- Mining was observed stopped 12.2–13.5 s after the gate closed. That passes
  the recorded 160 s terminal criterion and would fail the legacy 10 s
  serial-marker rule.
- The firmware exposes no frequency, voltage or fan-duty readback. The profile
  is evidenced by the signed grant, not observed directly.
- Window clock correlation uses host receipt time. The final soak ledger and
  released state were relayed through the Gate page.
- `authorization_high_water_match` turns false after the soak by design.
- Safety limits are judged on samples taken about once per second. Firmware
  revocation enforces them continuously.

## Seal deviation

`just hardware-operator owner-finish` stopped the attempt-001 server but its
own detector, which names no port, refused `ambiguous_serial_ports`, because
the BAP adapter is a second serial port. Both attempts were sealed with the
README's alternative path: a fresh detector naming the board's port, then
`just ultra205-soak finish`.

Private evidence root: `scratch/ultra205-soak-002-attempt-002/attempt-002`.

| Artifact         | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| Sealed inventory | `61e95ef9e0515b46d54da471a56b8a37fee7f5b68f701eb158ccd00f4675f10b` |
| Result           | `9d73ed8cc966ce048de69247d180ef64d6e0d732144801eb6d0d8ac751d01c12` |
| Gate samples     | `dc77d86f11151d86274ad5842d5482e742f0f1d6ae232187ec0dfd94d65154b5` |
| Observer journal | `85d8f3a72e81f45ad2babe7354b2e3807b37bb14829e0f1c124ebb15d9c66797` |

No TLS, Stratum V2, automatic fan mode, factory reset, erase, OTA, network
discovery, direct pin access or fault injection occurred. The only UART2
traffic was the read-only BAP probe described above.
