# Ultra 205: persisted best nonce difficulty

`task-parity-best-difficulty-persistence` replaced the per-connection
pool-difficulty `bestDiff` with upstream's nonce bests:
- an all-time best, restored from NVS `bestdiff` and written on each new record;
- a session best, which starts at 0 on every boot.

One upstream-default soak showed the bests rising while mining. An
identity-bound BAP probe after a reboot then read the restored all-time best.
Independent reviews agree with both runs. No parity row changes: the verified
rows that carry these values cannot be moved out of `verified` automatically.

| Item            | Value                                                              |
| --------------- | ------------------------------------------------------------------ |
| Firmware commit | `8e8a01c3fc2a8b045565dcde8f79c0e1be65e256` (clean, pushed)         |
| App ELF SHA-256 | `4b94b3ce1fce53a7a66a141eb8d672966a2e4385451679dc585e937a420c4289` |
| Package digest  | `97a2a359fcd990fb8a45f12910da7b97616a3850ebfbd560020e80c34ffac7c8` |
| Gate            | `d9023cb176336f74ebea20c2bb6f35319e489a75`                         |
| Reference       | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                         |
| Pool            | `pool_config: local-owner-supplied`; Stratum V1                    |

## Soak 003 attempt 001: the bests while mining (`result.json`: `passed`)

| Observation           | Measured result                                                                        |
| --------------------- | -------------------------------------------------------------------------------------- |
| Install               | State-preserving `flash-monitor`: `observed_exact_package`, `monitor_qualified: true`  |
| Active time           | 607,689 ms, ended by budget; safe-stop at `fan_paused`                                 |
| Soak ledger           | Ordinal 4 charged once; 2,476,200 ms total; next ordinal 5                             |
| Shares                | 27 accepted, 0 rejected, 28 submitted                                                  |
| Windows               | 20/20 credited; maximum gaps 4,781 ms WebSocket and 5,601 ms HTTP; zero reconnects     |
| Work-gate samples     | 5.235–5.278 V, 10.83–12.20 W, 34–52 C, 7,307–7,387 RPM, watchdog alive                 |
| Persistence           | `mineonboot=false`; pool settings retained                                             |
| `bestDiff`            | One boot session; 0 → 795 → 3,239 → 11,136 → 20,641 → 29,059; never decreased          |
| `bestSessionDiff`     | Equal to `bestDiff` in all 1,551 observer samples, since NVS held no earlier best      |

The first record, 795, was below the pool difficulty of 1,000. That confirms
nonces below the pool target count toward the best, as in the reference.

## BAP ordinal 010: the restored best after a reboot (`complete`, `bound`)

BAP ordinal 010 ran after the soak was sealed.

1. A state-preserving reinstall of the same package rebooted the board. NVS
   was kept, and the boot ordinal went from 1 to 2 with reset category
   `other`.
2. The identity-bound probe then ran on that one boot.

The binding held: the uptime advanced 81,080 ms against 80,799 ms of host
time, and the probe record was written between the two observer reads.

The `best_difficulty` subscription answered **`29.06k`**. That is the
reference `suffixString` of the soak's final all-time best, 29,059, restored
from `bestdiff`. Every other step passed with values in the reference format.

## Caveats from the independent reviews

- One submitted share was never answered: 28 submitted, 27 accepted, 0
  rejected. Follow-up: `task-stratum-unanswered-submission`.
- One about 5 s HTTP stall happened with no BAP traffic at all. Soak 002 had
  attributed its paired gaps to BAP; this points instead at the socket write
  held under the publication lock (`task-operator-snapshot-io-outside-lock`).
- The soak judge's `mineonboot` check reads a field that its samples never
  carry, so the check cannot fail. `mineonboot=false` is proven by the
  released final state instead. Follow-up: `task-soak-judge-mineonboot-field`.
- The soak cannot show individual NVS writes, and a value held for under
  about 2 s can fall between samples. Persistence is proven by the
  after-reboot BAP read.
- `bestSessionDiff` = 0 after the reboot is unit-tested only. Reading it live
  would need HTTP, which needs a new Gate session.

## Non-claims

- Stratum V2 nonces carry no nonce difficulty, so they do not count.
- Upstream's screen shows `Best: session/all-time`; this firmware still shows
  one value, the session best.
- JSON byte formatting: this firmware prints `0.0` where upstream cJSON prints
  `0`.
- Other boards.

## Private evidence

| Artifact                      | SHA-256                                                            |
| ----------------------------- | ------------------------------------------------------------------ |
| Soak sealed inventory         | `c0108b20ab0cd882d1446fae5406a5a59f666c041eb7916a8ae21d94b1136e41` |
| Soak result                   | `45ab8594e043df976438416da5643ce7ed973561214eda3b5e1c66cb995266a0` |
| Soak observer journal         | `f57a7e875a17b518bce05def725a97b120f5edcac27014f2b4811bc4558a061d` |
| BAP 010 probe record          | `63bc90ccf1d496c50e407daa60dfc61ff04665649cf0b63240c944dbf05bf076` |
| BAP 010 binding record        | `1786dd7d9b4ff379e54cc27ce81830e1db683de115bba39643ecab84d6ae9201` |

The roots stay uncommitted under ignored mode-0700 directories. The pool
values, endpoints and device identities are not recorded here.
