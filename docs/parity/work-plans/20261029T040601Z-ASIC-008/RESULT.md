# Parity work result

- Parity row: `ASIC-008`
- Final status: `verified`
- Implementation commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pure core and its golden fixtures are in
`docs/parity/evidence/asic008-bm1370-core/summary.md`. The firmware drives
it through `firmware/bitaxe/src/asic_adapter/board_601.rs`,
`firmware/bitaxe/src/asic_adapter/production/board_601.rs`,
`firmware/bitaxe/src/asic_adapter/production/bm1370_init.rs` and
`firmware/bitaxe/src/mining_actuation_adapter/gamma601.rs`.

Each run below used a repo-owned command on the Gamma 601's one BM1370,
sealed its contract, and was independently checked against its private
captures:

| Run | Build | Profile | Result |
| --- | --- | --- | --- |
| [Chip check](../../evidence/20261024-gamma601-chip-check.md) | test, `824d7e20` | 1.15 V, the chip's reset frequency | one chip counted |
| [Hash check](../../evidence/20261024-gamma601-hash-check.md) | test, `92d865c4` | 300 MHz / 1150 mV | 8 verified nonces, none invalid, on a known job |
| [First local mining](../../evidence/20261025-gamma601-first-local-mining.md) | test, `cb4b8956` | 400 / 1150 | 601 s, 32 shares accepted, 0 rejected, mean 813 GH/s |
| [Default profile](../../evidence/20261026-gamma601-default-profile-check.md) | test, `68c8bf8f` | 525 / 1150 | 601 s, 29 / 0, mean 1054 GH/s |
| [4 h soak](../../evidence/20261026-gamma601-4h-soak.md) | test, `e1fa5a47` | 525 / 1150 | 4 h of mining, 440 / 0, mean 1068 GH/s, one 55.7 s Wi-Fi outage recovered |
| [Boot resume](../../evidence/20261027-gamma601-boot-resume-check.md) | production, `33fea4d9` | 525 / 1150 | resumed on boot, 302 s, 28 / 0, mean 1060 GH/s |
| [Self-test](../../evidence/20261028-gamma601-production-self-test.md) | production, `bb9f2ae7` | 525 / 1150 | 1028.4 GH/s over 30.07 s, four domains 260 to 272 GH/s |
| [Smoke soak](../../evidence/20261028-gamma601-production-smoke-soak.md) | production, `69dafb3d` | 525 / 1150 | 600.5 s, 39 / 0, mean 1063 GH/s |

The test builds were retired at promotion (F4.4). The production 601 build
compiles their mining path.

## Revision

Status `verified`, evidence `unit,golden,hardware-smoke,hardware-regression,soak`.
The repeated soak and self-test runs are repo-owned scripted probes. The owned
target adds the four firmware files above. The notes become:

The pure BM1370 core (docs/parity/evidence/asic008-bm1370-core/summary.md) drives the Gamma 601 firmware through its 601 ASIC and mining adapters. On the 601's one BM1370, a bounded check counted one chip, and a known-job hash check returned 8 verified nonces and no invalid ones at 300 MHz. Local mining accepted shares with none rejected at 400 MHz and at upstream's 525 MHz / 1150 mV default, for 10 minutes each, and for 4 hours of mining (440 shares, mean 1068 GH/s, one Wi-Fi outage recovered) on test builds. The promoted production build then resumed mining on boot, passed the self-test at 1028.4 GH/s across four domains, and ran a 10-minute smoke soak (39 shares, mean 1063 GH/s). See docs/parity/work-plans/20261029T040601Z-ASIC-008/RESULT.md. Live frequency changes, mining from a power-on boot, multi-hour runs of the production build, the thermal margin in a warmer room (the 75 C trip ended one production run at this profile), other BM1370 boards and chip counts, and release readiness remain non-claims.

## Non-claims

- **Live frequency changes:** they are deferred on the 601 (ADR-0041
  item 15).
- **Mining from a power-on boot:** mining started only after installs and
  software restarts.
- **Multi-hour runs of the production build:** the 4 h soak ran a test build.
- **The thermal margin in a warmer room:** the smoke soak's attempt 003
  (`cfac7234`) tripped at 75 C after 171 s at this profile; the passing run
  had 6.9 C of margin in a cooled room.
- **Other boards and chip counts:** other BM1370 boards and multi-chip
  chains.
- **Release readiness.**
