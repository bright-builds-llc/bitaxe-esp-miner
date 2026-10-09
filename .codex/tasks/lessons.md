## lesson-esp-idf-service-ownership-and-redaction | 2026-07-02 23:29

1. Date: 2026-07-02
2. What went wrong: Wi-Fi startup initialized the default ESP-IDF event loop through raw `esp_event_loop_create_default()` before `EspSystemEventLoop::take()`, so esp-idf-svc's ownership tracker returned `ESP_ERR_INVALID_STATE`. The first hardware evidence run also showed that ESP-IDF Wi-Fi driver logs can expose the connected SSID outside JSON or `key=value` fields.
3. Preventive rule: Let esp-idf-svc own managed ESP-IDF service handles such as the default event loop; use raw idempotent init only for services without a wrapper ownership tracker. Redaction tests must include vendor log formats, not only project log formats.
4. Trigger signal to catch it earlier: A managed `take()` API fails with `ESP_ERR_INVALID_STATE` immediately after a raw init call, or sanitized serial evidence still contains natural-language Wi-Fi driver lines such as `wifi:connected with ...`.

## lesson-opaque-handoff-before-fallible-validation | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: A fresh exact-head attempt created a one-time opaque resume handle, then a fallible handoff assertion rejected the otherwise valid checkpoint before the handle reached the operator. The live private attempt could no longer be addressed through the normal handle-only cleanup path.
3. Preventive rule: Emit or durably escrow a one-time public locator before running fallible post-construction assertions. Provide a narrowly guarded, effect-free cleanup path for a uniquely identifiable pristine orphan without reconstructing or exposing the clear locator.
4. Trigger signal to catch it earlier: A command creates a private active record or capability and then performs validation, formatting, or output transformation before returning its only public locator.

## lesson-cross-process-tests-use-real-boundaries | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: In-process fixtures passed while the first real lifecycle continuation failed because a Unix-socket receiver mixed buffered line input with unbuffered payload reads. Other live-only failures involved process-group descendants, fresh-process capability parsing, and Bazel/runfiles execution resolving helpers or tools differently from the source-tree shell.
3. Preventive rule: Test IPC, process ownership, framing, capabilities, and Bazel/runfiles entrypoints through real fresh processes, Unix sockets, coalesced and fragmented writes, process groups, and mode-enforced files. Exercise sibling helper and tool resolution from the deployed layout, and prevent production children from invoking nested build tools. Keep pure tests too, but do not let them substitute for the operating-system boundary that production uses.
4. Trigger signal to catch it earlier: A test injects a function or prebuilt object where production crosses a socket, process, PTY, file-permission, process-group, or runfiles boundary; resolves helpers only from the source tree; or allows a launched child to call the build runner again.

## lesson-native-usb-capture-needs-prearmed-observation-or-replay | 2026-07-12 04:00

1. Date: 2026-07-12
2. What went wrong: A lifecycle waited for the operator to report barrel-then-USB restoration before opening the native USB monitor. The ESP32-S3 booted from barrel power before the serial node existed, so correct later ownership still captured zero early boot/listener markers.
3. Preventive rule: For native-USB cold-start evidence, arm the exact-node watcher before instructing physical restoration and start passive ownership automatically on node appearance. When the transport cannot preserve pre-enumeration bytes, validate replayable, session-tagged application proof instead of relying on an arbitrary countdown or post-plug acknowledgment.
4. Trigger signal to catch it earlier: A test requires early boot bytes from a serial device whose node is created only after power-up, or asks the operator to confirm plugging before the monitor process begins waiting.

## lesson-earliest-typed-failure-precedence | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: Cleanup or a later classifier result could replace the earliest typed failure, obscuring the boundary that actually stopped the workflow and routing recovery incorrectly.
3. Preventive rule: Capture the first typed failure once and preserve it through restoration, cleanup, sealing, and reporting. Later failures may be recorded separately but must not overwrite the original cause.
4. Trigger signal to catch it earlier: A mutable failure category is assigned in multiple phases after the first error, or a terminal report names cleanup instead of the earlier admission, discovery, transport, or validation boundary.

