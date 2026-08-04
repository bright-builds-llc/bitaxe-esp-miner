# Parity work plan

- Run ID: `20260804T205704Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `20c310dec6f0d95e922eeb64c7ea0b0ed35e2db7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-005`

## Selection

The clean synchronized `main` branch and deterministic selector resume only
`docs/parity/work-plans/20260804T204310Z-API-010/PLAN.md`; no other parity row
is eligible while this `API-010` lineage remains open. This immutable plan is
the direct continuation after the sole attempt-004 detector stopped before
capture.

Private classification against the protected attempt-004 detector and durable
USB child traces narrows the authoritative signature to
`terminal_category=bootloader_connect_failed`,
`espflash_detail=connection_failed`, `enumeration_changed=false`,
`same_physical_device=true`, and `cleanup_complete=true`. Source inspection
confirms this is the exact branch whose prescribed recovery is a full normal
barrel/USB power cycle. No device, port, USB identity, process identity, raw
child material, or other protected value was emitted.

The hardware-attempt policy permits a fresh ordinal only after an authorized
non-invasive remediation objectively changes the failed boundary. A user
report that the normal-connector cycle occurred is an occurrence checkpoint,
not proof; one successful fresh `just detect-ultra205` board-info transaction
is the required objective proof. No code change, timing-only retry, or repeated
detector run is accepted as progress.

## Scope and non-scope

Prepare exactly one attempt-005 continuation, then pause until the connected
Ultra 205 has both normal barrel/DC power and USB disconnected for at least ten
seconds and is reconnected using normal barrel power followed by USB. After
that occurrence, freeze the exact pushed planning-commit package and run one
fresh private detector command. Run the theme durability capture only if that
detector admits exactly one board 205 and board info succeeds.

The capture may perform one admitted exact-package flash-monitor transaction,
read the original theme, POST one generated non-secret alternate theme,
confirm immediate readback, request one normal software restart through the
typed device-session transaction, prove same-device exact-build boot ordinal
`N+1` and persisted theme state, restore the original theme, and confirm
restoration and cleanup. At most one built-in exact-package recovery flash is
allowed if normal restoration cannot be confirmed.

The supervisor-owned child is
`scratch/api010-theme-durability/attempt-005`; it must be absent before launch,
created exclusively by the supervisor as mode `0700`, and contain only
mode-`0600` private artifacts. The caller-owned sibling is
`scratch/api010-theme-durability/wrapper-005`; it is mode `0700` with separate
mode-`0600` detector/stdout/stderr files. The only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after complete success and semantic redaction.

Do not infer that the physical cycle happened; automate electrical power;
discover network origins; read or expose credentials; change Wi-Fi or pool
configuration; mine; enable ASIC work; change voltage, frequency, fan, thermal,
or power controls; exercise display input; perform OTA, erase, fault injection,
or raw partition writes; terminate foreign processes; use direct UART, pins,
pads, headers, GPIO, probes, jumpers, soldering, or injected signals; claim
installed AxeOS browser behavior; or run a second detector/capture attempt.
The repository hardware, device-session, evidence, architecture, code-shape,
verification, testing, Rust, and TypeScript standards govern this plan.

## Implementation

- [ ] Commit and push this immutable plan and the complete active task contract
      after all focused and mandatory software gates pass.
- [ ] Wait for the normal-connector power-cycle occurrence, build the exact
      planning-commit package, and run the one fresh protected detector.
- [ ] Treat successful board-info admission as objective remediation proof;
      otherwise record the typed detector outcome and stop without capture.
- [ ] If admitted, run exactly one bounded `attempt-005` theme durability
      transaction with a 360-second capture bound and shell wall clock above
      420 seconds.
- [ ] Validate package identity, private modes, the closed public projection or
      typed failure, persistence/restoration/cleanup, redaction, and non-claims.
- [ ] Create `RESULT.md` and transition only `API-010` to `verified` on complete
      evidence; otherwise withhold evidence, keep `implemented`, and stop.

## Verification and promotion

Before any new hardware action, run the focused automation/flash real-process
targets and the mandatory ordered Rust, Bright Builds, Bazel, parity,
progress, redaction, reference-cleanliness, immutable-plan, sensitive-output,
and diff checks. Commit and push the plan/task checkpoint. After the reported
normal power cycle, run only the task-recorded package, detector, and optional
capture commands.

If the detector reproduces the full attempt-004 signature after the targeted
normal power cycle, select `stop_repeated_boundary`; any other detector failure
selects `stop_hardware_blocker`. A detector failure consumes attempt-005 and
forbids capture. Capture launch failure, timeout, malformed/missing projection,
non-ready device session, persistence mismatch, restoration uncertainty,
cleanup/privacy failure, or a safety invariant violation also ends without
retry while preserving the earliest typed category.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean package and reference, one admitted board 205, a ready closed session for
the same physical device, one restart request, exact build recovery, changed
boot session, ordinal `N+1`, software reset, immediate and post-restart theme
equality, exact original-theme restoration, disabled mining and hardware
control, complete cleanup, and passed redaction. It may contain no origin, URL,
theme or hostname value, port, USB/network/process identifier, credential, raw
HTTP/serial/child material, or private path.
