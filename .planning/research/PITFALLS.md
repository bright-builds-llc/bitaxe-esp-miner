# Pitfalls Research: Bitaxe Rust ESP-IDF Firmware Rewrite

**Domain:** Rust rewrite of ESP-Miner firmware with device-user parity, hardware control, Bazel/ESP-IDF workflow, and GPL provenance guardrails
**Researched:** 2026-06-20
**Overall confidence:** MEDIUM-HIGH

Confidence is high for roadmap, parity, and provenance risks because the local project docs and ADRs are explicit. Confidence is medium for tooling risks because current ESP-IDF Rust docs confirm important constraints, but the repo does not yet contain the firmware workspace. Confidence is low for exact ASIC/power sequencing details until `reference/esp-miner` is initialized and real Gamma 601 traces are captured.

## Summary

The biggest failure mode is claiming parity too early. This project is not trying to prove that Rust code exists; it is trying to prove that a Bitaxe user, API client, pool, and flashing workflow observe compatible behavior. Every roadmap phase should therefore produce evidence in `docs/parity/checklist.md`, and `verified` should remain blocked unless the evidence type is appropriate for the surface. Safety-critical surfaces need hardware evidence, not only unit tests.

The second major failure mode is unsafe bring-up. Gamma 601 BM1370 should start with a minimal boot/log path, then explicit board defaults, then a staged ASIC initialization path with voltage, reset, fan, thermal, watchdog, and fail-closed behavior proved on hardware. Mining should not be the first live hardware milestone.

The third major failure mode is provenance leakage. Upstream ESP-Miner is GPL-3.0 and should remain a pinned read-only reference. Original Rust should stay MIT-first only when it is independently authored. Any intentional port of GPL-covered source expression, generated fixture derived from upstream expression, or distributed firmware image needs explicit license review and conservative labeling.

The fourth major failure mode is letting Bazel, Cargo, ESP-IDF, and `just` become parallel build systems. ESP-IDF Rust tooling is practical, but the `esp-idf-*` crates document that they are community-maintained, may lag stable ESP-IDF, and currently lack HIL tests. The roadmap should pin toolchains early, make Bazel the canonical graph, and ensure `just` routes through the same targets.

Local finding: this checkout currently has no `reference/` directory and `git submodule status --recursive` produced no submodules. That should be treated as an immediate foundation-phase guardrail, not a later cleanup item.

## Pitfalls

