# Parity work plan

- Run ID: `20260813T045300Z-UI-004`
- Parity row: `UI-004`
- Initial status: `implemented`
- Source commit: `2b8f8c6374c757a36d1cea5598ec18f2a882c58c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui004-live-browser-attempt-001`

Continues `docs/parity/work-plans/20260804T190000Z-UI-004/PLAN.md`.

## Selection

The clean synchronized selector returned no open plan. `API-009` is sealed at
its repeated attempt-007 command-effect boundary and attempt-008 is prohibited.
`THR-001` requires vendor-safe overheat or thermal-fault stimulus that the
repository does not own. `IO-002` requires an independent calibrated electrical
reference unavailable through the authorized USB path. `UI-001` and `UI-002`
require a trusted physical panel image, while `UI-003` requires a recorded
physical button interaction; the user's fresh report that the panel now shows
more information is corroborating context, not typed visual evidence.
`SELF-001` requires qualified self-test hardware stimulus and `BAP-002` requires
a compatible accessory and separately authorized electrical attachment.

`UI-004` is the first actionable row. The production static interface, its
synthetic real-browser contracts, exact embedded static serving, theme and
settings durability, retained/live logs, and firmware-only update behavior
already have separate committed evidence. The remaining joinable gap is one
current exact-package browser session proving that those production assets are
served and usable on the admitted Ultra 205 at responsive desktop and mobile
sizes. This plan composes those authoritative boundaries instead of replaying
settings mutations, firmware upload, or OTA.

## Scope and non-scope

Add a typed `bitaxe-ui-workflow-evidence-v1` projector and independent Rust
validator. The private-first projector must bind the current board-205 package,
source/reference commits, application ELF and static-asset digests, one closed
operator-snapshot projection, one isolated real-browser attestation, and the
existing independently validated theme durability, settings patch, log buffer,
static route, firmware OTA, and rollback projections. It may publish only
closed schema/provenance fields, evidence digests, route and viewport category
counts, same-origin and console-clean booleans, write-only-secret/update-guard
facts, safe-state/cleanup facts, and redaction facts.

The browser session is read-only. It may load `/`, the dashboard, network,
pool, settings, logs, update, and theme routes; receive the same-origin API and
log WebSocket traffic those pages normally request; resize between desktop and
mobile viewports; open and close mobile navigation; confirm that password
fields remain blank; confirm the no-file firmware action is disabled and
OTAWWW remains unavailable; and save private accessibility snapshots and
screenshots. It must not submit settings, invoke pause/resume/restart/identify,
dismiss a block, upload firmware, invoke OTA or OTAWWW, persist theme, start
mining, or touch any hardware-control surface.

The one hardware transaction may install the exact current package with
replacement NVS containing only owner-supplied Wi-Fi credentials and
`mineonboot=false`, capture receive-only USB boot evidence, perform one normal
software restart through the canonical device-session transaction, make
bounded same-origin HTTP/WebSocket reads, and use one exact-package factory
recovery flash only if final package/safe-state restoration cannot otherwise be
confirmed. Recovery cannot convert failure into evidence.

Private artifacts belong beneath mode-`0700`, gitignored
`scratch/ui004-live-workflows` and `output/playwright/ui004-attempt-001` roots;
every private file must be mode `0600`. Origins, hostnames, addresses, ports,
USB/network/process identities, credentials, HTTP bodies, WebSocket frames,
page text containing device values, commands, raw traces, and screenshots must
never enter terminal or Git output. No camera, physical panel claim, button,
direct UART, pins, pads, headers, GPIO, probes, external service, pool input,
mining, ASIC work, voltage, frequency, fan, thermal, or power effect is
authorized.

## Implementation

- [ ] Add the closed UI-workflow evidence schema, generated TypeScript binding,
      private-first projector, independent Rust validator, CLI/Just surface,
      redaction admission, and exact evidence-source joins.
- [ ] Add focused tests for package/static digest binding, route and viewport
      quorum, same-origin browser facts, blank write-only secrets, update and
      OTAWWW guards, console/network failure withholding, malformed or missing
      private artifacts, source-evidence drift, primary failure precedence,
      protected modes, and public-output redaction.
- [ ] Add a real-child projector/validator regression so file/process behavior
      cannot be replaced by an in-process fake, and re-run the production static
      UI/browser contracts plus the real firmware/package build.
- [ ] Commit and push the implementation before spending exactly one detector
      and at most one conditional hardware/browser attempt.
- [ ] Validate the final public projection independently and promote only
      `UI-004` when the complete evidence join passes.

## Verification and promotion

Run focused automation-contract, projector, CLI, static UI, source-ownership,
redaction, and real-child tests plus the real ESP32-S3 package build. Then run,
in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. generated-contract, immutable-plan, task-uniqueness, private-mode,
    sensitive-output, reference-cleanliness, no-public-output, and diff checks

After the implementation is committed, pushed, clean, and package-admitted,
run exactly one attempt:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/ui004-live-workflows/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/ui004-live-workflows/wrapper-001 && just detect-ultra205 > scratch/ui004-live-workflows/wrapper-001/detector.stdout 2>&1)`
3. Only after detector success, run
   `just capture-operator-snapshot-evidence --private-root scratch/ui004-live-workflows/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --port <detector-port> --projection scratch/ui004-live-workflows/wrapper-001/operator-snapshot-projection.private.json --capture-timeout-seconds 600`, with stdout/stderr redirected to mode-`0600` wrapper files.
4. From exactly one origin admitted by that same protected attempt, run one
   named isolated Playwright CLI session from
   `output/playwright/ui004-attempt-001`, capture the exact route/viewport
   quorum above without mutation, close the session, and write one mode-`0600`
   private browser attestation containing only validator-required facts and
   private artifact digests.
5. Run the repo-owned projector once to write
   `docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json`, then
   validate that public projection with the independent Rust validator.

The wrapper, attempt, and browser roots must be absent before use. Detector
failure stops before writes outside its wrapper. Starting step 3 consumes
attempt-001; do not retry the ordinal. Preserve the earliest typed failure
through cleanup and optional recovery. Accepted non-success categories are
`package_invalid`, `process_failed`, `timeout`, `hardware_blocked`,
`browser_blocked`, `evidence_invalid`, and `recovery_failed`. Any non-success
withholds final evidence and leaves `UI-004` at `implemented`.

Promotion requires exact current source/reference/package and static digests;
same-device exact-package boot and one normal restart; safe mining-disabled and
hardware-control-disabled state; all seven routes loaded from the embedded
assets at desktop and mobile sizes; working mobile-menu state changes;
same-origin API/log traffic; blank write-only secrets; disabled no-file update;
visible OTAWWW-unavailable behavior; zero browser console errors and zero
unexpected request failures; complete browser/device cleanup; protected modes;
redaction; valid joined theme/settings/log/static/firmware-update evidence; and
every software gate passing. Panel rendering, physical input, OTAWWW updates,
scoreboard/swarm population, mining telemetry, other boards, and release
readiness remain explicit non-claims.
