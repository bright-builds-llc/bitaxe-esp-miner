# Parity work result

- Parity row: `UI-003`
- Final status: `verified`
- Implementation commit: `67c45e6f81c46910485373677e2a139d32b10d2a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The exact clean pushed implementation commit was packaged with `just package`.
Its manifest binds source commit
`67c45e6f81c46910485373677e2a139d32b10d2a`, pinned reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`, and app ELF SHA-256
`ab88a37ccd9a59c3cf6e5b0bd8d2d3a7cf725839d3acd072aa64e03fb5864239`.
One `just detect-ultra205` admitted exactly one repository-supervised ESP32-S3
USB device and passed its board-info gate. The selected port remained private.

The sole effectful command was:

`just input-uat --board 205 --port <detector-port> --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --private-root scratch/ui003-input/attempt-002 --plan docs/parity/work-plans/20260816T102741Z-UI-003/PLAN.md --projection docs/parity/evidence/ui003-input/input-uat-projection.json`

After the exact-package factory flash, the receive-only observer admitted two
monotonic runtime attestations and the exact source/reference input semantics,
then published its durable checkpoint. The operator briefly pressed and
released BOOT once after that checkpoint. The observer reported
`input_uat: verified`, released USB ownership, and published only
`docs/parity/evidence/ui003-input/input-uat-projection.json`.

The committed mode-`0644` projection has SHA-256
`5469f53e0fc20f4a1f6feb3788d3d126148c053568b0848b5969087c20ca334f`.
It binds board 205, the exact source/reference/package/plan identities, GPIO0
active-low pull-up ownership, 10 ms sampling, 30 ms debounce, the exact
2,000 ms long-press boundary, a checkpoint before input, exactly one physical
short click, production screen advance, no long press, complete cleanup,
disabled mining and hardware control, no retained serial transcript, and
passed redaction. `just validate-input-uat-evidence
docs/parity/evidence/ui003-input/input-uat-projection.json` accepted it.

Before hardware, verification included nine focused input-UAT Cargo tests,
strict package Clippy, the Bazel flash suite, the ordered all-target Cargo
format/lint/build/test sequence, Bright Builds checks, `just test`, parity and
progress validation, redaction, reference cleanliness, package construction,
the independent evidence-contract target, immutable-plan, projection-absence,
and diff checks. Focused tests cover arbitrary split markers, every terminal
runtime-attestation category, attempt admission, success, interruption,
cleanup, exact closed failure reasons, and projection withholding.

## Conclusion

The validator-accepted exact-package observation proves the provided Ultra 205
GPIO0 BOOT input produced one post-checkpoint short click through the retained
production input owner and routed it to screen advance. The sealed source and
reference semantics plus deterministic debounce/boundary tests prove the
bounded active-low pull-up and 10/30/2,000 ms classification contract. This
complete quorum supports promoting only `UI-003` from `implemented` to
`verified` with `unit,workflow,hardware-smoke` evidence.

## Non-claims and residual risks

This result does not claim a physical long press, configuration-AP transition,
self-test cancellation, identify cancellation, every display state, another
board, mining, hardware control, credentials, network behavior, OTA, recovery,
direct UART, BAP, or electrical pin/pad/header/probe work. It proves one normal
short click and its screen-advance route; the 2,000 ms long boundary remains
source-bound and deterministically tested rather than physically exercised.
Future changes to the admitted input owner, classifier, route, evidence
contract, package identity, or pinned reference require fresh evidence.