## lesson-esp-idf-main-task-runtime-capacity | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: Host checks passed while the ESP-IDF main task lacked the runtime capacity required by the composed firmware startup and service stack.
3. Preventive rule: Treat the ESP-IDF main-task stack setting as an explicit runtime contract, keep one authoritative assignment, and regression-test its minimum capacity alongside the code paths that depend on it.
4. Trigger signal to catch it earlier: Firmware adds startup, parsing, service, or orchestration work without checking the configured main-task stack, or multiple stack assignments make the effective capacity ambiguous.

## lesson-http-liveness-is-not-response-readiness | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: Route registration, server-start markers, connectivity, and continuing application liveness were treated as if they proved that an HTTP request could deliver a complete parseable response.
3. Preventive rule: Keep connection establishment, request transmission, response status and headers, body receipt, and schema parsing as separate typed boundaries. Do not infer response readiness from route, startup, connectivity, or heartbeat markers.
4. Trigger signal to catch it earlier: Evidence shows a live application and registered route but has no independently observed response status, headers, body bytes, or completed parse.

## lesson-consume-qualified-transport-capabilities | 2026-07-22 20:41

1. Date: 2026-07-22
2. What went wrong: A phase-local reboot workflow hard-coded `espflash` as its runtime observer even though earlier hardware evidence in the same repository had already shown that passive espflash delivered zero application bytes while the receive-only OS-native reader delivered valid heartbeats.
3. Preventive rule: Model bootloader access, runtime observation, application control, and evidence proof as separate capabilities. Phase workflows must consume the repository's currently qualified backend for each capability instead of selecting a convenient tool locally.
4. Trigger signal to catch it earlier: A phase names a concrete transport executable directly even though a repository qualification, capability contract, or prior hardware result selects a different backend for that boundary.

## lesson-evaluator-identity-binds-transitive-validators | 2026-07-24

1. Date: 2026-07-24
2. What went wrong: The Phase 36 evidence evaluator identity omitted a materially reachable runtime-identity state reducer, so validator behavior could drift without rotating the evaluator or successor-contract identities.
3. Preventive rule: Bind every materially reachable repository-owned validator, including transitive reducers and models, through a versioned inventory of relative path and source bytes; declare every source in the build/runfiles graph and regression-test source, path, addition, removal, and replacement drift.
4. Trigger signal to catch it earlier: An evaluator inventory lists entrypoint validators but omits a reducer or model they call, accepts caller-authored digests, or lacks a test that membership drift rotates every derived identity.
## lesson-separate-flash-effect-from-monitor-proof | 2026-07-26 10:35

1. Date: 2026-07-26
2. What went wrong: An admitted factory write completed and the device ran normally, but the native reader attached after startup-only markers had passed, so the wrapper described missing monitor proof as a failed flash.
3. Preventive rule: Record flash effect completion, USB cleanup, original boot-transcript capture, and replayable exact-package runtime verification as separate outcomes. Runtime replay may establish only its own trust basis, and missing monitor proof must never recommend an unchanged automatic reflash.
4. Trigger signal: A post-flash log begins at nonzero uptime, contains healthy repeated same-session runtime output, and lacks startup-only markers even though the write and same-device cleanup completed.

## lesson-never-invite-ready-before-live-checkpoint | 2026-08-14 08:53

1. Date: 2026-08-14
2. What went wrong: After a hardware campaign had already failed before creating its ready checkpoint, the user was told they could reply `ready` within the new one-hour window. The wording implied that the window was live even though it had never opened, so the user's timely reply appeared to be ignored.
3. Preventive rule: Invite an operator readiness reply only after the current campaign's typed `required` checkpoint exists and the campaign is confirmed running. State explicitly when the window has not opened or has closed, and never describe a future or conditional window in language that sounds currently actionable.
4. Trigger signal to catch it earlier: A message mentions replying `ready`, a signal-sender command, or a window duration without first proving and stating that the matching live `required` checkpoint exists and the owning campaign is still running.

