# Ultra 205: boot-resume check and the PSRAM atomics panic

The boot-resume check (`task-local-mining-boot-resume`) turns
`startMiningOnBoot` on and restarts the device. Local mining must then come
back with no resume command. It turns the setting off, pauses and restarts
again, and the device must stay paused. Repo-owned commands drove it:
`just otawww-endpoint` and `just local-mining-soak --soak boot-resume-001`.

- **Attempt 001** found a panic in the restart path. The core dump led to a
  root cause and a fix (ADR-0039).
- **Attempt 002** passed on the fix. An independent review recomputed the
  verdict from the private journal. Parity is not promoted.

## Attempt 001: the restart panic (firmware `1dfca479`)

- The baseline was admitted, the pool was saved, and mining on boot was
  turned on and read back.
- The one `POST /api/system/restart` got no HTTP response. The next boot
  reported reset reason `panic`, so the check stopped
  (`boot_resume_reset_category`).
- That boot began local mining on its own 5.9 s after it started: mining
  on boot was honored even after a panic.
- Recovery worked as designed. The check paused at once, turned mining on
  boot off and read it back, and the ASIC powered down (0.6 W).

The core dump was read (`just core-dump-read`, stopped proof first) and
decoded against the preserved ELF, whose identity it matched. It was then
archived privately and cleared.
- **The assertion:** FreeRTOS asserted in `xTaskPriorityDisinherit`. The
  task giving a mutex was not its recorded holder.
- **Where:** the deferred-effect worker was unlocking a one-shot reply
  slot's mutex, the restart's ownership acknowledgement, which comes before
  any HTTP response. The recorded holder was httpd, and both tasks counted
  one held mutex.
- **Why** (inferred; the dump holds task stacks, not the slot): the slot
  was in PSRAM.
  - On the ESP32-S3, `S32C1I` is not atomic on PSRAM. ESP-IDF compiles
    its C with `-mdisable-hardware-atomics` for that reason.
  - rustc inlined `s32c1i` for every Rust atomic, so both tasks' lazy
    `OnceBox` initializations of the slot's mutex "won".
  - Each task locked its own mutex, and the worker unlocked httpd's.
  - No other explanation fits the dump.
- **Since when:** ordinary Rust allocations have been in PSRAM since
  `92f58abb` (2026-10-05). The image had 363 Rust functions with a
  hardware compare-exchange.

## The fix (`6f9a77d8`, ADR-0039)

- Rust for `xtensa-esp32s3-espidf` builds with `-C target-feature=-s32c1i`.
  Every Rust atomic now calls ESP-IDF's PSRAM-aware libcalls, which sit in
  IRAM.
- `//scripts:firmware_psram_atomics_test` (in `just test`) refuses a built
  ELF with a hardware compare-exchange outside an exact native allowlist.
  The crashing image fails with 363 functions; the fix passes with none.

## Attempt 002: passed (firmware `6f9a77d8`)

| Observation        | Result                                                                  |
| ------------------ | ----------------------------------------------------------------------- |
| Install            | `6f9a77d8`, clean package, state-preserving (`observed_exact_package`)  |
| Handoff            | First attempt (`failed_attempts: 0`)                                    |
| First restart      | Answered; new boot `software_cpu` in 10,573 ms (new session)            |
| Boot state         | Unpaused, mining on boot on; no resume command was sent                 |
| Mining start       | 8.3 s after the check first saw the boot (16 s of uptime)               |
| Mining             | 300,412 ms accumulated, no outages, 18 accepted shares, 0 rejected      |
| Mining samples     | 5,196–5,251 mV, at most 12.44 W, at most 60 C chip, at least 2,437 RPM  |
| Disable and pause  | Mining on boot read back off; ASIC down (0.84 W, then 0.91 W)           |
| Second restart     | Answered; new boot `software_cpu` in 12,158 ms (new session)            |
| Hold               | 58 samples, the last 121,837 ms after the boot's first; all paused, not mining, mining on boot off, 0.65–0.99 W |
| Final              | Paused, 0.9 W, mining on boot off                                       |
| Device             | Same physical device before and after; no serial holder after the run   |
| Retained log       | No panic or crash-loop guard line; `wifi_power_save=none`               |

**Cost of the fix**, over nonzero-hashrate mining samples after each run's
first minute:

| Run                   | Samples | Hashrate p5 / median / p95 (GH/s) | Power median |
| --------------------- | ------- | --------------------------------- | ------------ |
| Soak 005 (`be2ad02a`) | 13,782  | 366 / 432 / 497                   | 12.17 W      |
| Attempt 002 (fix)     | 115     | 359 / 437 / 502                   | 12.16 W      |

Five minutes cannot show a small loss. No loss is visible.

The check itself enforced, before journaling each new boot, that it ran the
same firmware with the next boot ordinal. The journal keeps neither field,
so the evidence for them is the check's `rebooted` events with no live
stop.

## Not shown here

- Response times with the fix, and hashrate over hours. The next soak
  measures them.
- That the lease handover panic (`30fc67e2`, 2026-10-18) had the same
  cause. It fits: it was a native panic at an idle status poll, which also
  uses reply slots. But its dump predated the image and could not be
  decoded.
- The crash-loop guard on hardware: software restarts clear its streak,
  and inducing panics is out of scope.
- A restart while mining; both restarts here were requested while paused.
