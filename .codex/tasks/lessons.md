## lesson-gsd-frontmatter-body-separators | 2026-06-28 14:14

1. Date: 2026-06-28
2. What went wrong: A GSD summary used standalone `---` body separators after YAML frontmatter. The GSD frontmatter parser scans all `--- ... ---` blocks and selected the last body pair, so lifecycle validation ignored the real frontmatter and failed.
3. Preventive rule: In GSD artifacts and other frontmatter-parsed Markdown, use standalone `---` only for the opening and closing YAML frontmatter delimiters at the top of the file. Use headings or `***` for body breaks instead. Markdown table separator rows such as `| --- |` remain valid.
4. Trigger signal to catch it earlier: Lifecycle validation reports missing frontmatter fields even though the file visibly has them near the top, or a Markdown artifact has more than two standalone `---` lines.

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

## lesson-espflash-no-reset-is-not-passive | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: The retained-runtime capture treated `espflash monitor --no-reset` as a passive serial open. In espflash 4.0.1 that flag suppresses the monitor's final application reset, but the default connection still drives reset lines, synchronizes with the bootloader, and may load the flasher stub.
3. Preventive rule: A passive ESP32-S3 monitor must use all three controls together: `--before no-reset-no-sync --after no-reset --no-reset`, with `--chip esp32s3`. Treat bare `--no-reset` as a reset-capable and bootloader-affecting command.
4. Trigger signal to catch it earlier: Any retained-runtime or no-flash capture renders `espflash monitor` with `--no-reset` but omits either explicit `--before no-reset-no-sync` or `--after no-reset`.

## lesson-power-and-usb-session-are-distinct | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: USB replug, barrel-power retention, both-power cold start, and warm reset were sometimes discussed as interchangeable recovery actions even though they preserve different MCU and USB-peripheral state.
3. Preventive rule: Record barrel/DC state and USB state independently, plus the USB enumeration epoch. Label every action as a USB re-enumeration, warm reset, or true both-power cold start; never infer one from another.
4. Trigger signal to catch it earlier: A hardware checkpoint says only `replug`, `power-cycle`, or `reset` without naming both power paths and the expected USB-session transition.

## lesson-native-usb-capture-needs-prearmed-observation-or-replay | 2026-07-12 04:00

1. Date: 2026-07-12
2. What went wrong: A lifecycle waited for the operator to report barrel-then-USB restoration before opening the native USB monitor. The ESP32-S3 booted from barrel power before the serial node existed, so correct later ownership still captured zero early boot/listener markers.
3. Preventive rule: For native-USB cold-start evidence, arm the exact-node watcher before instructing physical restoration and start passive ownership automatically on node appearance. When the transport cannot preserve pre-enumeration bytes, validate replayable, session-tagged application proof instead of relying on an arbitrary countdown or post-plug acknowledgment.
4. Trigger signal to catch it earlier: A test requires early boot bytes from a serial device whose node is created only after power-up, or asks the operator to confirm plugging before the monitor process begins waiting.

## lesson-boot-proof-replay-must-outlive-service-sessions | 2026-07-12 04:55

1. Date: 2026-07-12
2. What went wrong: The prearmed native-USB watcher acquired the correct node, held passive monitor ownership for the full capture, and cleaned up completely, but firmware emitted no replay markers. Source inspection showed replay was driven only from the live Stratum socket pump, so Wi-Fi or pool-session progress could prevent transport evidence from ever being replayed.
3. Preventive rule: Evidence needed to prove boot independently of external services must be scheduled by a boot-lifetime owner. Keep transport proof, boot proof, listener proof, and network/session proof as separate boundaries with separate failure categories.
4. Trigger signal to catch it earlier: A boot-evidence replay method is called only from a network, socket, pool, HTTP, ASIC-session, or other optional service loop, or a clean serial attachment captures zero bytes without an ownership failure.

## lesson-heartbeat-cannot-prove-over-silent-transport | 2026-07-12 14:03