## lesson-surface-preflight-exit-before-advancing | 2026-08-15 06:54

1. Date: 2026-08-15 06:54 CDT
2. What went wrong: A guessed manifest field made preflight exit nonzero, but empty output was mistaken for success and the next command launched without detector evidence.
3. Preventive rule: Validate package fields through repo-owned contracts and inspect every command exit code before advancing.
4. Trigger signal: A preflight produces no output or a required artifact is absent before an effect command.

## lesson-bind-telemetry-ranges-to-operating-state | 2026-08-15 17:18

1. Date: 2026-08-15 17:18 CDT
2. What went wrong: The ADC evidence validator rejected fresh `0 mV` readings using an unconditional 400–2,000 mV range even though the same evidence workflow deliberately kept the ASIC rail disabled and the typed acquisition path treats a successful zero as fresh truth.
3. Preventive rule: Validate telemetry units against the producer's real wire domain, and bind narrower expected operating ranges to independently validated device state. Never apply an energized-state range to disabled-state evidence.
4. Trigger signal: A telemetry validator has a fixed positive lower bound but does not consume power, enable, mode, or lifecycle state, or its accepted range contradicts a typed zero-value regression in the producer.

## lesson-trace-legacy-wire-units-through-the-ui | 2026-08-16 04:02

1. Date: 2026-08-16
2. What went wrong: SI-typed internal INA260 values in volts and amps were serialized directly into legacy API fields whose upstream contract transports millivolts and milliamps, so type-safe internals still produced reference-incompatible wire values.
3. Preventive rule: Keep internal engineering units explicit, then verify every compatibility boundary from sensor conversion through API serialization, statistics history, and reference UI normalization before claiming parity.
4. Trigger signal: The reference UI divides an API field by 1,000, the reference driver documents milli-units, or an internal `*_volts`/`*_amps` field is assigned directly to an unqualified legacy wire name such as `voltage` or `current`.

## lesson-distinguish-agent-runtime-from-host-runtime | 2026-08-17 12:05

1. Date: 2026-08-17 12:05 CDT
2. What went wrong: Repeated process-launch stalls and 300-second test timeouts inside the Codex execution environment were diagnosed as machine-wide macOS degradation requiring a reboot, but the user ran the exact uncached Bazel automation target in 68.1 seconds and the exact filtered Cargo command in 0.6 seconds from their normal shell.
3. Preventive rule: Treat timeouts observed only inside the agent execution environment as agent-session or sandbox failures until the same exact uncached command is independently reproduced in the user's normal shell. Do not recommend a host reboot or declare a machine-wide blocker from agent-only timing evidence.
4. Trigger signal: Agent tool calls show inconsistent multi-minute gaps between otherwise passing child processes, while an external user shell has not reproduced the delay or reports normal timings for the exact command.

## lesson-receive-only-serial-still-requires-raw-terminal-configuration | 2026-09-04 15:00

1. Date: 2026-09-04
2. What went wrong: The macOS USB observer opened the CDC callout node read-only and nonblocking but left its terminal line discipline in canonical mode, so binary and partial firmware evidence could remain withheld and healthy application execution appeared silent.
3. Preventive rule: A receive-only serial Adapter must still configure the admitted descriptor as raw at the expected baud, enable local receive, and disable hang-up-on-close while explicitly excluding payload writes, modem-control operations, DTR/RTS changes, and the maintenance baud.
4. Trigger signal to catch it earlier: A serial reader opens successfully but receives no bytes, while a PTY regression with bytes lacking a newline also returns empty or the Adapter never applies raw termios configuration.

## lesson-native-usb-and-wifi-share-internal-dma-heap | 2026-09-04 16:30