| ID | Pitfall | What goes wrong | Consequence | Confidence |
| --- | --- | --- | --- | --- |
| P-01 | Reference implementation is absent, mutable, or unpinned | Work proceeds without `reference/esp-miner`, or normal commands modify it | Parity evidence cannot be audited; GPL provenance becomes ambiguous | HIGH |
| P-02 | Parity checklist becomes a task list instead of audit evidence | Items move to `implemented` or `verified` without evidence links and command summaries | Roadmap reports progress that cannot support release readiness | HIGH |
| P-03 | Rewrite turns into C transliteration | Rust modules mirror upstream C files, task layout, and source expression instead of observable behavior | Maintainers inherit C complexity, lose Rust design benefits, and increase GPL contamination risk | MEDIUM-HIGH |
| P-04 | Gamma 601 board defaults are hand-coded incorrectly | BM1370, 525 MHz, 1150 mV, NVS defaults, pins, or board identity are scattered as magic constants | Hardware smoke may pass accidentally while config/API/user-visible behavior diverges | HIGH |
| P-05 | Hardware control is enabled before safety gates exist | ASIC reset, vcore, frequency, fan, thermal, or power code runs without staged preflight and kill behavior | Device damage, unstable mining, thermal runaway, or false hardware parity claims | HIGH |
| P-06 | Rust/FreeRTOS concurrency starves watchdog, Wi-Fi, or telemetry | Blocking ASIC or Stratum loops do not yield, priorities are copied blindly, or watchdog subscription is missing | Reboots, stuck networking, stale telemetry, or unreliable long-running mining | MEDIUM-HIGH |
| P-07 | Bazel, Cargo, ESP-IDF, and `just` diverge | Local builds depend on sourced shell state or Cargo downloads while CI/Bazel sees a different graph | Non-reproducible builds, slow onboarding, and firmware images that cannot be rebuilt from evidence | HIGH |
| P-08 | Flash image and partition behavior do not match ESP-Miner expectations | Only an ELF is flashed, partition offsets differ, OTA slots are missing, or SPIFFS/NVS layout is wrong | Boot failure, OTA breakage, lost settings, or hard-to-recover devices | HIGH |
| P-09 | NVS/settings/API compatibility drifts from AxeOS clients | Rust defaults, NVS key names, JSON fields, PATCH semantics, or WebSocket telemetry differ subtly | Existing UI/tools misconfigure devices or display wrong status | HIGH |
| P-10 | Stratum/mining parity is tested only against one happy-path pool | One live pool accepts shares, but reconnect, fallback, coinbase parsing, difficulty changes, or error frames fail | Apparent mining success hides production pool incompatibilities | MEDIUM-HIGH |
| P-11 | GPL-derived expression lands in MIT-only files | Ported constants, register sequences, comments, fixtures, or generated data are not isolated or labeled | License posture becomes misleading and release artifacts need emergency remediation | HIGH |
| P-12 | Roadmap expands board support before 601 evidence is stable | Non-601 boards are marked verified from code review or config table entries | Project loses credibility and hardware regressions multiply across board variants | HIGH |
| P-13 | Diagnostics and logs are not treated as parity surfaces | Rust boot, error, WebSocket log, and API telemetry formats diverge from expected operator workflows | Users and tooling cannot diagnose firmware the same way they diagnose ESP-Miner | MEDIUM |
| P-14 | Release packaging ignores source, license, and installation obligations | Firmware images are published as "MIT-only" without dependency inventory or corresponding source review | Public releases create compliance risk, especially if GPL-covered code or data is included | HIGH |

## Warning Signs

| Pitfall | Early warning signs |
| --- | --- |
| P-01 | `reference/esp-miner` missing; `git submodule status` empty; `just verify-reference` absent; CI can pass without upstream breadcrumbs; normal scripts write under `reference/`. |
| P-02 | Checklist rows have `verified` with `pending` evidence; safety-critical rows use only `unit` or `golden`; release notes say "parity" without evidence IDs; no command output summaries are stored. |
| P-03 | `crates/*` depends on ESP-IDF or FreeRTOS; files are named exactly after upstream C internals without a behavior reason; comments become line-by-line translations; no domain newtypes for board, voltage, frequency, ASIC model, or NVS keys. |
| P-04 | Gamma 601 constants appear in `firmware/bitaxe` instead of `crates/bitaxe-config`; config tests do not mention `gamma`, `601`, `BM1370`, `525 MHz`, and `1150 mV`; board support claims lack per-board evidence. |
| P-05 | Firmware sets voltage/frequency before proving fan and thermal telemetry; no "safe no-op" mode; hardware smoke tests run full mining first; no current, temperature, reset, or watchdog log checkpoints. |
| P-06 | Logs show task watchdog resets; hardware runs only under a debugger; tasks busy-loop while polling ASIC or sockets; Wi-Fi/API responsiveness drops during mining; no long-running soak test exists. |
| P-07 | `cargo build` works but `just build` fails; `just flash` bypasses Bazel packaging; `IDF_PATH` or `export-esp.sh` changes behavior silently; ESP-IDF `master` is used; first build downloads toolchains outside documented setup. |
| P-08 | Package artifacts lack an offset manifest; partition table is not generated or parsed in CI; image size is not checked against slots; OTA and factory boot paths are not separately tested. |
| P-09 | API models are manually invented before comparing upstream OpenAPI or captured responses; NVS keys exceed ESP-IDF's 15-character key limit; settings update tests do not reboot and re-read persisted values. |
| P-10 | Only live-pool smoke testing exists; no fake pool harness; job construction has no golden fixtures; reconnect and malformed-message behavior is untested. |
| P-11 | MIT files include copied upstream comments or code-shaped register sequences; SPDX headers are missing; fixture origin is not documented; generated files do not record source commit and license posture. |
| P-12 | Board expansion starts before 601 `hardware-smoke` and `hardware-regression`; 205 or other boards move beyond `implemented` without device logs; roadmap phases group all ASIC families together. |
| P-13 | Logs expose Rust-internal names instead of user-facing status; WebSocket log buffering is missing; boot logs omit firmware identity, ESP-IDF version, board, ASIC, partition, and reference commit. |
| P-14 | Release workflow lacks dependency license inventory; firmware image provenance is not tied to source commit and reference commit; no review distinguishes MIT-only source, GPL-derived source, and mixed artifacts. |

