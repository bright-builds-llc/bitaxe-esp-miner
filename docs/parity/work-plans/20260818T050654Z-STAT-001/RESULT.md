# Parity work result

- Parity row: `STAT-001`
- Final status: `verified`
- Implementation commit: `7d78889a82b5da9ef085290e29e37b5b7ddad310`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The immutable plan is
`docs/parity/work-plans/20260818T050654Z-STAT-001/PLAN.md` with SHA-256
`b9bc554eb3e49c685bcbd7a852a754febf015228df4ae89efe6e6b951eb65e24`.
The independently validated public evidence is
`docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json`.

Exact effect commands were commands 1 and 2 frozen under the plan's
`Authorized live commands and recovery` section: the one protected
`just detect-ultra205` invocation and the sole conditional
`just capture-hashrate-monitor-evidence` invocation for attempt-019. Both
exited zero; no retry or recovery rerun was used.

Pre-effect verification ran the focused Rust contract, TypeScript real-process,
flash, parity, generated-contract, firmware/package, redaction, reference,
selector, file-length, and diff checks. The mandatory ordered gate passed:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`

Post-effect validation passed with:

- `bazel run //crates/bitaxe-automation-contracts:validate_hashrate_monitor_evidence -- <absolute-projection-path>`
- `just verify-redaction`
- closed `jq -e` assertions over the sealed private result, network, and serial
  diagnostic documents without printing protected values
- exact mode checks for the mode-0700 roots, mode-0600 private files, and
  mode-0644 public projection

The projection binds attempt 19, board 205, exact implementation/reference/
package/plan identity, 21 current source paths, one BM1366, four hash domains,
1,000 ms cadence, all 20 required windows, work renewal, coherent changing
positive HTTP and WebSocket samples, warm rolling windows, terminal zeros,
trusted runtime identity, conservative 600-second campaign, active-then-paused
state, confirmed safe stop, complete cleanup, no rerun, and passed redaction.
The sealed private quorum additionally proves one accepted-or-rejected submit
outcome, correlation failure `none`, stable watchdog, terminal HTTP/WebSocket/
pool joins, mixed reset `none`, panic signature `none`, and zero panic count.

## Conclusion

Attempt-019 supplies the checklist's missing live hardware/API evidence for the
implemented hashrate monitor. The exact-package Ultra 205 run observed real
BM1366 register-derived rates across the complete bounded horizon and both
operator transports, while the independent validator proved identity, source
semantics, continuity, terminal zeroing, safety, cleanup, privacy, and
redaction. This supports transitioning only `STAT-001` from `implemented` to
`verified` with `unit,workflow,api-compare,hardware-smoke,hardware-regression`.

## Non-claims and residual risks

This result does not prove profitability, absolute laboratory-calibrated
hashrate accuracy, every arbitrary frequency/voltage/fan profile, every pool,
unbounded mining, other ASICs or boards, overclocking, fault injection, OTA,
release readiness, external UART/BAP, or electrical-interface behavior. It
does not publish credentials, endpoints, identities, exact sensor values, raw
serial, HTTP/WebSocket bodies, pool/owner/worker values, commands, PIDs, or
protected traces. Longer soak and other hardware variants remain separate.
