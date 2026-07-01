# Phase 15 Secret Redaction Review

## Artifact Scope

This review applies to Phase 15 evidence artifacts under
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/`, including
serial logs, JSON artifacts, API responses, WebSocket captures, terminal output,
pasted output, manual observations, mining allow manifests, package manifest
excerpts, and final Markdown evidence.

Current status: partial - `bm1366-chip-detect` and `bm1366-work-result`
passed redaction review; generated artifacts for later Phase 15 packs remain
pending before citation.

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
| `bm1366-chip-detect` | passed | Reviewed `detect-ultra205.log`, `allow-chip-detect.json`, `diagnostic-package-summary.json`, `package/bitaxe-ultra205-package.json`, `flash-command-evidence.json`, `flash-monitor.log`, `chip-detect.md`, and terminal output from diagnostic package, allow validation, and flash-monitor commands. Expected retained bench metadata: USB port, MAC address, WiFi feature label, NVS partition/status labels, PSRAM pool wording, source/reference commits, package paths, checksums, local absolute wrapper paths, and chip-detect serial markers. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |
| `bm1366-work-result` | passed | Reviewed `detect-ultra205.log`, `allow-work-result.json`, `diagnostic-package-summary.json`, `package/bitaxe-ultra205-package.json`, `flash-command-evidence.json`, `flash-monitor.log`, `work-result.md`, and terminal output from diagnostic package, allow validation, and flash-monitor commands. Expected retained bench metadata: USB port, MAC address, WiFi feature label, NVS partition/status labels, PSRAM pool wording, source/reference commits, package paths, checksums, local absolute wrapper paths, and work-result serial markers. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |
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
- 2026-07-01 chip-detect scan reviewed expected matches from:
  `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect`.
  All matches were expected non-secret category labels, bench identifiers, or
  firmware/platform status strings retained for evidence.
- 2026-07-01 work-result scan reviewed expected matches from:
  `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result`.
  All matches were expected non-secret category labels, bench identifiers,
  toolchain versions, or firmware/platform status strings retained for
  evidence.

## Conclusion

Conclusion: partial passed. The `bm1366-chip-detect` and
`bm1366-work-result` packs are cleared for their diagnostic citations.
`mining-smoke`, `bounded-soak`, `parity-redaction`, and `final-ledger` remain
pending until their generated artifacts are reviewed.
