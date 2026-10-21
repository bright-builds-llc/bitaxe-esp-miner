## lesson-esp-idf-service-ownership-and-redaction | 2026-07-02 23:29

1. Date: 2026-07-02
2. What went wrong: Wi-Fi startup initialized the default ESP-IDF event loop through raw `esp_event_loop_create_default()` before `EspSystemEventLoop::take()`, so esp-idf-svc's ownership tracker returned `ESP_ERR_INVALID_STATE`. The first hardware evidence run also showed that ESP-IDF Wi-Fi driver logs can expose the connected SSID outside JSON or `key=value` fields.
3. Preventive rule: Let esp-idf-svc own managed ESP-IDF service handles such as the default event loop; use raw idempotent init only for services without a wrapper ownership tracker. Redaction tests must include vendor log formats, not only project log formats.
4. Trigger signal to catch it earlier: A managed `take()` API fails with `ESP_ERR_INVALID_STATE` immediately after a raw init call, or sanitized serial evidence still contains natural-language Wi-Fi driver lines such as `wifi:connected with ...`.

## lesson-cross-process-tests-use-real-boundaries | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: In-process fixtures passed while the first real lifecycle continuation failed because a Unix-socket receiver mixed buffered line input with unbuffered payload reads. Other live-only failures involved process-group descendants, fresh-process capability parsing, and Bazel/runfiles execution resolving helpers or tools differently from the source-tree shell.
3. Preventive rule: Test IPC, process ownership, framing, capabilities, and Bazel/runfiles entrypoints through real fresh processes, Unix sockets, coalesced and fragmented writes, process groups, and mode-enforced files. Exercise sibling helper and tool resolution from the deployed layout, and prevent production children from invoking nested build tools. Keep pure tests too, but do not let them substitute for the operating-system boundary that production uses.
4. Trigger signal to catch it earlier: A test injects a function or prebuilt object where production crosses a socket, process, PTY, file-permission, process-group, or runfiles boundary; resolves helpers only from the source tree; or allows a launched child to call the build runner again.

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

## lesson-distinguish-agent-runtime-from-host-runtime | 2026-08-17 12:05

1. Date: 2026-08-17 12:05 CDT
2. What went wrong: Repeated process-launch stalls and 300-second test timeouts inside the Codex execution environment were diagnosed as machine-wide macOS degradation requiring a reboot, but the user ran the exact uncached Bazel automation target in 68.1 seconds and the exact filtered Cargo command in 0.6 seconds from their normal shell.
3. Preventive rule: Treat timeouts observed only inside the agent execution environment as agent-session or sandbox failures until the same exact uncached command is independently reproduced in the user's normal shell. Do not recommend a host reboot or declare a machine-wide blocker from agent-only timing evidence.
4. Trigger signal: Agent tool calls show inconsistent multi-minute gaps between otherwise passing child processes, while an external user shell has not reproduced the delay or reports normal timings for the exact command.

## lesson-independent-review-before-parity-promotion | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The STR-005 review numbers were correct, but an independent adversarial review still found overclaims: renewal coverage taken from an `unverified` seal without its caveats, a renewal reply described as signed when it is only matched, an older-image result cited as candidate evidence, and a blocking dependency described as non-blocking.
3. Preventive rule: Before any parity promotion, run an independent reviewer that re-derives every cited number from sealed roots and challenges each claim's framing. Evidence taken from an unverified seal must say so and carry its judge's blockers.
4. Trigger signal to catch it earlier: Evidence summaries written by the same agent that produced the runs, a covering result whose seal is not `complete`, or a protocol property asserted from documentation rather than the implementing source.

## lesson-gate-each-hardware-step-on-the-previous-result | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: In BWG-007 attempt 001 an install was chained with `;`, so it ran regardless of the preceding step's result; it stopped before transfer and wrote nothing. In attempt 006 the agent's driver issued the next scenario's steps without checking that the expiry Start had been rejected, so that segment sealed `terminal_reason_mismatch` instead of the real rejection.
3. Preventive rule: Join dependent hardware commands with `&&`, never `;`, so a failed precondition such as detection or `board-info` stops every later effect. Drive a multi-scenario run one scenario at a time and read each result before starting the next. This extends `lesson-surface-preflight-exit-before-advancing` to command chains and scenario drivers.
4. Trigger signal: A hardware command line has `;` between a precondition and an effect, or one driver call issues the steps of more than one scenario.

## lesson-never-edit-under-a-running-gate | 2026-10-17

1. Date: 2026-10-17
2. What went wrong: Editing files while the full gate ran produced spurious gate failures and wasted reruns. It recurred on 2026-10-21, when review fixes were applied under a running gate and its result had to be discarded.
3. Preventive rule: Stop a running gate before editing, or wait for it to finish; never edit the tree under a running gate. A gate result counts only for the tree it ran on.
4. Trigger signal: About to edit while a gate background task is active, or a review finishes while its gate still runs.

## lesson-pinned-evidence-is-immutable | 2026-10-21

1. Date: 2026-10-21
2. What went wrong: A dated note added to the soak 005 evidence file, which a verified-revision authority pins by SHA-256, made `just parity` fail ("verified revision evidence ... changed"). The failure went unnoticed for hours because the local gate script grepped the parity output without checking its exit status.
3. Preventive rule: Before editing anything under `docs/parity/evidence/`, search `docs/parity/verified-revisions/*.json` for its path. If an authority pins it, add a new sibling evidence file that references it instead. Gate scripts record every command's exit status, never only filtered output.
4. Trigger signal: Editing an existing evidence file, `just parity` reporting a changed evidence digest, or a gate summary that prints output lines but no exit codes.
