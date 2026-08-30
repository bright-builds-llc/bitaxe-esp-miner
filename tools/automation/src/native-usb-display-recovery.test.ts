import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { chmod, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { promisify } from "node:util";

import {
  createDisplayRecoveryRoot,
  displayCaptureRetryEligible,
  parseDisplayRecoveryArgs,
  projectDisplayRecovery,
} from "./native-usb-display-recovery.js";

const run = promisify(execFile);
const common = [
  "--board", "205", "--port", "/dev/cu.fixture",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--wifi-credentials", "wifi-credentials.json",
  "--pool-credentials", "pool-credentials.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--private-root", "scratch/native-usb-display-recovery/attempt-001",
  "--projection", "docs/parity/evidence/native-usb-display-recovery/recovery-projection-001.json",
  "--plan", "docs/parity/work-plans/20260830T161148Z-NATIVE-USB-DISPLAY-RECOVERY/PLAN.md",
  "--redact-evidence",
] as const;

test("display recovery parser accepts only the task-bound interface", () => {
  // Arrange / Act
  const parsed = parseDisplayRecoveryArgs("preflight", common);

  // Assert
  assert.equal(parsed.port, "/dev/cu.fixture");
  assert.equal(parsed.privateRoot, "scratch/native-usb-display-recovery/attempt-001");
  assert.throws(() => parseDisplayRecoveryArgs("preflight", [...common, "--board", "601"]));
});

test("display recovery creates its nested private root", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "display-root-"));

  // Act
  await createDisplayRecoveryRoot(workspace);

  // Assert
  const metadata = await stat(path.join(workspace, "scratch/native-usb-display-recovery/attempt-001"));
  assert.equal(metadata.isDirectory(), true);
  assert.equal(metadata.mode & 0o777, 0o700);
});

test("effect-free cancellation permits one fresh capture generation", () => {
  // Arrange / Act / Assert
  assert.equal(displayCaptureRetryEligible({ status: "cancelled" }, undefined), true);
  assert.equal(displayCaptureRetryEligible({ status: "accepted" }, undefined), false);
  assert.equal(displayCaptureRetryEligible(
    { status: "accepted" },
    { eligible: true, settings_request_count: 0 },
  ), true);
});

test("macOS capture helper writes one mode-0600 fixture result", async () => {
  // Arrange
  const directory = await mkdtemp(path.join(os.tmpdir(), "display-origin-"));
  const intent = path.join(directory, "intent.json");
  const result = path.join(directory, "result.json");
  await writeFile(intent, JSON.stringify({
    schema_version: "bitaxe-native-usb-display-origin-prompt-v1",
    operation: "fixture",
    generation: 1,
    fixture_status: "accepted",
    fixture_ipv4: "192.168.1.23",
  }), { mode: 0o600 });
  await chmod(intent, 0o600);
  const runfiles = process.env["TEST_SRCDIR"];
  const helperRoot = runfiles === undefined ? process.cwd() : path.join(runfiles, "_main");
  const helper = path.join(helperRoot, "tools/automation/src/macos-display-origin-capture.swift");

  // Act
  await run("/usr/bin/xcrun", ["swift", helper, "--intent", intent, "--result", result]);

  // Assert
  const value = JSON.parse(await readFile(result, "utf8")) as Record<string, unknown>;
  assert.equal(value["status"], "accepted");
  assert.equal(value["ipv4"], "192.168.1.23");
  assert.equal((await stat(result)).mode & 0o777, 0o600);
});

test("display recovery projection excludes raw private fields", () => {
  // Arrange
  const machine: Record<string, unknown> = {
    schema_version: "bitaxe-native-usb-display-recovery-machine-v1",
    source_commit: "1".repeat(40), reference_commit: "2".repeat(40),
    plan_sha256: "3".repeat(64), evaluator_sha256: "4".repeat(64),
    package_manifest_sha256: "5".repeat(64), restore_bundle_sha256: "6".repeat(64),
    capture_sha256: "7".repeat(64), usb_receipt_sha256: "8".repeat(64),
    display_origin_supplied: true, private_ipv4: true, usb_mac_bound: true,
    recovery_identity_exact: true, settings_exact: true, theme_exact: true,
    mineonboot_disabled: true, mining_inactive: true, zero_work: true,
    stable_physical_identity: true, cleanup_complete: true,
    settings_request_count: 1, theme_request_count: 1, reconciliation_read_count: 2,
    terminal_category: "complete", redaction_status: "passed",
    ipv4: "192.168.1.23", macAddr: "fixture",
  };

  // Act
  const projection = projectDisplayRecovery(machine, { exit_code: 0 });

  // Assert
  assert.equal(projection["schema_version"], "bitaxe-native-usb-display-recovery-projection-v1");
  assert.equal(Object.hasOwn(projection, "ipv4"), false);
  assert.equal(Object.hasOwn(projection, "macAddr"), false);
});
