# Parity work plan

- Run ID: `20260804T222559Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `dc2ea737236a8f0bff3b225218a7cd7cc6d29bc9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-006`

## Selection

The clean synchronized `main` branch and deterministic selector still return
only the open `API-010` lineage at
`docs/parity/work-plans/20260804T205704Z-API-010/PLAN.md`. Attempt-005 consumed
its one detector and capture and is truthfully recorded at pushed commit
`dc2ea737236a8f0bff3b225218a7cd7cc6d29bc9`; no other parity row is eligible.

Attempt-005 objectively changed the prior USB boundary: the detector admitted
one Ultra 205 and the exact-package flash effect completed. The capture then
closed as `evidence_invalid` before any theme read or mutation. Protected
offline classification found `runtime_origin_missing`, 27 distinct sequential
boot sessions and ordinals, 27 `panic` resets, no runtime-origin or connected
Wi-Fi marker, one safe-state/boot identity per boot, and only the allowlisted
stack-overflow panic category. No protected device, port, USB/network/process
identity, credential, origin, theme/hostname value, or raw trace was emitted.

The crash timing and code boundary provide new objective software information:
each boot reaches the safe-state marker and fails around the first 10-second
boot-identity replay, while the replay owner uses an 8 KiB stack and the same
identity emission succeeds during startup on the main task. The minimum
source-level remediation is to give that formatting/logging observer the
existing 16 KiB complex-runtime stack budget and lock the ownership/budget
contract with a focused source regression. Hardware success is still required
to verify the hypothesis.

## Scope and non-scope

Change only the boot-evidence observer stack budget from 8 KiB to 16 KiB, add a
focused host test proving the one named observer owns and uses that explicit
budget, and wire the test into Bazel. Do not alter replay cadence, boot/session
identity, safe-state attestation, HTTP/Wi-Fi behavior, theme semantics, display,
mining, ASIC work, voltage, frequency, fan, thermal, power, OTA, partitions,
credentials, or any other runtime owner.

After the focused and mandatory software gates pass, commit and push the fix,
then build the exact pushed package and run one fresh protected detector. The
source change and package identity are the progress-backed retry basis. Only if
the detector admits exactly one board 205 and board info succeeds may one
attempt-006 theme durability capture run.

The capture retains the prior contract: one admitted exact-package
flash-monitor transaction, one generated non-secret alternate theme, immediate
readback, one normal software restart through the typed device-session
transaction, exact-build same-device ordinal `N+1` proof, post-restart
persistence, exact original-theme restoration, and at most one built-in
exact-package recovery flash if normal restoration cannot be confirmed.

The supervisor-owned private child is
`scratch/api010-theme-durability/attempt-006`; it must be absent before launch,
created exclusively as mode `0700`, and contain only mode-`0600` files. The
caller-owned sibling is `scratch/api010-theme-durability/wrapper-006`, mode
`0700`, with mode-`0600` detector/stdout/stderr files. The only eligible public
artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after complete success and semantic redaction.

Do not retry unchanged code; infer network origins; read or expose credentials;
change Wi-Fi or pool settings; mine; enable ASIC work; change voltage,
frequency, fan, thermal, or power controls; exercise display input; perform
OTA, erase, fault injection, or raw partition writes; terminate foreign
processes; use direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals; or run a second detector/capture attempt.

## Implementation

- [ ] Raise only `OBSERVER_THREAD_STACK_BYTES` to 16 KiB and preserve the one
      boot-lifetime observer and all replay behavior.
- [ ] Add and wire a source-ownership regression proving the explicit budget,
      its sole `.stack_size` use, and the unchanged 10-second identity replay.
- [ ] Run the focused host target and canonical firmware/package targets, then
      the mandatory ordered repository gate.
- [ ] Commit and push the clean implementation before any hardware action.
- [ ] Run one exact package build, one protected detector, and—only after
      detector success—one bounded attempt-006 capture.
- [ ] Validate package identity, private modes, the closed projection or typed
      failure, restoration/cleanup, redaction, and non-claims. Create
      `RESULT.md` and promote only on complete evidence.

## Verification and promotion

Focused software verification is
`bazel test //firmware/bitaxe:boot_evidence_source_ownership_tests` followed by
`bazel build //firmware/bitaxe:firmware_image`. Then run, in order,
`cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`, plus semantic redaction, pinned-reference cleanliness,
immutable-plan, sensitive-output, protected-mode, selector, and diff checks.

After the implementation commit is clean and pushed, the only authorized
effectful sequence is:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-006 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-006 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-006/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-006 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-006 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-006/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-006/verify.stdout 2> scratch/api010-theme-durability/wrapper-006/verify.stderr)`

The capture shell wall clock must exceed 420 seconds. A detector failure
consumes attempt-006 and forbids capture. Any launch failure, timeout,
malformed/missing projection, non-ready device session, persistence mismatch,
restoration uncertainty, cleanup/privacy failure, repeated panic loop, or
safety invariant violation ends without retry while preserving the earliest
typed category.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean package and reference, one admitted board 205, a ready session for the
same physical device, one software restart, exact build recovery, changed boot
session, ordinal `N+1`, persisted theme equality, exact restoration, disabled
mining and hardware control, complete cleanup, and passed redaction. Otherwise
withhold evidence and `RESULT.md`, keep `API-010` at `implemented`, and stop.
