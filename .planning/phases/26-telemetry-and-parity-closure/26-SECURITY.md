---
phase: 26-telemetry-and-parity-closure
type: security
asvs_level: project-standard
status: secured
threats_total: 16
threats_closed: 16
threats_open: 0
unregistered_flags: 0
generated_by: gsd-security-auditor
generated_at: 2026-07-05T04:56:00Z
---

# Phase 26 Security Verification

## Scope

This audit verifies the threat mitigations declared in the Phase 26 plan threat models. It does not introduce new implementation findings and did not modify implementation files.

All registered threats use disposition `mitigate`. No registered Phase 26 threats use `accept` or `transfer`, so no accepted-risk or transfer documentation entries were required.

## Threat Verification

| Threat ID | Category | Component | Disposition | Status | Evidence |
| --- | --- | --- | --- | --- | --- |
| T-26-01 | Tampering | `RuntimeTelemetryProjection::fold` | mitigate | CLOSED | `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:133` rejects non-monotonic sequences before mutation; `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:184` rejects stale `PoolSessionGeneration`; stale-generation tests at `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:429` and `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:487` prove the invariant. |
| T-26-02 | Tampering | Share counter gate | mitigate | CLOSED | `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:184` checks current generation before the accepted/rejected match; `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:188` and `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:194` are the only projection branches that call counter mutation helpers; non-share classification tests at `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:450` keep other outcomes from advancing counters. |
| T-26-03 | Repudiation | Safe-stop state reset | mitigate | CLOSED | `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:206` advances generation, disconnects lifecycle, and blocks work submission on `SafeStopped`; `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:487` proves safe stop prevents stale active mining and stale counter updates. |
| T-26-04 | Information Disclosure | Projection debug/log labels | mitigate | CLOSED | `crates/bitaxe-stratum/src/v1/telemetry_projection.rs:340` renders projection/event debug output and checks the denylist for pool, target, extranonce, share payload, socket, device, Wi-Fi, NVS, token, and raw BM1366 labels. |
| T-26-05 | Information Disclosure | `project_api_views` and WebSocket payloads | mitigate | CLOSED | `crates/bitaxe-api/src/runtime_projection.rs:21` builds views from upstream-compatible DTOs; `crates/bitaxe-api/src/runtime_projection.rs:37` serializes `SystemInfoWire`; `crates/bitaxe-api/src/runtime_projection.rs:210` proves denylisted pool/device/raw ASIC fields stay out of public JSON. |
| T-26-06 | Tampering | Statistics sample materialization | mitigate | CLOSED | `crates/bitaxe-api/src/runtime_projection.rs:30` materializes statistics samples only from `maybe_sample_marker`; `crates/bitaxe-api/src/runtime_projection.rs:111` proves repeated request-time reads without a marker stay empty; `firmware/bitaxe/src/runtime_snapshot.rs:278` drains pending markers only through the snapshot helper. |
| T-26-07 | Tampering | Scoreboard view builder | mitigate | CLOSED | `crates/bitaxe-api/src/runtime_projection.rs:36` keeps projected scoreboard entries empty; `crates/bitaxe-api/src/runtime_projection.rs:152` proves blocked, fake-pool-only, stale, stopped, and missing outcome states remain empty; `crates/bitaxe-api/src/runtime_projection.rs:275` proves the public scoreboard response remains an upstream empty array. |
| T-26-08 | Repudiation | `/api/ws/live` safe-stop payloads | mitigate | CLOSED | `crates/bitaxe-api/src/runtime_projection.rs:184` tests live telemetry cadence after safe stop and rejects stale `"active"` output; `firmware/bitaxe/src/http_api.rs:114` and `firmware/bitaxe/src/http_api.rs:764` feed `/api/ws/live` cadence/connect frames from `projected_live_telemetry_payload`. |
| T-26-09 | Tampering | `live_stratum_runtime` producer wiring | mitigate | CLOSED | `firmware/bitaxe/src/runtime_snapshot.rs:237` creates monotonic `RuntimeTelemetrySequence` values and folds typed events; `firmware/bitaxe/src/live_stratum_runtime.rs:502` publishes submit classifications through the projection helper; grep found no `record_accepted_share` or `record_rejected_share` direct mutations under `firmware/bitaxe/src`. |
| T-26-10 | Repudiation | Sample marker producer boundary | mitigate | CLOSED | `firmware/bitaxe/src/live_stratum_runtime.rs:367`, `firmware/bitaxe/src/live_stratum_runtime.rs:383`, `firmware/bitaxe/src/live_stratum_runtime.rs:480`, `firmware/bitaxe/src/live_stratum_runtime.rs:507`, and `firmware/bitaxe/src/live_stratum_runtime.rs:513` create sample markers only from runtime producer boundaries; `firmware/bitaxe/src/runtime_snapshot.rs:308` drains the pending marker once. |
| T-26-11 | Information Disclosure | Firmware projection/log labels | mitigate | CLOSED | `firmware/bitaxe/src/live_stratum_runtime.rs:471` and `firmware/bitaxe/src/live_stratum_runtime.rs:478` retain fixed redaction-safe status/blocker labels; `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md:24` records the committed evidence denylist and `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md:53` records `redaction_status: passed`. |
| T-26-12 | Repudiation | HTTP/WebSocket safe-stop serialization | mitigate | CLOSED | `firmware/bitaxe/src/runtime_snapshot.rs:77`, `firmware/bitaxe/src/runtime_snapshot.rs:83`, `firmware/bitaxe/src/runtime_snapshot.rs:89`, and `firmware/bitaxe/src/runtime_snapshot.rs:95` expose projection-backed route helpers; `firmware/bitaxe/src/http_api.rs:239`, `firmware/bitaxe/src/http_api.rs:346`, `firmware/bitaxe/src/http_api.rs:355`, `firmware/bitaxe/src/http_api.rs:114`, and `firmware/bitaxe/src/http_api.rs:764` route HTTP and live WebSocket serialization through those helpers. |
| T-26-13 | Information Disclosure | Phase 26 evidence files | mitigate | CLOSED | `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md:12` records `redaction_status: passed`; `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md:13` and `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md:14` record no raw artifacts or raw pool values committed; targeted denylist grep found only denylist category labels. |
| T-26-14 | Repudiation | `docs/parity/checklist.md` | mitigate | CLOSED | `docs/parity/checklist.md:88`, `docs/parity/checklist.md:92`, `docs/parity/checklist.md:129`, `docs/parity/checklist.md:130`, and `docs/parity/checklist.md:152` cite exact Phase 26 artifacts, redaction status, exact non-claims, and conservative statuses. |
| T-26-15 | Tampering | `tools/parity` validator | mitigate | CLOSED | `tools/parity/src/main.rs:1052` implements `validate_phase26_telemetry_verified_row`; `tools/parity/src/main.rs:1060`, `tools/parity/src/main.rs:1067`, `tools/parity/src/main.rs:1074`, `tools/parity/src/main.rs:1081`, `tools/parity/src/main.rs:1089`, and `tools/parity/src/main.rs:1096` reject missing evidence, blocker language, missing redaction, missing exact non-claims, fabricated statistics, and unsupported scoreboard rows; regression tests start at `tools/parity/src/main.rs:2023`. |
| T-26-16 | Elevation of Privilege | Hardware verification path | mitigate | CLOSED | `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md:9`, `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md:20`, and `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md:75` record static-only detector/board-info state and `hardware_evidence_status: blocked_or_not_run`; `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md:77` keeps accepted/rejected shares and hardware-control surfaces as exact non-claims. |

## Threat Flags

No unregistered threat flags were found in the Phase 26 summaries:

- `26-01-SUMMARY.md`: no new network endpoints, auth paths, file access, or schema boundaries.
- `26-02-SUMMARY.md`: no new network endpoints, auth paths, file access, or schema changes.
- `26-03-SUMMARY.md`: reused existing HTTP/WebSocket routes with no new trust-boundary surfaces.
- `26-04-SUMMARY.md`: added planned evidence and parity guardrails only.

## Accepted Risks

None. No Phase 26 threat model entries used disposition `accept`.

## Transfer Documentation

None. No Phase 26 threat model entries used disposition `transfer`.

## Open Threats

None.

## Conclusion

All Phase 26 registered threat mitigations are present in the implementation or committed evidence artifacts. The security status for Phase 26 is `SECURED` with `threats_open: 0`.
