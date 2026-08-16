# Parity work plan

- Run ID: `20260816T030231Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `d0785ed7418a8a10b74ac013a24958125b951f63`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The deterministic selector returned no open plan and ordered the first six
unfinished rows as UI-001, UI-002, UI-003, SELF-001, BAP-002, and STAT-001.
UI-001 and UI-002 require trusted physical display/panel observation; UI-003
requires physical button input; SELF-001 lacks a production-safe hardware route;
and BAP-002 depends on BAP-001 plus a compatible accessory or direct electrical
attachment that is unavailable or unauthorized. STAT-001 is the first
actionable row because attempt-003's immutable closure requires this bounded
software-only diagnosis before any future hardware ordinal.

Active lesson inputs exceeded the deterministic loading budget. Every global
lesson and the repository safety, authorization, evidence, retry,
real-process-boundary, and telemetry blocks were loaded. The inventoried but
unloaded repository blocks were the unrelated GSD frontmatter block, legacy
USB/power/cold-boot transport blocks at lines 36-84, and the unrelated
human-checkpoint invitation block at lines 168-174. No lesson-audit trigger is
active.

## Scope and non-scope

Diagnose why valid-UTF-8 serial lines containing the runtime-attestation text
collapsed into the coarse `malformed` status. Carry a closed, redaction-safe
parse-failure discriminator with bounded per-category counts through the pure
runtime-attestation accumulator, production campaign serial diagnostics, and
sealed campaign result. Reproduce the exact source-owned producer/parser
boundary and apply only the targeted correction proven by that reproduction.

This plan authorizes local source edits, deterministic fixtures, host tests,
firmware builds, documentation, and ordinary git commits/pushes. It does not
authorize detector use, credentials, device access, flashing, mining, HTTP or
WebSocket observation, attempt-004, direct UART, pins/pads/headers/probes,
physical power actions, OTA, erase, fault injection, or any public parity
projection. No protected attempt-003 artifact or raw serial line may be read,
printed, copied, or committed.

The work is informed by `AGENTS.md`, `AGENTS.bright-builds.md`, the empty local
override table, and the architecture, code-shape, verification, testing, and
Rust standards. The implementation will keep parsing/classification pure and
the serial adapter thin.

## Implementation

- [ ] Add a closed runtime-attestation parse-failure vocabulary and saturating
      per-category counts without retaining field names, values, or source text.
- [ ] Reproduce the firmware's source-owned
      `runtime_boot_attestation=unavailable` diagnostic at the production serial
      boundary and prove the existing substring matcher misclassifies it as an
      attestation candidate.
- [ ] Require the stable marker token boundary so unavailable/deferred
      diagnostics cannot poison otherwise valid attestations, while preserving
      logger-prefix tolerance and fail-closed handling of genuine malformed
      marker candidates.
- [ ] Carry the closed discriminator/counts through private serial diagnostics
      and the sealed campaign result, with behavior-focused regression coverage.
- [ ] Append execution evidence to `WORKLOG.md`; leave the checklist row
      `implemented` because this software correction is not hardware parity
      evidence.

## Verification and promotion

Focused acceptance requires tests proving: each parser error maps to one closed
category; counts saturate; the exact firmware unavailable diagnostic is a
lookalike rather than a candidate; a genuine malformed stable marker remains
classified; and two valid logger-prefixed attestations remain trusted when a
lookalike is interleaved. The sealed result and diagnostics may contain only
closed labels and counts.

Run focused crate/tool tests, firmware build coverage, `just verify-redaction`,
`just verify-reference`, and the mandatory final sequence:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Review the complete diff and reference cleanliness. This plan cannot promote
STAT-001 or change its checklist fields. A truthful `CLOSURE.md` will record
the diagnosed boundary, verified software correction, remaining hardware
evidence gap, and the exact prerequisites for a separately planned attempt-004.
