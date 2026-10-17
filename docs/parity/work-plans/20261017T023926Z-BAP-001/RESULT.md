# Parity work result

- Parity row: `BAP-001`
- Final status: `verified`
- Implementation commit: `b27e753fd8ee671a3a2fb75237d659cba2515720`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summary is `docs/parity/evidence/20261016-ultra205-soak-010.md`. Soak 010 drove BAP settings with the repo's
`bap-client` during 600 s of mining on one boot of an exact installed package.
It sealed `passed`, and an independent review recomputed every operation and
segment from the private records.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, narrows the settings non-claim to `ssid` and `password`, which were not exercised on hardware: during a Work Lease on one boot of exact package `e9f19f95` (one boot session throughout the soak observer's journal), the firmware answered BAP `SET` as the reference does, with `show_new_block` and `found_block` acknowledged and read back over BAP (`block_found`), `frequency` acknowledged `400.00` and `485.00`, `asic_voltage` `1100` and `1200`, `fan_speed` with no acknowledgement and no refusal, and `auto_fan` `auto_fan_speed_set`. Frequency, core voltage and the fan mode took effect live: at 400 MHz and 1100 mV the measured core voltage fell about 100 mV and the counter hashrate followed 400/485, the manual fan at 80% ran 2.87 times faster than the automatic fan before it, and the automatic fan resumed after `auto_fan` 1; see docs/parity/evidence/20261016-ultra205-soak-010.md. Request answers while mining were observed for `show_new_block` and `found_block` only; the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summary is claimed. The status is unchanged.
