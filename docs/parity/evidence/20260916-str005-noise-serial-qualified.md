# Ultra 205 Serial Noise qualification

## Accepted result

Serial Noise attempt 1 completed under the frozen
[base contract](../../hardware/str005-noise-serial-qualification.md) and
[v2 amendment](../../hardware/str005-noise-parity-scope-amendment.md). The
[closed projection](str005-noise-serial/attempt-001.json) records one
configured-authority authenticated exchange, independent fixture decryption,
preservation, unchanged accounting, fresh restoration and complete cleanup. The
standalone read-only reviewer reproduced the sealed complete result.

| Identity         | Value                                                              |
| ---------------- | ------------------------------------------------------------------ |
| Board            | Ultra 205 / BM1366                                                 |
| Firmware         | `ad629679de9dcae33fdc1836372f73d20f4e0070`                         |
| Gate             | `5dc5ed6f39834cab0144af0a05f6c2535a39d827`                         |
| Application ELF  | `1427aa29d9b563041fe8b9833b178df833cb724264b3719df2d0ce71570c5fef` |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                         |
| Private result   | `9fe8f1ed6755e2c9a8573fa6dc5ae867fca044f6a79636cc584415c60744c40a` |
| Sealed inventory | `cc8e278543447e026b908b2a0e9e5b626ff1f7d47ec245060325ccf91d49f4d2` |

The projection additionally binds the package manifest, combined contract,
fixture and complete evaluator inventory. Private operational artifacts remain
protected; this report contains no device identifiers, endpoints or secrets.

## Observations

- The previously installed firmware supplied a fresh before-update baseline and
  accounting through the new Gate. One state-preserving candidate installation
  and four further update/reconnect cycles completed. Each cycle proved exact
  candidate identity, preserved Device Identity/settings and a fresh 65,536-byte
  request and response.
- The browser retained its original private baseline throughout. Each connection
  used native foreground permission and fresh authenticated Web Serial
  possession.
- Exactly one correlated fixture connection received the diagnostic's own
  64-byte act one. The configured authority authenticated successfully, and the
  fixture independently decrypted the exact 22-byte encrypted diagnostic frame.
  There were no unexpected peer connections.
- Device-local diagnostic completion took 1,368 ms; preparation took 67 ms.
  Actual socket/job cleanup took 10 ms. The fixture's host-observed lifetime was
  1,854 ms, and host cleanup took 7 ms. These are separate clock-domain
  observations, not cross-clock latency calculations.
- Fresh same-pair reconnection collected the retained terminal result and
  confirmed normal restoration. The candidate remains installed, mine-on-boot is
  false, leases are inactive, and browser, serial, fixture and supervisor
  resources were released. No rollback, factory reset or recovery-006 fallback
  occurred.
- Both ledgers remained unchanged: next qualification ordinal 18, last completed
  17, total charged 1,560,000 ms and no pending reservation. The original
  240,000-ms campaign remains exhausted. Fresh same-boot counter comparisons
  proved no new work or shares; historical cumulative counters were not required
  to be zero.

The clean native package repeated the
[software resource checks](../../hardware/str005-noise-v2-readiness.md) before
admission and acceptance. Its app occupied 4,167,616 bytes. The selected Noise
path, including measured thread entry frames, used 11,648 of 12,288 stack bytes,
leaving 640 against the prospective 512-byte margin. The worker is borrowed from
the existing idle pool transport; no new task stack or dependency fork was used.

## Scope and remaining work

This closes the current Serial Noise hardware obligation of
`task-str005-noise-auth-205`. Its failed historical diagnostic-001 and unchecked
recovery-006 execution items retain their original outcomes; the v2 successor
supersedes that restoration mechanism.

This run did not open a mining channel, receive jobs, drive ASIC work, submit
shares or consume a mining allowance. It does not repeat the September live
mining/heartbeat-loss campaigns on this pair. Channel/job and real V2 share
qualification remain separate tasks with their own prospective contracts.

The measured positive exchange does not prove universal in-call cancellation or
bounded cleanup after every opaque crypto stall. Selected native frame checks
are not a complete callgraph bound. Software negative tests are not hardware
negative evidence; negotiated-key possession is not a static device certificate;
fixture certificate acceptance does not establish synchronized device wall time.

Parity remains **90/95 active rows verified**. STR-005 promotion still requires
the independent channel/job, share and final evidence-composition obligations.
