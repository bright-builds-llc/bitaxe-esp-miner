# Ultra 205: local mining soak 006 (the PSRAM atomics fix)

Soak 006 ran local mining for 4 hours on firmware carrying the ADR-0039
fix: Rust atomics no longer use `S32C1I`, so each is now a call, and one on
PSRAM takes a shared spinlock. It was judged by the hashing-aware rules
and compared with soak 005, which ran before the fix. Repo-owned commands
drove it: `just otawww-endpoint` and `just local-mining-soak --soak 006`
(`task-local-mining-atomics-soak`).

Attempt 001 passed. An independent review recomputed the verdict and the
comparison from the private journals. Parity is not promoted.

| Observation      | Result                                                                   |
| ---------------- | ------------------------------------------------------------------------ |
| Firmware commit  | `78bcc988` (clean package, contains the fix `6f9a77d8`), state-preserving |
| Handoff          | First attempt (`failed_attempts: 0`)                                     |
| Mining time      | 14,402,050 ms of hashing, all one unbroken span                          |
| Outages          | None; Wi-Fi connected on every sample; no status-only sample             |
| Shares           | 424 accepted, 0 rejected                                                 |
| Mining samples   | 5,192.5–5,255 mV, at most 12.6 W, at most 61 C chip, at least 2,433 RPM  |
| Live stops       | None: no reset, device pause, fault or overheat; one boot session        |
| Final stop       | Paused; powered down (0.73 W, then 0.77 W)                               |
| Device           | Same physical device before and after; no serial holder afterwards       |

## Compared with soak 005

The window for each run is its first 4 hours of hashing, from 60 s after
its first hashing sample. Response time is a host-side proxy: each sample
interval minus the 2 s sleep, over pairs of hashing samples at most 10 s
apart, none next to a break-log read.

| Measure                    | Soak 005 (`be2ad02a`) | Soak 006 (fix)    |
| -------------------------- | --------------------- | ----------------- |
| Hashing samples            | 6,889                 | 6,825             |
| Hashrate p5 / p50 / p95    | 365 / 432 / 496 GH/s  | 367 / 432 / 493   |
| Median power               | 12.17 W               | 12.20 W           |
| Response p50 / p95 / p99   | 77 / 104 / 128 ms     | 90 / 152 / 183 ms |
| Intervals over 4 s         | 2                     | 3                 |
| RSSI p5 / p50              | −41 / −38 dBm         | −43 / −40 dBm     |
| Status-only samples, outages | 0, 0                | 0, 0              |

- **Hashrate and power:** no measurable change.
- **Response time:** the proxy rose about 13 ms at the median and about
  50 ms at p95. Its minimum did not move (62 against 63 ms), so the spread
  widened rather than the whole distribution shifting.
  - **Not the visible host load:** Bazel gates and firmware builds ran
    during about the first 80 minutes of soak 006, and not during soak 005.
    But the proxy was no lower after they stopped (p50 92 / p95 154 ms,
    against 86 / 146 during them).
  - **The fix is a plausible cause:** every Rust atomic is now a call, and
    one on PSRAM masks interrupts and takes a spinlock both cores share.
  - **Unproven either way:** the proxy is host-side, and unseen host
    activity cannot be ruled out. The 2 dB weaker RSSI is unlikely to
    matter at about −40 dBm.
  A device-side measurement of request handling would settle it.

## Not shown here

- Whether the response-time difference comes from the fix; it is
  unexplained.
- Mining past 4 hours on this firmware.
