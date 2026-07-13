---
phase: 16-current-commit-release-evidence-completion
plan: "06"
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-07-01T12-36-46
lifecycle_validated: true
status: passed
generated_at: 2026-07-01T15:34:20Z
---

# Phase 16 Plan 06 Final Verification

## Conclusion

Phase 16 final verification passed after the release-evidence identity gate was
tightened to support one explicit finalization pattern: a flashed package source
commit may be followed only by allowlisted evidence, release-documentation, and
GSD lifecycle commits. The strict default still rejects a package manifest whose
`source_commit` differs from current `HEAD`; the Phase 16 final command uses
`--allow-post-source-evidence-commits` and rejects any post-source code, tool,
firmware, script, or non-allowlisted path.

The current Phase 16 release-candidate package, release gate, detector-gated
serial boot evidence, redaction review, parity report, reference guard,
reference diff, Rust checks, Bazel tests, final release-evidence validator, and
GSD lifecycle validation passed.

This is not live HTTP, firmware OTA, rollback, erase, failed-update,
interrupted-update, or recovery proof. Those surfaces remain blocked or pending
where the evidence artifacts say they are blocked or pending.

## Command Results

| Command | Result | Notes |
| --- | --- | --- |
| `bash -n scripts/phase16-http-static-smoke.sh scripts/phase16-recovery-regression.sh` | passed | Shell syntax clean. |
| `bazel test //scripts:phase16_http_static_smoke_test //scripts:phase16_recovery_regression_test` | passed | Phase 16 helper tests passed. |
| `cargo test -p bitaxe-parity --all-features release_evidence` | passed | Focused release-evidence tests passed. |
| `bazel test //tools/parity:tests --test_filter=release_evidence` | passed | Bazel-filtered release-evidence tests passed. |
| `cargo fmt --all` | passed | Rust formatting clean. |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed | No clippy warnings. |
| `cargo build --all-targets --all-features` | passed | Workspace builds. |
| `cargo test --all-features` | passed | Workspace tests and doc-tests passed. |
| `just test` | passed | `bazel test //...` passed. |
| `just package` | passed | Generated `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`. |
| `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | passed | `release_gate: passed`. |
| `bazel run //tools/parity:report -- release-evidence --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --evidence-root docs/parity/evidence/phase-16-current-commit-release-evidence-completion --flash-evidence-json docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json --redaction-review docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md --require-redaction-passed --allow-post-source-evidence-commits` | passed | Final evidence identity accepted only because post-source changes are allowlisted evidence/docs/GSD closure paths. |
| `just parity` | passed | Parity report has no validation errors. |
| `just verify-reference` | passed | Reference guard reported pinned reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff -- reference/esp-miner --exit-code` | passed | No local reference implementation diff. |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 16 --expect-id 16-2026-07-01T12-36-46 --expect-mode yolo --require-plans --require-verification --raw` | passed | Lifecycle validator returned valid after this verification artifact was updated to `status: passed`. |

## Evidence Package Identity

The flashed Phase 16 evidence manifest at
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json`
records:

| Field | Value |
| --- | --- |
| source_commit | `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca` |
| observed firmware marker | `8490118a7e7f` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| `bitaxe-ultra205.elf` SHA-256 | `8d36ec4c19d1961fb540155be935f25c5f7fdf2622ccf8c9ebea7bb7a738b46f` |
| `esp-miner.bin` SHA-256 | `8113d28ca505ef3839f2f47757e905a0d3404f96c184b29b5108998f91b99320` |
| `www.bin` SHA-256 | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` SHA-256 | `d000050754659b52410658513264d8fa1b6667ee0ecacfd6dae6f8e210d082c7` |
| partition table SHA-256 | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| `otadata-initial.bin` SHA-256 | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

The final validator accepts this manifest after later evidence/doc closure
commits only because `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca` is an ancestor
of current `HEAD` and every post-source changed path is inside the explicit
Phase 16 evidence/documentation/GSD allowlist.

## Hardware Command Inventory

The final ledger cites these Phase 16 hardware/network/recovery commands and
artifacts:

| Plan | Command or gate | Result |
| --- | --- | --- |
| 16-02 | `just detect-ultra205` | Passed; exactly one Ultra 205 port and board-info succeeded. |
| 16-02 | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35` | Passed for release-candidate source commit `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca`. |
| 16-03 | `scripts/phase16-http-static-smoke.sh ...` | Blocked because `DEVICE_URL` was missing; no network scan ran. |
| 16-04 | `scripts/phase13-firmware-ota-smoke.sh ...` | Blocked before upload because `DEVICE_URL` was missing. |
| 16-05 | `scripts/phase16-recovery-regression.sh ...` | Pending because destructive allow flags were omitted; no failed-update upload, large erase, factory reflash, interrupted upload, rollback, or recovery action ran. |

## DEVICE_URL And Recovery Gates

`DEVICE_URL` remains missing for Phase 16. Live HTTP/static/recovery/API,
WebSocket, firmware OTA, invalid rejection, post-OTA identity, rollback, and
OTAWWW response evidence did not run.

Destructive and fault-injection operations remain pending. No failed-update
upload, large erase, factory restore, interrupted update, raw erase, raw write,
rollback proof, or post-restore HTTP/static proof was captured.

## Redaction Status

`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md`
records `redaction_status: passed`. The final review listed absent artifacts as
`absent - not cited` and found no private endpoint, pool credential, worker
secret, Wi-Fi credential, API token, NVS secret value, or local terminal secret.

## Reference Cleanliness

`just verify-reference` passed and printed reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.
`git diff -- reference/esp-miner --exit-code` also passed with no diff.

## Passed

- Phase 16 package and release gate passed.
- Phase 16 detector-gated serial boot evidence passed for source commit
  `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca`.
- Final release-evidence validation passed with explicit post-source
  evidence-commit allowance.
- Redaction review passed for all cited Phase 16 artifacts.
- Parity, reference guard, reference diff, Rust checks, Bazel helper tests,
  aggregate tests, package, release gate, and lifecycle validation passed.

## Blocked Or Pending Evidence

- Live HTTP/static/recovery/API/WebSocket evidence is blocked by missing
  `DEVICE_URL`.
- Firmware OTA is blocked by missing `DEVICE_URL`; no upload ran.
- Rollback and boot-validation proof remain blocked because OTA did not run.
- Failed-update, large-erase, interrupted-update, factory restore, and
  post-restore proof remain pending because allow flags were omitted.
- OTAWWW remains the REL-03 gap and below verified.

## Residual Risks

- The final release-evidence validator now depends on the post-source path
  allowlist staying narrow; future code, tool, firmware, script, or generated
  package changes after a flashed evidence source must still force new package
  and serial evidence.
- A reachable explicit `DEVICE_URL` is still required before live route, OTA,
  recovery, and OTAWWW evidence can be captured.
- Destructive recovery operations still require documented allow flags,
  detector/board-info gates, current factory image, abort steps, restore steps,
  and redaction review before any verified claim.

## Lifecycle Validation

Lifecycle validation command:

```bash
node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 16 --expect-id 16-2026-07-01T12-36-46 --expect-mode yolo --require-plans --require-verification --raw
```

Result: `valid`.
