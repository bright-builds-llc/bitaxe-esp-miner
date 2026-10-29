# Parity work plan

- Run ID: `20261029T040602Z-PWR-004`
- Parity row: `PWR-004`
- Initial status: `deferred`
- Source commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-gamma-601-bring-up` (F4.6; owner-approved plan
  `delightful-foraging-quill`, owner decisions 2026-10-26)

## Selection

PWR-004 owns the TPS546 power controller, which the Gamma 601 uses in place
of the DS4432U and INA260. Phases D to F4 configured it, enabled it with the
ASIC held in reset, then powered the BM1370 through mining, the self-test
and the promoted production build.

## Scope and non-scope

- A metadata-only change. It moves PWR-004 `deferred` -> `in-progress` ->
  `verified`, because the parity tool admits only `deferred` ->
  `in-progress` as an activation.
- Evidence becomes `unit,golden,hardware-smoke,hardware-regression,soak`.
- The owned target becomes the TPS546's pure configuration and enable
  model, its golden fixture and its firmware adapter files.
- No code, device, flash or network effect.

## Verification

- Every cited run sealed its contract, and an independent review agreed.
- `just parity` and `just verify-redaction` report no errors.
