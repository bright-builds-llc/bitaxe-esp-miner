# Repository Lesson Audits

## audit-repository-initial-baseline | 2026-07-19T16:54:17Z

- Audit timestamp: `2026-07-19T16:54:17Z`
- Trigger: `no baseline`; initial 75% crossing of both the 24,000-byte and summed 8,000-estimated-token loading limits
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md`
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Active lesson counts: global `4`; repository `17`; combined `21`
- Active byte counts: global `2,846`; repository `15,321`; combined `18,167`
- Conservative estimate: `ceil(2,846 / 3) = 949` global + `ceil(15,321 / 3) = 5,107` repository = `6,056` summed estimated tokens
- Illustrative combined estimate: `ceil(18,167 / 3) = 6,056`; this is illustrative only and does not replace the per-file-summed estimate
- Retained global lesson IDs: `lesson-use-source-vtt-for-caption-fixes`, `lesson-reproduce-ci-at-exact-boundary`, `lesson-diagnostic-completeness-before-one-shot-attempt`, `lesson-zsh-lowercase-path-mutates-path`
- Retained repository lesson IDs: `lesson-gsd-frontmatter-body-separators`, `lesson-esp-idf-service-ownership-and-redaction`, `lesson-opaque-handoff-before-fallible-validation`, `lesson-cross-process-tests-use-real-boundaries`, `lesson-espflash-no-reset-is-not-passive`, `lesson-power-and-usb-session-are-distinct`, `lesson-native-usb-capture-needs-prearmed-observation-or-replay`, `lesson-boot-proof-replay-must-outlive-service-sessions`, `lesson-heartbeat-cannot-prove-over-silent-transport`, `lesson-manual-removal-needs-owner-observation`, `lesson-physical-usb-identity-excludes-enumeration-fields`, `lesson-cold-boot-proof-needs-an-independent-observer`, `lesson-direct-uart-and-pin-access-requires-authorization`, `lesson-protected-evidence-root-ownership`, `lesson-earliest-typed-failure-precedence`, `lesson-esp-idf-main-task-runtime-capacity`, `lesson-http-liveness-is-not-response-readiness`
- Consolidated lesson IDs: none
- Archived lesson IDs: none
- Archive files created: none
- Next baseline:
  - Timestamp: `2026-07-19T16:54:17Z`; the 90-day changed-lessons trigger becomes eligible on `2026-10-17T16:54:17Z`
  - Counts: global `4`, repository `17`, combined `21`, with `0` new active lessons accumulated; the 10-new trigger occurs after 10 later additions
  - Bytes and estimates: global `2,846` / `949`, repository `15,321` / `5,107`, combined `18,167` / `6,056`
  - Active source SHA-256 values for change detection: global `65335d8a0b837714a14033fde85dd7214021216245fc2bff9c667c664d43b550`; repository `b1c798ff60abd3bf81d73d04ce1d089f1ef839a5ef00d4bcbbcc2ed56bc7c1fe`
  - Threshold state: above both 75% thresholds (`18,000` bytes and `6,000` estimated tokens); this crossing is consumed and cannot recursively retrigger without a distinct later trigger
  - Proposed appends must be measured against `24,000` combined bytes and `8,000` summed estimated tokens before writing