1. Date: 2026-09-04
2. What went wrong: Starting the optional TinyUSB Worker before Wi-Fi consumed and fragmented shared internal/DMA-capable heap, leaving the ESP32-S3 unable to satisfy a later 852-byte `DMA | 8BIT | INTERNAL` allocation; the allocator aborted directly and the board entered a panic reboot loop that looked like unstable USB enumeration. An attempted fix changed an unknown `TINYUSB_VENDOR_RX_BUFSIZE` symbol, and a later real TinyUSB stack reduction only moved the failure: after Wi-Fi became stable, the 12 KiB Worker pthread failed with `ENOMEM` because ordinary allocations had consumed the internal pool reserved by default at only 32 KiB.
3. Preventive rule: Treat USB, Wi-Fi, and forced-internal pthread stacks as one ordered budget. Allocate large required pthread stacks while internal memory is contiguous, defer optional USB installation until after Wi-Fi's fixed DMA resources, size the internal reserve from measured post-stage headroom plus the largest remaining forced-internal allocation, assert resolved Kconfig values at compile/build time, and retain stage-specific heap plus previous-boot failure evidence for field diagnosis.
4. Trigger signal to catch it earlier: An optional USB task starts before Wi-Fi, a forced-internal pthread is added without increasing or measuring the internal reserve, generated sdkconfig omits a requested setting, Worker enumerates briefly before repeated `panic` resets, an allocation receipt reports capability mask `0x0000080c`, or a deferred Worker spawn returns `ENOMEM` after Wi-Fi connects.

## lesson-clear-archived-core-dumps-before-the-next-panic | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The firmware stores flash core dumps with `CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE=y`. An earlier read returned a stale dump from a previous panic, and later the restart006 panic left no dump at all because the already-analysed queue boot-loop dump still occupied the partition, so its exact panic site was never captured.
3. Preventive rule: After a dump is captured, archived and analysed, clear it under its task contract so the partition is empty before the next effectful run. Before trusting a read, compare the dump's embedded app SHA-256 with the image that actually panicked.
4. Trigger signal to catch it earlier: A core-dump decode fails with `coredump SHA256 != app SHA256`, a read returns a dump whose app identity matches an older image, or an analysed dump has not been cleared when hardware work resumes.

## lesson-diagnose-heap-loss-from-a-passive-series | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: Two internal-heap readings, 9,687 and 607 bytes free, suggested an idle leak, but both were sampled during Worker connections. A once-a-minute idle sample read passively over serial showed persistent exhaustion with periodic 6 KB dips, not a leak. Separately, the retained `allocation_failure` row is the boot's first failed allocation, which was not necessarily the one that caused the panic.
3. Preventive rule: Before attributing internal-heap loss to a leak or to idle time, capture a passive time series (`internal_heap_sample` via receive-only `just monitor`, judged with `just internal-heap-series`) both idle and after the suspect session, without opening the connection under test. Treat a retained allocation-failure row as the first failure of that boot, not as the panic cause. This extends `lesson-native-usb-and-wifi-share-internal-dma-heap`.
4. Trigger signal to catch it earlier: A leak hypothesis rests on samples taken inside connections, the largest free internal block is below the size of the next required allocation, or a panic is explained by the retained allocation-failure row alone.

## lesson-stop-on-first-success-cannot-prove-later-events | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The accepted-share probe stopped as soon as the share was acknowledged, which on the candidate came before the Gate's first 20-second renewal, so the Renew path went unexercised. The renewal probe's 45-second window, with about 2.5 expected qualifying shares, also left about a 6% chance of no share; renew-current-001 hit it and sealed unverified.
3. Preventive rule: When a run must prove several events, make its stop rule require every claimed event, not the first success. Size each bounded window from measured event rates (probability of no event) and keep it inside the lease or authority expiry those events extend.
4. Trigger signal to catch it earlier: A claim needs two events from a run that stops on the first, an event scheduled after the stop condition is listed as covered, or a window's expected event count is below about 4.

