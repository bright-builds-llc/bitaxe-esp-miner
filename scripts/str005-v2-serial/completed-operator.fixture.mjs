// Entirely synthetic unit-fixture observations; not daemon lifecycle or hardware proof.
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";

export function syntheticBrowserWitness(context, last, observedAtUnixMs = Date.now()) {
  const managed = ["str005-v2-serial-context-v5", "str005-v2-serial-context-v6"].includes(context.schema);
  return { schema: managed ? "noise-serial-browser-closure-v3" : "noise-serial-browser-closure-v2",
    source: managed ? "native-ui-observer" : "parent-observed", contextSha256: sha256(JSON.stringify(context)), closed: true,
    lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs };
}

/** Complete the synthetic stopped daemon inventory joined to existing synthetic cleanup. */
export async function completedOperatorFixture(f, browser, supervisor) {
  const { root, context } = f;
  if (!["str005-v2-serial-context-v5", "str005-v2-serial-context-v6"].includes(context.schema)) return;
  const contextSha256 = sha256(JSON.stringify(context)), directory = `${root}.operator`;
  const owner = { pid: 81999, pgid: 81999, startedAt: "synthetic-operator" };
  await mkdir(directory, { mode: 0o700 });
  await writeNew(resolve(directory, "start.claim.json"), { schema: "str005-v2-operator-start-v1", contextSha256 });
  await writeNew(resolve(directory, "locator.json"), { schema: "str005-v2-operator-locator-v1", contextSha256, owner,
    socketPath: "/tmp/v2op-syntheticAbsent/s" });
  const operations = [...context.install_indices.map(index => ["install", { index }]),
    ["prepare-cleanup", {}], ["finish-cleanup", { browserWitness: browser }], ["stop", {}]];
  for (const [index, [action, payload]] of operations.entries()) {
    const requestId = (index + 1).toString(16).padStart(32, "0");
    await writeNew(resolve(directory, `request-${requestId}.json`), { schema: "str005-v2-operator-request-v1", contextSha256, requestId, action, payload });
    await writeNew(resolve(directory, `result-${requestId}.json`), { schema: "str005-v2-operator-response-v1", contextSha256, requestId, status: "succeeded", code: null });
  }
  const observationPath = "parent-cleanup-supervisor-observation.json";
  try {
    const existing = (await proof(root, observationPath)).value;
    assert.equal(existing.schema, "str005-v2-parent-process-observation-v1");
    for (const key of ["contextSha256", "owner", "code", "stopRequestedAtMs", "exitedAtMs"])
      assert.deepEqual(existing[key], supervisor[key], `existing supervisor observation ${key}`);
    assert.equal(existing.signal, null);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    await writeNew(resolve(root, observationPath), { ...supervisor, schema: "str005-v2-parent-process-observation-v1", signal: null });
  }
  await writeNew(resolve(directory, "stopped.json"), { schema: "str005-v2-operator-status-v1", contextSha256, phase: "stopped",
    maybeRequestId: operations.length.toString(16).padStart(32, "0"), maybeAction: "stop", maybeIndex: null, maybeCode: null,
    maybeSupervisorOrigin: (await proof(root, "server-owner.json")).value.origin, lastSequence: browser.lastSequence, lastStateSha256: browser.lastStateSha256 });
  await writeNew(resolve(directory, "disposition.json"), { schema: "str005-v2-operator-disposition-v1", contextSha256, owner,
    sourceSha256: context.evaluator.find(row => row.path === "scripts/str005-v2-serial/operator-daemon.mjs").sha256,
    hostStopped: true, cleanupRecorded: true,
    supervisorObservationSha256: (await proof(root, "parent-cleanup-supervisor-observation.json")).sha256 });
}
