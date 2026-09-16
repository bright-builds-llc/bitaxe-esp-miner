import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { fileDigest } from "../fixed-usb-qualification/contract.mjs";
import { processSnapshot } from "./host-resources.mjs";
import { startFixture } from "./fixture-owner.mjs";
import { nodeRuntimeEnvironment } from "./node-runtime.mjs";

for (const mode of ["owner-write-failure", "bad-ready"]) test(`real TERM-ignoring child is reaped after ${mode}`, async (t) => {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), "noise-child-")));
  t.after(() => rm(root, { recursive: true, force: true }));
  let child, childReady;
  const context = { attempt_id: "A".repeat(22), fixture_binary: process.execPath, fixture_sha256: await fileDigest(process.execPath) };
  const failures = [], began = performance.now();
  await assert.rejects(startFixture(root, context, "192.168.1.10", (code) => failures.push(code), {
    networkInterfaces: () => ({ fixture: [{ address: "192.168.1.20", family: "IPv4", netmask: "255.255.255.0", internal: false }] }),
    spawn(_program, _args, options) {
      assert.equal(options.env.PATH, "/usr/bin:/bin");
      assert.deepEqual(Object.keys(options.env).sort(), ["LANG", "LC_ALL", "PATH", "RUST_BACKTRACE"]);
      child = spawn(process.execPath, [fileURLToPath(new URL("./fixture-child.test-helper.mjs", import.meta.url)), resolve(root, "fixture-run"), mode], {
        ...options, env: { ...options.env, ...nodeRuntimeEnvironment() },
      });
      childReady = new Promise((done, reject) => {
        const timer = setTimeout(() => reject(new Error("synthetic fixture readiness timeout")), 5000);
        child.stdout.once("data", () => { clearTimeout(timer); done(); });
        child.once("error", (error) => { clearTimeout(timer); reject(error); });
        child.once("close", () => { clearTimeout(timer); reject(new Error("synthetic fixture exited before readiness")); });
      });
      return child;
    },
    processSnapshot: async () => { await childReady; return processSnapshot(); },
    ...(mode === "owner-write-failure" ? { writeOwner: async () => { throw Object.assign(new Error("synthetic write failure"), { code: "noise_owner_write_failed" }); } } : {}),
  }));
  assert(performance.now() - began < 6500);
  assert.equal(child.signalCode, "SIGKILL");
  assert.throws(() => process.kill(child.pid, 0), { code: "ESRCH" });
  assert.equal(failures[0], mode === "owner-write-failure" ? "noise_owner_write_failed" : "noise_object_shape");
});
