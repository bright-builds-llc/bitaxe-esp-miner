# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `413e29c5fbc88432a195c588d15d5b9e5d795a71f7b894e78dbbb0bb844a4a03`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact clean pushed source/package
`579f831521ef44d2a08325397d52d4e455bb068e`, the pinned reference, every
focused and mandatory software/privacy/package gate, and one protected detector
passed. The sole attempt-014 campaign stopped after 302,436 accumulated active
milliseconds and 3 of 20 credited network windows with the sealed signature
`hardware_blocked` / `watchdog_unresponsive` / `watchdog_unproved`, owner phase
`waiting_inbox`, and wait state `within_deadline`.

This is not recurrence of attempt-013's post-fix authoritative signature:
`watchdog_feed_stale` did not recur. The pushed coherent store resolved the
old-feed/new-wait contradiction, but the new sample projected no admitted latest
watchdog observation. Runtime/package identity and attestation were trusted;
active safety and same-package state were valid; terminal HTTP, reconstructed
WebSocket, and pool persistence passed; safe stop was confirmed; USB cleanup
was ready; all private files were mode 0600 beneath mode-0700 roots; and the
campaign-result and network-continuity digests matched. Redaction and projection
withholding passed, so no public evidence was written.

The current closed vocabulary maps a missing latest observation only to
`unproved`. It cannot distinguish a genuine pre-subscription observation,
bounded coherent-reader retry exhaustion, poisoned history, publication-
lifecycle loss, or a transport reconstruction defect. Inferring one cause
would overstate the evidence. Attempt-014 is consumed, no retry ran, terminal
outcome is `stop_hardware_blocker`, and STAT-001/checklist/progress remain
unchanged.

## Next safe action

Do not retry attempt-014 or authorize attempt-015. A fresh software-only
STAT-001 plan should add a closed value-free coherent-read outcome—covering at
least stable, uninitialized, retry-exhausted, and poisoned-history states—from
the firmware store through runtime health and sealed v15/v9 evidence. It should
reproduce the live-shaped active-to-unproved transition at the production
boundary, preserve earliest-failure semantics, determine the root cause, and
apply the minimum regression-backed correction. Hardware needs a separate
complete contract only after that work is pushed and fully gated.

## Non-claims

This closure does not verify STAT-001, prove reader contention, mutex poison,
subscription loss, scheduler behavior, or a transport defect; authorize
attempt-015; prove live BM1366 hashrate accuracy; complete twenty windows or
600 active seconds; establish work renewal or terminal-zero completion; or
claim profitability, arbitrary profiles/pools, other boards/ASICs,
update/recovery, or release readiness.
