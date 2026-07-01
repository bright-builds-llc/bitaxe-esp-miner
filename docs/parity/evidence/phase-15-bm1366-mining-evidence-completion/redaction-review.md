# Phase 15 Secret Redaction Review

## Artifact Scope

This review applies to Phase 15 evidence artifacts under
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/`, including
serial logs, JSON artifacts, API responses, WebSocket captures, terminal output,
pasted output, manual observations, mining allow manifests, package manifest
excerpts, and final Markdown evidence.

Current status: passed for cited artifacts - `bm1366-chip-detect`,
`bm1366-work-result`, `mining-smoke`, `bounded-soak`, `parity-redaction`, and
`final-ledger` passed redaction review. No API response bodies, WebSocket
captures, live-pool artifacts, pasted raw secrets, or additional manual
observation artifacts exist in Phase 15, and they are not cited for checklist
promotion.

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
| `mining-smoke` | passed | Reviewed `detect-ultra205.log`, `allow-mining-smoke.json`, `mining-smoke.log`, `mining-smoke.md`, and terminal output from detector, allow validation, and controlled wrapper commands. Expected retained bench metadata: USB port, MAC address, source/reference commits, package paths, controlled pool category, controlled no-share outcome, and safe-state markers. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |
| `bounded-soak` | passed | Reviewed `detect-ultra205.log`, `allow-bounded-soak.json`, `bounded-soak.log`, `bounded-soak.md`, and terminal output from detector, allow validation, and controlled wrapper commands. Expected retained bench metadata: USB port, MAC address, source/reference commits, package paths, controlled pool category, controlled no-share outcome, bounded duration, abort conditions, and safe-state markers. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |
| `parity-redaction` | passed | Reviewed `redaction-review.md` and the full evidence-tree secret-pattern scan. Expected retained wording names secret categories, bench metadata, missing prerequisite labels, and reviewed artifact paths. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |
| `final-ledger` | passed | Reviewed `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md` and its claim matrix. Expected retained wording names checklist rows, requirement IDs, absent live prerequisites, evidence paths, board, port, commits, and below-verified blockers. No pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, or local terminal secrets found. |

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
- 2026-07-01 mining-smoke scan reviewed expected matches from:
  `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke`.
  All matches were expected non-secret category labels, bench identifiers,
  package paths, controlled no-share status, and explicit missing-live-
  prerequisite labels retained for evidence.
- 2026-07-01 bounded-soak scan reviewed expected matches from:
  `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak`.
  All matches were expected non-secret category labels, bench identifiers,
  package paths, controlled no-share status, duration, abort conditions, and
  explicit missing-live-prerequisite labels retained for evidence.
- 2026-07-01 final-ledger and parity-redaction scan reviewed expected matches
  from:
  `rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion`.
  All matches were expected non-secret category labels, bench identifiers,
  firmware/platform labels, local wrapper paths, commit IDs, explicit missing
  live-prerequisite labels, and below-verified blocker wording retained for
  evidence.
- No API response bodies, WebSocket captures, live-pool artifacts, pasted raw
  secrets, or additional manual observation artifacts were generated in Phase
  15. They remain uncited and below verified.

## Conclusion

Conclusion: passed for cited artifacts. The `bm1366-chip-detect`,
`bm1366-work-result`, `mining-smoke`, `bounded-soak`, `parity-redaction`, and
`final-ledger` artifacts are cleared for their scoped citations. Absent
API/WebSocket/live-pool artifacts remain uncited and below verified.
