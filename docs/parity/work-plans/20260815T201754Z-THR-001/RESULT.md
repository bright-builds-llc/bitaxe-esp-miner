# Parity work result

- Parity row: `THR-001`
- Final status: `verified`
- Implementation commit: `e00b3665a20d6ab4b79a2ef952c8f137106e65e8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts after the replay-origin correction: one

## Evidence and verification

The detector-gated attempt-007 used the exact clean package built from the
pushed implementation commit. The bounded workflow ran these repo-owned
interfaces once each:

- `just detect-ultra205`
- `just capture-emc2101-thermal-fault-evidence --private-root scratch/thr001-emc2101-fault/attempt-007 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101-fault/wrapper-007/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-007.json --capture-timeout-seconds 120`

The committed
[`bitaxe-emc2101-thermal-fault-evidence-v1`](../../evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-007.json)
projection has SHA-256
`ab8852a111a8489a294218a956322685702a7373194e6f22291a94621a40dd5a`.
The independent Rust validator accepts it and proves:

- board 205, ordinal 7, detector admission, exact source/reference/application
  ELF/package identities, and the immutable plan digest;
- a healthy real EMC2101 baseline and continuing successful real reads while
  exactly five one-second typed invalid-temperature overlays were applied;
- the expected invalid-thermal-reading fault and one exact ordered
  `baseline_ready`, `fault_observed`, `recovered` witness through the admitted
  direct or retained replay firmware origins;
- consume-before-use intent handling, protected private modes, and current
  production source semantics;
- an ordinary exact-package restoration flash with fresh safe HTTP and
  WebSocket thermal truth below the throttle threshold, no remaining fault,
  and no replay of the consumed stimulus;
- mining disabled, hardware control disabled, complete cleanup, and redaction
  passed.

The following gates passed before hardware use:

- focused contract, flash-admission, and real-child automation tests;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just build` and `just test` (all 45 Bazel tests);
- `just parity`, `just parity-progress`, `just verify-redaction`, and
  `just verify-reference`;
- live-plan selection, stale-binding, generated-contract, sensitive-output,
  protected-mode, and `git diff --check` reviews.

After capture, the absolute-path independent validator, projection-to-package
identity join, redaction verifier, and OS holder checks passed. USB and workflow
process holder counts were both zero.

## Conclusion

The exact pushed Rust package demonstrated the narrow THR-001 fault model on a
detector-admitted Ultra 205: the production thermal owner retained healthy real
sensor acquisition while a bounded typed invalid-sample overlay drove the
expected fail-closed thermal fault, emitted an ordered machine witness, and
returned to fresh safe truth after ordinary exact-package restoration. This is
accepted `hardware-regression` evidence for THR-001.

## Non-claims and residual risks

The test uses a private one-shot NVS software stimulus; it does not claim
physical overheat, electrical EMC2101 open/short behavior, calibration
accuracy, thermal throttling under load, fan/voltage/frequency/power control,
mining, ASIC behavior, other boards, release readiness, external UART, or pin
manipulation. Raw temperatures, acquisition stamps, boot sessions, device and
network identities, credentials, origins, ports, commands, response bodies,
and traces remain only in ignored protected storage. The public projection
contains closed aggregate facts and cryptographic identities. Attempt-007 is
consumed and must never be retried.
