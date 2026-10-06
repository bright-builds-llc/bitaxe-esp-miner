# Parity work plan

- Run ID: `20261006T185454Z-OTA-002`
- Parity row: `OTA-002`
- Initial status: `implemented`
- Source commit: `306dbc8334689fa7a4a54c0b634861f0d5ad3a11`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ota002-www-hardware-verification`
- Continues plan: `docs/parity/work-plans/20261006T164502Z-OTA-002/PLAN.md`
- Hardware ordinal: `001`

## Selection

`OTA-002` is `implemented` with `unit` evidence. The owner asked on
2026-10-06 to build the hardware prerequisites and run them. The parity
validator requires `hardware-regression` evidence naming an
`interrupted-update` case before `verified`; this plan supplies exactly one
bounded attempt toward that.

## Scope and non-scope

One detector-gated run of `just capture-otawww-evidence` on one Ultra 205:

1. state-preserving exact-package install (`--phase install`, 60-second
   capture that only proves a completed boot, no Wi-Fi credentials, no
   factory reset);
2. one Gate session in the exclusive `stationEndpointHandoff` mode (Gate
   `f3c7f5a`) reads the station endpoint once, bound to fresh possession;
   `just otawww-endpoint` stores it privately, the Worker closes and the page
   leaves before any CLI serial use;
3. `--phase run`: the handed-off origin must report the installed boot
   ordinal and exact build, the passive safe state and the package label;
   then a complete OTAWWW of `www-probe.bin`, one proven restart, and served
   `version.txt`, `index.html` and `axeOSVersion` equal to the probe;
4. one reset-before-FIN OTAWWW of the package image with a 4096-byte prefix,
   retained `www_update_status=Protocol Error` in the same boot session, one
   proven restart, `axeOSVersion` `Unavailable` and the embedded recovery
   page served for static requests;
5. complete OTAWWW of the package image, one proven restart, package assets,
   build, hostname and stored-settings digest equal to the baseline.

The 60-second install capture is shorter than the general 360-second
guidance on purpose: since the fixed Serial/JTAG migration the install log
carries no identity or origin lines, so it only proves the flash completed
and the board booted. Identity, safe state and origin come from HTTP and
the Gate. Every restart uses `--capture-timeout-seconds 420`.

Non-scope: factory reset, Wi-Fi seeding, NVS writes, firmware
`/api/system/OTA`, erase-flash, raw writes, mining, pool, ASIC, voltage,
frequency, fan, thermal or power control, network discovery, direct UART,
pins, power interruption, and a second capture under this ordinal.

## Implementation

- [x] Probe image, route-parameterized interruption helper, two-phase
      supervisor, Gate endpoint handoff, contract, validator, redaction
      coverage and host tests (`306dbc83`, Gate `f3c7f5a`).
- [ ] Commit and push this plan, then build `just package` at the clean
      pushed HEAD.
- [ ] Run the six authorized commands of
      `task-parity-ota002-www-hardware-verification` once each, in order.

## Verification and promotion

Commands are recorded in `task-parity-ota002-www-hardware-verification`.
Evidence root: `scratch/ota002-otawww/attempt-001` (ignored, mode 0700,
files 0600). Projection:
`docs/parity/evidence/ota002-otawww/otawww-projection.json`, committed only
after `just verify-redaction` and an independent review re-derives it.

`verified` requires the closed `bitaxe-otawww-evidence-v1` projection to
validate, every stage postcondition above, no recovery flash, and the
review. Any other outcome keeps `OTA-002` at `implemented`, records the
earliest typed category in `CLOSURE.md`, and continues only under
`docs/hardware/hardware-attempt-policy.md` with a fresh ordinal and
verified progress.
