# Parity work plan

- Run ID: `20260821T180800Z-SELF-001`
- Parity row: `SELF-001`
- Initial status: `implemented`
- Source commit: `f4f9757319c9580acbb35c168d79beb949a14a60`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-self001-full-lifecycle`

## Selection

The clean synchronized selector returned no open plan and ordered `ASIC-009`,
`ASIC-010`, `SELF-001`, `BAP-001`, and `BAP-002`. The two ASIC rows require
unavailable BM1368/BM1397 boards. Both BAP rows require a compatible accessory
and external electrical authorization. `SELF-001` is the first row actionable
with the connected Ultra 205 after the user explicitly selected full lifecycle
evidence, private one-shot boot markers, a physical two-second BOOT-button
cancel path, controlled failure after stable load, and exact settings
restoration.

## Scope and non-scope

Implement one production-safe self-test owner that starts only from a complete
consume-before-use NVS tuple. It replaces the ordinary production-mining and
fan-controller owners for that boot, retains read-only Wi-Fi/API observability,
and exclusively owns validated safety/ASIC effects. Ordinary boots without the
tuple remain unchanged. No public self-test mutation endpoint is allowed.

The pure lifecycle must cover admission, preflight, preparation, warm-up,
diagnostic work, measurement, evaluation, safe-stop, failed wait, physical
cancel, pass, flag policy, and restart. The firmware must publish closed,
redacted runtime state and a lease-bound terminal receipt across restart.

The hardware campaign has two phases in one protected root. Phase A runs five
stable seconds of deterministic diagnostic work, injects only
`planned_evaluation_failure`, completes safe-stop, and exits at an unbounded
human checkpoint. The user then holds the built-in BOOT button for two seconds.
Phase B resumes only after proving cancellation and restart, runs the complete
upstream-compatible pass path, observes automatic restart, restores settings,
and independently validates evidence.

No sensor, thermal, power, fan, ASIC, communication, or electrical fault is
injected. No external UART, pins, pads, probes, jumpers, accessories, pool
connection, share submission, OTA, erase-flash, arbitrary raw write, or
unbounded load is authorized or claimed.

## Implementation

- [ ] Expand the pure lifecycle with exact Ultra 205 profile, stage, deadline,
      metric, terminal, failure, cancel, and receipt types plus focused tests.
- [ ] Add consume-before-use self-test admission, exclusive firmware ownership,
      deterministic BM1366 work, watchdog/telemetry projection, button routing,
      safe-stop, and restart receipts.
- [ ] Add `self-test-campaign start|resume`, settings reconstruction and exact
      restoration, protected evidence, independent validation, and real-child
      tests.
- [ ] Pass every software/package/privacy gate, commit and push exact source,
      then run one detector and the exact two-phase hardware campaign.
- [ ] Promote only on the complete independently validated quorum; otherwise
      preserve the earliest failure, restore safely, withhold `RESULT.md`, and
      stop without unchanged retry.

## Hardware contract

After implementation is clean, verified, committed, pushed, and package-bound,
run only:

1. `just package`
2. `test ! -e scratch/self001-full-lifecycle/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/self001-full-lifecycle/wrapper-001 && just detect-ultra205 > scratch/self001-full-lifecycle/wrapper-001/detector.stdout 2> scratch/self001-full-lifecycle/wrapper-001/detector.stderr)`
3. `test ! -e scratch/self001-full-lifecycle/attempt-001 && test ! -e docs/parity/evidence/self001-full-lifecycle/self-test-projection.json && (umask 077; just self-test-campaign start --private-root scratch/self001-full-lifecycle/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/self001-full-lifecycle/wrapper-001/detector.stdout --plan docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json > scratch/self001-full-lifecycle/wrapper-001/start.stdout 2> scratch/self001-full-lifecycle/wrapper-001/start.stderr)`
4. Only after the start command publishes a live safe checkpoint and the user
   performs the exact built-in BOOT hold: `just self-test-campaign resume --private-root scratch/self001-full-lifecycle/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --detector-output scratch/self001-full-lifecycle/wrapper-001/detector.stdout --plan docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`

The start command may install the exact package, seed only the reconstructed
configuration plus private self-test tuple, run Phase A, and reach the safe
checkpoint. The resume command may validate the cancellation receipt, seed
Phase B, observe the pass and restart, restore exact settings with
`mineonboot=false`, clean up, and publish only the redacted projection.

Exact active envelope: PSRAM present; 485 MHz; 1200 mV; fan 100% initial proof;
10-100% PID; warm to 55 C within 180 seconds; target 65 C; immediate stop above
70 C; 30-second workload at difficulty 16; total hashrate at least 85% of the
485 MHz x 894-core expectation; four domain averages within +/-33% with
upstream unreliable-counter handling; input 4.5-5.5 V; core voltage +/-10%;
power at most 15 W; final fan speed above 1,000 RPM. Safe-stop must disable
dispatch, reduce frequency/reset nonce, hold reset low, disable voltage and
ASIC, set fan 100%, cool to at most 45 C within 120 seconds, then set fan 30%.

The human wait has no elapsed deadline. Before it begins, safe-stop, cleanup,
and a self-describing checkpoint must be complete. Typed decline and resume are
required. Any missing settings/credentials, PSRAM, safety truth, detector,
identity, package, marker, receipt, restart, restoration, privacy, or cleanup
fact stops at the earliest boundary. One campaign only; no unchanged retry.

## Verification and promotion

Run focused pure, firmware ownership, NVS admission, process-boundary,
restoration, evidence, and redaction tests; build/package the real ESP32-S3
image; then run the ordered Cargo gates, Bright Builds, all Bazel tests,
`just parity`, `just parity-progress`, redaction, reference, selector,
sensitive-value, file-size, and diff checks.

Hardware acceptance requires both phases, one admitted board 205, exact
source/reference/package identity, advancing watchdog and safety observations,
complete rollback, physical cancel and restart, complete pass and auto-restart,
exact settings restoration, no pool traffic, cleanup, independent validation,
and redaction. On success create `RESULT.md`, transition only `SELF-001` to
`verified` with `unit,workflow,hardware-regression`, sync progress, archive the
task, final-verify, commit, and push.

## Non-claims

Actual overheat, zero-RPM, sensor, power, ASIC, or communication fault
injection; other boards; unbounded load; pool mining; external electrical
interfaces; OTA/recovery; and release readiness remain unverified.
