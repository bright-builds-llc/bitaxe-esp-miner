# Parity work plan

- Run ID: `20260813T061635Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `2feb5d4a2535b50d0568dd05a349dbba8ae31d6d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T235141Z-API-009/PLAN.md`

## Selection

The clean synchronized selector ranks API-009 first, so no candidate is
skipped. Its attempt-007 closure prohibits attempt-008 and any unchanged
hardware campaign, but explicitly permits a separately selected software
diagnosis that explains the nondeterministic post-pause safety-observation
loss and produces genuinely new boundary evidence.

Read-only source tracing identifies a missing host-orchestration join. The
command-effects observer treats API-visible `miningPaused=true` plus
`miningActivity=paused` as complete pause confirmation and immediately sends
resume. The production session publishes that logical state before its
synchronous resumable hardware safe-stop has completed. The observer therefore
races resume against safe-stop completion and the next fresh sensor epoch. The
same race crossed the boundary in attempts 005/006 and recurred as
`safety_prerequisites_stale` in attempts 004/007. The current campaign marker
does not expose a dedicated closed fact for resumable pause safe-stop, so the
host cannot make the required join without guessing from unrelated state.

The active lesson set remains at its audited 2026-08-03 baseline with no new
audit trigger. Complete safety, authorization, evidence, retry, redaction,
real-process, readiness, and host-boundary blocks are loaded; the disclosed
caption/VTT, small-table deduplication, legacy GSD separator, and manual-removal
blocks are unrelated omissions. Repo-local task/hardware/privacy guidance,
`AGENTS.bright-builds.md`, the empty effective overrides, and the architecture,
code-shape, verification, testing, Rust, and TypeScript standards govern this
continuation.

## Scope and non-scope

Add one versioned, closed `resumable_pause_safe_stop` state to the retained
campaign marker. Derive it only from the production-session snapshot owned by
the firmware campaign tracker: it may become `confirmed` only after a formerly
active command-effects campaign is logically paused, hardware is stopped, and
the same resumable lease remains armed. It must return to the non-confirmed
state before preparation or active mining resumes.

Carry only that closed state through the Rust marker parser and the in-memory
serial/network coordinator. Change the command-effects observer so one resume
request requires both API-visible logical pause and the matching same-session
serial safe-stop confirmation. Preserve request-once behavior and add a
bounded fail-closed wait for the join. Do not infer confirmation from elapsed
time, operator intent alone, a stale readiness transition, or terminal cleanup.

This is software-only work. It may use deterministic fixtures, local child
processes, builds, and repository verification. It may not read protected
attempt artifacts, use credentials, detect or access a device, package for an
effect, flash, reset, open USB/network/HTTP sessions, mine, initialize an ASIC,
change voltage/frequency/fan/thermal/power state, issue pause/resume/identify or
dismiss commands, perform OTA/recovery, use direct UART or pins, or create any
API-009 hardware ordinal. Attempt-008 remains prohibited.

## Implementation

- [ ] Add the closed resumable-pause safe-stop model and exact firmware-owned
      transition semantics without changing freshness limits or mining safety.
- [ ] Bind the marker state through the typed serial coordinator and require
      the logical-pause plus safe-stop join before the sole resume request.
- [ ] Add focused firmware, marker, serial/network, timing, request-once,
      malformed-state, regression, and sensitive-output tests at real
      production boundaries.
- [ ] Run every focused and mandatory gate, perform a simplification and diff
      review, then record the software-only result without promoting API-009.

## Verification and promotion

Focused verification must prove: logical pause alone never sends resume;
safe-stop confirmation before logical pause still joins correctly; the two
facts from one session produce exactly one resume; stale or malformed marker
states remain fail-closed; a bounded missing join terminates without a request;
repreparation clears the confirmation; and no raw sensor, device, network,
credential, path, or trace value enters public output. Run:

1. `cargo test -p bitaxe-flash campaign::network`
2. `bazel test //firmware/bitaxe:production_campaign_status_tests //firmware/bitaxe:sensor_source_ownership_tests //tools/flash:tests`
3. `bazel build //firmware/bitaxe:firmware`
4. `cargo fmt --all`
5. `cargo clippy --all-targets --all-features -- -D warnings`
6. `cargo build --all-targets --all-features`
7. `cargo test --all-features`
8. `bun scripts/bright-builds-check.ts all`
9. `just test`
10. `just parity`
11. `just parity-progress`
12. `just verify-redaction`
13. `just verify-reference`

Also require generated/build source ownership, immutable-plan digest, unique
task binding, selector closure, reference cleanliness, sensitive-output search,
`git diff --check`, and full diff review. This continuation cannot verify or
promote API-009 because the hardware effort is terminal and the complete
five-command device-user quorum remains absent. Leave the checklist unchanged,
create `CLOSURE.md`, and do not synchronize progress.
