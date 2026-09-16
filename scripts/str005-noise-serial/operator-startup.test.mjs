import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { fixture } from "./test-fixture.mjs";
import { observeCommand } from "./operator.mjs";
import { nodeRuntimeEnvironment } from "./node-runtime.mjs";

test("Node launcher environment excludes credentials and unrelated variables", () => {
  assert.deepEqual(nodeRuntimeEnvironment({ JS_BINARY__NODE_BINARY: "/node", SECRET: "not inherited", NODE_OPTIONS: "not inherited" }),
    { JS_BINARY__NODE_BINARY: "/node" });
});

for (const mode of ["missing-executable", "early-close", "never-ready"]) test(`automatic ${mode} fails without orphaned child or promise rejection`, async (t) => {
  const f = await fixture(t); let child;
  const unhandled = [];
  const record = (reason) => unhandled.push(reason);
  process.on("unhandledRejection", record); t.after(() => process.off("unhandledRejection", record));
  const began = performance.now();
  await assert.rejects(observeCommand(f.root, f.context, "detect", 0, {
    readyTimeoutMs: 100,
    spawn(_command, _args, options) {
      child = mode === "missing-executable" ? spawn("/nonexistent/noise-child", [], options) :
        spawn(process.execPath, [fileURLToPath(new URL("./operator-startup.test-helper.mjs", import.meta.url)), mode], options);
      return child;
    },
  }));
  assert(performance.now() - began < 6500);
  if (child.pid) assert.throws(() => process.kill(child.pid, 0), { code: "ESRCH" });
  assert.equal(unhandled.length, 0);
});