## lesson-sealed-run-bindings-must-accept-every-run-shape | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The lineage record-key check and the restart owner's generation lookup assumed one sealed-run shape (a heartbeat dispatch record). They refused a share run, which keeps a proof instead, and then a share run without a share, which keeps neither; each needed a separate fix while hardware work waited.
3. Preventive rule: When a new owner seals runs that later owners consume, enumerate every run shape it can produce, including failed and partial ones, and test each in the shared binding helpers (`deviceRecordAttemptId`, `startGeneration`) before the first hardware run.
4. Trigger signal to catch it earlier: `start_record_attempt_mismatch`, `preparation_parent_binding` or a refused previous Start right after a new owner or a new failure mode produced its first sealed run.

## lesson-serve-a-page-s-whole-module-graph | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The recovery client gained an import (`recovery-error-row.mjs`) that the shared current-recovery server never served. Every restart and preparation recovery page then failed to load before connecting, which surfaced only on hardware (restart005).
3. Preventive rule: A server that serves a browser module must own that module's whole import graph, derived in one place, with a test that walks the real imports and requires each to load. Never let individual callers list a page's modules.
4. Trigger signal to catch it earlier: A served page shows only the base Gate controls, the network log shows a non-200 for an `.mjs`, or a browser module gains an import while several servers serve it.

## lesson-join-random-ids-to-their-cli-flags | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The noise-serial owner passed a random base64url attempt id as a separate argument. One id began with `-`, the fixture's clap parser read it as a flag and exited with code 2 before listening, and the install attempt failed after five good installs.
3. Preventive rule: Pass generated or user-derived values to CLIs as `--flag=value`, never as a separate argument, and cover a leading `-` value in a regression on the real parser.
4. Trigger signal to catch it earlier: A child exits with code 2 and a short stderr before doing work, or an argv builder places a random token after its flag as its own element.

## lesson-independent-review-before-parity-promotion | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The STR-005 review numbers were correct, but an independent adversarial review still found overclaims: renewal coverage taken from an `unverified` seal without its caveats, a renewal reply described as signed when it is only matched, an older-image result cited as candidate evidence, and a blocking dependency described as non-blocking.
3. Preventive rule: Before any parity promotion, run an independent reviewer that re-derives every cited number from sealed roots and challenges each claim's framing. Evidence taken from an unverified seal must say so and carry its judge's blockers.
4. Trigger signal to catch it earlier: Evidence summaries written by the same agent that produced the runs, a covering result whose seal is not `complete`, or a protocol property asserted from documentation rather than the implementing source.

## lesson-update-validator-scope-lists-with-scope-changes | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: A June parity guard hard-coded STR-005 as deferred scope, from when only Stratum v1 was planned. After STR-005 became an active Ultra 205 row, the guard blocked its own promotion at the final `just parity` check.
3. Preventive rule: When a row's scope or deferral changes, search the parity tool for hard-coded row lists and update them in the same change, replacing a blanket exclusion with the precise rule it protected and keeping its original test case.
4. Trigger signal to catch it earlier: A validation error cites deferred or non-205 scope for an active row, or a checklist row's status leaves `deferred` without a parity-tool diff.

## lesson-retained-log-ring-loses-boot-lines | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: OTA-002 attempt 001 checked the installed boot's one-time `safe_state` line through `/api/system/logs` more than 20 minutes after boot. The 512 KiB retained log ring held only about 1,770 periodic snapshot and health lines by then, so the line had rotated out and the run stopped at its baseline.
3. Preventive rule: Read a boot's one-time retained lines only shortly after that boot is proven (for example right after a restart session). To prove the state of an older boot, use evidence bound to that boot, such as an authenticated session's Worker lease state, not the retained log.
4. Trigger signal to catch it earlier: A check searches `/api/system/logs` for a boot-time marker minutes after the boot, or the retained log is at its byte cap with only periodic records.

