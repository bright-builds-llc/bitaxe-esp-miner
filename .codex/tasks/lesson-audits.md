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

## audit-standing-authorization-lesson-baseline | 2026-08-03T22:37:17Z

- Audit timestamp: `2026-08-03T22:37:17Z`
- Trigger: the corrected standing-authorization lesson made the active combined total exceed both the `24,000`-byte and summed `8,000`-estimated-token limits
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md`
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Active lesson counts: global `6`; repository `23`; combined `29`
- Active byte counts: global `4,470`; repository `20,786`; combined `25,256`
- Conservative estimate: `ceil(4,470 / 3) = 1,490` global + `ceil(20,786 / 3) = 6,929` repository = `8,419` summed estimated tokens
- Retained lesson IDs: all `29` active lessons
- Consolidated lesson IDs: none; the new standing task-authorization lesson governs ordinary autonomous task continuation and is materially distinct from the direct-UART/pin authorization guardrail
- Archived lesson IDs: none; no lesson was proven obsolete, duplicate, or fully superseded
- Archive files created: none
- Next baseline:
  - Timestamp: `2026-08-03T22:37:17Z`; the 90-day changed-lessons trigger becomes eligible on `2026-11-01T22:37:17Z`
  - Counts: global `6`, repository `23`, combined `29`, with `0` new active lessons accumulated; the 10-new trigger occurs after 10 later additions
  - Bytes and estimates: global `4,470` / `1,490`, repository `20,786` / `6,929`, combined `25,256` / `8,419`
  - Active source SHA-256 values for change detection: global `a6eeee0d2e6715150dfc35397a7bb29b3292b46a0a1d367992911e6f3d13eff9`; repository `c101b8991f5a9761d8e19636466f227babd3ee3f37eac32810e08a2953c46cba`
  - Threshold state: above both hard loading limits; this proposed-append trigger is consumed and does not recursively trigger another audit without a distinct later trigger

## audit-display-origin-development-ip-baseline | 2026-08-30T16:17:00Z

- Audit timestamp: `2026-08-30T16:17:00Z`
- Trigger: nine active lessons accumulated after the 2026-08-03 baseline, and the proposed development-IP correction would be the tenth
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md`
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Active lesson counts: global `7`; repository `31`; combined `38`
- Active byte counts: global `5,230`; repository `28,410`; combined `33,640`
- Conservative estimate: `ceil(5,230 / 3) = 1,744` global + `ceil(28,410 / 3) = 9,470` repository = `11,214` summed estimated tokens
- Retained lesson IDs: all `38` active lessons
- Consolidated lesson IDs: none; no active lessons share the same durable cause, preventive rule, and trigger signal
- Archived lesson IDs: none; no lesson is obsolete, duplicate, or fully superseded
- Archive files created: none
- Next baseline:
  - Timestamp: `2026-08-30T16:17:00Z`; the 90-day changed-lessons trigger becomes eligible on `2026-11-28T16:17:00Z`
  - Counts: global `7`, repository `31`, combined `38`, with `0` new active lessons accumulated; the 10-new trigger occurs after 10 later additions
  - Bytes and estimates: global `5,230` / `1,744`, repository `28,410` / `9,470`, combined `33,640` / `11,214`
  - Active source SHA-256 values for change detection: global `664021be592cf86593dc360b54c3d21d1d6c6078f5ca5afb74d1dd3dbd8782ef`; repository `a613294cc49aec8e92c9acff8ee4eeacd0f8f79925df9c222d51c9a7f3957c67`
  - Threshold state: above both hard loading limits; this 10-new-lesson trigger is consumed and cannot recursively retrigger without a distinct later trigger

## audit-str005-promotion-lessons | 2026-10-06T18:00:00Z

