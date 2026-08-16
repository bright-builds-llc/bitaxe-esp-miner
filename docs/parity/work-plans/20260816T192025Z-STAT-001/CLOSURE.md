# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `dbe601a0a1d54999d4d73438bb6f00156d0813ec8c59f8525c2842674e47c3e6`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `145eff426cf4400f862c8a4727e5cc6c8937372e` corrects the
source-owned false-stale boundary identified after attempt-007. The pure
runtime-health evaluator no longer applies an unrelated 2,000-ms constant to
ESP task-watchdog feeds. Its named timing policy now receives the firmware's
compiled `CONFIG_ESP_TASK_WDT_TIMEOUT_S` value after checked conversion to
milliseconds.

Focused regressions prove that a 2,001-ms feed remains fresh under the
five-second production policy, the exact 5,000-ms boundary is fresh, and
5,001 ms is stale. Existing closed watchdog failures, API serialization, and
correlated projection behavior remain unchanged. The exact firmware and
package build succeeded, and the generated sdkconfig confirms the five-second
value. All mandatory software, firmware, package, privacy, reference, parity,
immutable-plan, and diff gates passed.

This software-only plan accessed no detector, device, credentials, protected
attempt artifacts, network runtime, or hardware effects. It therefore cannot
supply the accepted live evidence required to promote STAT-001.

## Next safe action

Run the clean synchronized selector in a new invocation. A fresh immutable
STAT-001 plan may consider exactly one attempt-008 only if it binds pushed
source `145eff426cf4400f862c8a4727e5cc6c8937372e` to a new exact package and
defines the complete detector, hardware-safety, privacy, recovery, cleanup,
retry, stop, and promotion contract. Never reuse attempt-007 or infer that
this source correction alone authorizes hardware.

## Non-claims

This closure does not verify live BM1366 counter accuracy, watchdog freshness
on the device, HTTP/WebSocket hashrate coherence, rolling-window values,
terminal zero behavior, all twenty network windows, mining outcomes, hardware
safety, or STAT-001 parity. It does not claim attempt-007 would have passed
with this correction. It establishes only the compiled-policy ownership,
software boundary behavior, unchanged closed failures, successful exact
firmware/package build, and full software gate results described above.
