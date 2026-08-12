import assert from "node:assert/strict";
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

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const readySession = {
  schema_version: "esp-device-session-v1",
  terminal_category: "ready",
  platform_category: "macos",
  board_category: "205",
  same_physical_device: true,
  stable_enumeration: true,
  reenumerated: false,
  reader_armed: true,
  pre_restart_serial_delivery: true,
  post_restart_serial_delivery: true,
  serial_delivery: "correlated",
  request_outcome: "response_received",
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
  schema: "mining-campaign-command-effects-v1",
  genuine_block_notification_observed: true,
  positive_block_count_observed: true,
  pause_request_count: 1,
  pause_confirmed: true,
  resume_request_count: 1,
  resume_confirmed: true,
  identify_request_count: 2,
  identify_rendered_confirmed: true,
  identify_cleared_confirmed: true,
  dismiss_request_count: 1,
  dismiss_confirmed: true,
  block_count_preserved: true,
  active_before_pause: true,
  active_after_resume: true,
  same_boot_and_package: true,
  safety_valid: true,
  terminal_http_valid: true,
  terminal_pool_persisted: true,
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

function fakePort(root: string, maybeSession: unknown = readySession, campaignFails = false) {
  return createFakeProcessPort(async (spec) => {
    if (spec.program === "/sbin/route") return ok("interface: en0\n");
    if (spec.program === "/usr/sbin/ipconfig") return ok("192.0.2.44\n");
    const attempt = path.join(root, "scratch", "attempt-001");
    if (spec.program === process.execPath) {
      await privateJson(path.join(attempt, "fixture-ready.private.json"), {
        status: "ready",
        fixture: "api-command-effects-v1",
        bound_port: 43210,
      });
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
      await privateJson(path.join(campaign, "campaign-result.json"), {
        schema: "mining-campaign-result-v5",
        stage: "command-effects",
        status: campaignFails ? "failed" : "accepted",
        terminal_category: campaignFails ? "command_request_failed" : "command_effects_complete",
        runtime_identity: "trusted",
        safe_stop: "confirmed",
        usb_cleanup: "ready",
        qualified_candidate_count: 1,
        redacted: true,
      });
      await privateJson(path.join(campaign, "campaign-network.private.json"), {
        status: campaignFails ? "failed" : "accepted",
        recovery_pause_request_count: campaignFails ? 1 : 0,
        command_effects: completeEffects,
      });
      await privateJson(path.join(campaign, "command-effects-reboot-intent.private.json"), {
        schema_version: "esp-device-session-reboot-intent-v1",
      });
      return { ...ok(), exitCode: campaignFails ? 1 : 0 };
    }
    if (spec.args[0] === "reboot-live") {
      const output = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
      await privateJson(output, maybeSession);
      return ok();
    }
    throw new Error(`unexpected child ${spec.program}`);
  });
}

test("complete command and reboot quorums publish only redacted typed evidence", async () => {
  // Arrange
  const value = await fixture();

  // Act
  const evidence = await captureApiCommandEffects(
    value.root,
    value.options,
    fakePort(value.root),
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
  );

  // Assert
  const published = await readFile(value.options.projection, "utf8");
  assert.equal((evidence as Record<string, unknown>)["schema_version"], "bitaxe-api-command-effects-evidence-v1");
  assert.equal((evidence as Record<string, unknown>)["hardware_control_state"], "disabled");
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
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
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
    path.join(value.root, "bin", "flash"),
    path.join(value.root, "bin", "device-session"),
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});
