# Phase 15 Secret Redaction Review

## Artifact Scope

This review applies to Phase 15 evidence artifacts under
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/`, including
serial logs, JSON artifacts, API responses, WebSocket captures, terminal output,
pasted output, manual observations, mining allow manifests, package manifest
excerpts, and final Markdown evidence.

Current status: pending - generated Phase 15 artifacts must be reviewed before
they are cited for checklist promotion.

## Review Checklist

- [ ] Pool credentials are absent or redacted.
- [ ] Worker names/secrets are absent or redacted.
- [ ] Wi-Fi credentials are absent or redacted.
- [ ] Private endpoints are absent or redacted.
- [ ] Private `DEVICE_URL` values are absent or redacted.
- [ ] API tokens are absent or redacted.
- [ ] NVS secret values are absent or redacted.
- [ ] Local terminal secrets are absent or redacted.
- [ ] Serial logs have been inspected for pool credentials, worker secrets,
  Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
  secret values, and local terminal secrets.
- [ ] JSON artifacts have been inspected for pool credentials, worker secrets,
  Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
  secret values, and local terminal secrets.
- [ ] Markdown evidence has been inspected for pool credentials, worker
  secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API
  tokens, NVS secret values, and local terminal secrets.
- [ ] API responses have been inspected for pool credentials, worker secrets,
  Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
  secret values, and local terminal secrets.
- [ ] WebSocket captures have been inspected for pool credentials, worker
  secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API
  tokens, NVS secret values, and local terminal secrets.
- [ ] Pasted output has been inspected for pool credentials, worker secrets,
  Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
  secret values, and local terminal secrets.
- [ ] Manual observations have been inspected for pool credentials, worker
  secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API
  tokens, NVS secret values, and local terminal secrets.

## Generated Artifact Review

| Artifact | Review status | Notes |
| --- | --- | --- |
| `bm1366-chip-detect` | pending | Review generated serial logs, JSON artifacts, command output, and Markdown summary before citation. |
| `bm1366-work-result` | pending | Review generated diagnostic work/result artifacts before citation. |
| `mining-smoke` | pending | Review pool category, share/no-share observations, telemetry artifacts, API responses, WebSocket captures, and safe-stop evidence before citation. |
| `bounded-soak` | pending | Review duration logs, thermal/power/watchdog observations, telemetry snapshots, reconnect/fallback notes, safe-stop evidence, and all generated artifacts before citation. |
| `parity-redaction` | pending | This file must be completed before redaction-governance claims are cited. |
| `final-ledger` | pending | Review the final Phase 15 ledger and checklist changes before citation. |

## Retained Evidence Fields

USB port, MAC address, source commit, reference commit, package path, checksum,
board-info output, controlled pool category, share/no-share outcome, and
bounded duration may be retained only when necessary for bench evidence and
after review. These fields are not credentials by themselves, but they can
identify a bench device or local environment and must be cited only where the
claim requires them.

## Review Notes

- Secret-pattern scan command:
  `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion`
- Expected matches must be reviewed manually because the templates intentionally
  name secret categories.
- Pool credentials, worker secrets, Wi-Fi credentials, private endpoints,
  private `DEVICE_URL`, API tokens, NVS secret values, and local terminal
  secrets must not be committed.
- If a generated artifact cannot be reviewed, record its component pack as
  blocked or `hardware evidence pending` and do not cite it for checklist
  promotion.
- Redaction review is artifact-specific. A passed review for one pack does not
  clear serial logs, JSON artifacts, Markdown evidence, API responses,
  WebSocket captures, terminal output, pasted output, or manual observations
  from another pack.

## Conclusion

Conclusion: pending. No Phase 15 generated evidence artifact is cleared for
citation until the relevant checklist items above are completed and the pack
review status is updated.
