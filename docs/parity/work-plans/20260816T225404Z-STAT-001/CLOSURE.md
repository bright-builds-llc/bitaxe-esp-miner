# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `a0e3c88f0f3c739051508660344ffc1201d9d45788174d20b4b8071ea6fb11e4`
- Plan commit: `24ccdc922438dffd98c356e4dd4a64c9837d5c0e`
- Implementation commit: `f5f1be9b4614c155df96aaa78a2271c60065f84f`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `f5f1be9b4614c155df96aaa78a2271c60065f84f`
corrects the production-owner progress boundary identified after attempts 007
and 008. The owner remains the sole ESP task-watchdog subscriber and feed
authority, but it now records progress after each completed session event and
effect, before each bounded inbox wait, and after campaign publication. It
does not feed while an effect is unfinished, and the compiled five-second
timeout is unchanged.

This matches the actual ownership difference from the pinned reference: the
reference hashrate monitor runs as its own periodic FreeRTOS task, whereas the
Rust hashrate service is one responsibility inside the broader production
owner. The old Rust feed occurred only after the entire orchestration pass, so
a long but completing feedback cascade was indistinguishable from a stalled
owner.

Focused regressions cover multi-event feedback progress, post-return effect
progress, failed-event behavior, source ownership, and the absence of ESP-IDF
calls from the pure driver. The exact firmware package and all mandatory
software, privacy, reference, parity-invariance, immutable-plan, and diff
gates pass. This plan authorized no detector, device, network runtime,
credentials, protected evidence, or hardware effects, so it cannot supply the
accepted live evidence required to promote STAT-001.

## Next safe action

Run the clean synchronized selector in a new invocation. A fresh immutable
STAT-001 plan may consider exactly one attempt-009 only if it binds pushed
source `f5f1be9b4614c155df96aaa78a2271c60065f84f` to a new exact package and
defines the complete detector, hardware-safety, privacy, recovery, cleanup,
retry, stop, and promotion contract. Attempts 007 and 008 remain consumed.

## Non-claims

This closure does not verify task-watchdog freshness on hardware, BM1366
counter accuracy, twenty-window continuity, full 600-second hashrate accuracy,
HTTP/WebSocket coherence, terminal zero behavior, mining outcomes, electrical
accuracy, profitability, extended soak, other boards or ASICs, update or
recovery behavior, release readiness, or STAT-001 parity. It does not claim
attempt 008 would have passed with this correction; it establishes only the
software progress semantics, ownership constraints, exact firmware/package
build, and repository gate results described above.