## lesson-fixed-serial-jtag-drops-runtime-log-markers | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: The OTAWWW supervisor took its device origin, boot session and safe state from `runtime_origin`, `runtime_boot_identity` and `safe_state` lines in the flash-monitor capture. Since the fixed Serial/JTAG migration the USB writer forwards only allowlisted `usb_*` records, so real captures contain none of those lines; host tests passed because the fake device emitted them.
3. Preventive rule: Take the station address from the authenticated Gate endpoint handoff (`just otawww-endpoint`) and identity and safe state from HTTP or session state. Build fake-device fixtures from what a recent real capture actually contains, not from what the firmware logs.
4. Trigger signal to catch it earlier: A host workflow parses `runtime_origin`, `runtime_boot_identity` or `safe_state` from a USB capture, or a fixture emits serial lines absent from recent post-migration captures.

## lesson-register-new-evidence-errors-in-typed-failure | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: The new OTAWWW supervisor's error class was not added to `tools/automation/src/typed-failure.ts`, so a typed `hardware_blocked` stop was reported as `process_failed`, the wrong category for the attempt policy.
3. Preventive rule: When adding an automation command with its own typed error class, register that class in the typed-failure mapping in the same change and test that a typed failure surfaces with its own category through the CLI result.
4. Trigger signal to catch it earlier: A new `*EvidenceError` class exists without an `instanceof` entry in `typed-failure.ts`, or a supervisor failure prints `process_failed` while its stderr names a specific stage.

## lesson-macos-holds-fresh-executables-at-launch | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: On this macOS host the first exec of a freshly written script or binary stalled 5–93 s at `_dyld_start` with 0% CPU while `syspolicyd` was busy; repeats were instant. Tests that exec freshly written fakes under 10 s bounds failed, and in BWG-007 a 309 s Bazel launch hold let the agent's tool timeout kill `owner-finish` and left a rerun's detector stale.
3. Preventive rule: For time-bounded steps such as detector freshness windows and seals, run already-built binaries directly instead of a path that first builds or writes a new executable. In tests, spawn an existing interpreter on a script (for example `node script.mjs`) rather than exec a freshly written executable, or size the bound for the first-exec hold.
4. Trigger signal: A test writes a fake executable and execs it under a bound of seconds, a freshness-windowed step runs through `just` or Bazel right after a rebuild, or a process sits at `_dyld_start` with 0% CPU.

## lesson-gate-each-hardware-step-on-the-previous-result | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: In BWG-007 attempt 001 an install was chained with `;`, so it ran regardless of the preceding step's result; it stopped before transfer and wrote nothing. In attempt 006 the agent's driver issued the next scenario's steps without checking that the expiry Start had been rejected, so that segment sealed `terminal_reason_mismatch` instead of the real rejection.
3. Preventive rule: Join dependent hardware commands with `&&`, never `;`, so a failed precondition such as detection or `board-info` stops every later effect. Drive a multi-scenario run one scenario at a time and read each result before starting the next. This extends `lesson-surface-preflight-exit-before-advancing` to command chains and scenario drivers.
4. Trigger signal: A hardware command line has `;` between a precondition and an effect, or one driver call issues the steps of more than one scenario.

## lesson-start-per-boot-attempts-on-a-fresh-boot | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: BWG-007 attempt 005 reused attempt 004's boot without a reinstall. The clock stimulus is spent once per boot and the rejection counter is per boot, so the first scenario failed `fact_stimulusCounterConsistent` before testing anything.
3. Preventive rule: When judged facts depend on per-boot one-shot device state, start every attempt on a fresh boot: an install, or the task-permitted `espflash board-info` chip reset. `scripts/bwg-restoration/README.md` states this precondition.
4. Trigger signal: An attempt is about to start on a boot an earlier attempt used, or a first scenario fails a per-boot counter or stimulus fact.
