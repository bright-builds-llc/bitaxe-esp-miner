# Parity work closure

- Parity row: `API-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `942264a2dccbf729001c3c40024659424842c125735bb6817d7b6114dbb5cd20`
- Active task: `task-parity-api002-system-info-contract`

## Closure reason

The one authorized detector-gated attempt flashed the admitted exact package,
but the capture ended as `evidence_invalid` during `system_info_capture` and
produced no public projection. Aggregate private diagnostics identified a
repeatable ESP-IDF `main` task stack overflow after boot readiness began. The
workflow preserved the primary failure, completed its one permitted exact-
package recovery flash, confirmed cleanup, and stopped without a retry.

The software root cause was startup readiness constructing the full operator
API snapshot on the main task. The follow-up fix gives startup a platform-only
snapshot path, bounds the inline `ApiSnapshot` footprint, and extends the real
firmware disassembly audit with a 1 KiB platform-readiness frame limit. A clean
firmware build measures that frame at 480 bytes. This software verification is
not hardware evidence.

## Next safe action

Resume `API-002` only through a fresh immutable plan and task contract that
admits an exact package containing the stack fix and authorizes a new bounded
detector-gated attempt. The new attempt must independently prove stable boot,
exact build identity, coherent system-info capture, cleanup, and every existing
privacy and evidence condition before promotion.

## Non-claims

This closure does not verify live `/api/system/info` parity, stable runtime on
the connected Ultra 205, conditional block fields, persisted-setting values,
mining behavior, hardware control, another board, release readiness, or that
the software-only stack fix resolves the observed device failure. `API-002`
remains `implemented`.