1. Date: 2026-07-12
2. What went wrong: An always-on boot-lifetime heartbeat passed strict reflash/reinit capture, but the retained both-power cold-start capture was still exactly empty after successful native-USB appearance, stable passive ownership, and a full bounded session. Moving evidence production earlier and making it service-independent did not restore byte delivery through a late-attached USB Serial/JTAG transport.
3. Preventive rule: Treat node appearance, serial ownership, firmware evidence production, and observed byte delivery as four separate boundaries. A heartbeat can measure boot age only after the transport proves it carries application bytes; it cannot substitute for that transport proof.
4. Trigger signal to catch it earlier: Reflash capture contains periodic application heartbeats, but an exact-node late-attach capture has zero bytes despite stable identity, expected ownership, and complete cleanup.

## lesson-manual-removal-needs-owner-observation | 2026-07-12 11:16

1. Date: 2026-07-12
2. What went wrong: A lifecycle accepted the operator's power-removal token before a persistent exact-node owner was watching for disappearance, so the token could attest intent while the transport transition itself remained unobserved.
3. Preventive rule: Start the lifecycle owner and exact-node removal watcher before publishing the removal action. Accept a manual response only after that owner records node disappearance after action publication, then require the complete bounded absence interval.
4. Trigger signal to catch it earlier: A hardware continuation starts its watcher inside `deliver`, or a token can advance state while the selected node is still present or has no owner-recorded disappearance timestamp.

## lesson-physical-usb-identity-excludes-enumeration-fields | 2026-07-12 17:27

1. Date: 2026-07-12
2. What went wrong: A cold-restore gate required both a new enumeration epoch and equality of a supposed physical-USB identity digest. On macOS that digest included `IOCalloutDevice`, `IODialinDevice`, `IOTTYDevice`, `IOTTYBaseName`, and the IORegistry entry ID, so the required re-enumeration could change the value and trigger `appearance_identity_changed` before capture.
3. Preventive rule: Model stable physical identity and enumeration identity separately. A physical-identity digest may use stable hardware attributes such as USB serial number, vendor/product IDs, and stable port location, but must exclude tty paths/names, device-node metadata, and IORegistry entry IDs that are expected to change across enumeration.
4. Trigger signal to catch it earlier: A lifecycle simultaneously requires `new_enumeration_epoch=true` and equality of a digest that contains callout/dial-in device names, tty base names, device-node inode data, or a registry-entry identifier.

## lesson-cold-boot-proof-needs-an-independent-observer | 2026-07-12 16:17

1. Date: 2026-07-12
2. What went wrong: Native USB was used as the authoritative cold-start evidence channel even though the same board power transition removes that transport and recreates it only after early application output may already have occurred. Watcher timing, passive ownership, replay, and heartbeat repairs could prove their own boundaries but could not make the late-enumerated channel preserve original bytes.
3. Preventive rule: When evidence must span destruction and recreation of a device-owned transport, use an independently powered receive-only observer that remains enumerated and open across the transition. Establish a quiet byte boundary while the target is unpowered, validate only post-boundary bytes, and keep target identity separate from observer identity and ownership.
4. Trigger signal to catch it earlier: A test requires original boot bytes while its authoritative reader node disappears with target power or cannot be opened until after the target has begun booting.

## lesson-direct-uart-and-pin-access-requires-authorization | 2026-07-12 18:42

1. Date: 2026-07-12
2. What went wrong: The next hardware plan treated a direct external-UART fixture as acceptable after native-USB evidence remained blocked, even though the user had not agreed to wire UART or manipulate board pads and pins.
3. Preventive rule: Default to the device's provided USB and barrel-power interfaces. Do not propose, request, instruct, or perform direct UART, probe, pin, pad, header, GPIO, jumper, solder, or injected-signal work unless the user explicitly requests that path, or a permanent blocker is documented after non-invasive paths are exhausted; in either case, obtain fresh explicit user authorization before physical instructions or hardware contact.
4. Trigger signal to catch it earlier: A plan or next action mentions RX/GND wiring, test pads, Tag-Connect pins, probes, soldering, jumpers, GPIO manipulation, or an external UART adapter without a recorded explicit authorization checkpoint.

