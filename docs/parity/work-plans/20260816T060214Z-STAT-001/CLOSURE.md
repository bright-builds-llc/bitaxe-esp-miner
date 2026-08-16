# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `734e670828393ae520b8c7b5c115201171bb839c748847415e8acc7e7c2811e4`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `f9232963a23313b15c34dc5b7a0845085b94aad3` closes the
attempt-005 diagnostic ambiguity in software. Every watchdog sample predicate
now produces one closed value-free earliest-failure label, and each closed
window independently distinguishes HTTP or WebSocket checkpoint or feed
advancement. The earliest label is preserved with the earliest terminal
category through later terminal observation and cleanup. Sealed network v5 and
campaign-result v11 evidence carry the label, while the hashrate wrapper
discloses it only for a mode-correct, seal-valid, v11 failed result whose
terminal category is `watchdog_unresponsive`.

Focused Rust tests reproduce all eight sample failures, all four transport
window failures, accepted `none`, first-failure precedence, non-watchdog
precedence, serialization, and value-free labels. Canonical automation tests
exercise all twelve public labels and reject an old schema, unknown label,
missing cause, mismatched category, and invalid seal. The complete ordered Rust,
Bright Builds, 45-target Bazel, firmware package, parity, progress, redaction,
and pinned-reference gates pass. No protected attempt artifact, credential,
detector, USB/device/network runtime, or private endpoint was accessed.

This plan cannot identify which new discriminator the connected Ultra 205 will
produce because hardware was explicitly outside scope. STAT-001 therefore
remains `implemented`, the checklist fields remain unchanged, and no progress
history or public evidence projection is written.

## Next safe action

A separately committed immutable STAT-001 plan may authorize one fresh
attempt-006 only after binding pushed source `f9232963` to a new exact clean
package and restating the complete detector, credential-presence, privacy,
safety, recovery, retry, cleanup, and promotion contract. The wrapper must
surface exactly one sealed closed watchdog discriminator if the campaign again
fails as `watchdog_unresponsive`. Apply a targeted source fix only when that
new evidence identifies a source-owned boundary; never retry unchanged.

## Non-claims

This closure does not verify STAT-001, watchdog responsiveness on hardware,
twenty-window continuity, full 600-second hashrate accuracy, work renewal,
electrical accuracy, profitability, extended soak, arbitrary pools or profiles,
other boards or ASICs, update/recovery behavior, or release readiness. Synthetic
classification and wrapper tests do not substitute for detector-gated live
hardware evidence.
