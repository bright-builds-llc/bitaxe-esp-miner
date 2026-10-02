# Control stack and port reuse successor amendment

## Status and precedence

Owner-authorized on 2026-10-02 under `task-control-stack-port-reuse-run`.
Contract ID: `str005-noise-serial-v2`, successor profile `control-stack-port-reuse`.

This amendment, the [v1 contract](str005-noise-serial-qualification.md), the
[v2 parity amendment](str005-noise-parity-scope-amendment.md) and the
[worker port reuse amendment](worker-port-reuse-amendment.md) together define
one new positive Noise attempt. It overrides only the requirements named below.
The [device Noise helper amendment](device-noise-helper-amendment.md) still
governs its own attempts, and its sealed results keep their meaning.

## Why a successor attempt

Two changes since attempt-003 need a real device run:

- **Control stack.** Firmware `5d0168a9` moved the general command `match` and
  the V2 admission and observation bodies out of the control-thread routing
  frames. On the 16 KiB control thread, the deepest resolved normal path fell
  from 13,536 to 10,000 bytes (see `task-control-stack-frame-pressure`). The
  baseline Connect, each Hello and probe, and the Noise Start, status and Stop
  all run through these routers.
- **Port reuse.** Gate `8b835c2c` (ADR-0101) reuses the single granted Ultra 205
  port on Connect. This run checks that every Connect completes without Chrome's
  port chooser.

## Overrides

| Requirement                     | Successor meaning                                                                                                                                                                                                                                                                                                     |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Predecessor                     | The sealed passing `device-noise-helper` attempt-003: result SHA-256 `6eab33d300e76159c5e1f7328edcd35f788204bcb1060ec122e767c181f6b098`, seal `9df0113f408b1473eacb0ffe14e5d83af8895ede71ac919e6a4f630d23e84a90`. Its after accounting must show a restored baseline, an inactive lease and `mine_on_boot=false`.  |
| Before identity and ledgers     | The candidate that attempt-003 left installed: source `68cb7e66d2bf8d046341f2c61f32cc9d875e495a`, ELF `b94d6886299a7b5322b99681ae820df96fa7532f190f3e2116404416c15cad8d`. Expected idle ledger next 22, last 21, charged 2,280,000 ms, with the original budget exhausted. Both must be observed unchanged before and after. |
| Attempt namespace and task gate | Private roots live under `scratch/control-stack-port-reuse/`. Preflight requires `task-control-stack-port-reuse-run` under `## Active` with the exact line `Control stack port reuse hardware: enabled.` A pass publishes to `docs/parity/evidence/control-stack-port-reuse/`.                                          |
| Port selection                  | Every Connect uses the Gate pinned in `MODULE.bazel`, so the worker port reuse amendment applies. The agent clicks **Connect Worker** with a real input event after a read-only visibility check. If Chrome's chooser appears, the owner may pick the Ultra 205. The attempt then still counts, but chooser-free Connect is not shown. |

Everything else stays as v2 and the helper amendment require:

- state-preserving initial installation plus four update, reconnect and
  maximum-exchange cycles, with at most five writes and 30-second installation
  captures;
- the helper amendment's native readiness, including the helper audit;
- no mining, Work Lease, grant, signer, pool credentials, Wi-Fi provisioning,
  factory reset, erase or rollback;
- the same retained page baseline;
- mandatory `finalize` and `review`, with `recover` still unavailable.

## Non-claims

A pass shows that the corrected control routing ran every Connect, Hello, probe,
Noise Start, status and Stop command on the device without a fault. A control
thread overflow would trip the FreeRTOS stack canary and fail the attempt. It
does not measure the control thread's runtime high-water mark, and it does not
exercise signed mining Start or renewal, whose separate offline audits still
govern their margin. It is not channel, share or mining evidence, and parity
stays 90/95.
