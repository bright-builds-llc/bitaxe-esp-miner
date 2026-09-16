import assert from "node:assert/strict";
import { once } from "node:events";
import { chmod, mkdir, readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import test from "node:test";
import { fixture, state, ledger, original } from "./test-fixture.mjs";
import { createSupervisor } from "./server.mjs";
import { installCandidate } from "./operator.mjs";
import { processSnapshot } from "./host-resources.mjs";
import { writeNew } from "./files.mjs";
import { quoteJustArgument } from "./operator-execution.mjs";

test("repo operator observes real gated child processes before any synthetic install effect", async (t) => {
  const f = await fixture(t), bin = resolve(f.base, "bin"); await mkdir(bin, { mode: 0o700 });
  const program = resolve(bin, "just");
  const script = resolve(bin, "fake-just.mjs");
  await writeFile(script, await readFile(new URL("./fake-just.test-helper.mjs", import.meta.url)), { mode: 0o600 });
  await writeFile(program, `#!/bin/sh\nexec ${quoteJustArgument(process.execPath)} ${quoteJustArgument(script)} "$@"\n`, { mode: 0o700 });
  f.operations.path = `${bin}:${process.env.PATH}`; f.operations.childProgram = fileURLToPath(new URL("./operator-child.test-helper.mjs", import.meta.url)); f.operations.processSnapshot = processSnapshot;
  // Delay the actual observer receipt to prove readiness awaits persisted arming.
  f.operations.writeObserverProof = async (path, value) => {
    if (path.endsWith(".observer-armed.json")) await new Promise((done) => setTimeout(done, 150));
    await writeNew(path, value);
  };
  const server = await createSupervisor(f.options, f.operations);
  server.listen(0, "127.0.0.1"); await once(server, "listening"); await server.qualificationReady;
  t.after(async () => { server.closeAllConnections(); await new Promise((done) => server.close(done)); await server.closeQualificationResources(); });
  const origin = `http://127.0.0.1:${server.address().port}`;
  async function post(path, input) {
    const response = await fetch(`${origin}${path}`, { method: "POST", headers: { Origin: origin, "Content-Type": "application/json" }, body: JSON.stringify(input) });
    const result = await response.json(); assert.equal(response.status, 200, JSON.stringify(result)); return result;
  }
  await post("/record", { state: state(f.context, "before") });
  await post("/accounting", { stage: "before-install", state: state(f.context, "before"), ledger, original_budget: original });
  await post("/record", { state: state(f.context, "before", true) });
  let installed;
  try { installed = await installCandidate(f.root, 0, f.operations); }
  catch (error) {
    const logs = await Promise.all(["failure.json", "install-0.detect.stderr.log", "install-0.stderr.log"].map(async (name) => {
      try { return await readFile(resolve(f.root, name), "utf8"); }
      catch (readError) { if (readError.code !== "ENOENT") throw readError; return "absent"; }
    }));
    assert.fail(`${error.code}: ${logs.join("\n")}`);
  }
  assert.deepEqual(installed, { install_verified: true, index: 0 });
  const claim = JSON.parse(await readFile(resolve(f.root, "install-0.claim.json")));
  const observation = JSON.parse(await readFile(resolve(f.root, "install-0.observation.json")));
  assert.equal(claim.index, 0); assert.equal(observation.failures.length, 0); assert.equal(observation.remaining.length, 0);
  assert(observation.started_at_unix_ms <= claim.atUnixMs);
  await assert.rejects(installCandidate(f.root, 0, f.operations));
});
