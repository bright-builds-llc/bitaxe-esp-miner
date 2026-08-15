import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  ApiCommandEffectsError,
  captureApiCommandEffects,
  type ApiCommandEffectsOptions,
} from "./api-command-effects.js";
import { createFakeProcessPort, type ProcessOutcome } from "./process.js";

const discardCheckpoint = () => undefined;
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const readySession = {
  schema_version: "esp-device-session-v1",
  terminal_category: "ready",
  platform_category: "macos",
  board_category: "205",
  same_physical_device: true, stable_enumeration: true,
  reenumerated: false, reader_armed: true,
  pre_restart_serial_delivery: true, post_restart_serial_delivery: true,
  serial_delivery: "correlated", request_outcome: "response_received",
  request_attempt_count: 1,
  service_loss_observed: true,
  trusted_origin_preserved: true,
  application_recovered: true,
  build_identity_matches: true,
  boot_session_changed: true,
  boot_ordinal_advanced_by_one: true,
  software_reset_observed: true,
  postcondition_matches: true,
  cleanup_complete: true,
  usb_disappearance_count: 0,
  enumeration_change_count: 0,
  serial_byte_count: 128,
  http_observation_count: 3,
  duration_millis: 1_000,
} as const;

const completeEffects = {
  schema: "mining-campaign-command-effects-v7",
  genuine_block_notification_observed: true,
  positive_block_count_observed: true,
  pause_request_count: 1,
  pause_confirmed: true,
  resume_request_count: 1,
  resume_intent_confirmed: true,
  resume_confirmed: true,
  identify_status_baseline_confirmed: true,
  identify_request_count: 1,
  identify_render_receipt_confirmed: true,
  identify_clear_receipt_confirmed: true,
  serial_transition_witnesses_confirmed: true,
  websocket_transition_witnesses_confirmed: true,
  identify_terminal_outcome: "none",
  dismiss_request_count: 1,
  dismiss_confirmed: true,
  block_count_preserved: true,
  active_before_pause: true,
  active_after_resume: true,
  recovery_pause_api_confirmed: false, recovery_pause_serial_confirmed: false,
  recovery_safe_stop_confirmed: false,
  recovery_terminal_outcome: "not_required",
  same_boot_and_package: true,
  safety_valid: true,
  terminal_http_valid: true,
  terminal_pool_persisted: true,
} as const;