## lesson-protected-evidence-root-ownership | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: A wrapper could pre-create the exact evidence child through output redirection, weakening the supervisor's exclusive creation and rejection boundary before admission or effects.
3. Preventive rule: Create one private parent, prove the supervisor-owned child is absent immediately before launch, and capture wrapper output in separately created private sibling files. The supervisor must reject any existing child before admission, discovery, sensitive-input access, or effects.
4. Trigger signal to catch it earlier: A caller redirects stdout or stderr beneath the requested child, creates the child on the supervisor's behalf, or launches without a fresh absence assertion.

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

## lesson-redact-after-private-classification | 2026-07-20 10:02

1. Date: 2026-07-20
2. What went wrong: Commit redaction transformed the same protected monitor artifact that the Boot A classifier still needed, so required private runtime-origin structure became invalid before the HTTP diagnostic boundary was reached.
3. Preventive rule: Remove `NeverPersistRaw` values before the first write, preserve the resulting mode-`0600` secret-sanitized input for private classification, and produce a distinct commit-redacted shareable copy; never run a lossy redactor in place before all authorized private classifiers have consumed their required fields.
4. Trigger signal to catch it earlier: A downstream classifier requires a sensitive structured field from an artifact that an upstream step also sanitizes, redacts, truncates, or rewrites for sharing.

## lesson-hardware-retries-require-new-information | 2026-07-20 23:43

1. Date: 2026-07-20
2. What went wrong: Repeating a hardware attempt without a verified fix or objectively changed boundary consumed a fresh ordinal but added no information and could reproduce the same failure indefinitely.
3. Preventive rule: Permit another hardware attempt only after one targeted fix is verified across the real failing boundary or an authorized non-invasive remediation objectively proves that boundary changed; stop when the same redacted authoritative boundary signature recurs after its targeted verified fix. A repeated coarse category may return to diagnosis only when closed discriminator fields prove a distinct signature.
4. Trigger signal to catch it earlier: A proposed continuation changes only the attempt number, evidence root, category label, timing, or hope of success while the code, inputs, physical state, and measured boundary signature remain unchanged.

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

## lesson-standing-task-authorization-avoids-confirmation-churn | 2026-08-03 17:37

1. Date: 2026-08-03 17:37 CDT
2. What went wrong: Repository workflows repeatedly required the user to authorize each fresh hardware-attempt ordinal even though the project already had standing authorization to execute its active tasks, creating artificial terminal blockers after every targeted fix.
3. Preventive rule: Treat active repository tasks as standing-authorized for autonomous execution, including selecting fresh attempt ordinals after verified progress, when their exact command, safety, privacy, evidence, recovery, retry, and stop contracts are complete. Do not ask for per-attempt confirmation. Keep materially different direct-UART, pin-manipulation, and ad hoc destructive or fault-injection actions behind their specific safety gates.
4. Trigger signal to catch it earlier: The next safe action is fully described by an active task and repo-owned command, but work is about to stop solely because the task text says a later ordinal needs fresh user authorization.

## lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing | 2026-08-13 14:49

1. Date: 2026-08-13
2. What went wrong: A 30-second physical IDENTIFY effect was triggered before the operator had confirmed they were watching, and the emitted checkpoint said only `rendered` without describing the expected frame. A later normal-screen report was then incorrectly treated as evidence that the frame never rendered even though the effect had already expired. The follow-up fix still guessed that the operator would return within one hour and propagated that estimate into the campaign, fixture, and parent-process lifetimes, which is incompatible with asynchronous work that may pause for hours or overnight.
3. Preventive rule: Never time-bound a safe wait for human availability. Pre-arm a self-describing checkpoint and either keep an explicitly operator-gated owner live or release resources behind a typed resume path. Before any finite physical observation effect, consume readiness locally, then enforce only the effect's exact evidence window; retain independent bounds for automated safety, protocol, recovery, cleanup, and resource phases. Classify late observations as expired authority boundaries rather than positive or negative device evidence.
4. Trigger signal to catch it earlier: Human readiness or response latency appears as a numeric timeout, fixture duration, parent-process budget, or task deadline; a physical effect starts before local readiness; a checkpoint omits the expected state or finite effect window; or a late report is used to classify what was displayed during an expired window.

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
