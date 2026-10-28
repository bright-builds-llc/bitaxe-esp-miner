# Gamma 601: the self-test on production

Phase F4.5 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 15) repeated SELF-001's two-phase lifecycle on the promoted production
601 build. It ran under F4.3's immutable plan,
`docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`, whose
objective names this production repeat. Attempt 009 completed. Its validated
projection is
`docs/parity/evidence/self001-gamma601/attempt-009/self-test-projection.json`.

- **Commands** (the F4.5 contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package --board 601`;
  2. `just self-test-campaign start --board 601` with:
     - the private root `scratch/self001-gamma601/attempt-009`;
     - the package's own manifest in `bazel-bin`;
     - the owner's Wi-Fi and pool credentials, by path only;
     - the detector output, the plan and the projection path;
  3. the owner's two-second BOOT press at `cancel_ready`;
  4. `just self-test-campaign resume` with the same options;
  5. `just monitor --board 601 --port <port> --capture-timeout-seconds 360`
     to confirm production. No reinstall was needed: the campaign's
     installs were already the production package at the attempt's commit.
- **Sources:** `bb9f2ae7`, the attempt's commit; the reference at
  `c1915b0`.
- **Package:** `firmware_variant=production`, board 601, source
  `bb9f2ae7`. The manifest SHA-256 and the application ELF SHA-256 are in
  the projection.
- **Pool:** `pool_config: local-owner-supplied`.
- **Wi-Fi:** the credentials were applied by the owner's request to go
  ahead with F4.5, whose approved plan runs this lifecycle.

## The planned failure

- **Start:** the lease tuple started the self-test in place of the mining
  owner and the Worker. It took the local generation
  (`self_test_authority=granted generation=1 bound_ms=240000`).
- **Run:** it went through preflight, preparing, warming and a 5 s measured
  load, then failed `planned_evaluation_failure` as planned.
- **Stop:** it stopped safely and held `cancel_ready` with
  `safe_state:true`.
- **Cancel:** the owner's BOOT press cancelled it. The projection records
  the long press, the cancellation receipt and the restart.

## The pass

| Metric | Value | Limit |
| --- | --- | --- |
| Profile | 525 MHz, 1150 mV set; 1145 mV read | core 1035 to 1265 mV |
| Measured window | 30.070 s at difficulty 16 | 30 s |
| Total hashrate (nonces) | 1028.4 GH/s (525 MHz × 2040 small cores ≈ 1071) | at least 910 GH/s (85 %) |
| Domain hashrates | 260.3, 271.8, 272.1, 266.3 GH/s | 227.6 GH/s ± 33 % (about 152 to 303) |
| Domain samples / rejected | 28, 28, 28, 19 / 0, 0, 0, 0 | |
| Power | 18.8 W | at most 22 W |
| Input voltage | 5.133 V (the TPS546's VIN) | 4.5 to 5.5 V |
| Chip temperature | at most 65.62 C | at most 70 C |
| Regulator | at most 54.75 C, no fault | at most 105 C |
| Fan | 6171 rpm | above 1000 rpm |

After the measurement:
- the pass receipt and the terminal record (`outcome:passed`,
  `safe_stop:true`, `restart_delay_ms:10000`);
- the automatic restart, into a boot with reset `software_cpu`;
- the settings restored over HTTP, and the projection published
  (`settings_restored`, `mine_on_boot_disabled`).

These numbers agree with F4.3's attempt 006 on the retired test build:
1032.8 GH/s, 18.9 W, chip 65.5 C, regulator 55.5 C.

- **Domain 3's samples:** 19 of about 28, none rejected, as in F4.3 (18).
  Its rate agrees with the others. The shortfall is recorded, not
  investigated.

## Attempts 007 and 008

- **007:** blocked at `failure_admission`, before any install. The
  operator had passed a copy of the manifest without its image files. The
  campaign had only read the settings backup over HTTP.
- **008:** the self-test passed: 1022.2 GH/s, domains 257.3 to 274.5 GH/s,
  18.9 W, chip 65.5 C. The settings restore after the pass then failed.
  - Its error was swallowed into one category, so the captures could not
    tell which step failed.
  - The campaign's recovery reinstalled the package and restored the
    settings with the same function, and production was confirmed.
  - The fix, `577d04ed`, records a failed restore's step in a private file,
    as value-free categories.
  - Attempt 009's restore then succeeded, so 008's failure is unexplained
    and did not recur. A recurrence will now name its step.

## Production afterwards

The receive-only capture after step 4 confirms production at `bb9f2ae7`:
- 57 safe-boot records, and 57 `usb_board_identity` records reading
  `built=601 stored=601 identity=match`;
- the USB profile `worker_started` with a confirmed baseline, and no panic;
- `boot_ordinal=17` with reset `software_cpu`, the pass's restart boot;
- the core-dump store valid for this boot and the one before it.

## Effects

In attempts 008 and 009:
- the factory installs with the owner's Wi-Fi credentials and the
  self-test tuple;
- the settings backup and restore over HTTP;
- the regulator on at 1150 mV under the local generation's 240 s bound in
  each phase, at 525 MHz;
- the fan from 10 to 100 %, and diagnostic work only;
- in 008, the campaign's recovery install.

Attempt 007 read the settings backup only. There was no pool work, Worker
Start or Work Lease, test build, other profile, direct UART or pin
access.

## Not shown here

- **Two restoration booleans,** `production_mining_never_started` and
  `pool_traffic_absent`, hold by the campaign's construction: a self-test
  boot starts no mining owner and no Worker. The campaign does not observe
  the network.
- **The 205:** its self-test rerun at the final F4 commit is phase F4.6's.
- **Upstream's bare `selftest=1` and BOOT held at power-on** stay
  non-claims on both boards.
- **Parity:** nothing is promoted here; F4.6 proposes it.
