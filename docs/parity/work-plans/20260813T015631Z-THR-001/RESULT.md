# Parity work result

- Parity row: `THR-001`
- Final status: `verified`
- Implementation commit: `021c061b26494a665e35b1e3068ec5b6a2775261`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The required Cargo format, warning-denied clippy, all-target build, all-feature
tests, Bright Builds checks, 41-target Bazel suite, parity report and progress,
redaction, reference-cleanliness, generated-contract, immutable-plan,
unique-task, source/reference, candidate-absence, and diff gates passed. The
focused host suite passes all 295 tests, including the production-shaped real
child-process boundary. The private Rust validator additionally proves exact
`u64` acquisition-stamp handling through `u64::MAX` and rejects unequal,
negative, fractional, overflow, string, stale, unsafe, and invalid-envelope
inputs without emitting raw values.

The exact clean package built from pushed commit
`021c061b26494a665e35b1e3068ec5b6a2775261` and passed independent package
validation. One protected `just detect-ultra205` invocation admitted exactly
one board 205. The sole attempt-003 then ran the immutable plan's exact
`just capture-emc2101-thermal-evidence` command and published only
`docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json`.

The closed `bitaxe-emc2101-thermal-evidence-v1` projection binds attempt
ordinal 3, the exact source, pinned reference, package manifest, workflow, and
protected inputs by digest. It proves current production source ownership and
semantics for read-only EMC2101 address `0x4c`, internal-temperature register
`0x00`, and the Ultra 205 `+5 C` offset. It also proves one finite plausible
fresh below-throttle sample, exact lossless HTTP/WebSocket temperature, state,
acquisition-stamp, boot-session, and package correlation, detector admission,
observed boot, disabled mining and hardware control, complete cleanup, no
recovery flash, valid private modes, and passed redaction.

The public projection passed the independent Rust
`validate_emc2101_thermal_evidence` binary. Both protected roots are mode
`0700`, every contained file is mode `0600`, and no process holds the attempt
root after completion.

## Conclusion

The exact-package Ultra 205 hardware observation closes the remaining live
EMC2101 thermal-reading gap. Together with the pure thermal model and source
ownership, this supports THR-001 at `verified` for the bounded read-only Ultra
205 thermal path.

## Non-claims and residual risks

This result does not verify an overheat stimulus, thermal fault injection or
recovery, fan/voltage/frequency/power control, mining under thermal load,
long-duration sensor stability, intermittent I2C failures, other sensor paths,
other boards, OTA/recovery behavior, or release readiness. Raw detector, USB,
flash, serial, network, origin, credential, process, HTTP, temperature, and
acquisition-stamp material remains only in ignored protected roots.
