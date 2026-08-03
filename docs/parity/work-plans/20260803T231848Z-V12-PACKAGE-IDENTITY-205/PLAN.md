# Parity work plan

- Run ID: `20260803T231848Z-V12-PACKAGE-IDENTITY-205`
- Parity row: `V12-PACKAGE-IDENTITY-205`
- Initial status: `implemented`
- Evidence source commit: `66cf184943d7f3a5aedfc99e692a9f500707de9e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-v12-package-identity-typed-evidence`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan and ranked `CFG-001` first. The prior row-specific V12 plan already records
why every earlier candidate is not currently promotable. Those gaps remain:
`CFG-001` needs a purpose-bound upstream-default actuation after its terminal
repeated-boundary soak; the other earlier rows still need broader persistence,
network, ASIC, Stratum, API, safety-control, logging, release, or recovery
evidence than the current artifact proves. This plan neither retries nor
relabels those attempts.

`V12-PACKAGE-IDENTITY-205` is the first actionable row because its current note
names one precise missing artifact: an exact-package typed hardware smoke using
schema `bitaxe-version-evidence-v1`. The subsequently verified `SYS-004`
workflow committed exactly that projection from a fresh package and one
detector-admitted Ultra 205. Reusing this immutable, redacted, schema-valid
projection closes the stated evidence-format gap without another hardware run.

## Scope and non-scope

Reconcile only the exact source, reference, package-manifest, and runtime
identity claim for board 205. Validate the committed typed projection and its
row-independent facts, run the existing package/runtime regressions, record a
new result, and transition only `V12-PACKAGE-IDENTITY-205` if every binding
passes.

No firmware, reference, package, raw hardware evidence, or credential content
will change. No detector, flash, reset, monitor, HTTP, WebSocket, OTA, mining,
network, voltage, fan, power, direct-UART, or pin effect is authorized or
required. The completed SYS-004 hardware attempt is not repeated or relabeled;
only its committed closed projection is reused for the overlapping package and
runtime identity facts.

This plan does not claim configuration persistence, operator-snapshot or
runtime-health substance, network longevity, mining, ASIC behavior, safety
controls, partitions, rollback, OTA recovery, other boards, or release
readiness.

## Implementation

- [x] Confirm the typed projection, SYS-004 result, package manifest identity,
      checklist row, and runtime implementation agree on source and reference.
- [x] Run focused projection validation, package-manifest, and runtime boot
      attestation regressions; add code only if they expose a row-specific
      defect.
- [x] Add a row-specific `RESULT.md` containing only closed facts, conclusion,
      and non-claims.
- [ ] Transition only `V12-PACKAGE-IDENTITY-205`, synchronize deterministic
      progress, complete the task record, and archive it atomically.

## Verification and promotion

Focused verification:

- `bazel run //crates/bitaxe-automation-contracts:validate_version_evidence -- docs/parity/evidence/sys004-version-reporting/version-projection.json`
- `cargo test -p bitaxe-api runtime_boot_attestation --all-features`
- `cargo test -p xtask package_manifest --all-features`
- `bazel test //tools/xtask:tests //crates/bitaxe-api:tests //crates/bitaxe-automation-contracts:tests`

Mandatory repository verification is the ordered Rust sequence, managed Bright
Builds checks, `just test`, `just parity`, `just parity-progress`, redaction,
reference cleanliness, and `git diff --check`.

Promotion requires schema `bitaxe-version-evidence-v1`, board 205, exact source
and reference identities, a manifest digest, safe boot, closed redaction,
same-origin API observation, matching manifest provenance, and matching
same-boot WebSocket projection. Evidence remains `workflow,hardware-smoke`.
Any invalid schema, identity mismatch, missing observation, privacy failure,
invalid transition, or repository gate failure leaves the row `implemented`.
No hardware recovery or retry exists because this plan performs no hardware
action.
