import assert from "node:assert/strict";
import { once } from "node:events";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { restartFixture } from "./reset-origin-restart-fixtures.mjs";
import { createRestartSupervisor } from "./reset-origin-restart-server.mjs";
const gatePackage = process.argv[2] ?? process.env.RESTART_TEST_GATE_PACKAGE;
test("HTTP context is accepted by the actual pinned Gate parser and its autoload path", { skip: !gatePackage }, async (t) => {
  let close = async () => {};
  t.after(() => close());
  const f = await restartFixture(t),
    server = await createRestartSupervisor(f.options, f.operations);
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  close = async () => {
    try {
      await server.closeQualificationResources();
    } finally {
      await new Promise((resolve) => server.close(resolve));
    }
  };
  const response = await fetch(`http://127.0.0.1:${server.address().port}/context`),
    value = await response.json();
  assert.equal(response.status, 200);
  const child = spawn(
    process.execPath,
    [fileURLToPath(new URL("./reset-origin-restart-gate-config-probe.mjs", import.meta.url)), resolve(gatePackage)],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  let stdout = "",
    stderr = "";
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  child.stdin.end(JSON.stringify(value));
  const [code] = await once(child, "close");
  assert.equal(code, 0, stderr);
  assert.deepEqual(JSON.parse(stdout), {
    actual_gate_configuration_accepted: true,
    autoload_uses_same_payload: true,
    phase_key_rejected: true,
  });
});
