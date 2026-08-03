import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort } from "./process.js";
import { captureVersionEvidence, hasPassiveSafeState } from "./version-evidence.js";

test("late-attached trusted runtime attestation proves the passive safe state", () => {
  // Arrange
  const trusted = "runtime_boot_attestation schema_version=1 mining=disabled work_submission=disabled hardware_control=disabled redacted=true";
  const activeMining = "runtime_boot_attestation schema_version=1 mining=active work_submission=enabled hardware_control=enabled redacted=true";

  // Act
  const trustedResult = hasPassiveSafeState(trusted);
  const activeResult = hasPassiveSafeState(activeMining);

  // Assert
  assert.equal(trustedResult, true);
  assert.equal(activeResult, false);
});

test("version workflow uses one typed exact-package effect and emits only a redacted projection", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-version-workflow-"));
  await mkdir(path.join(workspace, "package"));
  await mkdir(path.join(workspace, "scratch"));
  await writeFile(path.join(workspace, "package", "manifest.json"), JSON.stringify({
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
  }));
  await writeFile(path.join(workspace, "wifi.json"), "{}", { mode: 0o600 });
  const calls: string[][] = [];
  const fake = createFakeProcessPort(async (spec) => {
    calls.push([...spec.args]);
    if (spec.args[0] === "flash-monitor") {
      const evidenceIndex = spec.args.indexOf("--evidence-dir");
      const root = spec.args[evidenceIndex + 1];
      assert.notEqual(root, undefined);
      await writeFile(path.join(root as string, "flash-monitor.classifier-input.log"), [
        "runtime_boot_identity session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=1 reset_reason=power_on uptime_ms=1 redacted=true",
        "runtime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=1 device_url=http://device.test redacted=true",
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
      ].join("\n"), { mode: 0o600 });
    }
    return { exitCode: 0, stdout: "", stderr: "", timedOut: false };
  });
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => new Response(JSON.stringify({ version: "test" }), {
    status: 200,
    headers: { "content-type": "application/json" },
  });

  try {
    // Act
    const evidence = await captureVersionEvidence(workspace, {
      privateRoot: "scratch/attempt-001",
      packageManifest: "package/manifest.json",
      wifiCredentials: "wifi.json",
      port: "/dev/cu.test",
      projection: "shareable/version.json",
      captureTimeoutSeconds: 45,
    }, fake, "flash", "validator");

    // Assert
    assert.equal(calls.filter((call) => call[0] === "flash-monitor").length, 1);
    assert.equal(evidence.schema_version, "bitaxe-version-evidence-v1");
    const projection = await readFile(path.join(workspace, "shareable", "version.json"), "utf8");
    assert.equal(projection.includes("device.test"), false);
    assert.equal(projection.includes("/dev/cu.test"), false);
    assert.equal((await stat(path.join(workspace, "scratch", "attempt-001"))).mode & 0o777, 0o700);
  } finally {
    globalThis.fetch = originalFetch;
    await rm(workspace, { recursive: true });
  }
});
