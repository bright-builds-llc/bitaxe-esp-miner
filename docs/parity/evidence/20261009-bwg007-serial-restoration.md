# BWG-007: serial restoration campaign passed on Ultra 205

Attempt 008 of `task-bwg007-real-worker-restoration` passed all eight
restoration scenarios on one detector-admitted Ultra 205, through the
production Web Serial Gate and the owner's Stratum V1 pool
([ADR-0035](../../adr/0035-serial-bwg-restoration-campaign.md)). An independent
review recomputed every scenario from the private records and agrees, with the
caveats below. Parity is not promoted.

| Identity         | Value                                                              |
| ---------------- | ------------------------------------------------------------------ |
| Firmware commit  | `ffef193eddb19df1613acff95d8a40cebd7ef6b7`                         |
| App ELF SHA-256  | `941922fb85cd7c5bef0aa415da52d605cb3c863040de4b0d5c95b11c11d5a8ed` |
| Package manifest | `8b2f9808a36c55819a92f3f7385c57c3b7f589ae776647bce53196eb1bfe91c1` |
| Gate             | `8461ef026c80b09fc29829c155f64a6ebe52364f`                         |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                         |
| Pool             | `pool_config: local-owner-supplied`; Stratum V1                    |

| Scenario                | Terminal reason       | Key observations                                                                                                |
| ----------------------- | --------------------- | --------------------------------------------------------------------------------------------------------------- |
| completion              | `challenge_satisfied` | renewal accepted 22.2 s after start, then restore                                                               |
| pause                   | `paused`              | operator stop after the lease started                                                                           |
| cancel                  | `cancelled`           | operator stop after the lease started                                                                           |
| expiry                  | `lease_expired`       | device-ended 36.9 s after the loaded lease (minimum 27 s); no renewal or stop                                   |
| monotonic_uncertainty   | `monotonic_reset`     | stimulus acknowledged 5.0 s into the lease; device stop 6.8 s later; nothing else in the segment; counter 0 → 1 |
| disconnect              | `connectivity_lost`   | USB removal 5.4 s after the instruction; 13.7 s absence; same physical identity, new enumeration; same boot     |
| reboot                  | `reboot`              | both-power removal 3.9 s after the instruction; 22.2 s absence; new boot; stimulus reset; no high-water advance |
| authorization_negatives | `connectivity_lost`   | N1–N4 rejected and attributed (table below); 4 rejections, 0 accepted replays                                   |

| Leg | Replay             | Wire category           | Attribution (signature / context / replay guard)  | High-water advanced |
| --- | ------------------ | ----------------------- | ------------------------------------------------- | ------------------- |
| N1  | pre-reboot Start   | `authentication_failed` | valid / mismatch / at or below durable high-water | no                  |
| N2  | expired Start      | `admission_required`    | not evaluated / expired / not evaluated           | no                  |
| N3  | Start, new context | `authentication_failed` | valid / mismatch / fresh                          | no                  |
| N4  | accepted renewal   | `authentication_failed` | valid / current / at or below durable high-water  | yes                 |

N1's high-water fingerprint was unchanged across the reboot. Caps were 9 of 10
Starts, 2 of 2 renewals and 0 re-arms. The settle gate saw the device idle or
complete before every signing. Neither widened timing allowance was needed.
Private records contained no secret-like values.

## Caveats from the independent review

- `connectivity_lost` is a weak terminal fact for disconnect and N4. Every
  disconnect re-confirms that reason, so N4's `control_failed` stop is
  established by code, not observed on hardware.
- `reboot` proves a new boot, not a power-on reset. That both power sources
  were removed rests on the owner's action and the watcher seeing the device
  absent.
- Same-device evidence is strong but indirect: one detector identity, the
  identity-filtered watcher, per-boot stimulus continuity and high-water
  continuity. `sameDeviceAcrossScenarios` and `cleanupConfirmed` are asserted by
  the host projection, not measured. ADR-0019's per-scenario device-key
  fingerprint match is not implemented.
- Credential absence and same-key reacquisition have no dedicated judged fact.
  Live safety limits are firmware-enforced, not judged (a recorded non-claim).
  Pool work and shares are not judged.
- The settle gate can read a retained admission stage. Every settled reading
  here followed the previous lease's end.
- The seal covers the attempt root, not the parent's detector, install and
  finish logs.

## History

Attempts 001–007 each stopped on a verified fix or a recorded human-step or
precondition issue; see the task record. Fixes made along the way:

- the expiry anchor;
- the watcher tolerating a mid-probe removal;
- the N1 reboot-report carry;
- the fresh-boot precondition;
- the admission diagnostic and settle gate;
- wider latency allowances;
- the negatives' terminal reason.

Projections: [`bwg-worker-restoration/`](bwg-worker-restoration/), files
`bwg007-attempt-008-*.json`.
Private root: `scratch/bwg-restoration/run-20261009d/attempt-008`.
