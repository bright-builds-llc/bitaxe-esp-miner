# UI-004 work result

- Parity row: `UI-004`
- Final status: `implemented`
- Implementation commit: `89564440d56f174164666667200254baa2aa9d0e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The independent static interface provides a dark-first responsive operator
shell with accessible navigation for dashboard, network, pool, settings, logs,
firmware update, and theme workflows. Known direct routes, including the
configuration and system aliases, resolve to the production index through the
pure static planner while unknown assets retain the existing redirect and path
traversal remains rejected before filesystem access.

The browser core admits only known routes, formats unavailable and numeric
telemetry explicitly, builds settings patches from per-form allowlists, omits
blank write-only secrets, and returns closed public error text. The same-origin
adapter owns HTTP timeouts, JSON admission, command allowlisting, retained logs,
the raw log WebSocket, theme persistence, and the firmware-only upload route.
It does not expose response bodies in errors or use browser storage.

The DOM adapter renders device values with `textContent`, bounds the retained
log view, keeps password fields blank after reads and writes, requires
confirmation for commands and firmware upload, and accepts only a selected
`esp-miner.bin`. Whole-`www.bin` OTA remains visibly unavailable, and active
frequency, voltage, fan, power, erase, rollback, and recovery-fault controls are
absent. Closed mobile navigation is inert until the operator expands it.

The following gates passed on the implementation commit:

- nine focused static-route tests, including every admitted direct route,
  missing-file behavior, gzip selection, recovery, and traversal rejection;
- pure UI-core tests for route admission, allowlisted/write-only patches,
  value-free summaries, bounded theme behavior, and closed public errors;
- production static-contract tests for page inventory, same-origin endpoints,
  confirmation gates, text-only rendering, dark default, responsive layout,
  source disclosure, and absence of browser persistence or OTAWWW calls;
- headed Playwright CLI verification against synthetic same-origin responses,
  covering direct navigation, write-only settings submission, theme save,
  retained/live log filtering and pause/resume, pool/settings pages, disabled
  no-file update behavior, responsive mobile layout, and mobile-menu inertness;
- a final browser console check with zero errors and zero warnings;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 30 Bazel test targets passing and the ESP32-S3 firmware
  artifact produced from the production static tree;
- `just parity` with no validation errors and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, deterministic gzip
  comparison, sensitive-value scan, static provenance review, file-size review,
  and diff checks.

## Conclusion

The scoped AxeOS-compatible navigation, configuration, logs, theme, and
firmware-update user workflows are implemented in production static assets with
unit, static-route, workflow, and real-browser evidence. The implementation is
independent Rust-project expression; no Angular source expression or generated
asset was copied from the GPL reference tree.

## Non-claims and residual evidence gap

`UI-004` remains `implemented`, not `verified`. No exact-package static image
was flashed for this row, and no real device configuration, log session,
command, firmware upload, or update was exercised. Live embedded serving,
configuration persistence, responsive operator UAT, firmware upload/reboot,
OTAWWW, scoreboard/swarm population, external release checks, and end-to-end
hardware behavior remain below verified. No real origin, hostname, SSID,
address, port, USB identity, credential, pool field, worker, device identifier,
raw trace, hardware effect, direct UART, or pin manipulation appears in the
result or committed evidence.