const readyReadinessTransition = {
  wakeup: "observations_changed", previous_blocker: "safety_prerequisites_stale",
  current_blocker: "none", session_phase: "waiting_for_readiness",
  campaign_state: "armed", hardware_state: "stopped",
  safety_sample: "fresh", observation_epoch: "advanced",
  pending_observation_recovered: true,
} as const;

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value)}\n`, { mode: 0o600 });
  await chmod(output, 0o600);
}

async function fixture(): Promise<{ root: string; options: ApiCommandEffectsOptions }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-effects-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const wifi = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
  }));
  await writeFile(wifi, "{}\n");
  return {
    root,
    options: {
      privateRoot: "scratch/attempt-001",
      packageManifest: manifest,
      wifiCredentials: wifi,
      port: "/dev/private-sensitive-port",
      projection: path.join(root, "docs", "api-command-effects.json"),
      durationSeconds: 600,
    },
  };
}

function fakePort(
  root: string,
  maybeSession: unknown = readySession,
  campaignFails = false,
  flashDiagnosticsMode: "ready" | "malformed" | "missing" = "ready",
  protocolGate = "ready",
  readinessMode: "ready" | "malformed" | "missing" = "ready",
  checkpointMode: "absent" | "malformed" | "ordered" | "replayed" | "wrong_mode" | "wrong_order" = "ordered",
  commandEffects: Readonly<Record<string, unknown>> = completeEffects,
) {
  return createFakeProcessPort(async (spec) => {
    if (spec.program === "/sbin/route") return ok("interface: en0\n");
    if (spec.program === "/usr/sbin/ipconfig") return ok("192.0.2.44\n");
    const attempt = path.join(root, "scratch", "attempt-001");
    if (spec.program.endsWith("api-command-effects-stratum-pool")) {
      await privateJson(path.join(attempt, "fixture-ready.private.json"), {
        status: "ready",
        fixture: "api-command-effects-v1",
        bound_port: 43210,
      });
      while (true) {
        try {
          await readFile(path.join(attempt, "fixture.stop.private"), "utf8");
          break;
        } catch (error) {
          if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
          await new Promise((resolve) => setTimeout(resolve, 1));
        }
      }
      await privateJson(path.join(attempt, "fixture-report.private.json"), {
        status: "stopped",
        fixture: "api-command-effects-v1",
        method_counts: {
          "mining.configure": 1,
          "mining.subscribe": 1,
          "mining.authorize": 1,
          "mining.submit": 2,
        },
        configure_observed: true,
        subscribe_observed: true,
        authorize_observed: true,
        submit_observed: true,
        notify_sent_count: 2,
        accepted_submit_count: 2,
        source_work_fingerprint: "d".repeat(64),
        compact_network_target: "207fffff",
        raw_messages_committed: false,
        credential_contents_read: false,
      });
      return ok();
    }
    if (spec.program.endsWith("flash")) {
      assert.deepEqual(spec.args.filter((value) => value === "--stage" || value === "command-effects"), [
        "--stage", "command-effects",
      ]);
      const campaign = path.join(attempt, "campaign");
      await mkdir(campaign, { mode: 0o700 });
      if (checkpointMode === "ordered" || checkpointMode === "replayed") {
        await privateJson(path.join(campaign, "identify-ready.required.json"), {
          schema: "bitaxe-identify-checkpoint-v3",
          checkpoint: "ready",
          status: "required",
        });
        await privateJson(path.join(campaign, "identify-rendered.required.json"), {
          schema: "bitaxe-identify-checkpoint-v3",
          checkpoint: "rendered",
          status: "required",
        });
        if (checkpointMode === "replayed") {
          await privateJson(path.join(campaign, "identify-replayed.required.json"), {
            schema: "bitaxe-identify-checkpoint-v3",
            checkpoint: "replayed",
            status: "required",
          });
        }
        await privateJson(path.join(campaign, "identify-cleared.required.json"), {
          schema: "bitaxe-identify-checkpoint-v3",
          checkpoint: "cleared",
          status: "required",
        });
      } else if (checkpointMode === "malformed") {
        await privateJson(path.join(campaign, "identify-ready.required.json"), {
          schema: "private-invalid-schema",
          checkpoint: "ready",
          status: "required",
        });
      } else if (checkpointMode === "wrong_order") {
        await privateJson(path.join(campaign, "identify-rendered.required.json"), {
          schema: "bitaxe-identify-checkpoint-v3",
          checkpoint: "rendered",
          status: "required",
        });
      } else if (checkpointMode === "wrong_mode") {
        const ready = path.join(campaign, "identify-ready.required.json");
        await privateJson(ready, {
          schema: "bitaxe-identify-checkpoint-v3",
          checkpoint: "ready",
          status: "required",
        });
        await chmod(ready, 0o644);
      }
      const readyFlashDiagnostic = {
        schema_version: "esp-usb-command-diagnostic-v1",
        terminal_category: "ready",
        device_effect_state: "completed",
        termination: "exited_success",
        attempt_count: 1,
        connection_signature: "not_applicable",
        stdout_bytes: 0,
        stderr_bytes: 0,
        stdout_sha256: "0".repeat(64),
        stderr_sha256: "0".repeat(64),
        transfer_started: true,
        transfer_completed: true,
        raw_output_included: false,
      };
      const flashDiagnosticsDocument = flashDiagnosticsMode === "malformed"
        ? "{}\n"
        : `${JSON.stringify({
          schema: "mining-campaign-flash-diagnostics-v1",
          factory: readyFlashDiagnostic,
          nvs: readyFlashDiagnostic,
          raw_output_included: false,
        })}\n`;
      if (flashDiagnosticsMode !== "missing") {
        await writeFile(path.join(campaign, "campaign-flash.private.json"), flashDiagnosticsDocument, { mode: 0o600 });
      }
      await privateJson(path.join(campaign, "campaign-result.json"), {
        schema: "mining-campaign-result-v8",
        stage: "command-effects",
        status: campaignFails ? "failed" : "accepted",
        terminal_category: campaignFails ? "command_request_failed" : "command_effects_complete",
        terminal_reason: "none",
        runtime_identity: "trusted",
        protocol_gate: protocolGate,
        ...(readinessMode === "missing" ? {} : {
          readiness_transition: readinessMode === "malformed"
            ? { ...readyReadinessTransition, observation_epoch: "private-invalid-value" }
            : readyReadinessTransition,
        }),
        safe_stop: campaignFails ? "pending" : "confirmed",
        usb_cleanup: "ready",
        qualified_candidate_count: 1,
        flash_diagnostics_sha256: createHash("sha256").update(flashDiagnosticsDocument).digest("hex"),
        redacted: true,
      });
      await privateJson(path.join(campaign, "campaign-network.private.json"), {
        status: campaignFails ? "failed" : "accepted",
        recovery_pause_request_count: campaignFails ? 1 : 0,
        command_effects: campaignFails
          ? {
            ...commandEffects,
            recovery_pause_api_confirmed: true, recovery_pause_serial_confirmed: true,
            recovery_safe_stop_confirmed: true,
            recovery_terminal_outcome: "confirmed",
          }
          : commandEffects,
      });
      await privateJson(path.join(campaign, "command-effects-reboot-intent.private.json"), {
        schema_version: "esp-device-session-reboot-intent-v1",
        board_category: "205",
        trusted_origin: "http://192.0.2.44",
        baseline: {
          boot_session: "a".repeat(32),
          boot_ordinal: 7,
          source_commit: "a".repeat(40),
          reference_commit: "b".repeat(40),
          app_elf_sha256: "c".repeat(64),
        },
        expected_postcondition: { hostname_sha256: "d".repeat(64) },
      });
      return { ...ok(), exitCode: campaignFails ? 1 : 0 };
    }
    if (spec.args[0] === "transact-live") {
      const output = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
      await privateJson(output, maybeSession);
      return ok();
    }
    throw new Error(`unexpected child ${spec.program}`);
  });
}

test("the production campaign completes without operator checkpoints", async () => {
  // Arrange
  const value = await fixture();
  let checkpointSignalCount = 0;

  // Act
  await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, false, "ready", "ready", "ready", "absent"),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    () => { checkpointSignalCount += 1; },
  );

  // Assert
  assert.equal(checkpointSignalCount, 0);
});

test("campaign failure remains primary when its checkpoint is malformed", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, true, "ready", "ready", "ready", "malformed"),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  assert.equal(error.publicValue["safe_stop_confirmed"], true);
});

test("a non-ready protocol gate withholds the final projection", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, false, "ready", "transaction_unavailable"),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("a missing readiness transition withholds the final projection", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, false, "ready", "ready", "missing"),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("a non-closed readiness transition withholds the final projection", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, false, "ready", "ready", "malformed"),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("complete command and reboot quorums publish only redacted typed evidence", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const evidence = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  );

  // Assert
  const published = await readFile(value.options.projection, "utf8");
  const displayUatIntent = JSON.parse(await readFile(
    path.join(value.root, "scratch", "attempt-001", "display-uat-intent.private.json"),
    "utf8",
  )) as Record<string, unknown>;
  assert.equal((evidence as Record<string, unknown>)["schema_version"], "bitaxe-api-command-effects-evidence-v1");
  assert.equal((evidence as Record<string, unknown>)["hardware_control_state"], "disabled");
  assert.equal(displayUatIntent["schema_version"], "bitaxe-display-uat-intent-v1");
  assert.equal(displayUatIntent["programmatic_evidence_sha256"], createHash("sha256").update(published).digest("hex"));
  assert(!published.includes("192.0.2.44"));
  assert(!published.includes("/dev/private-sensitive-port"));
  assert(!published.includes("api009.fixture"));
  assert(!published.includes("poolPassword"));
});

test("campaign failure keeps its primary category and reports secondary recovery facts", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, readySession, true),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  assert.deepEqual(error.publicValue, {
    stage: "command_effects",
    safe_stop_confirmed: true,
    cleanup_complete: true,
    recovery_attempted: true,
    secondary_recovery_failure: false,
  });
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("a non-ready reboot withholds the final projection", async () => {
  // Arrange
  const value = await fixture();
  const session = { ...readySession, terminal_category: "device_not_reacquired" };

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root, session),
    path.join(value.root, "bin", "api-command-effects-stratum-pool"),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

for (const flashDiagnosticsMode of ["malformed", "missing"] as const) {
  test(`${flashDiagnosticsMode} flash diagnostics withhold the final projection`, async () => {
    // Arrange
    const value = await fixture();

    // Act
    const error = await captureApiCommandEffects(
      value.root,
      value.options,
      fakePort(value.root, readySession, false, flashDiagnosticsMode),
      path.join(value.root, "bin", "api-command-effects-stratum-pool"),
      path.join(value.root, "bin", "flash"),
      path.join(value.root, "bin", "device-session"),
      discardCheckpoint,
    ).then(() => undefined, (caught: unknown) => caught);

    // Assert
    assert(error instanceof ApiCommandEffectsError);
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
  });
}
