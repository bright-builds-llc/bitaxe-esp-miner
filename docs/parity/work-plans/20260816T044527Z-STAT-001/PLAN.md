# Parity work plan

- Run ID: `20260816T044527Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `bc11f23d570be3459979e312bca2995a9246b223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
still require trusted physical panel observation, UI-003 requires trusted
physical button observation, SELF-001 lacks a production-safe hardware
execution route, and BAP-002 depends on BAP-001 plus an unavailable authorized
accessory and qualified electrical UART setup. STAT-001 is the first actionable
row. Its sealed attempt-004 closure proves a deterministic software contract
gap: the conservative `LiveShare` campaign was accepted with trusted runtime
identity, clean parsing, safe stop, and cleanup, but
`CampaignNetworkCoordinator` classified `LiveShare` as `not_required` and
therefore could not produce the wrapper's required network quorum.

The active lessons total 29,963 bytes and an estimated 9,990 tokens, exceeding
both startup limits, so bounded whole-block loading and the existing audit flag
apply. Every global lesson plus repository service ownership/redaction, opaque
handoff, real-process boundaries, espflash reset, USB/power, native capture,
boot replay, heartbeat/transport, direct electrical authority, protected-root
ownership, earliest-failure precedence, HTTP readiness, private
classification, retry progress, qualified transport, evaluator identity,
flash/monitor separation, standing authorization, preflight exit, and
telemetry-state blocks informed this plan. Omitted repository blocks were
`lesson-gsd-frontmatter-body-separators`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-physical-usb-identity-excludes-enumeration-fields`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`,
`lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing`,
and `lesson-never-invite-ready-before-live-checkpoint`. The 2026-08-03 audit
baseline consumed the hard-limit crossing; only five active lessons have been
added since its 29-lesson baseline, fewer than 90 days have elapsed, and this
plan appends no lesson, so no distinct audit trigger exists.

## Scope and non-scope

Replace the repeated stage checks in the host campaign network coordinator with
one closed observation-mode policy. Map both conservative `LiveShare` and
`Soak` to the existing continuity observer, retain `CommandEffects` on its
dedicated observer, and keep `Observation` and `JobTransition` explicitly
`not_required`. Use that one mode for serial admission, worker selection,
command-effect timeout behavior, and finish semantics so the policy cannot
drift among branches. Add focused production-policy tests plus the existing
network accumulator and campaign tests needed to prove continuity and terminal
joins remain fail closed.

This is software-only host evidence infrastructure. It may modify
`tools/flash/src/campaign/network.rs`, its focused tests, the active STAT-001
task record, and this work-plan directory. It may use source, fixtures, builds,
and ordinary git operations. It must not read protected attempt-004 artifacts,
credentials, detector output, USB/device/network runtime, or private endpoints;
must not create a public projection; and must not detect, flash, monitor, mine,
actuate voltage/frequency/fan/power, update, erase, inject faults, manipulate
physical power, use external UART, or touch pins, pads, headers, GPIO, probes,
jumpers, solder, or signals. Attempt-004 remains consumed and this plan does not
authorize attempt-005 or any other hardware ordinal.

## Implementation

- [ ] Introduce one closed network-observation mode derived from
      `MiningCampaignStage` and consume it throughout
      `CampaignNetworkCoordinator`.
- [ ] Route `LiveShare` and `Soak` through the existing continuity observer,
      preserve the dedicated `CommandEffects` path, and retain explicit
      `not_required` behavior for `Observation` and `JobTransition`.
- [ ] Add focused Arrange/Act/Assert regressions proving the exact five-stage
      mapping and that live-share cannot take the `not_required` branch.
- [ ] Run focused network/campaign tests, privacy/reference/package checks, and
      every mandatory repository gate; commit and push before closing this
      non-verifying software plan.

## Verification and promotion

Before implementation, run the mandatory plan-only sequence and parity,
redaction, and reference checks. After implementation, run focused
`bitaxe-flash` network and campaign tests, build the canonical firmware package,
and run `just verify-redaction`, `just verify-reference`, then the mandatory
final sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires one production-owned closed policy mapping `LiveShare` and
`Soak` to continuity, `CommandEffects` to its dedicated observer, and only
`Observation`/`JobTransition` to `not_required`; every coordinator branch must
consume that policy; focused tests must fail against the prior source and pass
against the correction; existing HTTP/WebSocket, twenty-window, trusted-target,
terminal-zero, safe-stop handoff, cleanup, and failure-precedence tests must
remain green; the canonical package must build; reference integrity and
redaction must pass; and the final diff must contain no hardware evidence,
credentials, endpoints, or checklist overclaim.

This plan cannot verify STAT-001 because it authorizes no hardware. Do not
transition or synchronize the checklist when its status, target, and evidence
cells remain unchanged. After the pushed correction, create `CLOSURE.md` with
`Verification claimed: no`; keep STAT-001 `implemented`; and record that only a
fresh immutable plan may bind the fix to a newly built exact package and
authorize a separately bounded fresh hardware ordinal.
