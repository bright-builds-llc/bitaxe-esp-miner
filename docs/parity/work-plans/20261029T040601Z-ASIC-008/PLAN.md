# Parity work plan

- Run ID: `20261029T040601Z-ASIC-008`
- Parity row: `ASIC-008`
- Initial status: `implemented`
- Source commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-gamma-601-bring-up` (F4.6; owner-approved plan
  `delightful-foraging-quill`, owner decisions 2026-10-26)

## Selection

The pure BM1370 core reached `implemented` under
`docs/parity/work-plans/20261023T180521Z-ASIC-008/PLAN.md`, which left
`verified` to phase F's Gamma 601 hardware work. Phase F and F4 are now
done: a firmware adapter, the chip and hash checks, local mining, a 4 h
soak, the self-test and the promoted production build. ASIC-008 cannot
reuse Ultra 205 evidence (ADR-0014, ADR-0041 item 4).

## Scope and non-scope

- A metadata-only change. It moves ASIC-008 `implemented` -> `verified`
  and adds `hardware-smoke,hardware-regression,soak` to its evidence.
- The owned target adds the firmware's BM1370 adapter files.
- No code, device, flash or network effect.

## Verification

- Every cited run sealed its contract, and an independent review agreed.
- The row's text cites only Gamma 601 evidence.
- `just parity` and `just verify-redaction` report no errors.