- Audit timestamp: `2026-10-06T18:00:00Z`
- Trigger: a proposed append of eight lessons from the STR-005 integration and promotion work, with the combined active total already above the 24,000-byte and 8,000-token limits
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md`
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Retained lesson IDs: every active lesson except the two archived below
- Consolidated lesson IDs: none; `lesson-diagnose-heap-loss-from-a-passive-series` extends `lesson-native-usb-and-wifi-share-internal-dma-heap` but has a different cause, rule and trigger
- Archived lesson IDs:
  - `lesson-gsd-frontmatter-body-separators`: obsolete; GSD is sunset and no GSD artifacts may be created
  - `lesson-cdc-commit-receipt-requires-live-control-state`: obsolete; the TinyUSB CDC maintenance protocol no longer exists and USB Serial/JTAG is the sole controller (ADR-0021, ADR-0023)
- Archive files created: `.codex/tasks/lessons.archive.md`
- Added lesson IDs: `lesson-clear-archived-core-dumps-before-the-next-panic`, `lesson-diagnose-heap-loss-from-a-passive-series`, `lesson-stop-on-first-success-cannot-prove-later-events`, `lesson-sealed-run-bindings-must-accept-every-run-shape`, `lesson-serve-a-page-s-whole-module-graph`, `lesson-join-random-ids-to-their-cli-flags`, `lesson-independent-review-before-parity-promotion`, `lesson-update-validator-scope-lists-with-scope-changes`
- Next baseline:
  - Timestamp: `2026-10-06T18:00:00Z`; the 90-day changed-lessons trigger becomes eligible on `2027-01-04T18:00:00Z`
  - Counts: global `7`, repository `41`, combined `48`, with `0` new active lessons accumulated
  - Bytes and estimates: global `5,230` / `1,744`, repository `38,145` / `12,715`, combined `43,375` / `14,459`
  - Active source SHA-256 values for change detection: global `664021be592cf86593dc360b54c3d21d1d6c6078f5ca5afb74d1dd3dbd8782ef`; repository `fe5aec699f43ff1d3fb37b5cb1c12ec5e35091d6286cd002a12aa1295479a1b6`
  - Threshold state: above both hard loading limits; startup loading uses the priority-ordered whole-block rule, and this trigger is consumed

## audit-ota002-promotion-lessons | 2026-10-06T21:29:59Z

- Audit timestamp: `2026-10-06T21:29:59Z`
- Trigger: a proposed append of three lessons from the OTA-002 hardware verification, with the combined active total already above the 24,000-byte and 8,000-token limits (owner-requested on 2026-10-06)
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md`
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Review: every active block in both files was read in full. Global lessons are unrelated to this repository's evidence boundaries and none is obsolete.
- Retained lesson IDs: every active lesson except the one archived below
- Consolidated lesson IDs: none. The closest pairs differ in cause or trigger: `lesson-native-usb-capture-needs-prearmed-observation-or-replay` and `lesson-manual-removal-needs-owner-observation` (byte capture versus token acceptance); `lesson-hardware-retries-require-new-information`, `lesson-standing-task-authorization-avoids-confirmation-churn` and `lesson-hardware-fixes-do-not-require-plan-per-iteration` (retry evidence, confirmation churn and plan churn).
- Recurrence noted without a new lesson: attempt 001's endpoint preflight resolved a relative path inside the Bazel runfiles tree, which `lesson-cross-process-tests-use-real-boundaries` already covers.
- Archived lesson IDs:
  - `lesson-visible-cdc-is-not-flash-admission`: obsolete; the TinyUSB application CDC profile no longer exists (ADR-0021, ADR-0023), and its lasting rule is enforced by AGENTS.md and `just detect-ultra205`
- Archive files changed: `.codex/tasks/lessons.archive.md`
- Added lesson IDs: `lesson-retained-log-ring-loses-boot-lines`, `lesson-fixed-serial-jtag-drops-runtime-log-markers`, `lesson-register-new-evidence-errors-in-typed-failure`
- Next baseline:
  - Timestamp: `2026-10-06T21:29:59Z`; the 90-day changed-lessons trigger becomes eligible on `2027-01-04T21:29:59Z`
  - Counts: global `7`, repository `43`, combined `50`, with `0` new active lessons accumulated
  - Bytes and estimates: global `5,230` / `1,744`, repository `39,922` / `13,308`, combined `45,152` / `15,052`
  - Active source SHA-256 values for change detection: global `664021be592cf86593dc360b54c3d21d1d6c6078f5ca5afb74d1dd3dbd8782ef`; repository `88feb1d9bcc0fd68c50687f8f5b80368e3cd46238a7a51a93fbfeb6c97bc29f6`
  - Threshold state: above both hard loading limits; startup loading uses the priority-ordered whole-block rule, and this trigger is consumed

## audit-bwg-evidence-hardening-supersession | 2026-10-09T22:14:16Z

