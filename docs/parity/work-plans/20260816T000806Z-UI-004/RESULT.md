# Parity work result

- Parity row: `UI-004`
- Final status: `verified`
- Implementation commit: `19d8f99fd5969c87d9a55b0fefa9558875e9f0fd`
- Captured attempt source: `bf5b74f98cdb117ca5682b0118a61743db85856f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The preserved exact-package Ultra 205 attempt and read-only browser session from
the immutable prior plan completed successfully. The prior terminal closure
withheld promotion only because shell-created projector redirects had mode
`0644`; it did not identify a device, browser, source, cleanup, or quorum
failure. This continuation corrected that evidence boundary without rerunning
hardware or the browser.

The source-bound projector independently admitted the protected operator and
browser evidence, all seven desktop routes, all seven mobile routes, mobile
navigation open and closed states, blank write-only secrets, guarded firmware
update behavior, same-origin requests, the log WebSocket, zero console errors,
zero unexpected request failures, restart completion, disabled mining and
hardware control, device/browser cleanup, owner-only private modes, and passed
redaction. It also joined the previously validated theme, settings, retained
log, partition, and rollback evidence.

The public evidence is
`docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json`
(SHA-256
`28aca7f12400ebaf6e3da5896e21f240b77954a265e27f2096b5dfdc2e234441`).
It binds the prior plan, prior closure, this immutable plan, captured source,
projector source, package/image identities, and a ten-path served-UI and static
serving compatibility set. Git ancestry, exact byte equality from the captured
source through the projector commit, clean paths, and a clean synchronized
projector worktree all passed.

The exact evidence commands were:

```text
(umask 077; just project-ui-workflow-evidence --private-root scratch/ui004-live-workflows/attempt-001 --attempt-source-commit bf5b74f98cdb117ca5682b0118a61743db85856f --operator-snapshot-projection scratch/ui004-live-workflows/wrapper-001/operator-snapshot-projection.private.json --browser-attestation output/playwright/ui004-attempt-001/browser-attestation.private.json --projection docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json > scratch/ui004-live-workflows/wrapper-001/projector-002.stdout 2> scratch/ui004-live-workflows/wrapper-001/projector-002.stderr)
(umask 077; just validate-ui-workflow-evidence docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json > scratch/ui004-live-workflows/wrapper-001/validator-002.stdout 2> scratch/ui004-live-workflows/wrapper-001/validator-002.stderr)
```

Both exited zero. The public projection is mode `0644`; all four ignored
projector/validator captures are mode `0600`. No protected capture content was
printed or copied into Git.

Before the projector, the following gates passed on the implementation commit:

- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- the forced uncached focused UI workflow Bazel suite;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test`, with all 45 Bazel test targets passing;
- `just parity`, with no validation errors, and `just parity-progress`;
- `just verify-redaction` and `just verify-reference`;
- `just build`, producing the ESP32-S3 firmware artifact; and
- immutable-digest, task-uniqueness, protected-mode, output-absence,
  compatibility-path, generated-contract, and diff checks.

## Conclusion

The closed evidence proves that the production Rust firmware served the
responsive operator UI and that the expected dashboard, network, pool,
settings, logs, update, and theme workflows rendered and behaved correctly on
the connected Ultra 205 in both desktop and mobile browser viewports. The
projector correction changes only evidence admission: the served UI/static
source remained byte-identical to the exact captured firmware source, and no
new hardware or browser observation was substituted.

## Non-claims and residual risks

This result does not verify physical panel or button behavior, mutation of
device settings during this continuation, firmware or OTAWWW upload success,
mining behavior, direct UART, electrical pin/pad/header access, non-205 boards,
long-duration browser use, external release readiness, or any display/input
claim. It publishes no origin, address, port, hostname, device identity, page
value, response body, frame, screenshot, trace, credential, pool value,
worker, token, or NVS secret.
