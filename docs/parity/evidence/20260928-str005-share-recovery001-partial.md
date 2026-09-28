# Share001 recovery001: accounting measured, status proof incomplete

The published recovery-only host `4b88644d` collected independent accounting
without a Start, grant, renewal, reset, flash, core acquisition or clear.
Installed firmware remains f000872f/full ELF a3e25741, with Gate9643.

Fresh authenticated accounting measured **next21, completed20, charged2100000ms,
pending=false**. The original campaign remains masks7/7, charged240000ms,
pending=false. Thus ordinal20's completed charge is now measured; it was not
inferred from the failed share's issued grant or browser cache.

Current restoration, inactive authority, mining disabled, same-page preservation
and native Close passed their recorded observations. Finalization independently
confirmed host processes/listener/serial holders released. Diagnostics retained
boot16, reset_reason=panic. The result is nevertheless `current_safe_recovery=false`
and `fresh_effect_proof=false`: authenticated current V2 status is absent and the
collector retains two failures.

## Reproduced software boundaries

- Diagnostics were saved, but the response named `diagnostics.json`. The actual
  Gate export consumer requires a `diagnostic-export-*.json` review filename and
  rejected this receipt. Data presence does not erase the failed operation.
- Collection obtained possession before Stop and reused that context for status.
  Gate Stop invalidates the prepared context, so the status operation returned
  `v2_possession`. Status must obtain fresh possession after Stop; the independent
  collection-session binding remains separate from that command context.

Fix each actual production boundary with a regression before another collection.
The next root must be separate and the corrected source/contract published first.
No current safety proof or historical resource release is inferred from this
partial result. The raw crash partition remains untouched.

## Evidence

Private root: `scratch/str005-share-recovery/recovery001/attempt`.
Inventory seal:
`d5f0ea17ef24002985f884266fcd0c2efd838a45c8453c3eeb9e04d3e827b337`.
Earliest failure: diagnostics/operation_failed; secondary: status/v2_possession.
`accounting_measured=true`, `host_resources_released=true`, qualification=false.
Share001's original seal is unchanged. Parity remains90/95; no task is archived.

A separate local UI observation showed a non-authoritative8192-byte allocation
failure with capability mask00000804 and stage runtime_ready. This observation
is a diagnostic lead, not a decoded stack or proof of the panic cause; the closed
recovery projection intentionally retained boot diagnostics only.
