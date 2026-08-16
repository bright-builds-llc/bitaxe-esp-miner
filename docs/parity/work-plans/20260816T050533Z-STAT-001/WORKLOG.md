# Parity work log

## 2026-08-16T05:10:23Z | immutable attempt-005 plan pushed

- Source commit: `b37ebf1ab47a2af28882a142e5e06754893b5abd`
- Actions: Selected STAT-001 as the first actionable row after the pushed LiveShare observer correction; committed and pushed the immutable attempt-005 plan and task authorization as `81be00ad`.
- Verification: The plan-only ordered Rust sequence, Bright Builds checks, all 45 Bazel tests, parity validation/progress, redaction, pinned-reference verification, and canonical package build passed.
- Evidence: Immutable plan SHA-256 `c07d95b2ca7a7e064d4be8f5446cb551778cc535e8d02b5dd748f2bb5af71579`; pushed plan commit `81be00ada5e942344311aef6ffc138e5f8f80ec4`.
- Outcome: Attempt rebind implementation is authorized; device and credential access remain prohibited until that implementation is fully gated, committed, pushed, and repackaged.
- Blocker or next safe action: Rebind attempt-005 and strengthen the existing source admission for the millivolt core-voltage versus volt input-safety contract.

## 2026-08-16T05:37:17Z | attempt-005 rebind verified

- Source commit: `81be00ada5e942344311aef6ffc138e5f8f80ec4`
- Actions: Rebound private roots, immutable plan admission, attempt ordinal, Rust and generated TypeScript contracts, Bazel inputs, and real-process fixtures to attempt-005. Extended source admission from seven to ten paths to bind the conservative 400 MHz / 1,100 mV / 100% profile, volt-typed input-bus field and range, and upstream millivolt-to-volt control conversion.
- Verification: Four focused Rust evidence-contract tests passed; the explicit conservative-profile and input-voltage-boundary tests passed; generated contracts verified; filtered and full real-process automation passed; the campaign network suite passed 28/28; redaction, reference, and package gates passed; the ordered Rust sequence, Bright Builds checks, all 45 Bazel targets, parity validation, and parity progress passed.
- Evidence: The immutable plan digest remains `c07d95b2ca7a7e064d4be8f5446cb551778cc535e8d02b5dd748f2bb5af71579`. No detector, credential, USB, device, network runtime, or protected attempt path was accessed.
- Outcome: The attempt-005 evidence workflow is ready for an exact pushed source checkpoint. The unit boundary is explicit: core-voltage command `1_100` is millivolts; input bus voltage is finite volts within 4.5 through 5.5.
- Blocker or next safe action: Commit and push this verified implementation, rebuild the exact clean package, and only then run the plan's sole detector command.

## 2026-08-16T05:56:01Z | sole hardware attempt closed

- Source commit: `1090cf6eeb867345049ddb91cdcb7d5d382e264b`
- Actions: Rebuilt and validated the exact clean package; ran the frozen detector once; after successful one-device admission and closed credential/path checks, ran the sole attempt-005 capture once. No retry or additional hardware action ran.
- Verification: The sealed campaign-result v10 and network v4 digests joined; wrapper and attempt modes passed; the public projection remained absent. Closed evidence recorded trusted identity, zero parse failures, fresh safety, 11/20 windows, 310,615 active milliseconds, 61 HTTP successes, 298 WebSocket frames, zero transport failure counts, changing coherent positive hashrates and terminal zero on both transports, confirmed safe stop, terminal joins, persistence, and ready USB cleanup.
- Evidence: The wrapper returned `hardware_blocked` with `runtime_attestation_parse_failure: none`; the sealed terminal category was `watchdog_unresponsive`, with `watchdog_valid: false`. Raw protected artifacts, credentials, endpoints, identities, exact sensor values, and exact hashrates were not printed or promoted.
- Outcome: `stop_hardware_blocker`. Verification is not claimed; STAT-001 remains `implemented` and its checklist fields are unchanged.
- Blocker or next safe action: Add a software-only closed watchdog failure discriminator before considering another hardware ordinal. Attempt-005 is consumed and this plan authorizes no attempt-006.
