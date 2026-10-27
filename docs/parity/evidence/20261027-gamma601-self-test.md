# Gamma 601: the self-test

Phase F4.3 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 14) ran SELF-001's two-phase lifecycle on the 601 under the immutable
plan `docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`,
on the test-only `gamma601-mining` firmware. Attempt 006 passed. Its
validated projection is
`docs/parity/evidence/self001-gamma601/attempt-006/self-test-projection.json`.

- **Commands** (the contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package-gamma601-mining`;
  2. `just self-test-campaign start --board 601` with the private root
     `scratch/self001-gamma601/attempt-006`, the package manifest, the
     owner's Wi-Fi and pool credentials by path only, the detector output,
     the plan and the projection path;
  3. the owner's two-second BOOT press at `cancel_ready`;
  4. `just self-test-campaign resume` with the same options;
  5. `just package --board 601`, `just flash --board 601 --port <port>`,
     state-preserving, and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`.
- **Sources:** `22ee4801`, the attempt's commit; the reference at
  `c1915b0`.
- **Package:** `firmware_variant=gamma601-mining`, board 601, source
  `22ee4801`; manifest SHA-256 and application ELF SHA-256 in the
  projection.
- **Pool:** `pool_config: local-owner-supplied`. The Wi-Fi credentials were
  applied at the owner's request for this campaign only.

## The planned failure

The lease tuple started the self-test in place of the mining owner and the
Worker. It took the local generation (`self_test_authority=granted
generation=1 bound_ms=240000`), went through preflight, preparing, warming
and a 5 s measured load, failed `planned_evaluation_failure` as planned,
stopped safely and held `cancel_ready` with `safe_state:true`. The owner's
BOOT press cancelled it: the campaign's resume required the cancelled
receipt and the restart before the pass phase (the receipt is checked in
memory, not kept in the saved capture).

## The pass

| Metric | Value | Limit |
| --- | --- | --- |
| Profile | 525 MHz, 1150 mV set; 1145 mV read | core 1035 to 1265 mV |
| Measured window | 30.009 s at difficulty 16 | 30 s |
| Total hashrate (nonces) | 1032.8 GH/s (525 MHz × 2040 small cores ≈ 1071) | at least 910 GH/s (85 %) |
| Domain hashrates | 261.9, 272.4, 272.0, 258.7 GH/s | 227.5 GH/s ± 33 % (about 152 to 303) |
| Domain samples / rejected | 28, 28, 28, 18 / 0, 0, 0, 0 | |
| Power | 18.9 W | at most 22 W |
| Input voltage | 5.133 V (the TPS546's VIN) | 4.5 to 5.5 V |
| Chip temperature | at most 65.5 C | at most 70 C |
| Regulator | at most 55.5 C, no fault | at most 105 C |
| Fan | 6087 rpm | above 1000 rpm |

The pass receipt, the terminal record (`outcome:passed`, `safe_stop:true`,
`restart_delay_ms:10000`) and the automatic restart were observed. The
campaign then restored the settings and published the projection.

- **Domain 3's samples:** 18 of about 28, none rejected. Its rate agrees
  with the others; the shortfall is recorded, not investigated.
- **Input voltage:** upstream skips its input-voltage check on a board
  without an INA260. The 601 checks the TPS546's VIN instead, a deliberate
  deviation the plan records.

## Attempts 001 to 005

Each attempt's record is in `TASKS.md`. In short:

- **001, 002:** blocked at the settings backup before any write. The
  single USB writer did not forward the self-test's records (`52b3dbf7`),
  and the 601's seeded fallback pool could not be restored from the inputs
  (cleared over HTTP at the owner's decision).
- **003:** the planned failure ran correctly; the flash tool expected the
  Worker's baseline (`807ad692`).
- **004:** the pass failed `hashrate_low` (784 GH/s). The self-test read
  one result frame per 100 ms pass, resent its job every 2 s where upstream
  feeds the BM1370 every 500 ms, and counted the warm-up into the window
  (`cd9aee8e`).
- **005:** the pass failed `domain_failed` (1418 GH/s, a domain at 368).
  The self-test resent one identical job, so the count repeated one
  interval's sample; each dispatch now sends new work, as upstream's job
  pipeline does (`22ee4801`).

Attempts 001 and 002 stopped before any write. In attempts 003 to 005
every stop held the safe state, and production was reinstalled before each
analysis.

## Production afterwards

The receive-only capture after step 5 holds 56 safe-boot records with
`identity=match`. The USB profile reads `worker_started` with a confirmed
baseline at `22ee4801`; no panic.

## Effects

In attempts 003 to 006: the factory installs with the owner's Wi-Fi
credentials and the self-test tuple, the settings backup and restore over
HTTP, the regulator on at 1150 mV under the local generation's 240 s bound
in each phase, 525 MHz, the fan from 10 to 100 %, diagnostic work only, the
campaign's recovery install after a stopped phase (003 to 005), and the
production reinstall. Before attempt 003, one HTTP write cleared the
fallback pool. There was no pool work, Worker Start or lease, other
profile, direct UART or pin access.

## Not shown here

- **The projection's restoration booleans** `production_mining_never_started`
  and `pool_traffic_absent` hold by the campaign's construction (a
  self-test boot starts no mining owner and no Worker); the campaign does
  not observe the network.
- **The production build's self-test:** production 601 builds still refuse
  the power path; their self-test is phase F4.5's, after promotion.
- **The 205:** the fixes in `cd9aee8e` and `22ee4801` change both boards'
  measurement, so the 205's verified self-test numbers are no longer
  directly comparable. Its rerun is phase F4.6's.
- **Upstream's bare `selftest=1` and BOOT held at power-on** stay
  non-claims on both boards.
- **Parity:** nothing is promoted.
