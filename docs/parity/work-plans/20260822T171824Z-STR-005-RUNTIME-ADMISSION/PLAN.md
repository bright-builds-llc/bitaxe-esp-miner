# Parity work plan

- Run ID: `20260822T171824Z-STR-005-RUNTIME-ADMISSION`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `39aefd23d6f4f44a168ee3de01f0617bf65d2804`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-stratum-v2`
- Continues: `docs/parity/work-plans/20260822T165408Z-STR-005-RETRY2/CLOSURE.md`

## Selection

Attempt-003 passed the clean-source no-effect preflight and detector, then
stopped after about 20 seconds as `hardware_blocked` / `unclassified` before
private-root creation. Fixture, NVS, flash, pool, mining, share, and hardware-
control phases did not begin, USB cleanup is ready, and no process remains.

The remaining pre-root sequence is passive monitor process completion, exactly
one current runtime origin, same-origin system/theme reads, exact restoration-
input reconstruction, and prior-package selection. The earlier public result
collapsed all five. This plan adds one closed checkpoint per boundary and a
read-only command that must prove the whole sequence before attempt-004 can be
consumed.

## Scope and non-scope

Add checkpoints `runtime_monitor_process`, `runtime_origin`,
`runtime_settings`, `restoration_inputs`, and `restore_package`, plus successful
`runtime_admission_ready`. The read-only admission may use the detected board's
receive-only USB runtime observer, derive exactly one current origin from that
same capture, issue same-origin GETs, and compare protected local restoration
inputs only in memory. It may report only schema, ready/failed status, closed
category/checkpoint, and false effect/root booleans.

It must not create the attempt root, start the fixture, write NVS/settings,
flash/reset through a bootloader command, contact a pool, mine, initialize or
control the ASIC, change voltage/fan/power, publish evidence, or expose origins,
paths, settings, credentials, identities, responses, logs, or child output.
External pools, network scanning, direct UART, pins/pads/headers/GPIO, probes,
jumpers, soldering, injected signals, fault injection, OTA, erase, arbitrary
profiles, and unbounded mining remain prohibited.

## Implementation

- [x] Refactor the campaign so runtime admission is one shared read-only path
      consumed by both the diagnostic command and effectful campaign.
- [x] Add closed failure checkpoints for every remaining pre-root boundary and
      a closed `runtime_admission_ready` success result.
- [x] Add pure origin-cardinality tests, command/result redaction tests, and
      real-launch coverage while keeping the aggregate campaign tests active.
- [x] Bind only attempt-004 into the immutable parser and protected paths.
- [ ] Run every mandatory software, package, privacy, reference, and diff gate;
      commit and push the exact implementation before device access.

## Verification and conditional hardware continuation

After clean push and exact packaging, run the no-effect software preflight:

`just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Require `pre_effect_ready`, both false booleans, and absent private/public
targets. Then run `just detect-ultra205` once and, only after exactly one board
205 is admitted, run this read-only command:

`just stratum-v2-runtime-admission --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

The read-only command does not consume attempt-004. Continue only if it exits
zero with schema `bitaxe-stratum-v2-runtime-admission-v1`, status `ready`,
checkpoint `runtime_admission_ready`, `effect_started=false`, and
`private_root_created=false`, while both targets remain absent. Any failure
closes this plan without a campaign or new ordinal.

Only after that ready result may the sole attempt-004 effect command run:

`just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Attempt-004 inherits every objective, allowed effect, mode-0700/0600 privacy
rule, 400 MHz/1100 mV/100% fan profile, 180-second ceiling, safety limit,
single-share ceiling, terminal safe stop, exact settings/package restoration,
cleanup, independent validation, redaction, and public-withholding requirement
from `docs/parity/work-plans/20260822T040442Z-STR-005/PLAN.md`. It is consumed
only when the effect command starts. There is no unchanged retry or attempt-005
under this plan.

Before device access, pass ordered Cargo format/clippy/build/test, Bright
Builds, all Bazel tests, dedicated real-launch regression, canonical build and
package, parity/progress, redaction, reference cleanliness, selector lineage,
sensitive-value review, and final diff review. Promote only on a complete
independently accepted attempt-004 projection; otherwise preserve `implemented`,
withhold `RESULT.md`, and record the first closed category/checkpoint.
