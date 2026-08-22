# Parity work plan

- Run ID: `20260822T165408Z-STR-005-RETRY2`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `8e0e214ee78669856b132bc9831205e981a706c3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-stratum-v2`
- Continues: `docs/parity/work-plans/20260822T063702Z-STR-005-RETRY/CLOSURE.md`

## Selection

Attempt-002 stopped as the closed `evidence_invalid` category before passive
monitoring, private-root creation, fixture start, NVS construction, USB
ownership, flash, network, mining, or hardware control. Its outer launcher did
not identify which pre-effect predicate failed, and the consumed plan permits
no third attempt.

The diagnosed root cause is now deterministic. Workspace discovery returned
the source repository through `BUILD_WORKSPACE_DIRECTORY`, but every campaign
child inherited Bazel execroot as its current directory. The first real
predicate, `git check-ignore -q scratch/str005-stratum-v2/attempt-002`, returned
exit code 1 from that execroot and was flattened to `evidence_invalid`. The
campaign test files were compiled but absent from the aggregate test entrypoint,
so the real-launch regression never ran.

Attempt-003 becomes eligible only after the campaign passes the resolved source
workspace explicitly to every Git, flash, validator, route, and fixture child;
the exact ignored-path regression passes through `bazel run`; the campaign and
validator tests execute in the aggregate suite; a read-only preflight returns
the closed `pre_effect_ready` checkpoint; every mandatory gate passes; and
the exact implementation is committed, pushed, and packaged from clean HEAD.

## Scope and non-scope

Fix only campaign workspace ownership, test discovery, real-launch coverage,
and closed pre-effect discrimination. The no-effect preflight may report only
schema, ready/failed status, closed category/checkpoint, and false effect/root
booleans. It must not create the attempt root, start the fixture, open USB,
monitor the device, read credential contents, or mutate repository/device state.

The protocol, firmware, deterministic local Noise fixture, package identity,
privacy, safety, recovery, restoration, validation, and non-claim boundaries
from the original STR-005 plan remain unchanged. External production pools,
third-party endpoints, network scanning, direct UART, pins/pads/headers/GPIO,
probes, jumpers, soldering, injected signals, fault injection, unbounded mining,
arbitrary profiles, OTA, erase, and raw secret output remain prohibited.

## Implementation

- [x] Bind every campaign child and the owned fixture to the resolved source
      workspace rather than ambient process cwd.
- [x] Run campaign and validator tests from the aggregate Bazel entrypoint and
      keep a sub-second dedicated real-launch regression for workspace, child
      Git top-level, and ignored private-path admission.
- [x] Add the read-only `just stratum-v2-campaign-preflight` command and closed
      failure checkpoints without exposing messages, paths, credentials,
      identities, endpoints, or child output.
- [x] Bind only attempt-003 into the immutable parser and protected paths.
- [ ] Run every software, package, privacy, reference, and effect-eligibility
      gate on clean pushed source before any hardware command.

## Verification and promotion

After the implementation is committed and pushed, build the exact package and
run this no-effect preflight first:

`just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-003 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Continue only if it exits zero with schema
`bitaxe-stratum-v2-campaign-preflight-v1`, status `ready`, checkpoint
`pre_effect_ready`, `effect_started=false`, and `private_root_created=false`,
while the attempt root and projection remain absent. Any other result stops
before detector or hardware and does not consume attempt-003.

The only permitted hardware/effect commands are then:

1. `just detect-ultra205`
2. `just package`
3. `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-003 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Attempt-003 inherits the complete objective, preconditions, allowed effects,
mode-0700/0600 privacy contract, 400 MHz/1100 mV/100% fan profile, maximum
180-second lease, input/power/temperature/fan safety limits, one accepted-share
ceiling, terminal safe stop, exact settings/package restoration, cleanup,
independent validation, redaction, and public withholding rules from
`docs/parity/work-plans/20260822T040442Z-STR-005/PLAN.md`.

The attempt is consumed only when command 3 starts. Preserve the earliest
category and its closed checkpoint through recovery and cleanup. There is no
unchanged retry or attempt-004 under this plan. On any failure, attempt every
independent safe-stop/restoration step, withhold the projection and `RESULT.md`,
keep `STR-005` at `implemented`, and record a truthful closure.

Before command 3, run the ordered Cargo gates, Bright Builds, all Bazel tests,
the dedicated real-launch regression, canonical build/package, parity/progress,
redaction, reference cleanliness, source inventory, sensitive-value review,
and final diff review. `verified` requires one independently accepted
attempt-003 projection with exact source/reference/package identity, advancing
watchdog and telemetry, complete encrypted lifecycle, genuine ASIC work/result
and accepted share, terminal safe stop, exact restoration, cleanup, and
redaction. External production-pool interoperability remains a non-claim.
