# Device Noise completion exceeds the transport worker margin

Date: 2026-10-02. Owner: `task-device-noise-worker-stack`.
Method: offline static analysis only. No device, USB or network access occurred.

## Image

- Source: `998b50bf` (clean), built with `just build`.
- Device ELF: `bb3abf55ba018065b03e2434aabd816ed927e948f5fd8b53272e8eac8087f621`.
- Measurement: the repository frame parser and longest resolved crypto-family
  descent from `scripts/virtual-emulator/noise-stack-audit.mjs` (auditor v2).
  On the emulator, the same method was within a few hundred bytes of runtime:
  13,552 static against 13,368 bytes used in campaign003.

## Where Noise completes on the device

The device does not compile the retired `stratum_v2_session` module. Noise
completion runs in `bitaxe_stratum::v2::noise::diagnostic::run`, which calls
`complete_diagnostic_into`. Both entries that reach it run as jobs on the
`production_mining_session` transport worker, whose stack is 12 KiB
(`WORKER_STACK_BYTES`). With the repository's 2,048-byte margin, the budget is
10,240 bytes.

| Path                                       | Static bytes | Against 10,240 budget |
| ------------------------------------------ | -----------: | --------------------: |
| Share entry to certificate verification    |       11,424 |                −1,184 |
| Channel entry to certificate verification  |     ≥ 11,168 |                  −928 |
| Share entry to initiator preparation       |        5,088 |                +5,152 |

The channel entry is dispatched indirectly, so frames above it are not counted.
The share path leaves about 860 static bytes before the absolute end of the
12,288-byte stack.

## What dominates

The callers (`run_worker`, `v2::run`, `run_share_transport`, `run_share`,
`diagnostic::run`) total about 800 bytes. The rest is the crypto itself:
`complete_diagnostic_into` 1,168 bytes, the upstream `step_2_with_now` 3,472 bytes,
and Schnorr verification through Strauss and the odd-multiples table, about
4,800 bytes. Reshaping the callers cannot recover the deficit; the completion
frame or the worker stack must change.

## Limits

These are static lower bounds over named native paths. Indirect calls, calls
outside the crypto family and drop glue are not bounded. This does not identify
the Share002 or status001 cause, and it is not an observed overflow. It does put
Noise completion alongside the earlier signed-Start and renewal findings as a
concrete stack-pressure candidate on real hardware.

## Correction: completion on a PSRAM-stack helper

Internal RAM rules out growing the workers. Both 12 KiB workers start at boot,
and by the end of startup the device has only about 4.5–7.6 KB of internal/DMA
memory free, with a largest block of 2–7 KB. Reshaping callers cannot recover
the deficit either, because the completion return slot, the upstream
`step_2_with_now` and Schnorr verification alone exceed the budget.

The shared diagnostic now exposes `Observer::authenticate_act_two`. Its default
keeps the original inline completion. Both device observers override it with
`noise_completion_stack::authenticate_on_psram_stack`, which:

- applies `MALLOC_CAP_SPIRAM` stack caps through `esp_pthread_set_cfg` for one
  spawn, then restores the previous configuration;
- spawns a 16 KiB helper (its TCB, about 350 bytes, stays in internal RAM);
- runs only `complete_diagnostic_into` there, then joins and returns the
  unchanged result.

The helper performs no flash operation and never disables the cache, as PSRAM
stacks require. Socket I/O and every other step stay on the worker.

| Path (rebuilt device image)                 | Before | After  | Budget |
| ------------------------------------------- | -----: | -----: | -----: |
| Worker share entry, deepest crypto          | 11,424 |  5,088 | 10,240 |
| Worker channel entry, deepest crypto        | ≥11,168 |  4,832 | 10,240 |
| Worker share entry to helper spawn          |      — |  1,024 | 10,240 |
| Helper completion and certificate           |      — | 10,656 | 14,336 |

`just audit-device-noise-stack` re-measures these paths on any built image. It
fails if any function other than the helper entry directly calls completion
crypto, if the 12 KiB worker or the 16 KiB PSRAM helper contract changes, or if a
path exceeds its budget. Helper entry frames and indirect calls stay unbounded,
so this is still static evidence; only a device run can confirm runtime margin.
