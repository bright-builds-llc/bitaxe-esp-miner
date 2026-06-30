# Phase 13 HTTP Static Recovery Evidence

## Command Log

- command: `scripts/phase13-http-static-smoke.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery`
- http_static_status: blocked
- DEVICE_URL status: blocked - missing DEVICE_URL
- board: `205`
- selected port from Plan 13-02: `/dev/cu.usbmodem1101`
- source commit from package manifest: `190849539700b8f9a7909fd2b6ebd84142557968`
- reference commit from package manifest: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- sanitized DEVICE_URL source: not provided
- smoke log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery/http-static-smoke.log`

## Blocker

Live HTTP/static/recovery evidence is blocked because `DEVICE_URL` was not set for the just-flashed Ultra 205. Per D-06, the helper did not scan the network, infer a target from serial route registration, or promote live HTTP success from package or serial evidence.

The generated log records:

```text
DEVICE_URL status: blocked - missing DEVICE_URL
http_static_status: blocked
conclusion: blocked - live HTTP/static/recovery evidence requires an explicit DEVICE_URL
```

## Route Evidence

Because no explicit target URL was available, every live route probe below is not run and remains below verified.

| Request | Expected status/body marker | Actual status | Selected headers | Body/result | Route conclusion |
| --- | --- | --- | --- | --- | --- |
| `GET /` | `200` with `AxeOS unavailable`, `Open recovery`, and `Release metadata` | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /assets/app.css.gz` | `200` static CSS asset response with cache/static headers | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /phase13-missing-static` | missing static redirect with body `Redirect to the captive portal` | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /recovery` | `200` with `AxeOS Recovery` and `Response:` page markers | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /api/system/info` | known API route coexists with static wildcard | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /api/phase13-unknown` | `404` JSON body `{"error":"unknown route"}` | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /api/ws` | bounded WebSocket route response, not static wildcard | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `GET /api/ws/live` | bounded live WebSocket route response, not static wildcard | not run | not captured | blocked before curl | blocked - missing DEVICE_URL |
| `POST /api/system/OTAWWW` | `400` body `Wrong API input` | not run | not captured | blocked before curl | blocked - missing DEVICE_URL; REL-03 gap remains open |

## Recovery Page Evidence

`/recovery` page-load evidence was not captured because the device had no explicit reachable `DEVICE_URL`. The existing serial boot evidence still proves route registration only; it does not prove live recovery page behavior.

## OTAWWW REL-03 Gap

`/api/system/OTAWWW` remains the explicit REL-03 gap. Expected public response when live HTTP evidence is reachable is `Wrong API input`; this plan did not observe that response because `DEVICE_URL` was missing. Package generation of `www.bin`, recovery page copy, and serial route registration are not whole-`www` update parity evidence.

Owner: `phase-07-release`

Follow-up path: implement whole-`www` partition update with interruption/recovery evidence before any OTAWWW verified claim.

## Redaction

redaction: passed for `http-static-smoke.log` and this Markdown summary. No route headers, route body snippets, private endpoints, credentials, tokens, NVS secret values, pool data, or raw terminal secrets were generated because the helper stopped before curl.

## Conclusion

Conclusion: http_static_status: blocked - live HTTP/static/recovery evidence requires an explicit, reachable `DEVICE_URL` for the just-flashed Ultra 205. Checklist rows for live static, recovery, API coexistence, WebSocket coexistence, and OTAWWW response must remain below verified.
