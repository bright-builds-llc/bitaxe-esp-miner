# Parity work plan

- Run ID: `20261029T040600Z-CFG-002`
- Parity row: `CFG-002`
- Initial status: `deferred`
- Source commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-gamma-601-bring-up` (F4.6; owner-approved plan
  `delightful-foraging-quill`, owner decisions 2026-10-26)

## Selection

The Gamma 601 bring-up is promoted (F4.4, F4.5). F4.6 proposes the status
of its deferred and implemented parity rows from the committed 601
evidence. CFG-002 owns the 601's defaults from
`reference/esp-miner/config-601.cvs`. It cannot reuse Ultra 205 evidence
(ADR-0014, ADR-0041 item 4).

## Scope and non-scope

- A metadata-only change. It moves CFG-002 `deferred` -> `in-progress` ->
  `implemented`, because the parity tool admits only `deferred` ->
  `in-progress` as an activation.
- The owned target narrows to the files that hold, pin and install the
  601 seed.
- `verified` is not proposed. On the 205, CFG-001 needed a hardware
  readback of every configured default. The 601 build attests no defaults,
  and its mining profiles were saved by the harnesses rather than read
  from the seed.
- No code, device, flash or network effect.

## Verification

- The seed tests pass under `just test`
  (`the_601_seed_matches_its_upstream_config`, the matrix tests).
- The cited hardware run sealed its contract, and an independent review
  agreed.
- `just parity` and `just verify-redaction` report no errors.
