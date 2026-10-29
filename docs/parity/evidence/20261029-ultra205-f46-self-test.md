# Ultra 205: the self-test at the final F4 commit

Phase F4.6 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041)
reran SELF-001's two-phase lifecycle on the Ultra 205. The 601 bring-up had
changed the shared self-test code since attempt 005 (`a11b579b`): the
board-aware contract, the runtime and its measurement, the forwarded USB
records and the campaign. It ran under the immutable plan
`docs/parity/work-plans/20261029T050000Z-SELF-001-ULTRA205-RERUN/PLAN.md`,
on attempt 005's unchanged contract (`ultra205_full_v1`, 485 MHz /
1200 mV). Attempt 007 completed. Its validated projection is
`docs/parity/evidence/self001-full-lifecycle/attempt-007/self-test-projection.json`.

- **Commands** (the F4.6 205 self-test contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package`;
  2. `just self-test-campaign start --board 205` with:
     - the private root `scratch/self001-full-lifecycle/attempt-007`;
     - the package's own manifest in `bazel-bin`;
     - the owner's Wi-Fi and pool credentials, by path only;
     - the detector output, the plan and the projection path;
  3. the owner's two-second BOOT press at `cancel_ready`;
  4. `just self-test-campaign resume` with the same options;
  5. the production reinstall and its confirmation, state-preserving. The
     contract names `just flash` then `just monitor
     --capture-timeout-seconds 360`; the run used `just flash-monitor
     --board 205 --port <port> --capture-timeout-seconds 360`, which does
     both in one command.
- **Sources:** `b5940eba`, the attempt's commit; the reference at
  `c1915b0`.
- **Package:** `firmware_variant=production`, board 205, source
  `b5940eba`. The manifest SHA-256 and the application ELF SHA-256 are in
  the projection.
- **Pool:** `pool_config: local-owner-supplied`.
- **Wi-Fi:** the credentials were applied at the owner's request for this
  campaign (2026-10-28).

## Attempt 006

Attempt 006 (`29142412`) refused at the settings backup, before any write.
The 205's fallback pool, seeded by an earlier factory install, cannot be
restored exactly from the inputs; every other compared setting matched.
By owner decision, one HTTP settings write cleared it (the fallback URL
empty, the fallback off), as on the 601's F4.3 attempt 002. `b5940eba`
amends the contract to allow it. A read-back confirmed the fallback off
and every compared setting matching the inputs.

## The planned failure

- **Run:** the lease tuple started the self-test in place of the mining
  owner and the Worker. It went through preflight, preparing, warming and a
  5 s measured load, then failed `planned_evaluation_failure` as planned.
- **Stop:** it stopped safely and held `cancel_ready` with
  `safe_state:true`, replayed 27 times.
- **Cancel:** the owner's BOOT press cancelled it. The projection records
  the long press, the cancellation receipt and the restart.
- **No panic** appears in either phase's capture.

## The pass

| Metric | Value | Limit |
| --- | --- | --- |
| Profile | 485 MHz, 1200 mV set; 1198 mV read | core 1080 to 1320 mV |
| Measured window | 30.160 s at difficulty 16 | 30 s |
| Total hashrate (nonces) | 387.3 GH/s (485 MHz × 894 small cores ≈ 433.6) | at least 368.6 GH/s (85 %) |
| Domain hashrates | 104.1, 109.7, 116.3, 105.6 GH/s | 92.1 GH/s (a quarter of 368.6) ± 33 % (about 61.7 to 122.5) |
| Domain samples / rejected | 18, 11, 17, 4 / 0, 0, 0, 0 | |
| Power | 12.47 W | at most 15 W |
| Input voltage | 5.282 V | 4.5 to 5.5 V |
| Chip temperature | at most 60 C | at most 70 C |
| Fan | 2390 rpm | above 1000 rpm |

After the measurement:
- the pass receipt and the terminal record (`outcome:passed`,
  `safe_stop:true`, `restart_delay_ms:10000`);
- the automatic restart, into a boot with reset `software_cpu`;
- the settings restored over HTTP, and the projection published
  (`settings_restored`, `mine_on_boot_disabled`).

Notes on these numbers:
- **Attempt 005's total:** it read 1257.9 GH/s, about 2.9 times nominal.
  Its 30 s window also counted the warm-up's nonces, and it resent one
  identical job, which inflated the 601's F4.3 attempt 005 count. On the
  205, the share of each cause was not measured. `cd9aee8e`
  counts from the measuring stage on, as upstream does, and `22ee4801`
  sends new work on every dispatch. Attempt 007's total is the first 205
  measurement under both fixes. Attempt 005's domains (84.7 to 121.3
  GH/s), power (12.3 W) and chip temperature (61 C) agree with these.
- **The domain samples:** domain 4 has 4 samples and domain 2 has 11,
  against 17 and 18, with none rejected. Their rates agree with the others.
  The shortfall is recorded, not investigated, as for the 601's domain 3.

## Production afterwards

The campaign's installs were the production package at `b5940eba`. The
contract's state-preserving reinstall of that package and its capture
confirm production:
- 56 `usb_board_identity` records reading `built=205 stored=205
  identity=match`, the boot profile's `worker_started` records (the USB
  runtime's start, not a Worker Start command), and no panic;
- the core-dump store valid for this boot and the one before it.

A read-back over HTTP after the reinstall found mining on boot off, the
fallback off, and the Wi-Fi network and pool matching the inputs.

## Effects

- one HTTP settings write clearing the fallback pool, before attempt 007;
- attempt 006's settings backup, a read only;
- attempt 007's factory installs with the owner's Wi-Fi credentials and
  the self-test tuple, the settings backup and restore over HTTP, 485 MHz /
  1200 mV, the fan from 10 to 100 %, diagnostic work only;
- the state-preserving production reinstall.

There was no pool work, Worker Start or Work Lease, test build, other
profile, 601 package, direct UART or pin access, and no recovery install.

## Not shown here

- **Two restoration booleans,** `production_mining_never_started` and
  `pool_traffic_absent`, hold by the campaign's construction: a self-test
  boot starts no mining owner and no Worker. The campaign does not observe
  the network.
- **The fallback pool:** the 205 keeps none after this run; the owner can
  re-enter one in the web UI.
- **Upstream's bare `selftest=1` and BOOT held at power-on** stay
  non-claims on both boards.
- **Parity:** nothing is promoted here. SELF-001's passage for both boards
  follows, through a verified-revision authority.
