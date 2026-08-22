# SELF-001 Full-Lifecycle Evidence

## Provenance

| Field | Value |
| --- | --- |
| Parity row | `SELF-001` |
| Immutable plan | `docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md` |
| Plan SHA-256 | `0c9a03ec490967fc95989d88b91848c7d4ed740a76825822a8107d94e8fd7f84` |
| Plan commit | `d358d74055449a5799d61cf2c4610a75f1e240f1` |
| Implementation commit | `a11b579b62cb52a53bbf6072bde209d3eb3f17e2` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Hardware attempts | `attempt-005` accepted; 001-004 terminal and documented |

## Implemented route

The Rust functional core encodes the pinned 485 MHz, 1200 mV, difficulty-16,
55/65/70 C, 30-second, four-domain, electrical, fan, hashrate, deadline, and
unreliable-counter contract. The firmware consumes a complete private NVS tuple
before effects, starts self-test instead of production mining/fan control,
drives only the retained safety and ASIC owners, produces deterministic work,
feeds the task watchdog, executes terminal safe-stop, and persists lease-bound
cancel/pass receipts across restart.

The built-in input owner reads live self-test state. A short click remains
ignored while active; a two-second long press is admitted only after the failed
state has completed safe-stop. No public self-test mutation route exists.

The host supervisor owns exact `start` and `resume` actions. It validates the
clean pushed package, detector, plan/task, protected paths, current settings,
and matching local credential inputs; snapshots restorable settings before the
first write; seeds exact one-shot intents; validates the safe checkpoint and
both receipts; restores settings through the confirmed route; and publishes
only `bitaxe-self-test-evidence-v1` after independent validation.

## Software verification

- Pure tests cover exact pass metrics, unreliable-domain retention, every
  safety/performance failure category, clock regression, overflow, and exact
  deadlines.
- Firmware guards prove consume-before-use ordering, self-test/production
  exclusion, single ownership, private markers/receipts, button routing,
  deterministic work, terminal safe-stop, and absence of a public route.
- Host tests execute the protected start/resume boundary through real child
  interfaces, settings restoration, physical-cancel receipt, pass receipt,
  projection validation, and malformed invocation rejection.
- The canonical ESP32-S3 firmware and six-file package build successfully.
- Ordered Cargo gates, Bright Builds, all 51 Bazel tests, parity/progress,
  redaction, reference, contract generation, file-size, sensitive-value, and
  diff checks pass.

## Hardware verification

Detector-admitted Ultra 205 attempt-005 completed both phases under one
lease-bound campaign. The controlled failure reached a safe checkpoint only
after stable diagnostic load and complete safe-stop; the built-in BOOT hold
produced a persisted cancellation receipt and restart. The passing phase met
the thermal, workload, total/domain hashrate, voltage, power, fan, watchdog,
safe-stop, pass-receipt, and restart contract. Settings and theme were restored
exactly with `mineonboot=false`; production mining and pool transport remained
absent; cleanup completed.

The independently validated projection is
`self-test-projection.json` with SHA-256
`1cbc357cb76c51bb354162e48f485442a3ce5eaeb03aead3b71ed50fd3235090`
and `redaction_status=passed`.

## Conclusion and non-claims

The production-safe route and exact two-phase hardware campaign pass. This
supports `SELF-001` at `verified` with
`unit,workflow,hardware-regression` evidence.

Actual overheat, zero-RPM, sensor, power, ASIC, or communication faults; other
boards; unbounded load; pool mining; external electrical interfaces;
OTA/recovery; and release readiness remain non-claims.
