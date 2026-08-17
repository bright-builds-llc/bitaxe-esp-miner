# Parity work log

## 2026-08-17T06:47:21Z | coherent watchdog snapshot implementation

- Source commit: `f5a8fd144ada04503cd5fa49c7dcc175a112aaf6`
- Actions: Replaced independent feed-history and phase/deadline reads with one
  firmware-owned sequence-bracketed snapshot; added bounded fail-closed reads,
  store-level regressions, production ownership guards, and evaluator bindings.
- Verification: Stable, exact mixed-interleaving, retry-exhaustion, and poison
  tests pass; runtime-health, phase-34, automation, real ESP32-S3 firmware,
  package, redaction, reference, all 47 Bazel tests, and every mandatory gate
  pass. The recurring parity `os error 35` passed on one isolated tail retry.
- Evidence: The old-feed/new-wait injection invalidates its first read and
  returns the new feed plus new within-deadline wait on retry. Eight repeated
  races and poisoned history return the closed unavailable/non-waiting default.
- Outcome: Software correction complete; no checklist transition or hardware
  evidence claimed.
- Blocker or next safe action: Close and push. A separate immutable plan may
  bind this pushed source to fresh attempt-014 and its complete hardware
  contract; never reuse or retry attempt-013.
