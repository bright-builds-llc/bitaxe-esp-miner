# Parity work plan

- Run ID: `20260804T200849Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `053410ffd49824cf0737a581a94590db25c918bd`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-003`

## Selection

The clean `main` branch matched `origin/main`, the pinned reference was clean,
and the deterministic selector resumed
`docs/parity/work-plans/20260804T192918Z-API-010/PLAN.md` with no candidate
list. This plan is the next explicitly linked continuation of that immutable
`API-010` lineage.

The earlier `attempt-002` stopped before any route mutation or restart because
the exact-package flash-monitor transcript contained an ordered stale boot
prefix before the current ready epoch. Commit `67974ccc` added a strict
terminal-epoch classifier and a real-child-process production-shaped
regression; commit `053410ff` made this linked continuation selectable without
weakening ambiguity checks. This verified boundary change is the new
information required by the hardware-attempt policy for one fresh ordinal.

The local Wi-Fi credential input exists without being read. The private
attempt root, private wrapper root, and public projection destination for
`attempt-003` are absent. No other parity candidate is considered while the
selector has an open `API-010` plan.

## Scope and non-scope

Freeze an exact package from the pushed planning commit, privately capture one
detector admission for exactly one Ultra 205, and run exactly one bounded
`verify-theme-durability` transaction. The workflow may perform its admitted
exact-package flash, read the original theme, POST one generated non-secret
alternate theme, confirm immediate readback, request one normal software
restart through `device-session reboot-live`, prove same-device exact-build
boot ordinal `N+1` and persisted theme state, restore the original theme, and
confirm restoration and cleanup before publishing.

The supervisor-owned attempt root is
`scratch/api010-theme-durability/attempt-003`. It must be absent before launch,
mode `0700` after creation, and contain only mode-`0600` private artifacts. The
caller-owned sibling root is
`scratch/api010-theme-durability/wrapper-003`; it contains the mode-`0600`
detector and wrapper streams and never pre-creates the supervisor child. The
only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after the closed workflow succeeds and semantic redaction passes.

Do not read or expose credentials; discover network origins; change Wi-Fi or
pool configuration; mine; enable ASIC work; change voltage, frequency, fan,
thermal, or power controls; exercise display input; perform OTA or raw
partition writes; use direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals; claim installed AxeOS browser behavior; or run
a second hardware attempt. The architecture, code-shape, verification,
testing, Rust/TypeScript, ESP-device-session, hardware-attempt, and evidence
privacy standards govern this plan.

## Implementation

- [ ] Commit and push this immutable plan and complete active task contract
      before any detector or device interaction.
- [ ] Build the exact Ultra 205 package, capture one private detector transcript
      with mode-`0600`, and require exactly one admitted board 205.
- [ ] Run exactly one `attempt-003` capture with a 360-second workflow timeout
      and a shell wall clock exceeding 420 seconds.
- [ ] Validate private modes, the closed public projection, semantic redaction,
      exact package identity, same-device reboot facts, persisted theme,
      restoration, cleanup, and non-claims.
- [ ] Create `RESULT.md` and transition only `API-010` to `verified` if every
      criterion passes; otherwise record the earliest typed terminal category,
      withhold evidence, keep `implemented`, and stop without retry.

## Verification and promotion

Before hardware, run the focused terminal-baseline and real-child-process
regressions plus the repository-required pre-commit checks. After the pushed
plan, run only the exact commands recorded in the active task. After the single
attempt, run `cargo fmt --all`, strict Clippy, all-target/all-feature Cargo
build and tests, Bright Builds, `just test`, `just parity`, `just
parity-progress`, semantic redaction, pinned-reference cleanliness,
immutable-plan checks, sensitive-output review, private-mode checks, and diff
review.

Promotion requires the typed `bitaxe-theme-durability-evidence-v1` projection
to bind the exact clean package and reference, one admitted board 205, a ready
closed device session for the same physical device, one restart request, exact
build recovery, changed boot session, ordinal `N+1`, software reset, immediate
and post-restart theme equality, exact original-theme restoration, disabled
mining and hardware control, complete cleanup, and passed redaction. The
projection must contain no origin, URL, theme value, hostname, port, USB or
network identifier, credential, raw HTTP/serial/process material, or private
path.

If `baseline_multiple_sessions` recurs after its targeted verified fix, record
`stop_repeated_boundary`, withhold evidence, and stop. Any other typed failure
preserves its earliest category and recovery booleans, withholds promotion, and
ends this attempt without an unchanged retry. A distinct signature may justify
later diagnosis but is not authority for another invocation here.
