# STR-005 negative-test and native-coverage inventory

The [integration review](20261005-str005-integration-review.md) needs an
inventory of the negative tests and native resource checks behind each STR-005
checkpoint ([ADR-0029](../../adr/0029-piecewise-str005-qualification.md)).
This is that inventory for the final candidate.

| Item          | Identity                                                                       |
| ------------- | ------------------------------------------------------------------------------ |
| Firmware      | `2bff100442a7c5cbd4b806bdbbceff00f0c06a79`                                     |
| App ELF       | `9783dc74dfb369e633f9f3295021c34eefbd36fb50859a0d58a235447189b33a`             |
| Gate          | `86fc62d7a9d75da1affa2d51bc3b9eab41d86031`                                     |
| Test baseline | Repository `0baa3753`: `bazel test //... --local_test_jobs=4`, 296 of 296 pass |

At the default test parallelism, four slow tests timed out under load
(`str005_v2_serial_context_test`, `str005_v2_serial_operator_execution_test`,
`str005_v2_serial_cleanup_rehearsal_test`,
`usb_bootstrap_measure_regression_process_test`), and the firmware build
target failed while competing with them. The firmware build passes alone, and
the whole suite passes at four parallel tests. No firmware, crate or Cargo
file changed between the candidate and `0baa3753`.

## Negative tests

All negative tests are host-side: they run the production logic against
fakes, fixtures and the pinned Gate package, not the board. Counts are
approximate (tests whose names assert a refusal, failure or bound).

| Checkpoint                               | Negative tests | Main targets                                                                                                                                                                              | Covers                                                                                                                                 |
| ---------------------------------------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Install continuity, Noise authentication | ~110           | `//scripts:str005_noise_serial_*_test`, `//tools/stratum-v2-fixture:noise_serial_test`, `//crates/bitaxe-stratum:tests`, `//crates/bitaxe-simulation:tests`                               | wrong authority and tampered transport, nonce exhaustion, malformed frames, deadlines, cleanup preservation, ledger drift, privacy     |
| Startup, Start, dispatch, normal Stop    | ~150           | `//scripts:str005_startup_probe_test`, `//crates/bitaxe-worker-control:controller_tests`, `:authorization_tests`, `:framing_tests`, `//firmware/bitaxe:production_worker_admission_tests` | bad signatures, session/sequence/replay, Start admission, Stop and cleanup failures                                                    |
| Accepted share                           | ~50            | `//scripts:str005_share_probe_test`, `//tools/stratum-v2-fixture:v2_serial_test`, `//crates/bitaxe-stratum:tests`, `//crates/bitaxe-asic:tests`                                           | stale, duplicate and below-target shares, wrong channel, forged or missing acknowledgement, malformed ASIC results, deadlines          |
| Renewal                                  | ~13            | `//crates/bitaxe-worker-control:controller_tests`, `:authorization_tests`, `//scripts:str005_share_probe_test`                                                                            | replay, bad signature, foreign lease, non-canonical V2 window, renewal after expiry, no active lease, third renewal, missing minimum   |
| Heartbeat-loss revocation and shutdown   | ~65            | `//crates/bitaxe-runtime:tests`, `//scripts:str005_heartbeat_probe_test`, `//firmware/bitaxe:mining_actuation_tests`                                                                      | the 2,800 ms cutoff bounds, wrong generation, late signals, shutdown and cooling order, reason classification                          |
| Recovery, accounting, restoration        | ~75            | `//scripts:str005_share_recovery_test`, `:str005_panic_recovery_test`, `:str005_recovery_*_test`, `//crates/bitaxe-worker-control:qualification_budget_tests`                             | no refund or replay of the ledger, missing joins, cleanup preservation, panic and reset classification, refused effect routes, privacy |
| Internal-heap headroom                   | ~16            | `//scripts:internal_heap_series_test`, `//tools/automation:automation_test`, `//firmware/bitaxe:usb_startup_diagnostics_tests`                                                            | exhausted samples fail, the resolved PSRAM-first contract, the heap line only outside a session                                        |

