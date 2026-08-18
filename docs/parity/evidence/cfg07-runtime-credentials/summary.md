# CFG-07 Runtime Credential Evidence

cfg07_status: accepted
board: 205
implementation_commit: 04ecfab523bbeacead9871f4107e0d79426fe385
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
runtime_credentials_input: local-owner-supplied
committed_credential_values: none
raw_artifacts_committed: no
credential_contents_read_by_projector: false
redaction_status: passed
exact_non_claims: credential contents, credential rotation or persistence,
STR-09, ASIC-11, arbitrary profiles or pools, active control effects, self-test,
BAP/UART, other boards or ASICs, unbounded mining, OTA/recovery, and release
readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260818T150603Z-CFG-07/PLAN.md` | `be92a7b345f200028e2dec08fe5476f09d98dbb27fefe3c851f66ddeef9c91f1` |
| Accepted projection | `docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json` | `7840b62bf8aef9104e254202dbe007e00c54510ca30e30e1d0949f5ac437d206` |
| Accepted predecessor | `docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json` | `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e` |
| Attempt plan | `docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md` | `41ca445088dcf15c4c1c46e504a754c61260e7575eb16ccf68e0edb0fc742879` |
| Attempt closure | `docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md` | `350a56d6eaab1ea066f71a24d5a964a27e37d5472aca733fe912218afa87a79d` |
| Phase 30 conclusion | `docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md` | `789f18f29ca4ef864b2fcfa2b997e0680aa9c9d0239f67fabf87670d29334736` |

The accepted projection binds attempt source
`60a56d4935ced15eeb5ec6950b1ad4ea35fdf223`, implementation source
`04ecfab523bbeacead9871f4107e0d79426fe385`, and the pinned reference. Its
17-path evaluator identity contains seven credential-flow paths, eight current
validator/orchestrator paths, and two reference paths. The seven selected
attempt/current credential semantics are compatible.

## Closed Result

| Fact | Accepted result |
| --- | --- |
| Runtime credential category | `local-owner-supplied` |
| Wi-Fi input required | true |
| Pool input required | true |
| Both inputs forwarded to campaign | true |
| Live mining credentials consumed | true |
| Accepted submit observed | true |
| Detector admitted | true |
| Runtime identity | `trusted` |
| Campaign | `live-share` / `conservative` / `accepted` |
| Network status | `accepted` |
| Safe stop | `complete` |
| Cleanup | complete |
| Protected modes | valid |
| Committed credential values | none |
| Raw artifacts committed | no |
| Projector credential reads | none |
| Redaction | passed |

The same-chain proof is the join of the accepted SAFE-10 projection, the
immutable attempt-003 command requiring both runtime inputs, and attempt/current
credential-flow semantics. The source chain requires and validates both inputs,
creates private NVS material, seeds it before campaign runtime, and keeps secret-
bearing environment variables out of child process inheritance. The public
projector accepts only committed evidence paths and has no credential-path input.

## Phase 30 Admission

The canonical Phase 30 conclusion now records an explicit CFG-07-only promotion
with accepted outcome and passed current-source, detector, same-chain,
provenance, and redaction gates. Its exact CFG-07 fields are:

- `CFG-07.runtime_credentials_input: local-owner-supplied`
- `CFG-07.live_mining_credentials_consumed: true`
- `CFG-07.committed_credential_values: none`
- `CFG-07.safe_stop_status: complete`

STR-09 and ASIC-11 remain at their prior statuses and are not promoted by this
evidence.

## Verification

The following passed on the accepted implementation:

- CFG-07 Rust contract tests and independent validator
- public-only real-process projector tests, including source/live-proof drift
- newly enabled SAFE-10 predecessor projector tests
- actual attempt/current seven-path semantic compatibility
- specialized CFG-07 Bazel binary and validator builds
- real firmware package build
- independent SAFE-10 and CFG-07 validation
- Phase 30 structured admission tests
- Cargo format, lint, build, and all-feature tests
- Bright Builds checks
- all Bazel test targets
- parity and progress validation
- generic redaction command plus direct sensitive-pattern review
- reference cleanliness, file-size, source-inventory, and diff checks

The accepted projection is mode `0644`. Direct review found no credential path
or value, owner/pool/worker data, endpoint, port, USB/network identity, NVS
value, telemetry, raw log/payload, command, PID, trace, or protected identifier.

## Non-Claims

This evidence does not expose or independently validate credential contents or
prove rotation/persistence beyond the accepted campaign. It does not promote
STR-09 or ASIC-11; verify arbitrary profiles/pools; inject faults; verify
individual active controls, self-test, BAP/UART, other boards/ASICs, unbounded
mining, OTA/recovery, or release readiness.
