import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { chmod, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { promisify } from "node:util";

import {
  ownerRecoveryCanRecover,
  parseOwnerRecoveryArgs,
  projectOwnerRecovery,
} from "./native-usb-owner-recovery.js";

const run = promisify(execFile);
const args = [
  "--board", "205",
  "--port", "/dev/cu.fixture",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--private-root", "scratch/native-usb-owner-recovery/attempt-001",
  "--projection", "docs/parity/evidence/native-usb-owner-recovery/owner-projection-001.json",
  "--plan", "docs/parity/work-plans/20260901T161405Z-NATIVE-USB-SERIAL-OWNER-RECOVERY/PLAN.md",
  "--redact-evidence",
] as const;

test("owner recovery parser accepts only the immutable interface", () => {
  // Arrange / Act
  const parsed = parseOwnerRecoveryArgs("observe", args);

  // Assert
  assert.equal(parsed.port, "/dev/cu.fixture");
  assert.equal(parsed.privateRoot, "scratch/native-usb-owner-recovery/attempt-001");
  assert.throws(() => parseOwnerRecoveryArgs("start", args));
  assert.throws(() => parseOwnerRecoveryArgs(
    "observe",
    args.filter(value => value !== "--redact-evidence"),
  ));
});

test("manual BOOT RESET helper writes a mode-0600 fixture checkpoint", async () => {
  // Arrange
  const directory = await mkdtemp(path.join(os.tmpdir(), "owner-recovery-"));
  const intent = path.join(directory, "intent.json");
  const result = path.join(directory, "result.json");
  await writeFile(intent, JSON.stringify({
    schema_version: "bitaxe-native-usb-owner-recovery-prompt-v1",
    operation: "fixture",
  }), { mode: 0o600 });
  await chmod(intent, 0o600);
  const runfiles = process.env["TEST_SRCDIR"];
  const helperRoot = runfiles === undefined ? process.cwd() : path.join(runfiles, "_main");
  const helper = path.join(
    helperRoot,
    "tools/automation/src/macos-native-usb-owner-recovery.swift",
  );

  // Act
  await run("/usr/bin/xcrun", ["swift", helper, "--intent", intent, "--result", result]);

  // Assert
  const checkpoint = JSON.parse(await readFile(result, "utf8")) as Record<string, unknown>;
  assert.equal(checkpoint["status"], "accepted");
  assert.equal(checkpoint["action"], "manual_boot_reset");
  assert.equal((await stat(result)).mode & 0o777, 0o600);
});

test("manual helper preserves cancellation as a closed checkpoint", async () => {
  // Arrange
  const directory = await mkdtemp(path.join(os.tmpdir(), "owner-recovery-cancel-"));
  const intent = path.join(directory, "intent.json");
  const result = path.join(directory, "result.json");
  await writeFile(intent, JSON.stringify({
    schema_version: "bitaxe-native-usb-owner-recovery-prompt-v1",
    operation: "fixture",
    fixture_status: "cancelled",
  }), { mode: 0o600 });
  const runfiles = process.env["TEST_SRCDIR"];
  const helperRoot = runfiles === undefined ? process.cwd() : path.join(runfiles, "_main");
  const helper = path.join(
    helperRoot,
    "tools/automation/src/macos-native-usb-owner-recovery.swift",
  );

  // Act
  await run("/usr/bin/xcrun", ["swift", helper, "--intent", intent, "--result", result]);

  // Assert
  const checkpoint = JSON.parse(await readFile(result, "utf8")) as Record<string, unknown>;
  assert.equal(checkpoint["status"], "cancelled");
});

test("recovery admission is consume-once and stage-bound", () => {
  // Arrange / Act / Assert
  assert.equal(ownerRecoveryCanRecover("manual_required", true), true);
  assert.equal(ownerRecoveryCanRecover("rom_admitted", true), true);
  assert.equal(ownerRecoveryCanRecover("complete", true), false);
  assert.equal(ownerRecoveryCanRecover("manual_required", false), false);
});

test("owner recovery projection excludes private identity and raw transport fields", () => {
  // Arrange
  const machine: Record<string, unknown> = {
    schema_version: "bitaxe-native-usb-owner-recovery-private-v1",
    source_commit: "1".repeat(40),
    reference_commit: "2".repeat(40),
    plan_sha256: "3".repeat(64),
    manifest_sha256: "4".repeat(64),
    restore_bundle_sha256: "5".repeat(64),
    stage: "complete",
    initial_transport: "serial_jtag_runtime",
    passive_marker_status: "trusted",
    execution_owner: "application",
    rom_entry_path: "none",
    force_download_bit_category: "not_read",
    reset_adapter: "none",
    passive_observation_count: 1,
    rom_probe_count: 0,
    manual_prompt_count: 0,
    rom_admission_count: 0,
    force_bit_read_count: 0,
    application_exit_count: 0,
    enumeration_changed: false,
    physical_identity_match: true,
    physical_identity_digest: "6".repeat(64),
    device_write_observed: false,
    host_network_effect: false,
    cleanup_complete: true,
    terminal_category: "complete",
    redaction_status: "passed",
    port: "/dev/private",
    serial_bytes: "private",
  };

  // Act
  const projection = projectOwnerRecovery(machine, "7".repeat(64));

  // Assert
  assert.equal(
    projection["schema_version"],
    "bitaxe-native-usb-owner-recovery-projection-v1",
  );
  assert.equal(Object.hasOwn(projection, "physical_identity_digest"), false);
  assert.equal(Object.hasOwn(projection, "port"), false);
  assert.equal(Object.hasOwn(projection, "serial_bytes"), false);
});

test("owner recovery projection rejects request repetition", () => {
  // Arrange
  const repeated = {
    schema_version: "bitaxe-native-usb-owner-recovery-private-v1",
    source_commit: "1".repeat(40), reference_commit: "2".repeat(40),
    plan_sha256: "3".repeat(64), manifest_sha256: "4".repeat(64),
    restore_bundle_sha256: "5".repeat(64), stage: "complete",
    initial_transport: "serial_jtag_runtime", passive_marker_status: "trusted",
    execution_owner: "application", rom_entry_path: "manual_boot_reset",
    force_download_bit_category: "clear", reset_adapter: "managed_esptool_hard_reset",
    passive_observation_count: 1, rom_probe_count: 1, manual_prompt_count: 2,
    rom_admission_count: 1, force_bit_read_count: 1, application_exit_count: 1,
    enumeration_changed: true, physical_identity_match: true,
    device_write_observed: false, host_network_effect: false, cleanup_complete: true,
    terminal_category: "complete", redaction_status: "passed",
  };

  // Act / Assert
  assert.throws(() => projectOwnerRecovery(repeated, "6".repeat(64)));
});
