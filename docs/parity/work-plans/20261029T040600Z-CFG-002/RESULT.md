# Parity work result

- Parity row: `CFG-002`
- Final status: `implemented`
- Implementation commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

- **Unit and golden:** `board_default_seed_values("601")`
  (`crates/bitaxe-config/src/defaults/seed.rs`) is the 601's seed.
  - `crates/bitaxe-config/fixtures/gamma-601-defaults.csv` is
    `config-601.cvs` at the pinned reference.
  - `the_601_seed_matches_its_upstream_config` pins the seed to it.
  - The matrix tests pin the profile entry in
    `crates/bitaxe-config/fixtures/board-profile-defaults.json`: 525 MHz,
    1150 mV, BM1370, the `gamma` device model, the automatic fan, the
    manual fan at 100 % and `selftest` on.
  - `tools/parity/src/board_defaults.rs` compares the Rust matrix with
    every pinned `config-*.cvs` (CFG-006).
- **Hardware:** `docs/parity/evidence/20261022-gamma601-wifi-provisioning.md`.
  - A factory install of the 601 package (`tools/flash/src/wifi.rs`) wrote
    the seed (`nvs_seed_status: provided`).
  - The board then read its stored identity back from it
    (`built=601 stored=601 identity=match`), and joined Wi-Fi with the
    owner's credentials.
  - Later factory installs for the self-test found the seed's fallback
    pool (`docs/parity/evidence/20261027-gamma601-self-test.md`).

## Status

`implemented`, with `unit,golden,hardware-smoke`. `verified` would need a
hardware readback of every seeded default, as CFG-001's attestation gives
the 205. The 601 build has no such attestation. The frequency, voltage and
fan settings the 601 mined with were saved by the soak and self-test
harnesses, not read from the seed.

## Revision

Status `implemented`, evidence `unit,golden,hardware-smoke`. The owned
target becomes `crates/bitaxe-config/src/defaults/seed.rs`,
`crates/bitaxe-config/fixtures/gamma-601-defaults.csv`,
`crates/bitaxe-config/fixtures/board-profile-defaults.json` and
`tools/flash/src/wifi.rs`. The notes become:

The Gamma 601 seed (`board_default_seed_values("601")`) matches the pinned `config-601.cvs` in unit and golden tests: 525 MHz, 1150 mV, BM1370, the gamma device model and the 601's fan and self-test defaults. A factory install of the 601 package wrote the seed, and the board read its stored identity back from it (`built=601 stored=601 identity=match`); see docs/parity/evidence/20261022-gamma601-wifi-provisioning.md and docs/parity/work-plans/20261029T040600Z-CFG-002/RESULT.md. Verification needs a hardware readback of every seeded default, which the 601 build does not attest; the profiles the 601 mined with were saved by the harnesses, not read from the seed. Two differences from upstream remain: the factory install also saves mineonboot off, a key upstream lacks (ADR-0041 item 11), and the seed's selftest=1 does not start the self-test on first boot, which upstream does. Other Gamma boards remain non-claims.

## Non-claims

- A hardware readback of the seeded defaults, over the firmware or HTTP.
- Other Gamma boards and other numbered seeds on hardware.
- **Two differences from upstream:**
  - the factory install (`tools/flash/src/wifi.rs`) also saves `mineonboot`
    off, a key upstream does not have (ADR-0041 item 11);
  - the seed's `selftest=1` does not start the self-test on first boot, as
    upstream's does. Only the flash tool's lease tuple starts it, on either
    board.
- The surface label still reads "Deferred Gamma 601 defaults"; a parity
  transition cannot change it.
