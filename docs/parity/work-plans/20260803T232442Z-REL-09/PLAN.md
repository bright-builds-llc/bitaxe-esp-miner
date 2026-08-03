# Parity work plan

- Run ID: `20260803T232442Z-REL-09`
- Parity row: `REL-09`
- Initial status: `implemented`
- Evidence source commit: `66cf184943d7f3a5aedfc99e692a9f500707de9e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel09-typed-operator-evidence`

## Selection

The selector reported no open plan. The candidates before `REL-09` still need
their own broader or safety-critical evidence: defaults actuation, complete
settings persistence, reconnect longevity, ASIC initialization/work/result and
serial behavior, frequency transitions, live Stratum coordination, API command
effects, voltage/power/fan/thermal/self-test regressions, logging lifecycle,
or partition/recovery behavior. The terminal upstream-default soak is not a
promotion source and will not be retried or relabeled.

`REL-09` is the first actionable row. The typed automation migration downgraded
it only because the active release operator-evidence consumer requires the new
profile/disposition schema and fresh detector-gated evidence. The later SYS-004
attempt used the canonical typed detector-output path, one detector-admitted
Ultra 205, an exact package, safe boot, HTTP and WebSocket observation, private
raw storage, cleanup, and a committed redacted projection. Those immutable
facts can be represented in a new release-profile operator-evidence root
without another hardware run.

## Scope and non-scope

Create one new `release` operator-evidence root whose observed slots cite only
closed facts from the committed SYS-004 result and projection. Mark mining
share and production safe-stop slots deferred; do not turn safe no-op boot or
cleanup into mining proof. Validate the root through the typed
`capture-operator-evidence` command and promote only `REL-09` if the current
schema, inventory, redaction, and provenance checks pass.

No firmware, package, reference, historical evidence, or private artifact will
change. No hardware or network action is required. This plan does not claim
credentials during mining, settings durability, ASIC behavior, shares,
production safe stop, safety controls, OTA/recovery, other boards, or release
readiness.

## Implementation

- [x] Add the exact 11 release slots plus evidence contract with current
      profile/disposition fields and redacted source/reference provenance.
- [x] Validate through typed automation and focused parity/automation tests.
- [x] Add a row-specific result containing the supported workflow claim and
      explicit non-claims.
- [ ] Transition only `REL-09`, synchronize progress, and archive the task.

## Verification and promotion

Focused verification:

- `just capture-operator-evidence --profile release --evidence-root docs/parity/evidence/rel09-typed-operator-workflow --require-redaction-passed`
- `bazel test //tools/automation:automation_test //tools/parity:tests`

Mandatory verification is the ordered Rust sequence, Bright Builds checks,
`just test`, parity, progress, redaction, reference cleanliness, and diff
checks.

Promotion requires exact release inventory, current profile/disposition fields,
source/reference/package provenance, observed detector and board admission,
canonical typed command provenance, observed redacted boot/API/WebSocket facts,
explicitly deferred mining-only slots, and passing semantic redaction. Evidence
remains `workflow`. Any schema, inventory, provenance, privacy, transition, or
repository-gate failure leaves `REL-09` implemented.
