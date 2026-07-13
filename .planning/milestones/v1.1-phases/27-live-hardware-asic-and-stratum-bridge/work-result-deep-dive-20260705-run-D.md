# Work-Result Deep Dive Run D (require_diagnostic_nonce)

Date: 2026-07-05  
Investigation: `require_diagnostic_nonce` (W5 bootstrap disabled)

## Result

Run A/B/C with Wave 4 default already disable W5 bootstrap unless `initialized_no_mining_gate` is set. Run D behavior matches Run A:

- `bm1366_diagnostic_result=timeout`
- `asic_status=fail_closed reason=work_result_diagnostic_timeout`
- No `asic_work_result_trace=initialized_no_mining_bootstrap`

## Conclusion

Without bootstrap fallback, Phase 27 does **not** reach `asic_production_status=initialized` when diagnostic read is silent. Bridge E2E requires either UART proof or explicit `initialized_no_mining_gate` for pool-path testing.
