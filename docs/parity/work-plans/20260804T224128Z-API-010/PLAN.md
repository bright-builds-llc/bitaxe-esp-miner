# Parity work plan

- Run ID: `20260804T224128Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `486d0718d3cb9089fff8300e2a54b15b4b61c4d4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-007`

## Selection

The clean synchronized `main` branch and deterministic selector resume only
the `API-010` lineage. Attempt-006 implemented and pushed the minimal 16 KiB
boot-evidence observer stack fix with a red/green ownership regression and a
clean full software gate. Its exact remediation package then built, but the
sole detector stopped as `bootloader_connect_failed` before flashing; the
attempt-006 child and public projection were never created.

This plan directly continues
`docs/parity/work-plans/20260804T222559Z-API-010/PLAN.md`.

Protected recovery summaries prove retry admission and final cleanup each saw
the same accessible, holder-free physical device for three stable samples,
with unchanged enumeration. The repository's closed USB policy maps this exact
boundary to disconnecting normal USB and barrel/DC power for ten seconds, then
reconnecting normal power followed by USB. A user report proves only that the
manual occurrence happened; one successful fresh board-info detector is the
required objective boundary-change proof.

No other row is eligible while this open plan lineage remains. The attempt-006
detector outcome and software fix are pushed at
`486d0718d3cb9089fff8300e2a54b15b4b61c4d4`; no protected device, port,
USB/network/process identity, credential, origin, theme/hostname value, or raw
trace is copied into Git.

## Scope and non-scope

Prepare exactly one attempt-007 continuation and pause until the connected
Ultra 205 has both normal barrel/DC power and USB disconnected for at least ten
seconds, then is reconnected using normal barrel power followed by USB. After
the user reports that occurrence, build the exact pushed package and run one
fresh protected detector. Run the durability capture only if the detector
admits exactly one board 205 and board info succeeds.

The capture may perform one exact-package flash-monitor transaction containing
the pushed 16 KiB observer fix, read the original theme, POST one generated
non-secret alternate theme, confirm immediate readback, request one normal
software restart through the typed device-session transaction, prove the same
physical device and exact build at ordinal `N+1`, confirm persisted theme
state, restore the original theme, and confirm cleanup. At most one built-in
exact-package recovery flash is allowed if normal restoration cannot be
confirmed.

The supervisor-owned child is
`scratch/api010-theme-durability/attempt-007`; it must be absent before launch,
created exclusively as mode `0700`, and contain only mode-`0600` private
artifacts. The caller-owned sibling is
`scratch/api010-theme-durability/wrapper-007`, mode `0700`, with mode-`0600`
detector/stdout/stderr files. The only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after complete success and semantic redaction.

Do not infer or automate the manual occurrence; automate electrical power;
run hardware before the reported occurrence; discover network origins; read or
expose credentials; change Wi-Fi or pool configuration; mine; enable ASIC
work; change voltage, frequency, fan, thermal, or power controls; exercise
display input; perform OTA, erase, fault injection, or raw partition writes;
terminate foreign processes; use direct UART, pins, pads, headers, GPIO,
probes, jumpers, soldering, or injected signals; or run a second detector or
capture.

## Implementation

- [ ] Commit and push this immutable plan/task checkpoint after the complete
      software gate, without editing the already pushed fix.
- [ ] Wait for the normal-connector power-cycle occurrence, then build the
      exact pushed package and run the one protected detector.
- [ ] Treat successful board-info admission as objective remediation proof;
      otherwise record the closed detector outcome and stop without capture.
- [ ] If admitted, run exactly one bounded attempt-007 theme durability
      capture with a 360-second child bound and shell wall clock above 420
      seconds.
- [ ] Validate package identity, private modes, the closed projection or typed
      failure, persistence/restoration/cleanup, redaction, and non-claims.
- [ ] Create `RESULT.md` and transition only `API-010` to `verified` on
      complete evidence; otherwise withhold evidence, keep `implemented`, and
      stop.

## Verification and promotion

Before hardware, rerun the focused observer target and canonical firmware
build, then the ordered format, strict Clippy, all-target build, all-feature
Cargo tests, Bright Builds, all Bazel tests, parity validation/progress,
semantic redaction, pinned-reference cleanliness, immutable-plan,
sensitive-output, selector, and diff checks. Commit and push the contract.

After the user reports the manual occurrence, the only authorized effectful
sequence is:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-007 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-007 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-007/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-007 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-007 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-007/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-007/verify.stdout 2> scratch/api010-theme-durability/wrapper-007/verify.stderr)`

If the detector repeats the full unchanged bootloader boundary after the
reported power cycle, select `stop_repeated_boundary`; any other detector
failure selects `stop_hardware_blocker`. A detector failure consumes
attempt-007 and forbids capture. Capture launch failure, timeout,
malformed/missing projection, repeated panic loop, non-ready session,
persistence mismatch, restoration uncertainty, cleanup/privacy failure, or a
safety invariant violation also ends without retry and preserves the earliest
typed category.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean package and reference, one admitted board 205, a ready session for the
same physical device, one software restart, exact build recovery, changed boot
session, ordinal `N+1`, persisted theme equality, exact restoration, disabled
mining and hardware control, complete cleanup, and passed redaction. Otherwise
withhold evidence and `RESULT.md`, keep `API-010` at `implemented`, and stop.