- Audit timestamp: `2026-10-09T22:14:16Z`
- Trigger: a proposed append of three lessons from the BWG-007 restoration work, with the combined active total already above the `24,000`-byte and `8,000`-token limits; requested by `task-bwg-evidence-hardening`. Both active files were unchanged since the 2026-10-06 baseline (SHA-256 values matched).
- Active source paths:
  - Global: `/Users/peterryszkiewicz/.codex/tasks/lessons.md` (not modified; outside this task's scope)
  - Repository: `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/.codex/tasks/lessons.md`
- Review: every active block in both files was read in full and each repository block was compared with the current always-loaded AGENTS.md rules and the policy documents they cite.
- Before: global `7` lessons, `5,230` bytes / `1,744`; repository `43` lessons, `39,922` bytes / `13,308`; combined `50`, `45,152` bytes / `15,052` summed estimated tokens.
- Consolidated lesson IDs: none. No pair shares cause, rule and trigger. `lesson-native-usb-capture-needs-prearmed-observation-or-replay` and `lesson-never-invite-ready-before-live-checkpoint` overlap AGENTS.md only partly (its restoration pre-arm rule is Plan 13-specific; its invite-only-after-a-live-checkpoint rule is stricter than "say whether armed"), so both stay active.
- Archived lesson IDs (15, `14,642` bytes): each is a safety, privacy, authorization or evidence guardrail whose preventive rule AGENTS.md now states in full, so an equally strong, always-loaded replacement exists. Original text, date, rule and trigger signal are preserved in `.codex/tasks/lessons.archive.md` with the archive date, the exact AGENTS.md section as the reason, and `Replacement ID: none` (the replacement is a rule, not a lesson).
  - "Ultra 205 Serial Session Reuse": `lesson-espflash-no-reset-is-not-passive`, `lesson-power-and-usb-session-are-distinct`, `lesson-boot-proof-replay-must-outlive-service-sessions`, `lesson-heartbeat-cannot-prove-over-silent-transport`, `lesson-manual-removal-needs-owner-observation`, `lesson-physical-usb-identity-excludes-enumeration-fields`, `lesson-cold-boot-proof-needs-an-independent-observer` (also superseded in substance: its observer recommendation conflicts with the dormant external-UART policy)
  - "Direct UART And Pin-Manipulation Authorization": `lesson-direct-uart-and-pin-access-requires-authorization`
  - "Protected Evidence Root Ownership": `lesson-protected-evidence-root-ownership`
  - "Repository-Wide Evidence Privacy" and `docs/parity/evidence-policy.md`: `lesson-redact-after-private-classification`
  - "Progress-Gated Hardware Attempts" and `docs/hardware/hardware-attempt-policy.md`: `lesson-hardware-retries-require-new-information`
  - "Autonomous Ultra 205 Hardware Verification" with "Progress-Gated Hardware Attempts": `lesson-standing-task-authorization-avoids-confirmation-churn`, `lesson-development-ip-needs-share-redaction-not-interactive-secrecy`
  - "Iterative Hardware Fix Authorization": `lesson-hardware-fixes-do-not-require-plan-per-iteration`
  - "Asynchronous Human Checkpoints": `lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing`
- Added lesson IDs (3, `2,776` bytes):
  - `lesson-macos-holds-fresh-executables-at-launch`: new cause (macOS first-exec launch hold); distinct from `lesson-distinguish-agent-runtime-from-host-runtime`, which concerns agent-only timing.
  - `lesson-gate-each-hardware-step-on-the-previous-result`: recurrence of the cause behind `lesson-surface-preflight-exit-before-advancing` with new trigger signals (`;` chains, multi-scenario driver calls), so it extends rather than duplicates it.
  - `lesson-start-per-boot-attempts-on-a-fresh-boot`: new cause (spent per-boot one-shot state).
- After: global `7` lessons, `5,230` / `1,744`; repository `31` lessons, `28,057` bytes / `9,353`; combined `38`, `33,287` bytes / `11,097` summed estimated tokens.
- Remaining over budget: combined `9,287` bytes and `3,097` estimated tokens above the limits; the repository file alone exceeds both. Every remaining block records a distinct cause, rule and trigger not stated in AGENTS.md or another active lesson, and no remaining block is proven obsolete, duplicate or fully superseded. Archiving more on size alone is prohibited, so startup loading continues to use the priority-ordered whole-block rule. Reaching the budget would need promotion of further rules into AGENTS.md or another always-loaded source first, which is outside this audit's scope.
- Next baseline:
  - Timestamp: `2026-10-09T22:14:16Z`; the 90-day changed-lessons trigger becomes eligible on `2027-01-07T22:14:16Z`
  - Counts: global `7`, repository `31`, combined `38`, with `0` new active lessons accumulated
  - Bytes and estimates: global `5,230` / `1,744`, repository `28,057` / `9,353`, combined `33,287` / `11,097`
  - Active source SHA-256 values for change detection: global `664021be592cf86593dc360b54c3d21d1d6c6078f5ca5afb74d1dd3dbd8782ef`; repository `5c4c671df8bf0cff541a16a40f62042330fe4aa280b6f508728fe3d8a948ed74`
  - Threshold state: above both hard loading limits; this trigger is consumed
