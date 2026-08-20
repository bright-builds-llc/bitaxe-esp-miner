# Parity work plan

- Run ID: `20260820T224453Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `0ea4896559f24b5a3343940d4eeaf1c98a08ffb7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection and new information

The user authorizes the evidence plan and all work necessary to test and verify
`STAT-003`. The prior v1 recheck is closed and pushed. It produced no candidate
or projection and changed no protected attempt data.

That closed run established a new, specific contract fact: attempt-005 retained
the exact pre/post runtime package identity and the old capture terminal
boundary, but not the original package-manifest bytes. An exact-source detached
rebuild, then a rebuild with the retained original build timestamp, both
produced different app ELF hashes because the ELF identity is workspace/path
sensitive. Therefore the original manifest-byte SHA-256 is unrecoverable and
must not be fabricated.

The old capture terminal boundary nevertheless proves manifest admission. The
v1 workflow compared its admitted manifest source/reference/app identity to
both pre- and post-restart system snapshots before reaching the final
`scoreboard restart persistence is invalid` check. A v2 contract can represent
this truth directly by binding a digest of the retained capture package
identity plus the exact old capture plan/closure/terminal boundary.

## Scope and non-scope

Add `bitaxe-scoreboard-evidence-v2` while retaining v1 validation for historical
capture tests. V2 replaces the unavailable `package_manifest_sha256` claim with
`capture_package_identity_sha256`, computed from canonical exact capture source,
reference, app ELF SHA-256, and build timestamp retained in both system epochs.
It also records `evaluation_source_commit` and a v2 source block containing the
capture plan/closure and evaluation plan digests.

Update the independent Rust validator to dispatch strictly by schema version,
reject unknown/ambiguous shapes, and require:

- exact board/attempt/capture/evaluator/reference identities and closed digests;
- old capture manifest-admission terminal boundary, protected input digest,
  current 32-path semantics, campaign seals/quorum, and no hardware rerun;
- exact same-boot repeats, durable-only restart transformation, unchanged
  non-difficulty fields/order/count, live SPA, one session-changing ordinal+1
  software restart, disabled boot mining, safety, cleanup, modes, and redaction.

Make the recheck diagnostic-complete with a closed stage vocabulary emitted only
to the new caller-owned mode-0600 v2 streams. It may read the immutable attempt
and the existing wrapper capture/detector streams, but not rely on or parse the
failed v1 recheck streams beyond inventory/mode admission.

This plan authorizes one read-only protected v2 evaluation, one independently
validated redacted v2 projection, evidence/result commits, and conditional
promotion of only `STAT-003`. It does not authorize credentials, external
network, detector/device/USB access, flash, monitor, mining, shares, restart,
recovery, attempt-006, UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add v2 Rust/TypeScript contracts and strict schema-dispatch validation
      while preserving v1 capture validation.
- [ ] Update the recheck to bind retained capture identity and old manifest-
      admission boundary, add closed diagnostics, and remove every synthetic
      manifest/app constant.
- [ ] Add v2 positive/negative/unknown-schema/identity-drift/diagnostic tests,
      run all gates, commit, and push the exact evaluator.
- [ ] Run the sole v2 protected command, independently validate/redact, and
      promote only on the complete quorum.

## Authorized command, recovery, and stop conditions

After the v2 evaluator is fully verified, committed, pushed, and clean, run:

`test -d scratch/stat003-scoreboard/attempt-005 && test -d scratch/stat003-scoreboard/wrapper-005 && test ! -e scratch/stat003-scoreboard/wrapper-005/recheck-v2.stdout && test ! -e scratch/stat003-scoreboard/wrapper-005/recheck-v2.stderr && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && (umask 077; just recheck-scoreboard-evidence-v2 --private-root scratch/stat003-scoreboard/attempt-005 --wrapper-root scratch/stat003-scoreboard/wrapper-005 --capture-plan docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md --capture-closure docs/parity/work-plans/20260820T150151Z-STAT-003/CLOSURE.md --evaluation-plan docs/parity/work-plans/20260820T224453Z-STAT-003/PLAN.md --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json > scratch/stat003-scoreboard/wrapper-005/recheck-v2.stdout 2> scratch/stat003-scoreboard/wrapper-005/recheck-v2.stderr)`

Starting the command consumes the sole v2 evaluation. It may launch only local
Git and the Rust validator. Any unknown file/type/mode/symlink, plan/source/
reference drift, malformed capture boundary, package-identity disagreement,
seal/quorum/restart/scoreboard/privacy failure, validator rejection, or nonzero
exit withholds promotion. The CLI must publish a closed failure stage without
raw values. Remove only its candidate; never rewrite/delete protected evidence.

## Verification and promotion

Run focused v1/v2 validators and real-child recheck tests, ordered Cargo gates,
Bright Builds, all Bazel tests, firmware build/package, redaction, reference,
parity, progress, selector, sensitive-value, file-size, and diff checks before
protected access.

On success independently validate the v2 projection, run repo redaction, create
`RESULT.md` plus a redacted summary, and commit/push evidence without checklist
change. Then transition only `STAT-003` to `verified` with
`unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, final-gate, and push.

On failure create `CLOSURE.md`, retain `STAT-003` implemented and the active
task, record the first closed stage, and do not authorize hardware/attempt-006.

## Non-claims

This plan does not recover or claim original manifest bytes, pre-claim parity,
expose retained identity values, validate profitability/absolute difficulty,
arbitrary pools/profiles, other boards/ASICs, unbounded mining, OTA/recovery,
or release readiness.