## Prevention

| Pitfall | Prevention strategy | Concrete checks to add |
| --- | --- | --- |
| P-01 | Make the reference submodule a first foundation deliverable and fail fast when it is absent or dirty. | `just verify-reference` must fail if `reference/esp-miner` is missing, not a git submodule, dirty, or at an undocumented commit. CI should run it before build/test/package. |
| P-02 | Treat `docs/parity/checklist.md` as a schema-backed evidence ledger, not a markdown todo list. | Add `just parity` to reject `verified` rows with `pending` evidence, reject unknown statuses, require evidence type per row, and flag safety-critical rows verified without hardware evidence. |
| P-03 | Keep pure Rust behavior in crates and effects in `firmware/bitaxe`; use breadcrumbs at behavior boundaries only. | Bazel/Cargo dependency checks should prove `crates/bitaxe-core`, `bitaxe-asic`, `bitaxe-stratum`, `bitaxe-config`, and `bitaxe-api` do not depend on `esp-idf-*` crates unless explicitly allowed. |
| P-04 | Centralize board defaults and parse raw values into domain types before hardware use. | Unit/golden test `CFG-001` for Gamma 601: `devicemodel=gamma`, `boardversion=601`, `asicmodel=BM1370`, `asicfrequency=525`, `asicvoltage=1150`. Add range-checked newtypes for MHz and mV. |
| P-05 | Implement hardware control as staged state machines with safe defaults and explicit preflight gates. | Before enabling ASIC work: hardware log must show board detected, fan/thermal path ready or safely disabled by policy, voltage bounded, reset sequence complete, and fail-closed behavior tested. |
| P-06 | Design tasks around observable responsibilities, not upstream task names; explicitly handle watchdog and yielding. | Add a 30-60 minute Gamma 601 soak check for no watchdog reset, API responsiveness during mining, and periodic telemetry freshness. Require task watchdog subscription/reset decisions in phase plans. |
| P-07 | Pin Rust, ESP-IDF, esp-rs, Python, and flashing tools; make Bazel invoke the canonical package/build/test flow. | `just build`, `just test`, and `just package` should print toolchain versions and use Bazel targets. Add clean-checkout CI with no pre-sourced local shell state beyond documented setup. |
| P-08 | Make image layout a tested artifact with offsets, sizes, bootloader, partition table, firmware, and data partitions. | `just package` should emit a manifest with offsets and SHA256s. Verification should parse the partition table, check app size against slots, and smoke flash the merged image on 601. |
| P-09 | Generate or compare API/NVS behavior against upstream contracts before implementing handlers broadly. | Add API compare fixtures for system info, settings PATCH, logs, telemetry, and OTA routes. Add NVS roundtrip tests: write -> reboot/simulated reload -> read defaults and updates. |
| P-10 | Build deterministic Stratum fixtures and a fake pool harness before live pool claims. | Unit/golden tests for subscribe/authorize/notify/set_difficulty/submit messages, coinbase decoding, reconnect, malformed frames, and job construction. Hardware evidence should include accepted shares plus reconnect behavior. |
| P-11 | Track provenance per file, crate, fixture, and release artifact; isolate GPL-derived expression. | Require SPDX headers on new source files, fixture metadata with upstream commit and license posture, and a review step for any file that closely ports upstream source expression. |
| P-12 | Keep 601 as the hardware proof path until it is stable; expand board/ASIC support one evidence set at a time. | Parity tooling should block `verified` status for non-601 rows unless evidence names the physical board, command, log, firmware commit, and reference commit. |
| P-13 | Define boot, runtime, and WebSocket logs as compatibility artifacts. | First milestone boot smoke must capture firmware identity, board, ASIC target, ESP-IDF/Rust versions, partition/image identity, safe-mode state, and serial monitor command. Later API phases must compare log buffer and WebSocket behavior. |
| P-14 | Treat firmware release as a compliance gate, not just a build artifact. | Release phase must produce dependency license inventory, source/reference commit manifest, firmware artifact license assessment, source availability notes, and installation/flashing instructions. |

