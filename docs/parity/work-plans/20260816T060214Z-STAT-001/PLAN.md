# Parity work plan

- Run ID: `20260816T060214Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `f94659e891635e9532448c557c8384bc08d4ab5f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
still require trusted physical panel observation; UI-003 requires trusted
physical button observation; SELF-001 lacks a production-safe hardware
execution route; and BAP-002 depends on BAP-001 plus an unavailable authorized
compatible accessory and qualified electrical UART setup. STAT-001 is the
first actionable row because its attempt-005 closure identifies a deterministic
software diagnostic gap: the sealed result reports only `watchdog_valid:
false`, which cannot distinguish a sample predicate failure from HTTP or
WebSocket per-window checkpoint/feed advancement failure.

The active lesson inputs total 29,963 bytes with a conservative summed estimate
of 9,990 tokens, above both deterministic loading limits. Every lesson heading
was inventoried; all seven global blocks and the repository service ownership,
opaque handoff, real-process boundary, espflash reset, power/USB distinction,
native capture, boot replay, silent transport, direct electrical authority,
protected-root ownership, earliest failure, HTTP readiness, private
classification, progress-backed retry, qualified transport, evaluator
identity, flash/monitor separation, standing authorization, preflight exit, and
telemetry-state blocks informed this plan. Omitted repository blocks were
`lesson-gsd-frontmatter-body-separators`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-physical-usb-identity-excludes-enumeration-fields`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`,
`lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing`,
and `lesson-never-invite-ready-before-live-checkpoint`. The 2026-08-03 audit
baseline consumed the hard-limit crossing; only five new active lessons have
accumulated, fewer than 90 days have elapsed, and this plan appends no lesson,
so no distinct audit trigger exists.

## Scope and non-scope

Advance only STAT-001. Replace the lossy watchdog boolean diagnosis with one
closed, value-free earliest-failure discriminator. Sample failures must
distinguish supervisor availability, checkpoint health, checkpoint sequence
presence, watchdog participation, feed reason, feed sequence presence, feed age
presence, and stale feed age. Closed-window failures must independently
distinguish HTTP checkpoint advancement, HTTP feed advancement, WebSocket
checkpoint advancement, and WebSocket feed advancement. The accepted and
non-watchdog states use `none`; no raw values, sequence numbers, ages, samples,
origins, endpoints, identities, or protected content may enter the diagnostic.

Carry the discriminator through the accumulator, sealed private network
evidence, sealed campaign result, and the hashrate wrapper's nonzero failure
envelope. Bump the affected schemas so old evidence cannot be mistaken for the
new diagnostic contract. Preserve the earliest terminal category and earliest
watchdog discriminator; later safe-stop, terminal observations, or cleanup must
not overwrite either. The wrapper may disclose the closed discriminator only
after verifying private modes, the result seal, the new result schema, failed
status, and the `watchdog_unresponsive` terminal category.

This is software-only authorization for local source, fixtures, tests, builds,
documentation, and ordinary git operations. Do not read protected attempt-005
artifacts, credentials, detector output, USB/device/network runtime, or private
endpoints. Do not detect, flash, monitor, mine, actuate, update, erase, inject
faults, manipulate physical power, use external UART, touch pins, pads,
headers, GPIO, probes, jumpers, solder, or signals, create a public projection,
or start attempt-006. Completion proves only diagnostic completeness; STAT-001
remains `implemented` until a separate immutable hardware plan earns the
missing evidence.

## Implementation

- [ ] Define one serializable closed watchdog-failure type and make sample
      validation return its earliest failing predicate instead of a boolean.
- [ ] Split per-transport window advancement into checkpoint and feed facts,
      assigning the exact HTTP/WebSocket discriminator in deterministic order.
- [ ] Preserve the earliest discriminator beside the earliest terminal
      category and serialize it in network evidence and campaign-result v11.
- [ ] Extend the hashrate wrapper to validate and disclose only the sealed
      closed watchdog discriminator for a watchdog-blocked child result.
- [ ] Add exhaustive focused tests for every discriminator, success `none`,
      schema rejection, seal rejection, category gating, redaction, and
      earliest-failure precedence.
- [ ] Run all focused and mandatory software, firmware, privacy, reference,
      package, parity, and diff gates; commit and push the implementation.

## Verification and promotion

Run focused campaign-network, campaign-evidence, automation, generated-contract,
real-child, seal, protected-mode, failure-envelope, and redaction tests. Run
`just verify-redaction`, `just verify-reference`, `just package`, and the
mandatory final sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Review the complete diff and verify the immutable plan digest, task binding,
clean pinned reference, closed label vocabulary, absence of protected-value
projection, and unchanged STAT-001 checklist fields. Commit and push the exact
implementation. Because this plan authorizes no hardware and cannot produce
the missing verified evidence, record a truthful `CLOSURE.md`, update only this
task block with the completed software review and exact next safe action, run
the terminal gates, and push the closure. Do not transition the checklist or
synchronize parity progress when its status, evidence kinds, implementation
pointers, and evidence pointers are unchanged.