The tests that need the Gate source run in the Bazel suite: the targets pass
the pinned Gate package or fixtures, and their logs show no skips. They skip
only under a bare `node --test`.

### Added by this inventory

The survey found three renewal refusal branches without a test. They are now
covered in `crates/bitaxe-worker-control/tests/controller/renew_dispatch.rs`:

- a renewal for another lease fails authentication and restores first;
- a V2 renewal with a non-canonical window is refused before the session
  renews (30 s/10 s and 60 s/20 s-cadence variants, both of which pass
  general validation); removing the V2 rule makes this test fail;
- a renewal after the lease expired never reaches the session.

## Native resource checks on the candidate ELF

| Check                                 | Result on `9783dc74`                                            | Where it is bound                                                            |
| ------------------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Panic cutoff (`native-audit`)         | pass: wrapper and literals in IRAM, revoked before delegate     | sealed in heartbeat010 and renew-current-002 preflights                      |
| Signed Start stack (`start-audit`)    | 3,104 bytes headroom against 2,048 required                     | sealed in heartbeat010 and renew-current-002 preflights                      |
| Core-dump store seams (`store-audit`) | pass: 5 seams in IRAM, at most 240 of 256 bytes added           | sealed in heartbeat010 and renew-current-002 preflights                      |
| Fault provenance (`provenance-audit`) | pass: cache-safe hot closure, no forbidden calls                | sealed in heartbeat010 and renew-current-002 preflights                      |
| Noise, owner, telemetry and image fit | `selected_native_checks_passed`; image 4,067,184 bytes in 4 MiB | sealed in psram-default-install attempt-002 (`native/readiness.json`)        |
| Stack realignment                     | `no_runtime_realignment_callers`                                | [`stack-realignment.json`](str005-candidate-native/stack-realignment.json)   |
| Startup frames                        | `startup_frames_within_budget`                                  | [`startup-frames.json`](str005-candidate-native/startup-frames.json)         |
| Control stack                         | 4,256 bytes headroom against a 14,336 budget                    | [`control-stack.json`](str005-candidate-native/control-stack.json)           |
| Device-Noise helper stack             | completion only on the PSRAM helper; all paths within budget    | [`device-noise-stack.json`](str005-candidate-native/device-noise-stack.json) |
| Signed Renew stack                    | 4,608 bytes unclaimed headroom; Renew dispatch isolated         | [`signed-renew-stack.json`](str005-candidate-native/signed-renew-stack.json) |
| Native USB symbols                    | `native_usb_symbols=verified` (verifier `8f45f272…`)            | candidate preflights; rerun on the retained ELF for this inventory           |
| Resolved sdkconfig contracts          | memory (PSRAM-first, 96 KiB reserve), core dump, BBPLL          | package build, tested in `//tools/automation:automation_test` and `build.rs` |

The last five audit outputs were produced for this inventory from the ELF
retained in psram-default-install attempt-002; each records
`elf_sha256 = 9783dc74…`. Before this inventory, the stack-realignment and
startup-frames passes were recorded only as task text, and the control,
device-Noise and signed-Renew audits had not run on the candidate.

## Limitations

- **Host-only negatives.** No hardware run exercises a rejected share or a
  refused renewal. The only negative path exercised on the board is
  heartbeat loss (heartbeat010).
- **Heap threshold after the fact.** The 16 KiB / 8 KiB headroom criterion is
  judged on passive captures; no firmware check enforces it on the device.
- **`build.rs` required defaults** fail closed by panicking the build; there
  is no separate negative test for that list. The memory and core-dump
  values are also checked by the tested resolved-config contract.
- **Load-sensitive tests.** Four slow host tests time out at full parallelism
  on this host; they pass at four parallel tests.
