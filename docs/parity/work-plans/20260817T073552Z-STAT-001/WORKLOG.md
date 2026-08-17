# Parity work log

## 2026-08-17T08:14:14Z | watchdog read-outcome and tuple correction

- Source commit: pending implementation checkpoint
- Actions: Added the four-state coherent-read outcome; mapped retry exhaustion
  and poison to precise fail-closed reasons; carried the field through API,
  retained health, campaign result v15/network v9, and private-first wrapper;
  latched outcome/phase/wait with the earliest watchdog failure.
- Verification: Store/core/wire, attempt-014-shaped terminal-overwrite,
  unknown/outcome/failure vocabulary, generated-contract, real-child,
  source-inventory, firmware, package, privacy, reference, file-length, and all
  mandatory gates pass. The recurring parity `os error 35` passed on one
  isolated tail retry.
- Evidence: The live-shaped regression preserves
  `watchdog_unproved/uninitialized/unavailable/not_waiting` after a later
  `waiting_inbox/within_deadline` terminal sample; pre-fix code overwrote the
  tuple. Retry exhaustion and history poison now have distinct labels.
- Outcome: Software diagnostic and real evidence-boundary correction complete;
  no checklist transition or hardware evidence claimed.
- Blocker or next safe action: Commit/push the exact source, then close this
  plan. Any future hardware ordinal requires its own immutable contract.
