# Parity work plan

- Run ID: `20260812T132655Z-STR-007`
- Parity row: `STR-007`
- Initial status: `implemented`
- Source commit: `f7913ee207e71bc9f728fac589f0f494ce11fd08`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str007-mining-criteria-promotion`

## Selection and prior closure

The clean synchronized selector again selects `STR-007`. The preceding plan is
closed without verification because its one publication invocation repeated a
command token already injected by the Bazel target. Argument parsing rejected
that invocation before projector entry; neither a candidate nor a public
projection exists. The closed projector implementation at
`3978d828b55de61aa97d276528510cd1d66b6e3e` remains unchanged and passed its
complete implementation gate.

## Scope and non-scope

Add one behavior regression proving that the Bazel wrapper supplies
`project-mining-criteria-evidence` exactly once and that callers pass only
flags after `--`. Re-run the focused and complete gates, commit and push that
guard, then permit exactly one corrected software-only publication attempt:

```text
bazel run //tools/automation:project_mining_criteria_evidence -- \
  --summary <public-summary> \
  --smoke <public-smoke> \
  --soak <public-soak> \
  --coordinator-projection <public-str006-projection> \
  --projection <public-str007-projection>
```

The existing closed contract, independent validator, exact public-input
digests, source and cleanliness guards, atomic candidate publication, public
denylist, and promotion boundary remain authoritative. The attempt must use
only committed public evidence and current source.

No protected campaign artifact may be opened, copied, or summarized. No
detector, package build, flash, reset, USB/network session, credential input,
mining, pool contact, fan/voltage/power/ASIC actuation, recovery, direct UART,
pins, attempt-005, or other hardware effect is permitted. This plan does not
reopen or reinterpret the terminal default-profile attempt-004 continuity
task, and it does not claim accepted/rejected current soak behavior, successful
current 600-second continuity, arbitrary pools, other boards, updates,
recovery, profitability, unbounded mining, or release readiness.

## Implementation

- [ ] Add a regression that models Bazel's injected command plus caller flags,
      accepts the single-token shape, and rejects the duplicated-token shape.
- [ ] Run the focused contract and automation tests and the complete ordered
      repository gate.
- [ ] Commit and push the regression from a clean synchronized head before the
      single publication attempt.
- [ ] Run the corrected flags-only publication once, independently validate
      the closed projection, and publish no evidence on any failure.

## Verification and promotion

Run the mandatory ordered repository gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require focused wrapper, contract, projector, and real-child tests;
generated-contract verification; independent projection validation; exact
plan and admitted-input digests; mode `0644`; absent candidate; current source
and reference cleanliness; `just verify-redaction`; `just verify-reference`;
task uniqueness; a public sensitive-value scan; and `git diff --check`.

Promote only `STR-007` from `implemented` to `verified` with
`workflow,hardware-smoke,soak` if the closed projection passes every gate and
continues to bind only the committed bounded Phase 21 proof, verified STR-006
coordinator compatibility, and current fail-closed criteria. Any failure
withholds evidence, leaves the row implemented, closes this plan, and permits
no retry or hardware fallback.