## Phase Mapping

| Roadmap phase | Pitfalls to prevent | Required guardrails |
| --- | --- | --- |
| Foundation and Gamma 601 boot/log bring-up | P-01, P-02, P-07, P-08, P-11, P-13 | Add `reference/esp-miner` submodule, `verify-reference`, Bazel/Just skeleton, pinned toolchains, package manifest, provenance policy, parity report, and hardware boot log evidence. |
| Config and board model parity | P-02, P-04, P-09, P-12 | Implement `bitaxe-config` domain types, Gamma 601 golden defaults, NVS key model, status/evidence enforcement, and explicit non-601 `not-started` or `implemented` without hardware claims. |
| BM1370 ASIC protocol and safe initialization | P-03, P-04, P-05, P-06, P-11 | Split packet/protocol logic into `bitaxe-asic`; stage hardware adapter behind safe preflight; require breadcrumbs and hardware evidence before `ASIC-002` or power-adjacent rows become `verified`. |
| Stratum v1 mining loop | P-03, P-06, P-10, P-13 | Build deterministic Stratum fixtures and fake pool tests before live pool claims; prove task/watchdog/network behavior under mining load; record accepted-share and reconnect evidence. |
| AxeOS API, logs, and telemetry compatibility | P-02, P-09, P-13 | Compare OpenAPI/captured responses, WebSocket telemetry, log buffer behavior, settings PATCH, and reboot persistence. Keep UI rewrite out of scope. |
| Power, thermal, fan, and self-test parity | P-05, P-06, P-12 | Require hardware regression logs for voltage, fan, thermal thresholds, reset, self-test, and fail-closed behavior. Unit tests can mark implemented but not verified. |
| OTA, filesystem, packaging, and release parity | P-08, P-09, P-11, P-14 | Verify partition layout, SPIFFS/assets, OTA slots, rollback/recovery behavior, image size, release manifest, dependency licenses, source availability, and installation information. |
| Additional board and ASIC expansion | P-04, P-05, P-10, P-11, P-12 | Add one board/ASIC at a time with config golden tests, hardware smoke/regression logs, provenance review, and no blanket verification inherited from 601. |

## Recommended Guardrails

Add these checks to roadmap requirements or verification criteria:

