# Parity work log

## 2026-08-14T02:42:55Z | immutable-plan verification

- Plan SHA-256:
  `abca4697668c1648949f4198d9e0f25ac6c757f72f885058253ee84bc7cedd65`.
- Source commit: `0ae0842149e98d05e7ce03bf10071fd7071a2355`.
- Actions: Drafted one bounded attempt-014 contract after the clean
  synchronized selector ranked API-009 first and confirmed no open plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 44 canonical Bazel test targets, parity,
  parity-progress, focused activation/epoch and host-category regressions,
  real-process automation tests, redaction, reference cleanliness, the real
  ESP firmware build, task uniqueness, selector ownership, plan digest, and
  diff checks pass.
- Evidence: Immutable plan digest, public task binding, selector result, and
  category-only command outcomes. No credential, detector, protected attempt
  trace, USB, device, network, display, mining, or other hardware interface
  was accessed.
- Outcome: The exact-package, detector-gated attempt-014 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD, then perform exact-package admission before running the detector once.

## 2026-08-14T03:20:47Z | attempt-014 terminal checkpoint

- Source commit: `5d8108c2d4e1d33ea577111d6cc02d630a4a4918`.
- Package app ELF SHA-256:
  `7d9555ce5716c56ec14bbddf421f35e70e364c8f9697121ca17682d18e9c6c9d`.
- Actions: Built and independently validated the exact clean package, required
  the ignored Wi-Fi credential input without reading it, ran the detector once,
  and consumed the single authorized attempt-014. No operator checkpoint was
  emitted or confirmed.
- Verification: The detector admitted one board-205 device and closed USB
  cleanup. Factory and NVS flashes completed, runtime identity was trusted, the
  protocol gate was ready, and the local fixture exited cleanly. The sealed
  campaign closed `safety_stale` after retaining two milliseconds active; its
  final transition remained armed and hardware-stopped behind
  `operator_paused`. The wrapper preserved `hardware_blocked`, cleanup was
  complete, safe stop was unconfirmed, recovery was not attempted, every
  protected directory/file remained mode `0700`/`0600`, no holder or related
  process remained, and the public projection was absent.
- Evidence: Sealed category/boolean aggregates, exact package identity, private
  mode and digest checks, process/holder counts, and the public source contract.
  No origin, hostname, USB/network identity, credential, worker, address,
  password, token, sensor value, or raw trace was published.
- Outcome: `blocked`. The reflash succeeded, but command effects never reached
  a genuine notification, pause/resume, IDENTIFY, dismissal, or restart quorum.
  API-009 remains `implemented` and attempt-015 is not authorized.
- Blocker or next safe action: The production command-effects tracker forces
  `Run` only before its first active snapshot, then immediately reuses the
  unchanged boot-time `Paused` request seeded by `mineonboot=0`. Close this
  attempt and start a separate software-only plan for a lease-scoped initial
  run request that remains distinguishable from later explicit pause/resume.
