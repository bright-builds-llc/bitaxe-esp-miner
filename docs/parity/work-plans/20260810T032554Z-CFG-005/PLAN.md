# Parity work plan

- Run ID: `20260810T032554Z-CFG-005`
- Parity row: `CFG-005`
- Initial status: `implemented`
- Source commit: `15615d19fe756d180a7a53a51c27c08a190cfc98`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-cfg005-full-settings-persistence`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. The first candidate, `CFG-001`, is not actionable in this software-only
invocation: its exact defaults already have unit and golden evidence, while its
remaining frequency and voltage behavior requires safety-critical hardware
evidence.

`CFG-005` is the next candidate and the first actionable software row. Its pure
schema path already validates the complete upstream REST settings table and
emits typed NVS writes, but the production PATCH route grants persistence
authority only to hostname and the project-owned `startMiningOnBoot` setting.
All other valid upstream fields currently return compatibility success without
being stored. The pinned `PATCH_update_settings`, `check_settings_and_update`,
and `nvs_config.c` paths provide a closed reference contract that can be
implemented and tested without device access.

## Scope and non-scope

Implement one serialized, fail-closed firmware transaction for every validated
upstream settings write. The transaction must write every typed `NvsWrite`,
commit once, independently reread and reconcile the complete expected write
set, publish only the existing non-secret settings snapshot, and expose an
empty success response only after confirmation. Unknown fields remain ignored;
any invalid known field rejects the whole request before storage access.
Hostname remains the only best-effort live effect and may run only after
storage confirmation and response scheduling. Pool and credential values may
exist only in validated request and private NVS adapter memory; diagnostics,
tests, evidence, and public responses may contain keys or closed categories but
never values.

Retain the separately owned `startMiningOnBoot` extension and its production
session wakeup. Preserve compatibility writes, body-size/access gates,
serialized ownership, post-commit uncertainty reporting, and the existing
hardware, mining, safety, and credential-consumption gates. Persisting a
setting must not itself enable mining, submit ASIC work, actuate frequency,
voltage, fan, thermal, or power controls, connect to a pool, restart the device,
or widen any downstream authority.

Do not access hardware, credentials, USB, serial, external networks, OTA,
recovery, direct UART, pins, or private scratch evidence. Do not change or
promote `API-003` or any other checklist row. Live NVS durability, live hostname
application, credential use, and every hardware-control effect remain explicit
non-claims owned by their respective rows.

## Implementation

- [ ] Add an adapter-neutral full-settings persistence contract that carries
      the already validated write set through write, commit, independent
      reconciliation, non-secret publication, and public success.
- [ ] Implement all string, `u16`, `i32`, and `u64` NVS writes in the ESP-IDF
      adapter, including string-backed float and legacy compatibility writes,
      without logging values.
- [ ] Route valid upstream PATCH bodies through that transaction while keeping
      unknown-only requests inert and invalid-known requests atomic and
      fail-closed.
- [ ] Add reference-derived exhaustive golden cases, pure failure-order tests,
      firmware ownership/privacy regressions, and the real firmware build.
- [ ] Append verification to `WORKLOG.md`; create `RESULT.md` only if the full
      software contract and every repository gate pass.

## Verification and promotion

Focused verification will run the `bitaxe-config` and `bitaxe-api` Cargo and
Bazel tests, the firmware settings ownership tests, API comparison, and the
canonical ESP32-S3 firmware build. Tests must prove complete schema coverage,
typed write fidelity, unknown-field no-op behavior, all-or-nothing validation,
single-owner ordering, every adapter failure boundary, exact reconciliation,
secret-free diagnostics, and zero hardware effects.

Final verification will run, in order, `cargo fmt --all`, strict
all-target/all-feature Clippy, all-target/all-feature build, all-feature tests,
Bright Builds checks, `just test`, `just parity`, `just parity-progress`,
redaction, reference cleanliness, immutable-plan, and diff checks.

Promote only `CFG-005` to `verified` with `unit,golden,workflow` evidence when
the complete upstream REST settings schema reaches exact confirmed NVS writes
through the production adapter, all invalid inputs fail before storage, no
secret value reaches diagnostics or committed evidence, the real firmware
build passes, and every gate succeeds. Otherwise leave `CFG-005` at
`implemented`, record the exact blocker, and close the plan without a
verification claim. Hardware evidence is neither required nor accepted for
this software-only plan.
