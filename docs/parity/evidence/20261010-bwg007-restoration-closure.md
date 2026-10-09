# BWG-007: measured restoration closure on Ultra 205

Attempt 009 of `task-bwg007-real-worker-restoration` passed all eight
restoration scenarios again. This time every published boolean is measured
([ADR-0036](../../adr/0036-measured-bwg-restoration-closure-facts.md)):

- the device identity key is stable across every connection;
- the same key is reacquired after a USB-only disconnect and after a
  both-power reboot;
- the stored pool configuration is unchanged within each boot;
- the attempt evidence contains no credential values or token shapes.

An independent review recomputed every scenario and fact from the private
records and agrees, with the caveats below. Parity is not promoted. It follows
[attempt 008](20261009-bwg007-serial-restoration.md), whose caveats it partly
resolves.

| Identity        | Value                                                                    |
| --------------- | ------------------------------------------------------------------------ |
| Firmware commit | `8e16961bf3553e8b61c27531207fad8bee3bb609`                               |
| Gate            | `be55781d73d9ea190a8fc5c83ec69dac9557d83d`                               |
| Reference       | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                               |
| Pool            | `pool_config: local-owner-supplied`; Stratum V1                          |
| Projections     | profile `bwg-worker-restoration-result/0.3`, `bwg007-attempt-009-*.json` |

| Measurement                      | Result                                                                                                  |
| -------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Scenarios                        | 8/8 passed: completion, pause, cancel, expiry, monotonic reset, USB disconnect, reboot, N1–N4           |
| Device identity                  | epoch 1 in all 74 page states; 54 possession-bound observations in one page lifetime                    |
| Same key after USB disconnect    | observations 30 → 33, epoch 1                                                                           |
| Same key after both-power reboot | observations 37 → 40, epoch 1; device reported `reboot`                                                 |
| Pool configuration               | all 54 status reports were `worker-preservation-v2` with `pool_configuration_unchanged_since_boot` true |
| Credential scan at seal          | 15 files, 0 hits; the reviewer's rescan of the whole run directory found 38 files, 0 hits               |
| Caps                             | 9 of 10 Starts, 2 of 2 renewals, 0 re-arms; settle gate idle or complete before every signing           |
| Authorization negatives          | N1–N4 rejected with the expected attributions; ordinals 1–4; no replay accepted                         |

## Caveats from the independent review

- Pool continuity is proven per boot. The reboot scenario's last few seconds
  before power loss, and equality of the two boots' captures, are not
  observed. "Unchanged" also cannot exclude a write of identical values.
- The credential scan proves only that the listed values and shapes are
  absent. The pool password is shorter than the 8-character scan minimum, and
  finish cannot prove it received the same pool file serve used.
- The identity tracker is page-local. The host trusts the pinned Gate code,
  whose page and bundle digests preflight verifies. It proves one key; the
  watcher's USB identity covers the physical device.
- `reboot` proves a new boot, not that barrel power was removed. N4's
  `control_failed` stop is established by code, not observed. Live safety
  limits and pool shares are not judged.
- The seal is a local digest inventory, not a signature.

Private root: `scratch/bwg-restoration/run-20261010/attempt-009`.
