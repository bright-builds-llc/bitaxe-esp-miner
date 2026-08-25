# Restore and verify continuation closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `946ec6b353add5e2ef08fe9047640f9271a68556021ca92e47661f6393103c1a`
- Active task: `task-str005-restore-and-verify-continuation`
- Remediation ordinal 2 consumed: `yes`
- Campaign attempt-005 consumed: `no`

## Closure reason

Remediation ordinal 2 restored the exact original firmware, settings, and theme,
but the immutable plan's final state requires both exact `mineonboot=false`
settings and `miningActivity=safe_blocked`. The restored historical firmware
intentionally projects the operator-paused blocker as `miningActivity=paused`.
Those literal requirements cannot both be satisfied without changing exact
settings or manufacturing an unrelated blocker, so the closed outcome is
`stop_impossible_contract`.

## Software and executor result

Pushed source/package `e3bd08bb` passed ordered Cargo formatting, clippy,
all-target build, and all-feature tests; Bright Builds; all 55 Bazel tests;
canonical build/package; parity/progress; redaction; reference cleanliness;
selector lineage; sensitive-value review; and diff review.

The generic executor still rejects non-espflash programs. The new private
managed-esptool command admits only the exact ESP32-S3 eight-range transaction
and uses the existing USB lease, owned-child supervision, same-device
reacquisition, retry rules, cleanup, and closed effect diagnostics. A real-child
regression proves partial-transfer classification without exposing output.

## Restoration outcome

Admission-only preflight and two fresh one-board detectors passed. The snapshot
and Wi-Fi commands each completed on their first attempt with terminal category
`ready`, device effect `completed`, termination `exited_success`, transfer
started/completed, and no raw output. USB cleanup passed.

The board now proves:

- original source `a11b579b62cb52a53bbf6072bde209d3eb3f17e2`;
- original app digest
  `32e2de545a0a89f97153baaa0e4c169f5064d47358d208732cf2b2ac8be3a4c4`;
- pinned reference and factory partition;
- exact restorable settings and exact theme;
- `mineonboot=false`;
- mining activity `paused`;
- zero hashrate; and
- zero accepted/rejected shares.

The protected state reached `settings_restored`. The public restoration
projection and `RESULT.md` were withheld because the literal final predicate
did not pass.

## Impossibility proof

In the restored source, the production-session projection maps
`ProductionSessionBlocker::OperatorPaused` to `MiningActivityStatus::Paused`;
other blockers map to `SafeBlocked`. Exact `mineonboot=false` supplies the
operator-paused intent. Producing `safe_blocked` would require changing that
intent, removing exact configuration, or deliberately creating another blocker.
None is authorized or truthful exact restoration evidence.

## Terminal decision

No attempt-005 root, fixture, campaign flash, mining, ASIC work, or share effect
was created. STR-005 remains `implemented`, no hardware-regression evidence was
published, and the blocked task remains active rather than archived.

## Next safe action

A new immutable continuation may replace the contradictory literal with a
closed `mining_inactive` proof accepting only `paused` or `safe_blocked` while
still requiring `mineonboot=false`, zero hashrate/shares, exact settings/theme,
and exact original identity. Because the board is already restored, that
continuation should first close restoration through read-only validation and
must not reflash before campaign admission.

## Non-claims

This closure does not verify Stratum V2 hardware, Noise/channel/job/share
behavior, campaign safe-stop, external pools, mixed-protocol fallback, other
boards, direct UART/pins, raw NVS/coredump access, fault injection, OTA, erase,
unbounded mining, release readiness, or verified STR-005 parity.
