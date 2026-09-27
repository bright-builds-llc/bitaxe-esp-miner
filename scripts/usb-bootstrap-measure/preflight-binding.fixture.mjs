// Explicit context-admission fixture. Full interrupted-corpus verification has separate real-file tests.
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { proof } from "../str005-noise-serial/files.mjs";
import { sources } from "./sources.mjs";
import { check, sha256, BEFORE } from "./values.mjs";
export async function prepareSyntheticClosure(f, parent, predecessor) {
  const root = resolve(parent, "attempt-001"), path = `${root}.preflight-closure.json`;
  const context = { schema: "usb-bootstrap-measure-context-v1", firmwareRoot: f.options.firmwareRoot,
    package: { firmware_commit: "f".repeat(40), manifest_sha256: "e".repeat(64) }, sourceInventory: [], beforeSource: BEFORE,
    originalCampaign: predecessor.originalCampaign, expectedAccounting: predecessor.expectedAccounting };
  const hash = sha256(JSON.stringify(context));
  await f.put(resolve(root, "context.json"), JSON.stringify({ context, sha256: hash }));
  await f.put(resolve(parent, "attempt-ordinal-1.json"), JSON.stringify({ schema: "usb-bootstrap-measure-assignment-v1", root, contextSha256: hash, attemptId: "prior-synthetic" }));
  const bytes = await readFile(resolve(parent, "attempt-ordinal-1.json"));
  await f.put(path, JSON.stringify({ schema: "synthetic-closure-fixture", failedContextSha256: hash, assignment: { path: "attempt-ordinal-1.json", sha256: sha256(bytes), length: bytes.length },
    checkerIdentity: { commit: "a".repeat(40), sources: await sources(f.options.firmwareRoot) } }));
  return path;
}
export async function inspectSyntheticClosure(path, operations, { deep } = {}) {
  const root = path.slice(0, -".preflight-closure.json".length), record = await proof(resolve(root, ".."), path);
  check(record.value.schema === "synthetic-closure-fixture", "bootstrap_test_closure");
  const stored = (await proof(root, "context.json")).value;
  check(stored.sha256 === sha256(JSON.stringify(stored.context)) && stored.sha256 === record.value.failedContextSha256, "bootstrap_test_closure");
  const marker = await proof(resolve(root, ".."), "attempt-ordinal-1.json"); check(marker.sha256 === record.value.assignment.sha256, "bootstrap_test_closure");
  return { root, context: stored.context, contextSha256: stored.sha256, assignment: record.value.assignment, closurePath: path, closureSha256: record.sha256,
    checkerIdentity: record.value.checkerIdentity, effectsObserved: false, classification: "interrupted_before_effects", status: "unverified" };
}
