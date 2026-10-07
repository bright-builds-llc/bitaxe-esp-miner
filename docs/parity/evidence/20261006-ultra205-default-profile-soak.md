# Ultra 205: upstream-default soak verified

Soak attempt 001 under `task-ultra205-default-profile-soak-reverification`
passed. The Ultra 205 mined for 600 active seconds at the upstream-default
profile (485 MHz, 1200 mV, 100% fan). It ran through the browser Gate
(ADR-0033), on Stratum V1 against the owner's pool. The device closed the work
gate by its budget, completed safe-stop and returned to the paused baseline. An
independent review of the private evidence agrees with `passed`, with the
caveats below. Parity is not promoted.

| Observation            | Measured result                                                    |
| ---------------------- | ------------------------------------------------------------------ |
| Firmware commit        | `f2306620fbd0f29668be30ec894929391e38e2fd`                         |
| App ELF SHA-256        | `2aae3863b890e2213e9eabfe6abb727d83b2bdd7506e556253e808420d8c3579` |
| Package manifest       | `09b233d78fb87512f4bf227f9987f1ccca301a8e592631e4d8845ad8018e2acb` |
| Gate                   | `6fd3ff1414b018834030177d642644682e6086d0`                         |
| Reference              | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                         |
| Board / profile        | 205 / signed `upstream-default`                                    |
| Pool                   | `pool_config: local-owner-supplied`; Stratum V1                    |
| Active time at stop    | 607,781 ms (work gate 600,000 ms; budget 619,050 ms)               |
| Soak ledger            | Ordinal 1 charged once; 619,050 ms total; next ordinal 2           |
| Renewals               | 28 of 36 pre-signed                                                |
| Shares                 | 17 accepted, 0 rejected                                            |
| Windows                | 20/20 credited; HTTP 14–15 and WebSocket 48–51 samples per window  |
| Continuity             | Max WebSocket gap 843 ms; max HTTP gap 2,265 ms; zero reconnects   |
| Clock residual         | 79 ms (limit 1,000 ms)                                             |
| Input voltage / power  | 5.26–5.47 V / 0.77–12.76 W                                         |
| ASIC temperature / fan | 36–57 C / 7,346–7,681 RPM                                          |
| Terminal               | Mining stopped on HTTP and WebSocket; paused baseline after        |
| Persistence            | `mineonboot=false`; pool settings retained                         |
| Cleanup                | Server gone, port free, same-device detector, no serial holder     |

The idle WebSocket pre-phase passed over 94,563 ms with 48 HTTP and 164
WebSocket samples, one connection and no broken events. That
is the hardware regression for attempt-004's repeated idle-reconnect boundary
(116 reconnects, 6,455 ms maximum WebSocket gap).

## Caveats from the independent review

- The firmware exposes no frequency, voltage or fan-duty readback. The profile
  is evidenced by the signed grant and its fixed-for-life enforcement, not
  observed directly.
- Mining was observed stopped about 16–17.6 s after the gate closed. That passes
  the recorded 160 s terminal criterion (a deliberate deviation in the task
  contract). It would fail the legacy 10 s serial-marker rule.
- Window clock correlation uses host receipt time.
- The final soak ledger and released state were relayed through the Gate page.
  They were not read independently by the host.
- `authorization_high_water_match` turns false after the soak by design, because
  renewals advance the high-water mark.
- Safety limits are judged on samples taken about once per second. Firmware
  revocation enforces them continuously.
- Artifact integrity is internal consistency: digests and the sealed inventory.

## Seal deviation

The contract's `owner-finish` step failed twice:

- The helper quoted arguments for the positional `ultra205-soak` recipe, which
  produced `soak_action`.
- The 60 s final-detector freshness window was too short for this host's
  Bazel overhead, which was 80–128 s per command.

The root was sealed by running the built `ultra205_soak finish` launcher
directly, immediately after a fresh detector. Both bugs are fixed with
regressions. The earlier stale detector logs stay in the private parent.

Private evidence root: `scratch/ultra205-soak/attempt-001`.

| Artifact         | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| Sealed inventory | `a7cf7de1447ad95e3099cb0167e1e48ef44452fb707670750ef2d28bdee96035` |
| Result           | `705a1148a0dc729675e471d7d9d36a68e6598193684297590fb5ab4d86a0bf41` |
| Gate samples     | `0eee6c2924c6142f5cd690e81e9f65705ac0d18373bfb44c2d7a01246b315c4e` |
| Observer journal | `7bf76628f4c1ddac163611dcecf642700c06d70633408710ed4f33bbef1baa95` |

No TLS, Stratum V2, automatic fan mode, factory reset, erase, OTA, network
discovery, direct UART or pin access, or fault injection occurred.
