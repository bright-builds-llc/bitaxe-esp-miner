# Phase 23 Command Slot

slot: command
slot_status: passed
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: blocked-mode-workflow
detector_evidence: `just detect-ultra205` required for hardware mode
command_category: repo-owned-phase23-evidence
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

Repo-owned commands only. Ad hoc raw Stratum, raw BM1366, voltage-control, fan-control, erase, rollback, interrupted-update, or network scan commands are not accepted for Phase 23 evidence.

The allowed operator command shape is:

```bash
just phase23-evidence --evidence-root docs/parity/evidence/phase-23-redacted-operator-evidence-workflow --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --mode blocked
```

## Conclusion

The command slot supports the Phase 23 workflow claim only. Hardware mode remains detector-gated and must keep local credential files as runtime-only inputs.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
