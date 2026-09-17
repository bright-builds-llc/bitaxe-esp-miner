import assert from "node:assert/strict";
import test from "node:test";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { main } from "./main.mjs";
import { permissionFixture } from "./permission-closure.test-helper.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";

test("recovery rejects before reading a nonexistent or inaccessible attempt", async () => {
  await assert.rejects(main(["recover", "--private-root", "/unreadable/share-001"]), { code: "v2_serial_recovery_unavailable" });
});

test("serve requires protected stdout before context or signing authority access", () => {
  // Arrange / Act: the child has a pipe even when Bazel directs this test to a protected file.
  const child = spawnSync(process.execPath, [fileURLToPath(new URL("main.mjs", import.meta.url)),
    "serve", "--private-root", "/unreadable/share-001", "--authority-directory", "/unreadable/authority"],
  { encoding: "utf8", timeout: 5000, env: { PATH: "/usr/bin:/bin", ...nodeRuntimeEnvironment() } });
  // Assert
  assert.equal(child.error, undefined); assert.equal(child.status, 1);
  assert.equal(JSON.parse(child.stdout).error, "v2_protected_stdout_required");
});

test("unknown commands cannot import effect handlers", async () => {
  await assert.rejects(main(["flash", "--private-root", "/unreadable/share-001"]), { code: "v2_action_invalid" });
});

test("permission CLI closure and review expose only the allowlisted effect-free summary", async t => {
  // Arrange
  const f = await permissionFixture(t);
  // Act
  const closed = await main(["close-permission", "--private-root", f.root], f.operations);
  f.operations.processSnapshot = () => { assert.fail("historical review must not rescan owners"); };
  f.operations.execFileSync = () => { assert.fail("historical review must not inspect holders"); };
  const reviewed = await main(["review-permission", "--private-root", f.root], f.operations);
  // Assert
  assert.deepEqual(reviewed, closed);
  assert.deepEqual(Object.keys(closed).sort(), ["status", "classification", "hardware_qualified", "device_effects", "context_sha256", "closure_sha256"].sort());
  assert.equal(closed.hardware_qualified, false); assert.equal(closed.device_effects, false);
  for (const secret of [f.root, f.context.firmware_root, f.context.gate_root, "evaluator", "permission-correction"])
    assert(!JSON.stringify(closed).includes(secret));
});
