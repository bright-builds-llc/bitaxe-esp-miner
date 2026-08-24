# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `9be990644575c4f3023362e9c41ecfc2d26a1229e7e7ccf9eda89989a26d2390`
- Active task: `task-parity-str005-installed-package-recovery`
- Attempt-004 consumed: `no`

## Closure reason

Clean pushed implementation `5a0a30109777550bbb5488acf4eaaa4690cecb0d`
and its exact canonical package passed the mandatory software gates. A fresh
detector admitted exactly one Ultra 205 before recovery.

Bounded artifact search did not select an exact schema-v3 package. The one
timestamp-pinned clean rebuild did not match the installed app-ELF identity, so
the owner entered the firmware-only snapshot fallback. It completed three
allowed ranges, explicitly skipping NVS and coredump storage, then stopped at
earliest category `hardware_blocked`, checkpoint `snapshot_capture`. The 4 MiB
factory read reached the original fixed 300-second child limit and was
terminated before producing bytes. No restore bundle or public readiness
projection was created.

The recovery command performed no flash write, NVS write, new-baseline
adoption, fixture start, pool connection, mining, ASIC control, settings change,
or campaign effect. Attempt-004 remains unused; its private root and public
projection remain absent. The interrupted zero-byte target was protected to
mode `0600`, every private file and directory passed the required mode check,
and a post-run detector again reported exactly one ready Ultra 205 with no
owned recovery process remaining.

## Diagnosed implementation defect

The readback owner used the serial tool's implicit baud while imposing a
300-second limit on every range. That bound was insufficient for the 4 MiB
read. It also applied mode `0600` only after child success, leaving an
interrupted target dependent on host umask until cleanup. The follow-up
software correction pre-creates targets at `0600`, reasserts protection after
all child outcomes, and renders an explicit 460800-baud read with a 600-second
ceiling. This correction does not authorize or perform another recovery.

## Next safe action

Keep `STR-005` at `implemented`. Do not retry this recovery or attempt-004 under
the closed plan. Any further readback requires a new active task and immutable
continuation contract that binds the corrected command, a fresh private root,
cleanup, evidence rules, and a new recovery ordinal. Attempt-004 remains
available only because no campaign effect began; no later campaign ordinal is
authorized.

## Non-claims

This closure does not verify an Ultra 205 Noise handshake, V2 channel, ASIC
work, target-qualified nonce, encrypted share, accepted response, terminal safe
stop, package/settings restoration, external-pool interoperability, mixed-
protocol fallback, other boards, unbounded mining, OTA, or release readiness.
It does not create `RESULT.md`, hardware-regression evidence, or `verified`
status.
