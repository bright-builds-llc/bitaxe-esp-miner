# Parity work plan

- Run ID: `20260822T063702Z-STR-005-RETRY`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `1c587e03d5dd5d356142d10b067243cab067aba3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-stratum-v2`
- Continues: `docs/parity/work-plans/20260822T040442Z-STR-005/PLAN.md`

## Selection

The exact attempt-001 outer command started once and stopped as
`evidence_invalid` before passive monitoring, fixture start, NVS construction,
USB acquisition, flash, network connection, mining, or hardware control. Under
the original immutable consumption rule, attempt-001 is consumed even though no
effect occurred.

The confirmed cause is a real-child launcher boundary: the standalone Bazel
`js_binary` anchored the workspace to its runfiles current directory rather
than the `BUILD_WORKSPACE_DIRECTORY` supplied by `bazel run`. That made the
otherwise-valid ignored paths, clean Git state, and exact manifest resolve
outside the repository. Read-only checks reconfirmed mode-0600 Wi-Fi/pool
inputs, ignored absent private/projection paths, clean pushed HEAD, and exact
source/reference package identity.

Attempt-002 is eligible only after a regression-backed workspace-root fix is
committed and pushed, all original software/effect gates pass again, and the
exact clean package is rebuilt. This is a changed launcher boundary, not an
unchanged retry.

## Scope and non-scope

Fix only standalone campaign workspace discovery by using the ordered existing
launcher policy: `BUILD_WORKSPACE_DIRECTORY`, then current directory, walking
to a real `MODULE.bazel`. Add a real Bazel-launch regression that proves the
outer CLI anchors repository paths before any monitor or effect.

All protocol, firmware, fixture, privacy, safety, recovery, evidence, and
non-claim boundaries from the continued plan remain unchanged. Do not weaken
path, Git, package, settings, prior-package, detector, safe-stop, restoration,
cleanup, or independent-validation checks. Do not run attempt-001 again.

## Implementation

- [ ] Correct the standalone launcher workspace root.
- [ ] Add direct and real-launch regression coverage.
- [ ] Re-run every original software, package, privacy, and effect-eligibility
      gate on clean pushed source.
- [ ] Invoke at most the exact attempt-002 command once.

## Verification and promotion

The only permitted effect commands are:

1. `just detect-ultra205`
2. `just package`
3. `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-002 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Attempt-002 inherits the complete objective, preconditions, allowed/prohibited
effects, mode-0700/0600 privacy contract, safety limits, terminal safe stop,
exact settings/package restoration, cleanup, independent validation, and
redacted projection requirements from the continued plan. It is consumed when
the outer command starts. No second retry is authorized by this plan; any
attempt-002 failure is recorded truthfully without `RESULT.md` or promotion.

Before the effect, run the ordered Cargo gates, Bright Builds, all Bazel tests,
canonical build/package, parity/progress, redaction, reference cleanliness,
source inventory, sensitive-value review, and final diff review. `verified`
still requires one accepted attempt-002 projection and every original hardware
acceptance criterion; otherwise `STR-005` remains `implemented`.