1. **Reference guard:** `just verify-reference` fails when `reference/esp-miner` is missing, not initialized, dirty, or not at a recorded upstream commit.
1. **Parity status guard:** `just parity` rejects `verified` rows without non-`pending` evidence and rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`.
1. **Evidence manifest:** Every hardware evidence entry records board, port, command, firmware commit, reference commit, timestamp, and captured log path.
1. **Pure crate boundary check:** Pure crates must not depend on ESP-IDF, FreeRTOS, flashing tools, or hardware adapters.
1. **Board defaults test:** Gamma 601 defaults must be tested from reference-derived fixtures before ASIC init work begins.
1. **Hardware safe-mode boot:** First boot firmware must log that ASIC power/work submission is disabled or safely staged until explicit bring-up phases enable it.
1. **Voltage/frequency bounds:** Vcore and ASIC frequency APIs must accept only range-checked domain types, not raw integers from API/NVS.
1. **Thermal/fan preflight:** Any phase enabling hashing must prove fan and thermal behavior or document a fail-closed deferral that blocks verified mining parity.
1. **Watchdog/soak test:** Mining and API phases need a repeatable soak check that detects watchdog resets, stale telemetry, and dropped API responsiveness.
1. **Package manifest:** Build artifacts must include offsets, partition table summary, image sizes, SHA256s, Rust/ESP-IDF versions, firmware commit, and reference commit.
1. **NVS/API compare:** Settings and system-info phases need fixtures that compare key names, defaults, JSON fields, status codes, reboot persistence, and PATCH behavior.
1. **Stratum fake pool:** Mining loop acceptance must include deterministic protocol tests and fake pool scenarios before relying on a public pool smoke test.
1. **SPDX/provenance lint:** New source and fixture files must include SPDX or documented license posture; GPL-derived expression must be isolated from MIT-only files.
1. **Dependency license inventory:** Release phase must generate an inventory covering Rust crates, Bazel external repos, ESP-IDF/esp-rs components, flashing tools, and static assets.
1. **No broad board claims:** Parity tooling or release checklist must block "all boards verified" language unless each board row has evidence for that board.
1. **Reference refresh protocol:** Any submodule pointer update must produce a parity impact note listing changed upstream surfaces and rows needing revalidation.
1. **Command surface consistency:** `just build`, `just test`, `just package`, `just flash`, and `just monitor` must either call Bazel targets or print the exact target/script they delegate to.
1. **Current checkout blocker:** Foundation planning should include an explicit task to create and initialize `reference/esp-miner`; this research observed that it is absent in the current worktree.

## Sources

- Local: `.planning/PROJECT.md`, `docs/project/first-milestone.md`, `docs/parity/checklist.md`, `PROVENANCE.md`, `docs/project/project-decisions.md`, `docs/project/seed-layout.md`, ADRs `0001` through `0013`.
- Local observation: `reference/` is absent and `git submodule status --recursive` returned no entries in this checkout on 2026-06-20.
- ESP-IDF Rust template: `esp-idf-template` documents ESP-IDF Rust STD support, tool prerequisites, ESP-IDF version choices, `espflash`, and environment requirements. https://github.com/esp-rs/esp-idf-template
- ESP-IDF Rust crates: `esp-idf-svc` supports ESP-IDF services including Wi-Fi, HTTP, WebSocket, NVS, and OTA, but documents the `esp-idf-*` crates as community-maintained, potentially lagging stable ESP-IDF, lacking HIL tests, and needing more documentation. https://github.com/esp-rs/esp-idf-svc
- `esp-idf-sys`: build is Cargo-driven by default and can download/configure ESP-IDF and toolchains, which is a Bazel hermeticity concern unless pinned and wrapped deliberately. https://docs.rs/crate/esp-idf-sys/latest
- Bazel `rules_rust`: official docs provide Rust rules for Bazel and support explicit Rust toolchain version configuration. https://bazelbuild.github.io/rules_rust/
- ESP-IDF OTA docs: OTA requires correct partition tables and OTA data partitions; bootloader, partition table, and data partition update modes have different safety characteristics; watchdogs can be triggered during large erase operations. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html
- ESP-IDF partition docs: partition tables define app/data offsets and are binary artifacts generated from CSV; OTA tables include `otadata`, `ota_0`, and `ota_1`. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/partition-tables.html
- ESP-IDF NVS docs: NVS stores key-value pairs in flash partitions and keys are ASCII strings with a 15-character maximum length. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/storage/nvs_flash.html
- ESP-IDF FreeRTOS docs: IDF FreeRTOS is a modified SMP implementation, so task behavior, affinity, critical sections, and scheduling need deliberate review rather than generic FreeRTOS assumptions. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/freertos_idf.html
- ESP-IDF watchdog docs: interrupt and task watchdogs detect blocked scheduling or tasks that do not yield/reset, and OpenOCD can disable watchdogs while debugging. https://espressif-docs.readthedocs-hosted.com/projects/esp-idf/en/stable/api-reference/system/wdts.html
- ESP-IDF power management docs: power locks, CPU/APB frequency changes, and light sleep affect peripherals and interrupt latency. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/power_management.html
- GPL-3.0 text via SPDX: object-code distribution requires corresponding source options, and user-product distribution can require installation information. https://spdx.org/licenses/GPL-3.0-only
- SPDX guidance: SPDX identifiers are machine-readable, precise, and can be applied per file; GNU licenses should use `-only` or `-or-later` suffixes. https://spdx.dev/learn/handling-license-info/
